#requires -Version 7.0
<#
Pure validation of a proposed bounded iNavi run. The caller must collect
Observed independently, immediately before launch, and must separately reserve
approval/ZIP use under an exclusive lock in both durable ledgers. This module
does not read a path, grant approval, mutate a ledger, back up a mount or start
a process. Rejecting a manifest here is safe; accepting one is not authority
to launch without the effectful worker's remaining checks.
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
    # One pure list for the validator and the effectful worker's independent
    # file-hash collector. Add a file here before sealing a new run build.
    return @(
        'src/main.rs', 'src/remote_server.rs', 'src/ce/coredll.rs',
        'src/ce/kernel.rs', 'src/ce/remote.rs', 'src/ce/gwe.rs',
        'tests/basic_subsystems.rs', 'src/ce/memory.rs',
        'src/emulator/unicorn.rs', 'Cargo.toml', 'Cargo.lock',
        '.gitattributes',
        'scripts/inavi_bounded_e22.ps1', 'scripts/inavi_run_preflight_core.ps1',
        'scripts/backup_rw_mounts.ps1', 'scripts/test_inavi_run_preflight.ps1',
        'scripts/backup_rw_mounts_inventory_core.ps1',
        'scripts/test_backup_rw_mounts_inventory.ps1',
        'scripts/inavi_summary_observer_core.ps1',
        'scripts/test_inavi_summary_observer.ps1',
        'scripts/test_inavi_runner_guard.ps1'
    )
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

function Test-InaviRunManifestCore {
    [CmdletBinding()]
    param(
        [AllowNull()][object]$Manifest,
        [AllowNull()][object]$Observed,
        [AllowNull()][object]$LocalLedger,
        [AllowNull()][object]$TrackedLedger
    )

    $codes = [System.Collections.Generic.List[string]]::new()
    if (-not (Test-InaviCoreMap $Manifest)) {
        Add-InaviCoreCode $codes 'SCHEMA_MISSING'
        return New-InaviCoreResult $codes
    }

    foreach ($field in @('schemaVersion', 'runTag', 'approvalReference',
            'approvalObservedUtc',
            'sourceSeal', 'sourceSealFile', 'buildReceipt', 'executable', 'pdb', 'config',
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
    foreach ($map in @($seal, $Manifest.sourceSealFile, $receipt,
            $Manifest.executable, $Manifest.pdb,
            $config, $Manifest.backupManifest, $caps)) {
        if (-not (Test-InaviCoreMap $map)) {
            Add-InaviCoreCode $codes 'SCHEMA_INVALID'
        }
    }
    if ($codes.Count -gt 0) { return New-InaviCoreResult $codes }

    foreach ($field in @('head', 'parentHead', 'sourceRoot', 'sourceFiles',
            'patchSha256', 'kernelSha256', 'remoteSha256', 'gweSha256',
            'basicSubsystemsSha256')) {
        if (-not (Test-InaviCoreField $seal $field)) {
            Add-InaviCoreCode $codes 'SCHEMA_MISSING'
        }
    }
    foreach ($field in @('path', 'sha256', 'sourceHead', 'sourcePatchSha256',
            'sourceSealSha256',
            'wB1cParentHead', 'focusedPassed', 'releasePassed',
            'noFeaturePassed', 'alternateFeaturesPassed', 'executableSha256',
            'pdbSha256')) {
        if (-not (Test-InaviCoreField $receipt $field)) {
            Add-InaviCoreCode $codes 'SCHEMA_MISSING'
        }
    }
    foreach ($record in @($Manifest.sourceSealFile, $Manifest.executable, $Manifest.pdb,
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
        $Manifest.schemaVersion -ne 1 -or
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
        -not (Test-InaviCoreCommit $seal.parentHead) -or
        $seal.parentHead -ne '0e06e2c5c662ad224c9b472e7798f4604240a6b6' -or
        $seal.head -eq $seal.parentHead) {
        Add-InaviCoreCode $codes 'SOURCE_SEAL_MISMATCH'
    }
    foreach ($field in @('patchSha256', 'kernelSha256', 'remoteSha256',
            'gweSha256', 'basicSubsystemsSha256')) {
        if (-not (Test-InaviCoreHash $seal[$field])) {
            Add-InaviCoreCode $codes 'SOURCE_SEAL_MISMATCH'
        }
    }
    $requiredSourceFiles = @(Get-InaviRequiredSourceFiles)
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
        -not (Test-InaviCoreCommit $receipt.wB1cParentHead) -or
        $receipt.sourceHead -ne $seal.head -or
        $receipt.sourcePatchSha256 -ne $seal.patchSha256 -or
        $receipt.sourceSealSha256 -ne $Manifest.sourceSealFile.sha256 -or
        $receipt.wB1cParentHead -ne $seal.parentHead -or
        $receipt.executableSha256 -ne $Manifest.executable.sha256 -or
        $receipt.pdbSha256 -ne $Manifest.pdb.sha256 -or
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
            'parsedBuildReceipt',
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
        foreach ($field in @('path', 'sourceHead',
                'sourceSealSha256',
                'sourcePatchSha256', 'wB1cParentHead', 'focusedPassed',
                'releasePassed', 'noFeaturePassed', 'alternateFeaturesPassed',
                'executableSha256', 'pdbSha256')) {
            if (-not (Test-InaviCoreField $Observed.parsedBuildReceipt $field) -or
                $receipt[$field] -ne $Observed.parsedBuildReceipt[$field]) {
                Add-InaviCoreCode $codes 'BUILD_RECEIPT_MISMATCH'
            }
        }
    }

    $fileHashes = $Observed.fileSha256
    Test-InaviCoreHashRecord $Manifest.sourceSealFile $fileHashes $codes
    Test-InaviCoreHashRecord $receipt $fileHashes $codes
    Test-InaviCoreHashRecord $Manifest.executable $fileHashes $codes
    Test-InaviCoreHashRecord $Manifest.pdb $fileHashes $codes
    Test-InaviCoreHashRecord $Manifest.backupManifest $fileHashes $codes
    foreach ($name in @('registry', 'devices', 'mounts', 'app')) {
        Test-InaviCoreHashRecord $config[$name] $fileHashes $codes
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
        $Manifest.buildReceipt.path, $Manifest.backupManifest.path,
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
