#requires -Version 7.0
<#
Offline guards for the effectful iNavi runner. Parse and define selected
in-memory functions from the runner; never dot-source it. No emulator, mounts, backup,
ledger or worker process is opened by this test.
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$path = Join-Path $PSScriptRoot 'inavi_bounded_e22.ps1'
$tokens = $null
$errors = $null
$ast = [System.Management.Automation.Language.Parser]::ParseFile(
    $path, [ref]$tokens, [ref]$errors)
if ($errors.Count -ne 0) { throw "Runner has $($errors.Count) parse error(s)." }
$definitions = @($ast.FindAll({ param($node)
    $node -is [System.Management.Automation.Language.FunctionDefinitionAst]
}, $true))
foreach ($name in @('Assert-InaviWorkerClaim', 'New-InaviUseEvent',
        'Get-InaviPdbRsds', 'Get-Sha256',
        'Get-InaviLedgerEvidenceHashes', 'Get-InaviReconciliationProofHashes',
        'Get-InaviHistoricalRunEvidenceHashes',
        'Stop-InaviProcessObject',
        'Complete-InaviSpawnCommit')) {
    $definition = @($definitions | Where-Object Name -eq $name)
    if ($definition.Count -ne 1) { throw "Expected one pure runner function: $name" }
    . ([scriptblock]::Create($definition[0].Extent.Text))
}

$passed = 0
function Assert-Guard([bool]$Condition, [string]$Name) {
    if (-not $Condition) { throw "Offline runner guard failed: $Name" }
    $script:passed++
}
function Assert-Reject([scriptblock]$Action, [string]$Name) {
    $rejected = $false
    try { & $Action } catch { $rejected = $true }
    Assert-Guard $rejected $Name
}

$nonce = 'A' * 32
$hash = 'B' * 64
$approval = 'fresh-human-one-run-approval'
$manifestPath = 'D:\GitHub\Wince_emulation_v3\evidence\new-run-manifest.json'
$directory = 'D:\GitHub\Wince_emulation_v3\evidence\live-fresh-run'
$claim = @{
    runTag = 'fresh-run'; launchNonce = $nonce
    manifestPath = $manifestPath; manifestSha256 = $hash
    approvalReference = $approval
    approvalObservedUtc = '2026-10-15T00:00:00Z'
}
$request = $claim.Clone()
Assert-InaviWorkerClaim $claim $request $directory $manifestPath $hash $approval $nonce
Assert-Guard $true 'matching claimed worker request'
$wrong = $request.Clone(); $wrong.launchNonce = 'C' * 32
Assert-Reject { Assert-InaviWorkerClaim $claim $wrong $directory $manifestPath $hash $approval $nonce } 'nonce mismatch rejects'
Assert-Reject { Assert-InaviWorkerClaim $claim $request 'D:\GitHub\Wince_emulation_v3\evidence\live-other' $manifestPath $hash $approval $nonce } 'run-directory mismatch rejects'
Assert-Reject { Assert-InaviWorkerClaim $claim $request $directory $manifestPath $hash 'old-approval' $nonce } 'approval mismatch rejects'
$wrong = $request.Clone(); $wrong.approvalObservedUtc = '2026-10-14T00:00:00Z'
Assert-Reject { Assert-InaviWorkerClaim $claim $wrong $directory $manifestPath $hash $approval $nonce } 'approval observation mismatch rejects'

$manifest = @{ approvalReference = $approval; backups = @(
        @{ archiveSha256 = '1' * 64 }, @{ archiveSha256 = '2' * 64 },
        @{ archiveSha256 = '3' * 64 }) }
$reserved = New-InaviUseEvent 'fresh-run' $hash $manifest 'reserved' $null
Assert-Guard ($reserved.state -eq 'reserved' -and
    $reserved.approvalReference -eq $approval -and
    $reserved.zipSha256.Count -eq 3 -and
    $reserved.zipSha256[0] -eq ('1' * 64) -and
    -not $reserved.ContainsKey('pid')) 'reservation identifies approval and ZIPs before a PID'
