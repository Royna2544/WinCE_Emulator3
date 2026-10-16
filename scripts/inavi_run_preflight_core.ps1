#requires -Version 7.0
<#
Validation of a proposed bounded iNavi run. The caller must collect Observed
independently, immediately before launch, and must separately reserve
approval/ZIP use under an exclusive lock in both durable ledgers. Production
resolves only a read-only, repo-scoped Git source-path inventory; offline tests
inject that inventory so their behavioral checks remain process-free. This
module does not hash mounted data, grant approval, mutate a ledger, back up a
mount or start the emulator. Rejecting a manifest here is safe; accepting one
is not authority to launch without the effectful worker's remaining checks.
#>

function Test-InaviCoreMap([object]$Value) {
    return $Value -is [System.Collections.IDictionary]
}

function Test-InaviCoreField([object]$Value, [string]$Name) {
    return (Test-InaviCoreMap $Value) -and $Value.Contains($Name) -and $null -ne $Value[$Name]
}

function Test-InaviCoreHash([object]$Value) {
    return $Value -is [string] -and
        $Value -cmatch '^[0-9A-Fa-f]{64}$' -and
        $Value -cnotmatch '^([0-9A-Fa-f])\1{63}$'
}

function Test-InaviCoreCommit([object]$Value) {
    return $Value -is [string] -and
        $Value -cmatch '^[0-9A-Fa-f]{40}$' -and
        $Value -cnotmatch '^([0-9A-Fa-f])\1{39}$'
}

function ConvertTo-InaviCoreUtc([object]$Value) {
    if ($Value -isnot [string] -or
        $Value -cnotmatch '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d{1,7})?Z$') {
        return $null
    }
    [DateTimeOffset]$parsed = [DateTimeOffset]::MinValue
    if (-not [DateTimeOffset]::TryParse(
            $Value,
            [System.Globalization.CultureInfo]::InvariantCulture,
            [System.Globalization.DateTimeStyles]::AssumeUniversal,
            [ref]$parsed
        )) {
        return $null
    }
    return $parsed.ToUniversalTime()
}

function Test-InaviCoreInteger([object]$Value) {
    return $Value -is [int] -or $Value -is [long]
}

function Test-InaviCorePath([object]$Value) {
    return $Value -is [string] -and $Value -cmatch '^[A-Za-z]:\\' -and
        $Value -cnotmatch '[\x00-\x1F]' -and
        $Value -cnotmatch '(^|[\\/])\.\.([\\/]|$)'
}

