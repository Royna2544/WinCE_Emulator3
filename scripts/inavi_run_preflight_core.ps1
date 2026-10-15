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
            'sourceSeal', 'buildReceipt', 'executable', 'pdb', 'config',
            'backupManifest', 'backups', 'caps')) {
        if (-not (Test-InaviCoreField $Manifest $field)) {
            Add-InaviCoreCode $codes 'SCHEMA_MISSING'
            if ($field -eq 'approvalReference') {
                Add-InaviCoreCode $codes 'APPROVAL_MISSING'
            }
        }
    }
    if ($codes.Count -gt 0) { return New-InaviCoreResult $codes }

    $seal = $Manifest.sourceSeal
    $receipt = $Manifest.buildReceipt
    $config = $Manifest.config
    $caps = $Manifest.caps
    foreach ($map in @($seal, $receipt, $Manifest.executable, $Manifest.pdb,
            $config, $Manifest.backupManifest, $caps)) {
        if (-not (Test-InaviCoreMap $map)) {
            Add-InaviCoreCode $codes 'SCHEMA_INVALID'
        }
    }
    if ($codes.Count -gt 0) { return New-InaviCoreResult $codes }

    foreach ($field in @('head', 'parentHead', 'patchSha256', 'kernelSha256',
            'remoteSha256', 'gweSha256', 'basicSubsystemsSha256')) {
        if (-not (Test-InaviCoreField $seal $field)) {
            Add-InaviCoreCode $codes 'SCHEMA_MISSING'
        }
    }
    foreach ($field in @('path', 'sha256', 'sourceHead', 'sourcePatchSha256',
            'wB1cParentHead', 'focusedPassed', 'releasePassed',
            'noFeaturePassed', 'alternateFeaturesPassed', 'executableSha256',
            'pdbSha256')) {
        if (-not (Test-InaviCoreField $receipt $field)) {
            Add-InaviCoreCode $codes 'SCHEMA_MISSING'
        }
    }
    foreach ($record in @($Manifest.executable, $Manifest.pdb,
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
    foreach ($field in @('sha256', 'sourcePatchSha256', 'executableSha256',
            'pdbSha256')) {
        if (-not (Test-InaviCoreHash $receipt[$field])) {
            Add-InaviCoreCode $codes 'BUILD_RECEIPT_MISMATCH'
        }
    }
    if (-not (Test-InaviCorePath $receipt.path) -or
        -not (Test-InaviCoreCommit $receipt.sourceHead) -or
        -not (Test-InaviCoreCommit $receipt.wB1cParentHead) -or
        $receipt.sourceHead -ne $seal.head -or
        $receipt.sourcePatchSha256 -ne $seal.patchSha256 -or
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
    foreach ($field in @('currentSourceSeal', 'parsedBuildReceipt',
            'fileSha256', 'executableRsds', 'pdbRsds',
            'writableMountRoots', 'verifiedBackupManifest',
            'existingEvidenceTags')) {
        if (-not (Test-InaviCoreField $Observed $field)) {
            Add-InaviCoreCode $codes 'OBSERVATION_MISSING'
        }
    }
    if ($codes.Contains('OBSERVATION_MISSING')) { return New-InaviCoreResult $codes }

    if (-not (Test-InaviCoreMap $Observed.currentSourceSeal)) {
        Add-InaviCoreCode $codes 'SOURCE_SEAL_MISMATCH'
    } else {
        foreach ($field in @('head', 'parentHead', 'patchSha256',
                'kernelSha256', 'remoteSha256', 'gweSha256',
                'basicSubsystemsSha256')) {
            if (-not (Test-InaviCoreField $Observed.currentSourceSeal $field) -or
                $seal[$field] -ne $Observed.currentSourceSeal[$field]) {
                Add-InaviCoreCode $codes 'SOURCE_SEAL_MISMATCH'
            }
        }
    }
    if (-not (Test-InaviCoreMap $Observed.parsedBuildReceipt)) {
        Add-InaviCoreCode $codes 'BUILD_RECEIPT_MISMATCH'
    } else {
        foreach ($field in @('path', 'sha256', 'sourceHead',
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
    if (-not (Test-InaviCoreMap $verified) -or
        -not (Test-InaviCoreField $verified 'sha256') -or
        $verified.sha256 -ne $Manifest.backupManifest.sha256 -or
        -not (Test-InaviCoreField $verified 'completed') -or
        $verified.completed -isnot [array] -or
        $verified.completed.Count -ne 3) {
        Add-InaviCoreCode $codes 'BACKUP_MANIFEST_MISMATCH'
    } elseif ($Manifest.backups -is [array] -and $Manifest.backups.Count -eq 3) {
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
        }
    }

    foreach ($value in @($Manifest.runTag, $Manifest.executable.path,
            $Manifest.pdb.path, $Manifest.backupManifest.path)) {
        if ($value -is [string] -and
            $value -match '(^|[\\/_.-])e37([\\/_.-]|$)') {
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
    foreach ($entry in $LocalLedger) {
        if (-not (Test-InaviCoreMap $entry) -or
            -not (Test-InaviCoreField $entry 'eventId') -or
            -not (Test-InaviCoreField $entry 'approvalReference') -or
            -not (Test-InaviCoreField $entry 'zipSha256') -or
            -not (Test-InaviCoreField $entry 'state') -or
            -not (Test-InaviCoreField $entry 'utc') -or
            $entry.zipSha256 -isnot [array]) {
            Add-InaviCoreCode $codes 'LEDGER_INVALID'
            continue
        }
        if ($entry.eventId -isnot [string] -or
            [string]::IsNullOrWhiteSpace($entry.eventId) -or
            $entry.approvalReference -isnot [string] -or
            [string]::IsNullOrWhiteSpace($entry.approvalReference) -or
            $entry.state -isnot [string] -or
            $entry.state -notin @('reserved', 'spent', 'reconciled') -or
            $entry.utc -isnot [string] -or
            $entry.utc -cnotmatch '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$') {
            Add-InaviCoreCode $codes 'LEDGER_INVALID'
            continue
        }
        if ($entry.approvalReference -eq $Manifest.approvalReference) {
            if ($entry.state -eq 'reserved') {
                Add-InaviCoreCode $codes 'APPROVAL_RESERVED'
            } else {
                Add-InaviCoreCode $codes 'APPROVAL_REUSED'
            }
        }
        foreach ($zip in $entry.zipSha256) {
            if (-not (Test-InaviCoreHash $zip)) {
                Add-InaviCoreCode $codes 'LEDGER_INVALID'
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
    }
    return New-InaviCoreResult $codes
}