$fakeProcess = [pscustomobject]@{ Id = 1234; StartTime = [DateTime]::UtcNow }
$spent = New-InaviUseEvent 'fresh-run' $hash $manifest 'spent' $fakeProcess
Assert-Guard ($spent.state -eq 'spent' -and $spent.pid -eq 1234 -and
    $spent.processStartUtc -match '^\d{4}-\d{2}-\d{2}T') 'spent entry records exact process identity'
Assert-Reject { New-InaviUseEvent 'fresh-run' $hash $manifest 'spent' $null } 'spent without process rejects'

# Inject a failed spent append after a synthetic spawn. The post-spawn commit
# wrapper must close the held process object, not reopen a PID or skip cleanup.
$faulted = [pscustomobject]@{
    Id = 7654; HasExited = $false; KillCount = 0; WaitCount = 0
    StartTime = [DateTime]::UtcNow
}
$faulted | Add-Member ScriptMethod Refresh { }
$faulted | Add-Member ScriptMethod Kill {
    $this.KillCount++; $this.HasExited = $true
}
$faulted | Add-Member ScriptMethod WaitForExit {
    param($milliseconds)
    $this.WaitCount++; return $this.HasExited
}
$script:recordedPostSpawnFailure = $null
Assert-Reject {
    Complete-InaviSpawnCommit $faulted {
        param($spawnedApp, $startUtc)
        throw 'synthetic spent-append failure'
    } {
        param($failure, $spawnedApp)
        $script:recordedPostSpawnFailure = $failure
    }
} 'spent-append failure rejects after synthetic spawn'
Assert-Guard ($faulted.KillCount -eq 1 -and $faulted.WaitCount -eq 1 -and
    $faulted.HasExited -and
    $script:recordedPostSpawnFailure -like '*synthetic spent-append failure*') `
    'spent-append failure records and closes held process object'

# An immediate exit can make Process.StartTime unreadable. It must still
# produce failure evidence, without killing a recycled PID.
$immediate = [pscustomobject]@{ Id = 7655; HasExited = $true; KillCount = 0; WaitCount = 0 }
$immediate | Add-Member ScriptProperty StartTime { throw 'synthetic immediate-exit StartTime failure' }
$immediate | Add-Member ScriptMethod Refresh { }
$immediate | Add-Member ScriptMethod Kill { $this.KillCount++ }
$immediate | Add-Member ScriptMethod WaitForExit {
    param($milliseconds)
    $this.WaitCount++; return $true
}
$script:recordedImmediateFailure = $null
Assert-Reject {
    Complete-InaviSpawnCommit $immediate { throw 'commit must not run' } {
        param($failure, $spawnedApp)
        $script:recordedImmediateFailure = $failure
    }
} 'immediate exit rejects without readable StartTime'
Assert-Guard ($immediate.KillCount -eq 0 -and $immediate.WaitCount -eq 1 -and
    -not [string]::IsNullOrWhiteSpace($script:recordedImmediateFailure)) `
    'immediate exit records failure without PID-only kill'

# A four-block in-memory MSF7 fixture is written only to one random temp file
# because the production parser accepts an exact PDB path. It is removed in
# finally, including on failure; no user PDB or run ledger is opened.
$scratchPdb = Join-Path ([System.IO.Path]::GetTempPath()) (
    'inavi-rsds-offline-' + [Guid]::NewGuid().ToString('N') + '.pdb')
