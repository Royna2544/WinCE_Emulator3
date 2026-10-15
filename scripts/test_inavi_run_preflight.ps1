#requires -Version 7.0
<#
Offline acceptance controls for a future bounded-run preflight. This file never
dot-sources the old runner or backup script, creates fixture files, enumerates
mounts, launches a process, or writes a use ledger. All paths are synthetic
strings under this repository's evidence directory.

Proposed pure contract in scripts/inavi_run_preflight_core.ps1:
  Test-InaviRunManifestCore -Manifest <hashtable-or-null> -Observed <hashtable>
    -LocalLedger <array> -TrackedLedger <array>
returns { accepted: bool, codes: string[] }. Observed values are independently
collected by the future monitor worker; the production worker must recheck its
own observations immediately before spawn. Core validation cannot itself
grant or spend approval. The exclusive dual-ledger reservation is separate.

-FixturesOnly checks the in-memory fixture inventory. Until the core exists,
normal execution produces a dependency red, not a behavioral validator red.
#>
[CmdletBinding()]
param([switch]$FixturesOnly)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Copy-Fixture([object]$Value) {
    return ($Value | ConvertTo-Json -Depth 24 | ConvertFrom-Json -AsHashtable)
}

function New-Baseline {
    $root = 'D:\GitHub\Wince_emulation_v3\evidence\stage3-live-evidence-20261015\offline-only'
    $paths = @{
        exe = "$root\fresh.exe"; pdb = "$root\fresh.pdb"
        build = "$root\build-receipt.json"; backups = "$root\backup-manifest.json"
        registry = "$root\registry.reg"; devices = "$root\serial_devices.json"
        mounts = "$root\mounts.toml"; app = "$root\app.exe"
    }
    $roots = @("$root\SD", "$root\ResidentFlash", "$root\EmulatorRoot")
    $backups = @(
        @{ name = 'SD'; sourceRoot = $roots[0]; archivePath = "$root\SD.zip"; archiveSha256 = ('A' * 64) },
        @{ name = 'ResidentFlash'; sourceRoot = $roots[1]; archivePath = "$root\ResidentFlash.zip"; archiveSha256 = ('B' * 64) },
        @{ name = 'EmulatorRoot'; sourceRoot = $roots[2]; archivePath = "$root\EmulatorRoot.zip"; archiveSha256 = ('C' * 64) }
    )
    $seal = @{
        head = 'synthetic-telemetry-gate-head'; parentHead = '0e06e2c5'
        patchSha256 = ('D' * 64); kernelSha256 = ('1' * 64)
        remoteSha256 = ('2' * 64); gweSha256 = ('3' * 64)
        basicSubsystemsSha256 = ('4' * 64)
    }
    $receipt = @{
        path = $paths.build; sha256 = ('E' * 64); sourceHead = $seal.head
        sourcePatchSha256 = $seal.patchSha256; wB1cParentHead = '0e06e2c5'
        focusedPassed = $true; releasePassed = $true
        noFeaturePassed = $true; alternateFeaturesPassed = $true
        executableSha256 = ('5' * 64); pdbSha256 = ('6' * 64)
    }
    $manifest = @{
        schemaVersion = 1; runTag = 'offline-fixture-fresh'
        approvalReference = 'call_offline_fresh_approval'
        sourceSeal = $seal; buildReceipt = $receipt
        executable = @{ path = $paths.exe; sha256 = ('5' * 64); rsds = '11111111-1111-1111-1111-111111111111:1' }
        pdb = @{ path = $paths.pdb; sha256 = ('6' * 64); rsds = '11111111-1111-1111-1111-111111111111:1' }
        config = @{
            registry = @{ path = $paths.registry; sha256 = ('7' * 64) }
            devices = @{ path = $paths.devices; sha256 = ('8' * 64) }
            mounts = @{ path = $paths.mounts; sha256 = ('9' * 64) }
            app = @{ path = $paths.app; sha256 = ('0' * 64) }
        }
        backupManifest = @{ path = $paths.backups; sha256 = ('F' * 64) }
        backups = $backups
        caps = @{ appSeconds = 330; watchdogSeconds = 390; rssMiB = 2048; rawLogMiB = 512 }
    }
    $files = @{}
    foreach ($item in @(
        @($paths.exe, ('5' * 64)), @($paths.pdb, ('6' * 64)),
        @($paths.build, ('E' * 64)), @($paths.backups, ('F' * 64)),
        @($paths.registry, ('7' * 64)), @($paths.devices, ('8' * 64)),
        @($paths.mounts, ('9' * 64)), @($paths.app, ('0' * 64))
    )) { $files[$item[0]] = $item[1] }
    foreach ($item in $backups) { $files[$item.archivePath] = $item.archiveSha256 }
    $observed = @{
        currentSourceSeal = Copy-Fixture $seal
        parsedBuildReceipt = Copy-Fixture $receipt
        fileSha256 = $files
        executableRsds = $manifest.executable.rsds
        pdbRsds = $manifest.pdb.rsds
        writableMountRoots = $roots
        verifiedBackupManifest = @{ sha256 = ('F' * 64); completed = Copy-Fixture $backups }
        existingEvidenceTags = @()
    }
    return @{ manifest = $manifest; observed = $observed; localLedger = @(); trackedLedger = @() }
}

function New-UseEntry([string]$Approval, [string]$State, [string[]]$ZipHashes) {
    return @{
        eventId = "offline-$State-$Approval"
        approvalReference = $Approval; zipSha256 = @($ZipHashes)
        state = $State; utc = '2026-10-15T00:00:00.0000000Z'
    }
}

$base = New-Baseline
$cases = [System.Collections.Generic.List[object]]::new()
$cases.Add(@{ name = 'fresh_baseline'; data = Copy-Fixture $base; accepted = $true; code = $null })