function Test-InaviCoreSamePath([object]$Left, [object]$Right) {
    if (-not (Test-InaviCorePath $Left) -or -not (Test-InaviCorePath $Right)) {
        return $false
    }
    return [string]::Equals(
        ([string]$Left).TrimEnd('\'), ([string]$Right).TrimEnd('\'),
        [System.StringComparison]::OrdinalIgnoreCase)
}

function Get-InaviRequiredSourceFiles {
    [CmdletBinding()]
    param(
        # Tests inject paths to keep the validator's behavioral fixtures
        # process-free. Production discovers both tracked and non-ignored
        # untracked files from the one authorized source checkout.
        [AllowNull()][string[]]$DiscoveredPaths = $null,
        [string]$SourceRoot = 'D:\GitHub\Wince_emulation_v3'
    )

    $pathspecs = @(
        'src', 'tests', 'scripts', 'vendor/unicorn-engine-sys',
        'Cargo.toml', 'Cargo.lock', '.gitattributes', 'build.rs',
        '.cargo', 'rust-toolchain', 'rust-toolchain.toml'
    )
    $repositoryInventory = $null -eq $DiscoveredPaths
    if ($repositoryInventory) {
        if (-not (Test-InaviCoreSamePath $SourceRoot 'D:\GitHub\Wince_emulation_v3')) {
            throw "Unexpected source checkout for run seal: $SourceRoot"
        }
        # Repo-scoped index plus non-ignored untracked files: a newly added
        # Rust module cannot enter a build without entering the source seal.
        $DiscoveredPaths = @(& git -C $SourceRoot -c core.quotePath=false `
            ls-files --cached --others --exclude-standard --full-name -- @pathspecs)
        if ($LASTEXITCODE -ne 0) {
            throw "Could not inventory source inputs from Git: $SourceRoot"
        }
    }

    $root = [System.IO.Path]::GetFullPath($SourceRoot).TrimEnd('\') + '\'
    $required = [System.Collections.Generic.SortedSet[string]]::new(
        [System.StringComparer]::Ordinal)
    foreach ($relative in $DiscoveredPaths) {
        if ([string]::IsNullOrWhiteSpace($relative) -or
            $relative -cnotmatch '^[A-Za-z0-9._/-]+$' -or
            $relative -cmatch '(^|/)\.\.?(/|$)|//' -or
            $relative.StartsWith('/') -or
            $relative.Contains('\')) {
            throw "Non-canonical source path in run seal inventory: $relative"
        }
        $full = [System.IO.Path]::GetFullPath((Join-Path $SourceRoot $relative))
        if (-not $full.StartsWith($root, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Source path escapes the run checkout: $relative"
        }
        if ($repositoryInventory) {
            $cursor = Get-Item -LiteralPath $full -Force -ErrorAction Stop
            if ($cursor -is [System.IO.DirectoryInfo]) {
                throw "Source inventory entry is not a file: $relative"
            }
            while ($null -ne $cursor) {
                if ($cursor.Attributes -band [System.IO.FileAttributes]::ReparsePoint) {
                    throw "Source inventory path contains a reparse point: $relative"
                }
                if ([string]::Equals($cursor.FullName.TrimEnd('\'),
                        $root.TrimEnd('\'), [System.StringComparison]::OrdinalIgnoreCase)) {
                    break
                }
                $cursor = if ($cursor -is [System.IO.DirectoryInfo]) {
                    $cursor.Parent
                } else {
                    $cursor.Directory
                }
            }
            if ($null -eq $cursor) {
                throw "Source inventory path could not reach the run checkout: $relative"
            }
        }
        $allowed = $relative -in @(
            'Cargo.toml', 'Cargo.lock', '.gitattributes', 'build.rs',
            'rust-toolchain', 'rust-toolchain.toml'
        ) -or $relative -cmatch '^(src|tests|scripts|vendor/unicorn-engine-sys|\.cargo)/'
        if (-not $allowed) {
            throw "Unexpected source path in run seal inventory: $relative"
        }
        [void]$required.Add($relative)
    }
    foreach ($anchor in @(
        'Cargo.toml', 'Cargo.lock', 'src/main.rs', 'src/lib.rs',
        'scripts/inavi_bounded_e22.ps1',
        'scripts/inavi_run_preflight_core.ps1',
        'scripts/backup_rw_mounts.ps1',
        'vendor/unicorn-engine-sys/build.rs'
    )) {
        if (-not $required.Contains($anchor)) {
            throw "Required source anchor absent from run seal inventory: $anchor"
        }
    }
    return [string[]]@($required)
}

function Add-InaviCoreCode([System.Collections.Generic.List[string]]$Codes, [string]$Code) {
    if (-not $Codes.Contains($Code)) { $Codes.Add($Code) }
}

function New-InaviCoreResult([System.Collections.Generic.List[string]]$Codes) {
    return [pscustomobject]@{
        accepted = ($Codes.Count -eq 0)
        codes = [string[]]$Codes.ToArray()
    }
}

function Test-InaviCoreEqual([object]$Left, [object]$Right) {
    if ($null -eq $Left -or $null -eq $Right) {
        return $null -eq $Left -and $null -eq $Right
    }
    if ((Test-InaviCoreMap $Left) -or (Test-InaviCoreMap $Right)) {
        if (-not (Test-InaviCoreMap $Left) -or
            -not (Test-InaviCoreMap $Right) -or
            $Left.Count -ne $Right.Count) {
            return $false
        }
        foreach ($key in $Left.Keys) {
            if (@($Right.Keys) -cnotcontains $key -or
                -not (Test-InaviCoreEqual $Left[$key] $Right[$key])) {
                return $false
            }
        }
        return $true
    }
    if ($Left -is [array] -or $Right -is [array]) {
        if ($Left -isnot [array] -or $Right -isnot [array] -or
            $Left.Count -ne $Right.Count) {
            return $false
        }
        for ($i = 0; $i -lt $Left.Count; $i++) {
            if (-not (Test-InaviCoreEqual $Left[$i] $Right[$i])) {
                return $false
            }
        }
        return $true
    }
    return $Left.GetType() -eq $Right.GetType() -and [object]::Equals($Left, $Right)
}

function Get-InaviCoreUseIdentity([object]$Entry) {
    # A spent event closes only its own earlier reservation. JSON encodes
    # field boundaries, so arbitrary approval text cannot alias another key.
    return ConvertTo-Json -InputObject @(
        [string]$Entry.attemptId,
        ([string]$Entry.manifestSha256).ToUpperInvariant(),
        [string]$Entry.approvalReference,
        @($Entry.zipSha256 | ForEach-Object { ([string]$_).ToUpperInvariant() })
    ) -Compress -Depth 4
}

function Test-InaviCoreEvidencePath([object]$Path, [object]$SourceRoot) {
    if (-not (Test-InaviCorePath $Path) -or
        -not (Test-InaviCorePath $SourceRoot)) {
        return $false
    }
    try {
        $evidenceRoot = [System.IO.Path]::GetFullPath(
            (Join-Path ([string]$SourceRoot) 'evidence')).TrimEnd('\') + '\'
        $candidate = [System.IO.Path]::GetFullPath([string]$Path)
        return $candidate.StartsWith(
            $evidenceRoot, [System.StringComparison]::OrdinalIgnoreCase)
    } catch {
        return $false
    }
}

function Test-InaviCoreHashRecord(
    [object]$Record,
    [object]$FileHashes,
    [System.Collections.Generic.List[string]]$Codes
) {
    if (-not (Test-InaviCoreMap $Record) -or
        -not (Test-InaviCoreField $Record 'path') -or
        -not (Test-InaviCoreField $Record 'sha256')) {
        Add-InaviCoreCode $Codes 'SCHEMA_MISSING'
        return
    }
    if (-not (Test-InaviCorePath $Record.path) -or
        -not (Test-InaviCoreHash $Record.sha256)) {
        Add-InaviCoreCode $Codes 'SCHEMA_INVALID'
        return
    }
    if (-not (Test-InaviCoreMap $FileHashes) -or
        -not $FileHashes.Contains($Record.path) -or
        -not (Test-InaviCoreHash $FileHashes[$Record.path]) -or
        $FileHashes[$Record.path] -ne $Record.sha256) {
        Add-InaviCoreCode $Codes 'HASH_MISMATCH'
    }
}

function Test-InaviCoreLogArgv([object]$Argv, [string]$Profile) {
    if ($Argv -isnot [array] -or $Argv.Count -eq 0 -or
        @($Argv | Where-Object { $_ -isnot [string] }).Count -ne 0) {
        return $false
    }
    $releaseFeatures = @('test', '--release', '--features',
        'unicorn,trace,win32-desktop')
    switch ($Profile) {
        'focused' {
            if ($Argv.Count -le 4 -or
                -not (Test-InaviCoreEqual @($Argv[0..3]) $releaseFeatures)) {
                return $false
            }
            $tail = @($Argv[4..($Argv.Count - 1)])
            $next = 1
            if ($tail[0] -ceq '--test') {
                if ($tail.Count -le $next -or
                    $tail[$next] -cnotmatch '^[A-Za-z0-9_-]+$') {
                    return $false
                }
                $next++
            } elseif ($tail[0] -cne '--lib') {
                return $false
            }
            if ($next -lt $tail.Count -and $tail[$next] -cne '--') {
                if ($tail[$next] -cnotmatch '^[A-Za-z0-9_][A-Za-z0-9_:.+-]*$') {
                    return $false
                }
                $next++
            }
            if ($next -eq $tail.Count) { return $true }
            # Only a display option may cross the libtest argument boundary.
            # --no-run, --list, --ignored and --skip can exit zero without
            # executing the focused test and are not evidence of a pass.
            return $tail.Count -eq ($next + 2) -and
                $tail[$next] -ceq '--' -and
                $tail[$next + 1] -ceq '--nocapture'
        }
        'release' {
            return Test-InaviCoreEqual $Argv @(
                'test', '--release', '--features',
                'unicorn,trace,win32-desktop', '-j', '1')
        }
        'noFeature' { return Test-InaviCoreEqual $Argv @('test', '-j', '1') }
        'alternateUnicorn' {
            return Test-InaviCoreEqual $Argv @('check', '--features', 'unicorn')
        }
        'alternateDesktop' {
            return Test-InaviCoreEqual $Argv @(
                'check', '--features', 'unicorn,win32-desktop')
        }
        'clean' {
            return Test-InaviCoreEqual $Argv @(
                'clean', '--release', '-p', 'wince_emulation_v3')
        }
        'build' {
            return Test-InaviCoreEqual $Argv @(
                'build', '--release', '--features',
                'unicorn,trace,win32-desktop', '-j', '1')
        }
    }
    return $false
}

function Test-InaviCoreStampedLog(
    [object]$Record,
    [string]$Profile,
    [object]$Observed,
    [object]$SourceFiles,
    [object]$SourceRoot,
    [string]$ExpectedHead,
    [System.Collections.Generic.List[string]]$Codes
) {
    if (-not (Test-InaviCoreMap $Record) -or
        -not (Test-InaviCoreField $Record 'path') -or
        -not (Test-InaviCoreField $Record 'sha256') -or
        -not (Test-InaviCoreEvidencePath $Record.path $SourceRoot)) {
        Add-InaviCoreCode $Codes 'BUILD_RECEIPT_MISMATCH'
        return
    }
    Test-InaviCoreHashRecord $Record $Observed.fileSha256 $Codes
    $facts = if ((Test-InaviCoreMap $Observed.rawLogFacts) -and
        $Observed.rawLogFacts.Contains($Record.path)) {
        $Observed.rawLogFacts[$Record.path]
    } else { $null }
    if (-not (Test-InaviCoreMap $facts)) {
        Add-InaviCoreCode $Codes 'BUILD_RECEIPT_MISMATCH'
        return
    }
    foreach ($field in @('attempt', 'command', 'commandArgv',
            'sourceHeadBefore', 'sourceHeadAfter', 'sourceSha256Before',
            'sourceSha256After', 'sourceChangedDuringRun', 'exitCode',
            'runStartUtc', 'runEndUtc', 'compiledPackage', 'finishedRelease')) {
        if (-not (Test-InaviCoreField $facts $field)) {
            Add-InaviCoreCode $Codes 'BUILD_RECEIPT_MISMATCH'
            return
        }
    }
    $start = ConvertTo-InaviCoreUtc $facts.runStartUtc
    $end = ConvertTo-InaviCoreUtc $facts.runEndUtc
    if (-not (Test-InaviCoreInteger $facts.attempt) -or $facts.attempt -le 0 -or
        -not (Test-InaviCoreLogArgv $facts.commandArgv $Profile) -or
        $facts.command -isnot [string] -or
        $facts.command -cne ('cargo ' + ($facts.commandArgv -join ' ')) -or
        -not (Test-InaviCoreCommit $facts.sourceHeadBefore) -or
        $facts.sourceHeadBefore -cne $ExpectedHead -or
        $facts.sourceHeadBefore -cne $facts.sourceHeadAfter -or
        -not (Test-InaviCoreEqual $facts.sourceSha256Before $SourceFiles) -or
        -not (Test-InaviCoreEqual $facts.sourceSha256After $SourceFiles) -or
        $facts.sourceChangedDuringRun -isnot [bool] -or
        $facts.sourceChangedDuringRun -or
        -not (Test-InaviCoreInteger $facts.exitCode) -or
        $facts.exitCode -ne 0 -or
        $null -eq $start -or $null -eq $end -or $end -le $start -or
        $facts.compiledPackage -isnot [bool] -or
        $facts.finishedRelease -isnot [bool] -or
        ($Profile -eq 'build' -and
            (-not $facts.compiledPackage -or -not $facts.finishedRelease))) {
        Add-InaviCoreCode $Codes 'BUILD_RECEIPT_MISMATCH'
    }
    if ($Profile -eq 'focused' -and
        (-not (Test-InaviCoreField $facts 'executedTests') -or
         -not (Test-InaviCoreInteger $facts.executedTests) -or
         $facts.executedTests -lt 1)) {
        Add-InaviCoreCode $Codes 'BUILD_RECEIPT_MISMATCH'
    }
    if ($Profile -eq 'build') {
        foreach ($field in @('buildOutputExePath', 'buildOutputPdbPath',
                'buildOutputExeSha256', 'buildOutputPdbSha256')) {
            if (-not (Test-InaviCoreField $facts $field)) {
                Add-InaviCoreCode $Codes 'BUILD_RECEIPT_MISMATCH'
                return
            }
        }
        $targetExe = Join-Path $SourceRoot 'target\release\wince_emulation_v3.exe'
        $targetPdb = Join-Path $SourceRoot 'target\release\wince_emulation_v3.pdb'
        if (-not (Test-InaviCoreSamePath $facts.buildOutputExePath $targetExe) -or
            -not (Test-InaviCoreSamePath $facts.buildOutputPdbPath $targetPdb) -or
            -not (Test-InaviCoreHash $facts.buildOutputExeSha256) -or
            -not (Test-InaviCoreHash $facts.buildOutputPdbSha256)) {
            Add-InaviCoreCode $Codes 'BUILD_RECEIPT_MISMATCH'
        }
    }
}

function Test-InaviRunManifestCore {
    [CmdletBinding()]
    param(
        [AllowNull()][object]$Manifest,
        [AllowNull()][object]$Observed,
        [AllowNull()][object]$LocalLedger,
        [AllowNull()][object]$TrackedLedger,
        [AllowNull()][string[]]$DiscoveredSourcePaths = $null
    )

    $codes = [System.Collections.Generic.List[string]]::new()
    if (-not (Test-InaviCoreMap $Manifest)) {
        Add-InaviCoreCode $codes 'SCHEMA_MISSING'
        return New-InaviCoreResult $codes
    }

    foreach ($field in @('schemaVersion', 'runTag', 'approvalReference',
            'approvalObservedUtc',
            'sourceSeal', 'sourceSealFile', 'sourcePatchFile', 'buildReceipt',
            'executable', 'pdb', 'config',
            'backupManifest', 'backups', 'caps')) {
        if (-not (Test-InaviCoreField $Manifest $field)) {
            Add-InaviCoreCode $codes 'SCHEMA_MISSING'
            if ($field -in @('approvalReference', 'approvalObservedUtc')) {
                Add-InaviCoreCode $codes 'APPROVAL_MISSING'
            }
        }
    }
    if ($codes.Count -gt 0) { return New-InaviCoreResult $codes }

    $seal = $Manifest.sourceSeal
    $receipt = $Manifest.buildReceipt
    $config = $Manifest.config
    $caps = $Manifest.caps
    foreach ($map in @($seal, $Manifest.sourceSealFile, $Manifest.sourcePatchFile, $receipt,
            $Manifest.executable, $Manifest.pdb,
            $config, $Manifest.backupManifest, $caps)) {
        if (-not (Test-InaviCoreMap $map)) {
            Add-InaviCoreCode $codes 'SCHEMA_INVALID'
        }
    }
    if ($codes.Count -gt 0) { return New-InaviCoreResult $codes }

    foreach ($field in @('head', 'baselineHead', 'sourceRoot', 'sourceFiles',
            'patchSha256', 'kernelSha256', 'remoteSha256', 'gweSha256',
            'basicSubsystemsSha256')) {
        if (-not (Test-InaviCoreField $seal $field)) {
            Add-InaviCoreCode $codes 'SCHEMA_MISSING'
        }
    }
    foreach ($field in @('path', 'sha256', 'kind', 'rustRebuiltForThisReceipt',
            'sourceHead', 'sourceBaselineHead', 'sourcePatchSha256',
            'sourceSealSha256', 'sourceValidationReceipt', 'cleanBuildReceipt',
            'focusedPassed', 'releasePassed', 'noFeaturePassed',
            'alternateFeaturesPassed', 'executableSha256', 'pdbSha256', 'rsds')) {
        if (-not (Test-InaviCoreField $receipt $field)) {
            Add-InaviCoreCode $codes 'SCHEMA_MISSING'
        }
    }
    foreach ($record in @($Manifest.sourceSealFile, $Manifest.sourcePatchFile,
            $Manifest.executable, $Manifest.pdb,
            $Manifest.backupManifest)) {
        foreach ($field in @('path', 'sha256')) {
            if (-not (Test-InaviCoreField $record $field)) {
                Add-InaviCoreCode $codes 'SCHEMA_MISSING'
            }
        }
    }
    foreach ($record in @($Manifest.executable, $Manifest.pdb)) {
        if (-not (Test-InaviCoreField $record 'rsds')) {
            Add-InaviCoreCode $codes 'SCHEMA_MISSING'
        }
    }
    foreach ($name in @('registry', 'devices', 'mounts', 'app')) {
        if (-not (Test-InaviCoreField $config $name) -or
            -not (Test-InaviCoreMap $config[$name])) {
            Add-InaviCoreCode $codes 'SCHEMA_MISSING'
        } else {
            foreach ($field in @('path', 'sha256')) {
                if (-not (Test-InaviCoreField $config[$name] $field)) {
                    Add-InaviCoreCode $codes 'SCHEMA_MISSING'
                }
            }
        }
    }
    foreach ($field in @('appSeconds', 'watchdogSeconds', 'rssMiB', 'rawLogMiB')) {
        if (-not (Test-InaviCoreField $caps $field)) {
            Add-InaviCoreCode $codes 'SCHEMA_MISSING'
        }
    }
    if ($codes.Count -gt 0) { return New-InaviCoreResult $codes }

    if (-not (Test-InaviCoreInteger $Manifest.schemaVersion) -or
        $Manifest.schemaVersion -ne 2 -or
        $Manifest.runTag -isnot [string] -or
        $Manifest.runTag -cnotmatch '^[A-Za-z0-9][A-Za-z0-9._-]{1,127}$' -or
        $Manifest.approvalReference -isnot [string] -or
        $Manifest.approvalReference.Length -gt 256) {
        Add-InaviCoreCode $codes 'SCHEMA_INVALID'
    }
    if ($Manifest.approvalReference -isnot [string] -or
        [string]::IsNullOrWhiteSpace($Manifest.approvalReference)) {
        Add-InaviCoreCode $codes 'APPROVAL_MISSING'
    }
    $approvalObservedUtc = ConvertTo-InaviCoreUtc $Manifest.approvalObservedUtc
    if ($null -eq $approvalObservedUtc) {
        Add-InaviCoreCode $codes 'APPROVAL_MISSING'
    }
    if (-not (Test-InaviCoreCommit $seal.head) -or
        -not (Test-InaviCoreCommit $seal.baselineHead) -or
        $seal.head -eq $seal.baselineHead) {
        Add-InaviCoreCode $codes 'SOURCE_SEAL_MISMATCH'
    }
    foreach ($field in @('patchSha256', 'kernelSha256', 'remoteSha256',
            'gweSha256', 'basicSubsystemsSha256')) {
        if (-not (Test-InaviCoreHash $seal[$field])) {
            Add-InaviCoreCode $codes 'SOURCE_SEAL_MISMATCH'
        }
    }
    if (-not (Test-InaviCoreEvidencePath $Manifest.sourcePatchFile.path $seal.sourceRoot) -or
        $Manifest.sourcePatchFile.sha256 -ne $seal.patchSha256) {
        Add-InaviCoreCode $codes 'SOURCE_SEAL_MISMATCH'
    }
    $requiredSourceFiles = if ($null -eq $DiscoveredSourcePaths) {
        @(Get-InaviRequiredSourceFiles)
    } else {
        @(Get-InaviRequiredSourceFiles -DiscoveredPaths $DiscoveredSourcePaths)
    }
    if (-not (Test-InaviCoreSamePath $seal.sourceRoot 'D:\GitHub\Wince_emulation_v3') -or
        -not (Test-InaviCoreMap $seal.sourceFiles) -or
        $seal.sourceFiles.Count -ne $requiredSourceFiles.Count) {
        Add-InaviCoreCode $codes 'SOURCE_SEAL_MISMATCH'
    } else {
        foreach ($relative in $requiredSourceFiles) {
            if (-not (Test-InaviCoreField $seal.sourceFiles $relative) -or
                -not (Test-InaviCoreHash $seal.sourceFiles[$relative])) {
                Add-InaviCoreCode $codes 'SOURCE_SEAL_MISMATCH'
            }
        }
        foreach ($pair in @(
            @('kernelSha256', 'src/ce/kernel.rs'),
            @('remoteSha256', 'src/ce/remote.rs'),
            @('gweSha256', 'src/ce/gwe.rs'),
            @('basicSubsystemsSha256', 'tests/basic_subsystems.rs')
        )) {
            if ($seal[$pair[0]] -ne $seal.sourceFiles[$pair[1]]) {
                Add-InaviCoreCode $codes 'SOURCE_SEAL_MISMATCH'
            }
        }
    }
    foreach ($field in @('sha256', 'sourcePatchSha256', 'executableSha256',
            'pdbSha256', 'sourceSealSha256')) {
        if (-not (Test-InaviCoreHash $receipt[$field])) {
            Add-InaviCoreCode $codes 'BUILD_RECEIPT_MISMATCH'
        }
    }
    if (-not (Test-InaviCorePath $receipt.path) -or
        -not (Test-InaviCoreCommit $receipt.sourceHead) -or
        -not (Test-InaviCoreCommit $receipt.sourceBaselineHead) -or
        $receipt.sourceHead -ne $seal.head -or
        $receipt.sourcePatchSha256 -ne $seal.patchSha256 -or
        $receipt.sourceSealSha256 -ne $Manifest.sourceSealFile.sha256 -or
        $receipt.sourceBaselineHead -ne $seal.baselineHead -or
        $receipt.kind -cne 'fresh-local-rebuild' -or
        $receipt.rustRebuiltForThisReceipt -isnot [bool] -or
        -not $receipt.rustRebuiltForThisReceipt -or
        $receipt.executableSha256 -ne $Manifest.executable.sha256 -or
        $receipt.pdbSha256 -ne $Manifest.pdb.sha256 -or
        $receipt.rsds -ne $Manifest.executable.rsds -or
        $receipt.rsds -ne $Manifest.pdb.rsds -or
        $receipt.focusedPassed -isnot [bool] -or -not $receipt.focusedPassed -or
        $receipt.releasePassed -isnot [bool] -or -not $receipt.releasePassed -or
        $receipt.noFeaturePassed -isnot [bool] -or -not $receipt.noFeaturePassed -or
        $receipt.alternateFeaturesPassed -isnot [bool] -or
        -not $receipt.alternateFeaturesPassed) {
        Add-InaviCoreCode $codes 'BUILD_RECEIPT_MISMATCH'
    }
    if (-not (Test-InaviCoreInteger $caps.appSeconds) -or
        -not (Test-InaviCoreInteger $caps.watchdogSeconds) -or
        -not (Test-InaviCoreInteger $caps.rssMiB) -or
        -not (Test-InaviCoreInteger $caps.rawLogMiB) -or
        $caps.appSeconds -ne 330 -or $caps.watchdogSeconds -ne 390 -or
        $caps.rssMiB -ne 2048 -or $caps.rawLogMiB -ne 512) {
        Add-InaviCoreCode $codes 'CAP_MISMATCH'
    }

    if (-not (Test-InaviCoreMap $Observed)) {
        Add-InaviCoreCode $codes 'OBSERVATION_MISSING'
        return New-InaviCoreResult $codes
    }
    foreach ($field in @('currentSourceSeal', 'parsedSourceSeal',
            'currentSourceRoot', 'sourceFileSha256', 'currentManifestSha256',
            'parsedBuildReceipt', 'parsedSourceValidationReceipt',
            'parsedCleanBuildReceipt', 'rawLogFacts',
            'fileSha256', 'executableRsds', 'pdbRsds',
            'writableMountRoots', 'verifiedBackupManifest',
            'existingEvidenceTags')) {
        if (-not (Test-InaviCoreField $Observed $field)) {
            Add-InaviCoreCode $codes 'OBSERVATION_MISSING'
        }
    }
    if ($codes.Contains('OBSERVATION_MISSING')) { return New-InaviCoreResult $codes }

    if (-not (Test-InaviCoreMap $Observed.currentSourceSeal) -or
        -not (Test-InaviCoreEqual $seal $Observed.currentSourceSeal) -or
        -not (Test-InaviCoreMap $Observed.parsedSourceSeal) -or
        -not (Test-InaviCoreEqual $seal $Observed.parsedSourceSeal) -or
        -not (Test-InaviCoreSamePath $Observed.currentSourceRoot $seal.sourceRoot) -or
        -not (Test-InaviCoreHash $Observed.currentManifestSha256) -or
        -not (Test-InaviCoreMap $Observed.sourceFileSha256)) {
        Add-InaviCoreCode $codes 'SOURCE_SEAL_MISMATCH'
    } else {
        foreach ($relative in $requiredSourceFiles) {
            if (-not (Test-InaviCoreField $Observed.sourceFileSha256 $relative) -or
                $seal.sourceFiles[$relative] -ne $Observed.sourceFileSha256[$relative]) {
                Add-InaviCoreCode $codes 'SOURCE_SEAL_MISMATCH'
            }
        }
    }
    if (-not (Test-InaviCoreMap $Observed.parsedBuildReceipt)) {
        Add-InaviCoreCode $codes 'BUILD_RECEIPT_MISMATCH'
    } else {
        # `sha256` is the externally computed digest of the receipt file;
        # demanding that value inside the parsed file would be circular.
        foreach ($field in @('path', 'kind', 'rustRebuiltForThisReceipt',
                'sourceHead', 'sourceBaselineHead', 'rsds',
                'sourceSealSha256',
                'sourcePatchSha256', 'sourceValidationReceipt',
                'cleanBuildReceipt', 'focusedPassed',
                'releasePassed', 'noFeaturePassed', 'alternateFeaturesPassed',
                'executableSha256', 'pdbSha256')) {
            if (-not (Test-InaviCoreField $Observed.parsedBuildReceipt $field) -or
                -not (Test-InaviCoreEqual $receipt[$field] $Observed.parsedBuildReceipt[$field])) {
                Add-InaviCoreCode $codes 'BUILD_RECEIPT_MISMATCH'
            }
        }
    }

    $fileHashes = $Observed.fileSha256
    Test-InaviCoreHashRecord $Manifest.sourceSealFile $fileHashes $codes
    Test-InaviCoreHashRecord $Manifest.sourcePatchFile $fileHashes $codes
    Test-InaviCoreHashRecord $receipt $fileHashes $codes
    Test-InaviCoreHashRecord $Manifest.executable $fileHashes $codes
    Test-InaviCoreHashRecord $Manifest.pdb $fileHashes $codes
    Test-InaviCoreHashRecord $Manifest.backupManifest $fileHashes $codes
    foreach ($name in @('registry', 'devices', 'mounts', 'app')) {
        Test-InaviCoreHashRecord $config[$name] $fileHashes $codes
    }

    $seenLogs = [System.Collections.Generic.HashSet[string]]::new(
        [System.StringComparer]::OrdinalIgnoreCase)
    $validation = $Observed.parsedSourceValidationReceipt
    $cleanBuild = $Observed.parsedCleanBuildReceipt
    foreach ($record in @($receipt.sourceValidationReceipt,
            $receipt.cleanBuildReceipt)) {
        if (-not (Test-InaviCoreMap $record) -or
            -not (Test-InaviCoreField $record 'path') -or
            -not (Test-InaviCoreEvidencePath $record.path $seal.sourceRoot)) {
            Add-InaviCoreCode $codes 'BUILD_RECEIPT_MISMATCH'
        } else {
            Test-InaviCoreHashRecord $record $fileHashes $codes
        }
    }
    if (-not (Test-InaviCoreMap $validation) -or
        -not (Test-InaviCoreField $validation 'sourceSealSha256') -or
        -not (Test-InaviCoreField $validation 'sourcePatchSha256') -or
        $validation.sourceSealSha256 -ne $Manifest.sourceSealFile.sha256 -or
        $validation.sourcePatchSha256 -ne $seal.patchSha256 -or
        -not (Test-InaviCoreField $validation 'logs') -or
        -not (Test-InaviCoreMap $validation.logs)) {
        Add-InaviCoreCode $codes 'BUILD_RECEIPT_MISMATCH'
    } else {
        foreach ($profile in @('focused', 'release', 'noFeature',
                'alternateFeatures')) {
            $rows = $validation.logs[$profile]
            if ($rows -isnot [array] -or $rows.Count -lt 1 -or
                ($profile -eq 'alternateFeatures' -and $rows.Count -ne 2)) {
                Add-InaviCoreCode $codes 'BUILD_RECEIPT_MISMATCH'
                continue
            }
            for ($i = 0; $i -lt $rows.Count; $i++) {
                $row = $rows[$i]
                if (-not (Test-InaviCoreMap $row) -or
                    -not (Test-InaviCoreField $row 'path') -or
                    -not $seenLogs.Add([string]$row.path)) {
                    Add-InaviCoreCode $codes 'BUILD_RECEIPT_MISMATCH'
                    continue
                }
                $expected = if ($profile -eq 'alternateFeatures') {
                    if ($i -eq 0) { 'alternateUnicorn' } else { 'alternateDesktop' }
                } else { $profile }
                Test-InaviCoreStampedLog $row $expected $Observed `
                    $seal.sourceFiles $seal.sourceRoot $seal.head $codes
            }
        }
    }
    if (-not (Test-InaviCoreMap $cleanBuild) -or
        -not (Test-InaviCoreField $cleanBuild 'sourceSealSha256') -or
        -not (Test-InaviCoreField $cleanBuild 'sourcePatchSha256') -or
        -not (Test-InaviCoreField $cleanBuild 'cleanLog') -or
        -not (Test-InaviCoreField $cleanBuild 'buildLog') -or
        -not (Test-InaviCoreField $cleanBuild 'executableSha256') -or
        -not (Test-InaviCoreField $cleanBuild 'pdbSha256') -or
        -not (Test-InaviCoreField $cleanBuild 'rsds') -or
        $cleanBuild.sourceSealSha256 -ne $Manifest.sourceSealFile.sha256 -or
        $cleanBuild.sourcePatchSha256 -ne $seal.patchSha256 -or
        $cleanBuild.executableSha256 -ne $Manifest.executable.sha256 -or
        $cleanBuild.pdbSha256 -ne $Manifest.pdb.sha256 -or
        $cleanBuild.rsds -ne $receipt.rsds -or
        -not (Test-InaviCoreMap $cleanBuild.cleanLog) -or
        -not (Test-InaviCoreMap $cleanBuild.buildLog) -or
        $cleanBuild.cleanLog.path -eq $cleanBuild.buildLog.path) {
        Add-InaviCoreCode $codes 'BUILD_RECEIPT_MISMATCH'
    } else {
        if ((Test-InaviCoreMap $validation) -and
            (Test-InaviCoreMap $validation.logs)) {
            foreach ($row in @($cleanBuild.cleanLog, $cleanBuild.buildLog)) {
                if ((Test-InaviCoreField $row 'path') -and
                    -not $seenLogs.Add([string]$row.path)) {
                    Add-InaviCoreCode $codes 'BUILD_RECEIPT_MISMATCH'
                }
            }
        }
        Test-InaviCoreStampedLog $cleanBuild.cleanLog 'clean' $Observed `
            $seal.sourceFiles $seal.sourceRoot $seal.head $codes
        Test-InaviCoreStampedLog $cleanBuild.buildLog 'build' $Observed `
            $seal.sourceFiles $seal.sourceRoot $seal.head $codes
        $cleanFacts = if ((Test-InaviCoreMap $Observed.rawLogFacts) -and
            $Observed.rawLogFacts.Contains($cleanBuild.cleanLog.path)) {
            $Observed.rawLogFacts[$cleanBuild.cleanLog.path]
        } else { $null }
        $buildFacts = if ((Test-InaviCoreMap $Observed.rawLogFacts) -and
            $Observed.rawLogFacts.Contains($cleanBuild.buildLog.path)) {
            $Observed.rawLogFacts[$cleanBuild.buildLog.path]
        } else { $null }
        if (-not (Test-InaviCoreMap $cleanFacts) -or
            -not (Test-InaviCoreMap $buildFacts) -or
            $null -eq (ConvertTo-InaviCoreUtc $cleanFacts.runEndUtc) -or
            $null -eq (ConvertTo-InaviCoreUtc $buildFacts.runStartUtc) -or
            (ConvertTo-InaviCoreUtc $cleanFacts.runEndUtc) -ge
                (ConvertTo-InaviCoreUtc $buildFacts.runStartUtc)) {
            Add-InaviCoreCode $codes 'BUILD_RECEIPT_MISMATCH'
        }
        if (-not (Test-InaviCoreMap $buildFacts) -or
            -not (Test-InaviCoreField $buildFacts 'buildOutputExeSha256') -or
            -not (Test-InaviCoreField $buildFacts 'buildOutputPdbSha256') -or
            $buildFacts.buildOutputExeSha256 -ne $Manifest.executable.sha256 -or
            $buildFacts.buildOutputPdbSha256 -ne $Manifest.pdb.sha256) {
            Add-InaviCoreCode $codes 'BUILD_RECEIPT_MISMATCH'
        }
    }

    $rsdsPattern = '^[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}:[1-9][0-9]*$'
    if ($Manifest.executable.rsds -isnot [string] -or
        $Manifest.pdb.rsds -isnot [string] -or
        $Observed.executableRsds -isnot [string] -or
        $Observed.pdbRsds -isnot [string] -or
        $Manifest.executable.rsds -cnotmatch $rsdsPattern -or
        $Manifest.pdb.rsds -cnotmatch $rsdsPattern -or
        $Manifest.executable.rsds -ne $Manifest.pdb.rsds -or
        $Manifest.executable.rsds -ne $Observed.executableRsds -or
        $Manifest.pdb.rsds -ne $Observed.pdbRsds) {
        Add-InaviCoreCode $codes 'RSDS_MISMATCH'
    }

    if ($Observed.existingEvidenceTags -isnot [array]) {
        Add-InaviCoreCode $codes 'OBSERVATION_MISSING'
    } elseif ($Manifest.runTag -in $Observed.existingEvidenceTags) {
        Add-InaviCoreCode $codes 'EVIDENCE_TAG_EXISTS'
    }

    if ($Manifest.backups -isnot [array] -or $Manifest.backups.Count -ne 3 -or
        $Observed.writableMountRoots -isnot [array] -or
        $Observed.writableMountRoots.Count -ne 3) {
        Add-InaviCoreCode $codes 'MOUNT_ROOT_MISMATCH'
    } else {
        $names = @('SD', 'ResidentFlash', 'EmulatorRoot')
        $seenPaths = [System.Collections.Generic.HashSet[string]]::new(
            [System.StringComparer]::OrdinalIgnoreCase)
        $seenHashes = [System.Collections.Generic.HashSet[string]]::new(
            [System.StringComparer]::OrdinalIgnoreCase)
        for ($i = 0; $i -lt 3; $i++) {
            $backup = $Manifest.backups[$i]
            if (-not (Test-InaviCoreMap $backup) -or
                -not (Test-InaviCoreField $backup 'name') -or
                -not (Test-InaviCoreField $backup 'sourceRoot') -or
                -not (Test-InaviCoreField $backup 'archivePath') -or
                -not (Test-InaviCoreField $backup 'archiveSha256')) {
                Add-InaviCoreCode $codes 'SCHEMA_MISSING'
                continue
            }
            if (-not (Test-InaviCorePath $backup.sourceRoot)) {
                Add-InaviCoreCode $codes 'SCHEMA_INVALID'
                continue
            }
            if ($backup.name -cne $names[$i] -or
                -not (Test-InaviCoreSamePath $backup.sourceRoot $Observed.writableMountRoots[$i])) {
                Add-InaviCoreCode $codes 'MOUNT_ROOT_MISMATCH'
            }
            if (-not (Test-InaviCorePath $backup.archivePath) -or
                -not (Test-InaviCoreHash $backup.archiveSha256)) {
                Add-InaviCoreCode $codes 'SCHEMA_INVALID'
            } else {
                if (-not $seenPaths.Add($backup.archivePath) -or
                    -not $seenHashes.Add($backup.archiveSha256)) {
                    Add-InaviCoreCode $codes 'BACKUP_MANIFEST_MISMATCH'
                }
                Test-InaviCoreHashRecord @{
                    path = $backup.archivePath
                    sha256 = $backup.archiveSha256
                } $fileHashes $codes
                $parent = [System.IO.Path]::GetDirectoryName(
                    ([string]$backup.sourceRoot).TrimEnd('\'))
                $archiveParent = [System.IO.Path]::GetDirectoryName(
                    [string]$backup.archivePath)
                if (-not [string]::Equals($parent, $archiveParent,
                        [System.StringComparison]::OrdinalIgnoreCase) -or
                    -not ([string]$backup.archivePath).EndsWith('.zip',
                        [System.StringComparison]::OrdinalIgnoreCase)) {
                    Add-InaviCoreCode $codes 'BACKUP_MANIFEST_MISMATCH'
                }
            }
        }
    }
    $verified = $Observed.verifiedBackupManifest
    $backupScriptSha256 = if (Test-InaviCoreMap $seal.sourceFiles) {
        $seal.sourceFiles['scripts/backup_rw_mounts.ps1']
    } else {
        $null
    }
    if (-not (Test-InaviCoreMap $verified) -or
        -not (Test-InaviCoreField $verified 'sha256') -or
        $verified.sha256 -ne $Manifest.backupManifest.sha256 -or
        -not (Test-InaviCoreField $verified 'schemaVersion') -or
        $verified.schemaVersion -ne 2 -or
        -not (Test-InaviCoreField $verified 'tag') -or
        $verified.tag -isnot [string] -or
        $verified.tag -cnotmatch '^\d{8}T\d{6}Z$' -or
        -not (Test-InaviCoreField $verified 'scriptSha256') -or
        $verified.scriptSha256 -ne $backupScriptSha256 -or
        -not (Test-InaviCoreField $verified 'startedUtc') -or
        -not (Test-InaviCoreField $verified 'updatedUtc') -or
        -not (Test-InaviCoreField $verified 'complete') -or
        $verified.complete -isnot [bool] -or
        -not $verified.complete -or
        -not (Test-InaviCoreField $verified 'completed') -or
        $verified.completed -isnot [array] -or
        $verified.completed.Count -ne 3) {
        Add-InaviCoreCode $codes 'BACKUP_MANIFEST_MISMATCH'
    } elseif ($Manifest.backups -is [array] -and $Manifest.backups.Count -eq 3) {
        $backupStartedUtc = ConvertTo-InaviCoreUtc $verified.startedUtc
        $backupUpdatedUtc = ConvertTo-InaviCoreUtc $verified.updatedUtc
        if ($null -eq $backupStartedUtc -or $null -eq $backupUpdatedUtc -or
            $null -eq $approvalObservedUtc -or
            $backupStartedUtc -le $approvalObservedUtc -or
            $backupUpdatedUtc -lt $backupStartedUtc -or
            $verified.tag -cne $backupStartedUtc.ToString(
                'yyyyMMddTHHmmssZ',
                [System.Globalization.CultureInfo]::InvariantCulture
            )) {
            Add-InaviCoreCode $codes 'BACKUP_NOT_FRESH'
        }
        $archiveStems = @('INAVI', 'ResidentFlash', 'EmulatorRoot')
        for ($i = 0; $i -lt 3; $i++) {
            $actual = $verified.completed[$i]
            $claimed = $Manifest.backups[$i]
            if (-not (Test-InaviCoreMap $actual) -or
                -not (Test-InaviCoreMap $claimed)) {
                Add-InaviCoreCode $codes 'BACKUP_MANIFEST_MISMATCH'
                continue
            }
            foreach ($field in @('name', 'sourceRoot', 'archivePath',
                    'archiveSha256')) {
                if (-not (Test-InaviCoreField $actual $field) -or
                    -not (Test-InaviCoreField $claimed $field) -or
                    $actual[$field] -ne $claimed[$field]) {
                    Add-InaviCoreCode $codes 'BACKUP_MANIFEST_MISMATCH'
                }
            }
            if ((Test-InaviCoreField $claimed 'archivePath') -and
                [System.IO.Path]::GetFileName([string]$claimed.archivePath) -ine
                "$($archiveStems[$i])-pre-live-$($verified.tag).zip") {
                Add-InaviCoreCode $codes 'BACKUP_NOT_FRESH'
            }
            if (-not (Test-InaviCoreField $actual 'postBackupInventoryVerified') -or
                $actual.postBackupInventoryVerified -isnot [bool] -or
                -not $actual.postBackupInventoryVerified -or
                -not (Test-InaviCoreField $actual 'postBackupSourceManifest') -or
                -not (Test-InaviCorePath $actual.postBackupSourceManifest) -or
                -not (Test-InaviCoreField $actual 'postBackupSourceManifestSha256') -or
                -not (Test-InaviCoreHash $actual.postBackupSourceManifestSha256)) {
                Add-InaviCoreCode $codes 'BACKUP_MANIFEST_MISMATCH'
            }
        }
    }

    $provenancePaths = @($Manifest.runTag, $Manifest.executable.path,
        $Manifest.pdb.path, $Manifest.sourceSealFile.path,
        $Manifest.sourcePatchFile.path, $Manifest.buildReceipt.path,
        $Manifest.backupManifest.path,
        $config.registry.path, $config.devices.path, $config.mounts.path,
        $config.app.path)
    if ($Manifest.backups -is [array]) {
        foreach ($backup in $Manifest.backups) {
            if (Test-InaviCoreMap $backup) {
                $provenancePaths += $backup.archivePath
            }
        }
    }
    foreach ($value in $provenancePaths) {
        if ($value -is [string] -and
            ($value -match '(^|[\\/_.-])e37([\\/_.-]|$)' -or
                $value -match 'pre-live-mount-backup-20261014')) {
            Add-InaviCoreCode $codes 'LEGACY_E37_ARTIFACT'
        }
    }
    if ($Manifest.approvalReference -eq 'call_93cb71e3f4dc432c84262f0a340f3091') {
        Add-InaviCoreCode $codes 'APPROVAL_REUSED'
    }

    if ($null -eq $LocalLedger -or $null -eq $TrackedLedger -or
        $LocalLedger -isnot [array] -or $TrackedLedger -isnot [array]) {
        Add-InaviCoreCode $codes 'LEDGER_UNAVAILABLE'
        return New-InaviCoreResult $codes
    }
    if (-not (Test-InaviCoreEqual $LocalLedger $TrackedLedger)) {
        Add-InaviCoreCode $codes 'LEDGER_DIVERGENCE'
        return New-InaviCoreResult $codes
    }
    $seenEventIds = [System.Collections.Generic.HashSet[string]]::new(
        [System.StringComparer]::Ordinal)
    $reservationsById = [System.Collections.Generic.Dictionary[string, object]]::new(
        [System.StringComparer]::Ordinal)
    $reservationOrder = [System.Collections.Generic.List[object]]::new()
    $openReservationIds = [System.Collections.Generic.HashSet[string]]::new(
        [System.StringComparer]::Ordinal)
    $reconciledIdentities = [System.Collections.Generic.HashSet[string]]::new(
        [System.StringComparer]::Ordinal)
    $usedApprovals = [System.Collections.Generic.HashSet[string]]::new(
        [System.StringComparer]::OrdinalIgnoreCase)
    $usedAttempts = [System.Collections.Generic.HashSet[string]]::new(
        [System.StringComparer]::OrdinalIgnoreCase)
    $withdrawnApprovals = [System.Collections.Generic.HashSet[string]]::new(
        [System.StringComparer]::OrdinalIgnoreCase)
    $withdrawnAttempts = [System.Collections.Generic.HashSet[string]]::new(
        [System.StringComparer]::OrdinalIgnoreCase)
    $nonWithdrawnZipHashes = [System.Collections.Generic.HashSet[string]]::new(
        [System.StringComparer]::OrdinalIgnoreCase)
    $withdrawnZipHashes = [System.Collections.Generic.HashSet[string]]::new(
        [System.StringComparer]::OrdinalIgnoreCase)
    $sourceRootForProof = if (Test-InaviCoreField $Observed 'currentSourceRoot') {
        $Observed.currentSourceRoot
    } else { $null }
    foreach ($entry in $LocalLedger) {
        if (-not (Test-InaviCoreMap $entry) -or
            -not (Test-InaviCoreField $entry 'eventId') -or
            -not (Test-InaviCoreField $entry 'attemptId') -or
            -not (Test-InaviCoreField $entry 'zipSha256') -or
            -not (Test-InaviCoreField $entry 'state') -or
            -not (Test-InaviCoreField $entry 'utc') -or
            $entry.zipSha256 -isnot [array]) {
            Add-InaviCoreCode $codes 'LEDGER_INVALID'
            continue
        }
        $isWithdrawn = $entry.state -eq 'withdrawn'
        if (($isWithdrawn -and $entry.zipSha256.Count -gt 3) -or
            (-not $isWithdrawn -and
                ($entry.zipSha256.Count -ne 3 -or
                    -not (Test-InaviCoreField $entry 'manifestSha256')))) {
            Add-InaviCoreCode $codes 'LEDGER_INVALID'
            continue
        }
        $hasManifestDigest = $entry.Contains('manifestSha256')
        if ($entry.eventId -isnot [string] -or
            [string]::IsNullOrWhiteSpace($entry.eventId) -or
            -not $seenEventIds.Add($entry.eventId) -or
            $entry.attemptId -isnot [string] -or
            $entry.attemptId -cnotmatch '^[A-Za-z0-9][A-Za-z0-9._-]{1,127}$' -or
            (($hasManifestDigest -or -not $isWithdrawn) -and
                -not (Test-InaviCoreHash $entry.manifestSha256)) -or
            $entry.state -isnot [string] -or
            $entry.state -notin @('reserved', 'spent', 'reconciled', 'legacy-deny', 'historical-spent', 'withdrawn') -or
            $entry.utc -isnot [string] -or
            $entry.utc -cnotmatch '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$') {
            Add-InaviCoreCode $codes 'LEDGER_INVALID'
            continue
        }
        if ($entry.state -eq 'legacy-deny') {
            # Old launches with an unrecorded approval must never acquire a
            # fabricated call ID. They deny reuse of their three ZIP hashes.
            if (($entry.Contains('approvalReference') -and
                    $null -ne $entry.approvalReference) -or
                -not (Test-InaviCoreField $entry 'provenance') -or
                $entry.provenance -isnot [string] -or
                [string]::IsNullOrWhiteSpace($entry.provenance) -or
                -not (Test-InaviCoreField $entry 'reason') -or
                $entry.reason -isnot [string] -or
                [string]::IsNullOrWhiteSpace($entry.reason)) {
                Add-InaviCoreCode $codes 'LEDGER_INVALID'
                continue
            }
        } elseif (-not (Test-InaviCoreField $entry 'approvalReference') -or
            $entry.approvalReference -isnot [string] -or
            [string]::IsNullOrWhiteSpace($entry.approvalReference)) {
            Add-InaviCoreCode $codes 'LEDGER_INVALID'
            continue
        }
        if ($entry.state -in @('spent', 'historical-spent') -and
            (-not (Test-InaviCoreField $entry 'pid') -or
                -not (Test-InaviCoreField $entry 'processStartUtc') -or
                -not (Test-InaviCoreInteger $entry.pid) -or
                $entry.pid -le 0 -or
                $entry.processStartUtc -isnot [string] -or
                $entry.processStartUtc -cnotmatch '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$')) {
            Add-InaviCoreCode $codes 'LEDGER_INVALID'
            continue
        }
        $seenZipHashes = [System.Collections.Generic.HashSet[string]]::new(
            [System.StringComparer]::OrdinalIgnoreCase)
        if ($entry.state -ne 'legacy-deny' -and
            $entry.approvalReference -eq $Manifest.approvalReference) {
            if ($entry.state -eq 'reserved') {
                Add-InaviCoreCode $codes 'APPROVAL_RESERVED'
            } else {
                Add-InaviCoreCode $codes 'APPROVAL_REUSED'
            }
        }
        $validZipHashes = $true
        foreach ($zip in $entry.zipSha256) {
            if (-not (Test-InaviCoreHash $zip) -or
                -not $seenZipHashes.Add($zip)) {
                Add-InaviCoreCode $codes 'LEDGER_INVALID'
                $validZipHashes = $false
            } elseif ($Manifest.backups -is [array]) {
                foreach ($backup in $Manifest.backups) {
                    if ((Test-InaviCoreMap $backup) -and
                        (Test-InaviCoreField $backup 'archiveSha256') -and
                        $zip -eq $backup.archiveSha256) {
                        Add-InaviCoreCode $codes 'ZIP_REUSED'
                    }
                }
            }
        }
        if (-not $validZipHashes) { continue }
        if (-not $isWithdrawn) {
            # Historical rows and reserved/spent pairs may share a ZIP, but
            # no row may reuse a ZIP already sealed by a withdrawal.
            foreach ($zip in $entry.zipSha256) {
                if ($withdrawnZipHashes.Contains($zip)) {
                    Add-InaviCoreCode $codes 'LEDGER_INVALID'
                }
                [void]$nonWithdrawnZipHashes.Add($zip)
            }
        }
        if ($entry.state -in @('reserved', 'spent', 'historical-spent', 'legacy-deny')) {
            if ($withdrawnAttempts.Contains($entry.attemptId) -or
                ($entry.state -ne 'legacy-deny' -and
                    $withdrawnApprovals.Contains($entry.approvalReference))) {
                Add-InaviCoreCode $codes 'LEDGER_INVALID'
            }
            [void]$usedAttempts.Add($entry.attemptId)
            if ($entry.state -ne 'legacy-deny') {
                [void]$usedApprovals.Add($entry.approvalReference)
            }
        }
        if ($isWithdrawn) {
            # This terminal deny is for an approval abandoned before a run
            # manifest or process existed. It cannot close a reservation, and
            # its evidence bytes must be independently hashed by the caller.
            $allowedFields = @(
                'eventId', 'attemptId', 'approvalReference', 'state', 'utc',
                'zipSha256', 'reason', 'withdrawalEvidence', 'manifestSha256')
            if (@($entry.Keys | Where-Object { $_ -notin $allowedFields }).Count -gt 0 -or
                -not (Test-InaviCoreField $entry 'reason') -or
                $entry.reason -isnot [string] -or
                [string]::IsNullOrWhiteSpace($entry.reason) -or
                $entry.reason.Length -gt 2048 -or
                $entry.reason -match '[\x00-\x1F]' -or
                $null -eq (ConvertTo-InaviCoreUtc $entry.utc) -or
                -not (Test-InaviCoreField $entry 'withdrawalEvidence') -or
                -not (Test-InaviCoreMap $entry.withdrawalEvidence) -or
                $entry.withdrawalEvidence.Count -ne 2 -or
                -not (Test-InaviCoreField $entry.withdrawalEvidence 'path') -or
                -not (Test-InaviCoreField $entry.withdrawalEvidence 'sha256') -or
                -not (Test-InaviCoreEvidencePath $entry.withdrawalEvidence.path $sourceRootForProof) -or
                -not (Test-InaviCoreHash $entry.withdrawalEvidence.sha256)) {
                Add-InaviCoreCode $codes 'LEDGER_INVALID'
                continue
            }
            $withdrawalHashes = if (Test-InaviCoreField $Observed 'verifiedWithdrawalEvidenceSha256') {
                $Observed.verifiedWithdrawalEvidenceSha256
            } else { $null }
            if (-not (Test-InaviCoreMap $withdrawalHashes) -or
                -not $withdrawalHashes.Contains($entry.withdrawalEvidence.path) -or
                -not (Test-InaviCoreHash $withdrawalHashes[$entry.withdrawalEvidence.path]) -or
                $withdrawalHashes[$entry.withdrawalEvidence.path] -ne
                    $entry.withdrawalEvidence.sha256) {
                Add-InaviCoreCode $codes 'LEDGER_WITHDRAWAL_UNVERIFIED'
                continue
            }
            foreach ($zip in $entry.zipSha256) {
                if ($nonWithdrawnZipHashes.Contains($zip) -or
                    -not $withdrawnZipHashes.Add($zip)) {
                    Add-InaviCoreCode $codes 'LEDGER_INVALID'
                }
            }
            if ($usedApprovals.Contains($entry.approvalReference) -or
                $usedAttempts.Contains($entry.attemptId) -or
                $withdrawnApprovals.Contains($entry.approvalReference) -or
                $withdrawnAttempts.Contains($entry.attemptId)) {
                Add-InaviCoreCode $codes 'LEDGER_INVALID'
            }
            [void]$withdrawnApprovals.Add($entry.approvalReference)
            [void]$withdrawnAttempts.Add($entry.attemptId)
        } elseif ($entry.state -eq 'historical-spent') {
            # These launches predate the ledger. Their observed process and
            # approval provenance is recorded honestly as a terminal deny,
            # never as an invented reservation/spent transition.
            if (-not (Test-InaviCoreField $entry 'runEvidence') -or
                -not (Test-InaviCoreMap $entry.runEvidence) -or
                -not (Test-InaviCoreField $entry.runEvidence 'path') -or
                -not (Test-InaviCoreField $entry.runEvidence 'sha256') -or
                -not (Test-InaviCoreEvidencePath $entry.runEvidence.path $sourceRootForProof) -or
                -not (Test-InaviCoreHash $entry.runEvidence.sha256) -or
                -not (Test-InaviCoreField $entry 'approvalProvenance') -or
                $entry.approvalProvenance -notin @('direct', 'inferred') -or
                -not (Test-InaviCoreField $entry 'approvalSourceReference') -or
                $entry.approvalSourceReference -isnot [string] -or
                [string]::IsNullOrWhiteSpace($entry.approvalSourceReference) -or
                $entry.approvalSourceReference.Length -gt 2048 -or
                $entry.approvalSourceReference -match '[\x00-\x1F]' -or
                $null -eq (ConvertTo-InaviCoreUtc $entry.processStartUtc) -or
                $null -eq (ConvertTo-InaviCoreUtc $entry.utc) -or
                (ConvertTo-InaviCoreUtc $entry.utc) -lt
                    (ConvertTo-InaviCoreUtc $entry.processStartUtc)) {
                Add-InaviCoreCode $codes 'LEDGER_INVALID'
                continue
            }
            $historicalHashes = if (Test-InaviCoreField $Observed 'verifiedHistoricalRunEvidenceSha256') {
                $Observed.verifiedHistoricalRunEvidenceSha256
            } else { $null }
            if (-not (Test-InaviCoreMap $historicalHashes) -or
                -not $historicalHashes.Contains($entry.runEvidence.path) -or
                -not (Test-InaviCoreHash $historicalHashes[$entry.runEvidence.path]) -or
                $historicalHashes[$entry.runEvidence.path] -ne
                    $entry.runEvidence.sha256) {
                Add-InaviCoreCode $codes 'LEDGER_HISTORICAL_UNVERIFIED'
            }
        } elseif ($entry.state -eq 'reserved') {
            $identity = Get-InaviCoreUseIdentity $entry
            $reservationsById.Add($entry.eventId, $entry)
            $reservationOrder.Add(@{ eventId = $entry.eventId; identity = $identity })
            [void]$openReservationIds.Add($entry.eventId)
        } elseif ($entry.state -eq 'spent') {
            $identity = Get-InaviCoreUseIdentity $entry
            if ($reconciledIdentities.Contains($identity)) {
                # A later spent event contradicts the human no-spawn finding.
                Add-InaviCoreCode $codes 'LEDGER_INVALID'
            }
            $matchedReservation = $false
            foreach ($reservation in $reservationOrder) {
                if ($reservation.identity -ceq $identity -and
                    $openReservationIds.Contains($reservation.eventId)) {
                    [void]$openReservationIds.Remove($reservation.eventId)
                    $matchedReservation = $true
                    break
                }
            }
            if (-not $matchedReservation) {
                # Even a well-formed PID/start record cannot claim a launch
                # that was never reserved in this append-only ledger.
                Add-InaviCoreCode $codes 'LEDGER_INVALID'
            }
        } elseif ($entry.state -eq 'reconciled') {
            # A reconciliation is a manually reviewed no-spawn attestation,
            # not proof inferred from the absence of a process file. The
            # caller must independently hash its preserved investigation
            # receipt under the ledger lock and supply that observed digest.
            if (-not (Test-InaviCoreField $entry 'reservationEventId') -or
                $entry.reservationEventId -isnot [string] -or
                [string]::IsNullOrWhiteSpace($entry.reservationEventId) -or
                -not (Test-InaviCoreField $entry 'resolution') -or
                $entry.resolution -cne 'confirmed-no-spawn' -or
                -not (Test-InaviCoreField $entry 'reviewReference') -or
                $entry.reviewReference -isnot [string] -or
                [string]::IsNullOrWhiteSpace($entry.reviewReference) -or
                $entry.reviewReference.Length -gt 256 -or
                $entry.reviewReference -eq $entry.approvalReference -or
                -not (Test-InaviCoreField $entry 'reviewedUtc') -or
                $null -eq (ConvertTo-InaviCoreUtc $entry.reviewedUtc) -or
                -not (Test-InaviCoreField $entry 'noSpawnProof') -or
                -not (Test-InaviCoreMap $entry.noSpawnProof) -or
                -not (Test-InaviCoreField $entry.noSpawnProof 'path') -or
                -not (Test-InaviCoreField $entry.noSpawnProof 'sha256') -or
                -not (Test-InaviCoreEvidencePath $entry.noSpawnProof.path $sourceRootForProof) -or
                -not (Test-InaviCoreHash $entry.noSpawnProof.sha256)) {
                Add-InaviCoreCode $codes 'LEDGER_INVALID'
                continue
            }
            if (-not $reservationsById.ContainsKey($entry.reservationEventId) -or
                -not $openReservationIds.Contains($entry.reservationEventId)) {
                Add-InaviCoreCode $codes 'LEDGER_INVALID'
                continue
            }
            $prior = $reservationsById[$entry.reservationEventId]
            if ((Get-InaviCoreUseIdentity $prior) -cne
                    (Get-InaviCoreUseIdentity $entry) -or
                (ConvertTo-InaviCoreUtc $entry.reviewedUtc) -lt
                    (ConvertTo-InaviCoreUtc $prior.utc) -or
                (ConvertTo-InaviCoreUtc $entry.utc) -lt
                    (ConvertTo-InaviCoreUtc $entry.reviewedUtc)) {
                Add-InaviCoreCode $codes 'LEDGER_INVALID'
                continue
            }
            $proofHashes = if (Test-InaviCoreField $Observed 'verifiedReconciliationProofSha256') {
                $Observed.verifiedReconciliationProofSha256
            } else { $null }
            if (-not (Test-InaviCoreMap $proofHashes) -or
                -not $proofHashes.Contains($entry.noSpawnProof.path) -or
                -not (Test-InaviCoreHash $proofHashes[$entry.noSpawnProof.path]) -or
                $proofHashes[$entry.noSpawnProof.path] -ne
                    $entry.noSpawnProof.sha256) {
                Add-InaviCoreCode $codes 'LEDGER_RECONCILIATION_UNVERIFIED'
                continue
            }
            [void]$openReservationIds.Remove($entry.reservationEventId)
            [void]$reconciledIdentities.Add((Get-InaviCoreUseIdentity $entry))
        }
    }
    if ($openReservationIds.Count -gt 0) {
        Add-InaviCoreCode $codes 'LEDGER_UNRESOLVED_RESERVATION'
    }
    return New-InaviCoreResult $codes
}