try {
    $pdbBytes = [byte[]]::new(2048)
    [System.Text.Encoding]::ASCII.GetBytes("Microsoft C/C++ MSF 7.00`r`n`u{001A}DS") |
        ForEach-Object -Begin { $i = 0 } -Process { $pdbBytes[$i++] = $_ }
    [BitConverter]::GetBytes([uint32]512).CopyTo($pdbBytes, 32)
    [BitConverter]::GetBytes([uint32]4).CopyTo($pdbBytes, 40)
    [BitConverter]::GetBytes([uint32]16).CopyTo($pdbBytes, 44)
    [BitConverter]::GetBytes([uint32]1).CopyTo($pdbBytes, 52)
    [BitConverter]::GetBytes([uint32]2).CopyTo($pdbBytes, 512)
    [BitConverter]::GetBytes([uint32]2).CopyTo($pdbBytes, 1024)
    [BitConverter]::GetBytes([uint32]28).CopyTo($pdbBytes, 1032)
    [BitConverter]::GetBytes([uint32]3).CopyTo($pdbBytes, 1036)
    [BitConverter]::GetBytes([uint32]2).CopyTo($pdbBytes, 1544)
    $fixtureGuid = [Guid]::Parse('E072B53A-6F13-4C91-B379-2C5E166A419B')
    $fixtureGuid.ToByteArray().CopyTo($pdbBytes, 1548)
    [System.IO.File]::WriteAllBytes($scratchPdb, $pdbBytes)
    Assert-Guard ((Get-InaviPdbRsds $scratchPdb) -eq
        'E072B53A-6F13-4C91-B379-2C5E166A419B:2') 'PDB RSDS GUID and age parsed from stream 1'
    [BitConverter]::GetBytes([uint32]0).CopyTo($pdbBytes, 1544)
    [System.IO.File]::WriteAllBytes($scratchPdb, $pdbBytes)
    Assert-Reject { Get-InaviPdbRsds $scratchPdb } 'zero-age PDB rejects'
} finally {
    if (Test-Path -LiteralPath $scratchPdb) {
        Remove-Item -LiteralPath $scratchPdb -Force
    }
}

$source = [System.IO.File]::ReadAllText($path)
$lockAt = $source.IndexOf('$ledgerLock = [System.IO.FileStream]::new(', [StringComparison]::Ordinal)
$preflightAt = $source.IndexOf('$checked = Assert-InaviManifestPreflight $ManifestPath $ManifestSha256', [StringComparison]::Ordinal)
$observerAt = $source.IndexOf('Assert-InaviObserverLive $Directory', [StringComparison]::Ordinal)
$reserveAt = $source.IndexOf('Add-InaviDualLedgerEntry $checked.ledgers $reservation', [StringComparison]::Ordinal)
$appAt = $source.IndexOf('Start-Process -FilePath ([string]$manifest.executable.path)', [StringComparison]::Ordinal)
$spentAt = $source.IndexOf('Add-InaviDualLedgerEntry $checked.ledgers $spent', [StringComparison]::Ordinal)
Assert-Guard ($lockAt -ge 0 -and $preflightAt -gt $lockAt -and
    $observerAt -gt $preflightAt -and $reserveAt -gt $observerAt -and
    $appAt -gt $reserveAt -and
    $spentAt -gt $appAt) 'worker lock/preflight/reserve/spawn/spent order'
Assert-Guard ($source -notmatch '\$request\.arguments|Test-OriginalMountPreflight|e37-live-|wince_emulation_v3-e37\.exe') 'legacy request and E-37 launch path removed'
Assert-Guard ($source.Contains('Assert-InaviBackupComplete') -and
    $source.Contains('Assert-InaviCheapMountInventory') -and
    $source.Contains('Assert-SlowBootMarkers')) 'worker retains backup, mount and marker gates'
Assert-Guard ($source.Contains("phase = 'prelaunch'") -and
    $source.Contains('launchRelativeHorizonSeconds = 420') -and
    $source.Contains('observer-worker-heartbeat.json') -and
    $source.Contains('coverageIncomplete = $coverageIncomplete')) 'observer has launch-relative horizon and coverage classification'