$sample = Copy-Fixture $base
$sample.manifest = $null
$cases.Add(@{ name = 'missing_manifest'; data = $sample; accepted = $false; code = 'SCHEMA_MISSING' })

$sample = Copy-Fixture $base
$sample.manifest.sourceSeal.head = 'e37-stale-source'
$sample.manifest.executable.path = 'D:\GitHub\Wince_emulation_v3\evidence\op4-fresh-view-readback-20261014\wince_emulation_v3-e37.exe'
$cases.Add(@{ name = 'stale_e37_binary_and_source'; data = $sample; accepted = $false; code = 'SOURCE_SEAL_MISMATCH' })

$sample = Copy-Fixture $base
$oldApproval = 'call_93cb71e3f4dc432c84262f0a340f3091'
$sample.manifest.approvalReference = $oldApproval
$entry = New-UseEntry $oldApproval 'spent' @((('1' * 64)), (('2' * 64)), (('3' * 64)))
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'stale_e37_spent_approval'; data = $sample; accepted = $false; code = 'APPROVAL_REUSED' })

$sample = Copy-Fixture $base
$entry = New-UseEntry $sample.manifest.approvalReference 'spent' @(('D' * 64))
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'already_spent_approval'; data = $sample; accepted = $false; code = 'APPROVAL_REUSED' })

$sample = Copy-Fixture $base
$entry = New-UseEntry $sample.manifest.approvalReference 'reserved' @(('D' * 64))
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'interrupted_reservation'; data = $sample; accepted = $false; code = 'APPROVAL_RESERVED' })

$sample = Copy-Fixture $base
$entry = New-UseEntry 'call_other_offline_approval' 'spent' @(('A' * 64))
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'reused_zip_hash'; data = $sample; accepted = $false; code = 'ZIP_REUSED' })

$sample = Copy-Fixture $base
$sample.manifest.backups[0].archiveSha256 = ('D' * 64)
$sample.observed.verifiedBackupManifest.completed[0].archiveSha256 = ('D' * 64)
$cases.Add(@{ name = 'wrong_zip_hash'; data = $sample; accepted = $false; code = 'HASH_MISMATCH' })

$sample = Copy-Fixture $base
$sample.observed.existingEvidenceTags = @($sample.manifest.runTag)
$cases.Add(@{ name = 'duplicate_evidence_tag'; data = $sample; accepted = $false; code = 'EVIDENCE_TAG_EXISTS' })

$sample = Copy-Fixture $base
$sample.manifest.backups[1].sourceRoot = 'D:\GitHub\Wince_emulation_v3\evidence\stage3-live-evidence-20261015\offline-only\WrongRoot'
$sample.observed.verifiedBackupManifest.completed[1].sourceRoot = $sample.manifest.backups[1].sourceRoot
$cases.Add(@{ name = 'mismatched_writable_root'; data = $sample; accepted = $false; code = 'MOUNT_ROOT_MISMATCH' })

$sample = Copy-Fixture $base
$sample.localLedger = @(New-UseEntry 'call_other_offline_approval' 'spent' @(('D' * 64)))
$cases.Add(@{ name = 'local_tracked_ledger_divergence'; data = $sample; accepted = $false; code = 'LEDGER_DIVERGENCE' })

if ($cases.Count -ne 11) { throw "Unexpected fixture count: $($cases.Count)" }
foreach ($test in $cases) {
    if ($null -ne $test.data.manifest -and $test.data.manifest.runTag -notlike 'offline-fixture-*') {
        throw "Non-synthetic run tag in $($test.name)"
    }
    if ($null -ne $test.data.manifest -and $test.data.manifest.backups.Count -ne 3) {
        throw "Fixture lacks exactly three backup rows in $($test.name)"
    }
}
Write-Output "FIXTURES_OK count=$($cases.Count); no files, mounts, ledgers or processes touched"
if ($FixturesOnly) { exit 0 }

$corePath = Join-Path $PSScriptRoot 'inavi_run_preflight_core.ps1'
if (-not (Test-Path -LiteralPath $corePath -PathType Leaf)) {
    [Console]::Error.WriteLine("DEPENDENCY_RED: pure preflight core is not implemented at $corePath; no behavioral validator result is claimed")
    exit 2
}
. $corePath
if ($null -eq (Get-Command Test-InaviRunManifestCore -CommandType Function -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('DEPENDENCY_RED: Test-InaviRunManifestCore is absent; no behavioral validator result is claimed')
    exit 2
}

$failures = [System.Collections.Generic.List[string]]::new()
foreach ($test in $cases) {
    try {
        $result = Test-InaviRunManifestCore -Manifest $test.data.manifest -Observed $test.data.observed -LocalLedger $test.data.localLedger -TrackedLedger $test.data.trackedLedger
        if ($null -eq $result -or $null -eq $result.accepted -or $null -eq $result.codes) {
            throw 'Core did not return accepted and codes fields'
        }
        $accepted = [bool]$result.accepted
        $codes = @($result.codes)
        if ($accepted -ne [bool]$test.accepted) {
            $failures.Add("$($test.name): accepted=$accepted, expected=$($test.accepted)")
        } elseif (-not $accepted -and $test.code -notin $codes) {
            $failures.Add("$($test.name): missing $($test.code), got $($codes -join ',')")
        } else {
            Write-Output "PASS $($test.name) accepted=$accepted codes=$($codes -join ',')"
        }
    } catch {
        $failures.Add("$($test.name): $($_.Exception.Message)")
    }
}
if ($failures.Count -gt 0) {
    foreach ($failure in $failures) { Write-Error "FAIL $failure" }
    exit 1
}
Write-Output "PASS all $($cases.Count) offline preflight controls"
exit 0