$observerDefinition = @($definitions | Where-Object Name -eq 'Invoke-Observer')
if ($observerDefinition.Count -ne 1) { throw 'Expected one observer function.' }
$observerText = $observerDefinition[0].Extent.Text
$requestAt = $observerText.IndexOf('$firstFrameRequestTicks = [System.Diagnostics.Stopwatch]::GetTimestamp()', [StringComparison]::Ordinal)
$frameAt = $observerText.IndexOf('$frame = Invoke-GetCapture $client "$base/api/v1/frame.jpg"', [StringComparison]::Ordinal)
$statusAt = $observerText.IndexOf('$status = Invoke-GetCapture $client "$base/api/v1/status"', [StringComparison]::Ordinal)
$summaryAt = $observerText.IndexOf('$summary = Invoke-SummaryCapture', [StringComparison]::Ordinal)
Assert-Guard ($requestAt -ge 0 -and $frameAt -gt $requestAt -and
    $statusAt -gt $frameAt -and
    $summaryAt -gt $statusAt -and
    $observerText.Contains('$firstFrameRequestTicks = [System.Diagnostics.Stopwatch]::GetTimestamp()') -and
    $observerText.Contains('($firstFrameRequestTicks - $launchTicks) -gt (2L * $frequency)') -and
    $observerText.Contains('$firstFrameReceivedTicks = [long]$frame.ReceivedTicks') -and
    $observerText.Contains('($firstFrameReceivedTicks - $launchTicks) -gt (2L * $frequency)') -and
    $observerText.Contains('$frame.Status -ne 200') -and
    $observerText.Contains('firstFrameRequestTicks = $firstFrameRequestTicks') -and
    $observerText.Contains('firstFrameReceivedTicks = $firstFrameReceivedTicks')) `
    'first frame GET precedes status/summary and is bounded by launch-relative coverage'
$firstFrameBlock = $observerText.Substring($frameAt, $statusAt - $frameAt)
Assert-Guard ($firstFrameBlock.Contains("$" + 'coverageIssues.Add(') -and
    $firstFrameBlock.Contains('observer-first-frame.json') -and
    -not $firstFrameBlock.Contains('throw ')) `
    'first-frame HTTP failure is retained as incomplete evidence without stopping later capture'
Assert-Guard ($observerText.Contains("coverageStatus = 'pending-first-frame'") -and
    $observerText.Contains('descriptorOutsideTwoSeconds = $coverageIncomplete')) `
    'pre-frame launch receipt does not claim completed coverage'
Assert-Guard ($source.Contains('Test-InaviMonitorHeartbeatFresh $runDirectory $appDescriptor $launchTicks') -and
    $source.Contains('Stop-InaviSpawnedFallback $runDirectory $monitorProcess $ManifestSha256') -and
    $source.Contains('Stop-InaviUniqueLaunchFallback $runDirectory')) 'driver checks heartbeat and pre-descriptor exact fallback'
Assert-Guard ($source.Contains('$rawLogBytes += (Get-Item -LiteralPath $streamPath).Length') -and
    $source.Contains('if ($rawLogBytes -gt 512MB)')) 'monitor caps combined raw-log bytes'
$coreSource = [System.IO.File]::ReadAllText((Join-Path $PSScriptRoot 'inavi_run_preflight_core.ps1'))
$coreTokens = $null; $coreErrors = $null
$coreAst = [System.Management.Automation.Language.Parser]::ParseInput(
    $coreSource, [ref]$coreTokens, [ref]$coreErrors)
$coreFunctions = @($coreAst.FindAll({ param($node)
    $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
    $node.Name -in @('Get-InaviRequiredSourceFiles', 'Test-InaviCorePath')
}, $true))
if ($coreErrors.Count -ne 0 -or $coreFunctions.Count -ne 2) {
    throw 'Offline runner guard could not load the shared source-file list.'
}
foreach ($coreFunction in $coreFunctions) {
    . ([scriptblock]::Create($coreFunction.Extent.Text))
}
$requiredFiles = @(Get-InaviRequiredSourceFiles)
Assert-Guard ($source.Contains('foreach ($relative in (Get-InaviRequiredSourceFiles))') -and
    $coreSource.Contains('$requiredSourceFiles = @(Get-InaviRequiredSourceFiles)') -and
    $requiredFiles.Count -eq 21 -and '.gitattributes' -in $requiredFiles -and
    'scripts/test_inavi_runner_guard.ps1' -in $requiredFiles) 'worker and validator use one 21-file source map'

$repository = 'D:\GitHub\Wince_emulation_v3'
$emptyProofs = Get-InaviReconciliationProofHashes @(@{ state = 'reserved' }) $repository
Assert-Guard ($emptyProofs.Count -eq 0) 'unreconciled events require no proof-file read'
$emptyHistorical = Get-InaviHistoricalRunEvidenceHashes @(@{ state = 'legacy-deny' }) $repository
Assert-Guard ($emptyHistorical.Count -eq 0) 'legacy-deny does not invent historical run evidence'
$forgedOutside = @{ state = 'reconciled'; noSpawnProof = @{
        path = 'C:\Temp\forged-no-spawn-proof.json'; sha256 = 'A' * 64 } }
Assert-Reject {
    Get-InaviReconciliationProofHashes @($forgedOutside) $repository
} 'reconciliation proof outside checkout evidence rejects'
$historicalOutside = @{ state = 'historical-spent'; runEvidence = @{
        path = 'C:\Temp\forged-historical-run.json'; sha256 = 'A' * 64 } }
Assert-Reject {
    Get-InaviHistoricalRunEvidenceHashes @($historicalOutside) $repository
} 'historical run evidence outside checkout rejects'
$missingInside = @{ state = 'reconciled'; noSpawnProof = @{
        path = (Join-Path $repository ('evidence\missing-offline-' + [Guid]::NewGuid().ToString('N') + '.json'))
        sha256 = 'A' * 64 } }
Assert-Reject {
    Get-InaviReconciliationProofHashes @($missingInside) $repository
} 'missing reconciliation proof rejects'
$historicalMissing = @{ state = 'historical-spent'; runEvidence = @{
        path = $missingInside.noSpawnProof.path; sha256 = 'A' * 64 } }
Assert-Reject {
    Get-InaviHistoricalRunEvidenceHashes @($historicalMissing) $repository
} 'missing historical run evidence rejects'

# A unique proof fixture under this checkout's evidence directory exercises
# the positive hash path. It is never added to either ledger and is deleted
# in finally; no mounted tree, archive or emulator is touched.
$proofFixture = Join-Path $repository ('evidence\runner-proof-offline-' +
    [Guid]::NewGuid().ToString('N') + '.json')
try {
    [System.IO.File]::WriteAllText($proofFixture, '{"offlineFixture":true}')
    $proofHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $proofFixture).Hash
    $validProof = @{ state = 'reconciled'; noSpawnProof = @{
            path = $proofFixture; sha256 = $proofHash } }
    $verified = Get-InaviReconciliationProofHashes @($validProof) $repository
    Assert-Guard ($verified.Count -eq 1 -and $verified[$proofFixture] -eq $proofHash) `
        'reconciliation proof under checkout evidence is independently hashed'
    $validProof.noSpawnProof.sha256 = 'F' * 64
    Assert-Reject {
        Get-InaviReconciliationProofHashes @($validProof) $repository
    } 'reconciliation proof hash mismatch rejects'
    $historical = @{ state = 'historical-spent'; runEvidence = @{
            path = $proofFixture; sha256 = $proofHash } }
    $historicalHashes = Get-InaviHistoricalRunEvidenceHashes @($historical) $repository
    Assert-Guard ($historicalHashes.Count -eq 1 -and
        $historicalHashes[$proofFixture] -eq $proofHash) `
        'historical run receipt is independently hashed under checkout evidence'
    $historical.runEvidence.sha256 = 'F' * 64
    Assert-Reject {
        Get-InaviHistoricalRunEvidenceHashes @($historical) $repository
    } 'historical run receipt hash mismatch rejects'
} finally {
    if (Test-Path -LiteralPath $proofFixture) {
        Remove-Item -LiteralPath $proofFixture -Force
    }
}
Assert-Guard ($source.Contains('$observed.observation.verifiedReconciliationProofSha256 =') -and
    $source.Contains('Get-InaviReconciliationProofHashes $ledgers.local $Repository') -and
    $source.Contains('[System.IO.FileAttributes]::ReparsePoint')) `
    'worker independently hashes scoped no-spawn proofs before core decision'
Assert-Guard ($source.Contains('$observed.observation.verifiedHistoricalRunEvidenceSha256 =') -and
    $source.Contains('Get-InaviHistoricalRunEvidenceHashes $ledgers.local $Repository')) `
    'worker independently hashes named historical receipts before core decision'

Write-Output "Offline runner guards: $passed passed, 0 failed."
