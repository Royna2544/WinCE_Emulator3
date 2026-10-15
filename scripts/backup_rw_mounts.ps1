#requires -Version 7.0
<#
Create verified ZIP backups beside the three exact writable iNavi mounts.
The original trees and earlier archives are never modified. Every file in a
new archive is hashed against the source after archive creation; a ZIP is
published only after all entries have passed verification. The sequential
archives are not an atomic snapshot: all three sources must stay quiescent
until the final content-hash comparison completes.
#>
[CmdletBinding()]
param()

$backupStarted = [DateTime]::UtcNow
$backupStartedUtc = $backupStarted.ToString('O')
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$utf8 = [System.Text.UTF8Encoding]::new($false)
$repository = Split-Path -Parent $PSScriptRoot
if ($repository -ne 'D:\GitHub\Wince_emulation_v3') {
    throw "Unexpected checkout: $repository"
}

$mounts = @(
    @{ Name = 'SD'; ArchiveStem = 'INAVI'; Source = 'D:\INAVI_Emulator\INAVI'; Parent = 'D:\INAVI_Emulator' },
    @{ Name = 'ResidentFlash'; ArchiveStem = 'ResidentFlash'; Source = 'D:\INAVI_Emulator\DUMPPLZ\ResidentFlash'; Parent = 'D:\INAVI_Emulator\DUMPPLZ' },
    @{ Name = 'EmulatorRoot'; ArchiveStem = 'EmulatorRoot'; Source = 'D:\INAVI_Emulator\EmulatorRoot'; Parent = 'D:\INAVI_Emulator' }
)
$tag = $backupStarted.ToString('yyyyMMddTHHmmssZ')
$evidence = Join-Path $repository "evidence\pre-live-mount-backup-$tag"
if (Test-Path -LiteralPath $evidence) { throw "Evidence directory already exists: $evidence" }
[void][System.IO.Directory]::CreateDirectory($evidence)
$activePartial = $null
$completed = [System.Collections.Generic.List[object]]::new()
$initialInventories = [System.Collections.Generic.List[object]]::new()
$manifestPath = Join-Path $evidence 'manifest.json'
. (Join-Path $PSScriptRoot 'backup_rw_mounts_inventory_core.ps1')

function Write-Json([string]$Path, [object]$Value) {
    $json = $Value | ConvertTo-Json -Depth 12
    [System.IO.File]::WriteAllText($Path, $json + "`n", $utf8)
}

function Write-JsonAtomic([string]$Path, [object]$Value) {
    $temporary = Join-Path (Split-Path -Parent $Path) (
        '.manifest-' + [guid]::NewGuid().ToString('N') + '.tmp'
    )
    try {
        Write-Json $temporary $Value
        if (Test-Path -LiteralPath $Path -PathType Leaf) {
            [System.IO.File]::Replace($temporary, $Path, [NullString]::Value)
        } else {
            [System.IO.File]::Move($temporary, $Path)
        }
    } finally {
        if (Test-Path -LiteralPath $temporary -PathType Leaf) {
            Remove-Item -LiteralPath $temporary -Force
        }
    }
}

$manifestState = @{
    schemaVersion = 2
    tag = $tag
    startedUtc = $backupStartedUtc
    updatedUtc = [DateTime]::UtcNow.ToString('O')
    repository = $repository
    scriptSha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $PSCommandPath).Hash.ToUpperInvariant()
    complete = $false
    completed = @()
}

function Publish-Manifest {
    Assert-BackupStartedUtcContract -Manifest $manifestState -ExpectedStartedUtc $backupStartedUtc
    $manifestState.updatedUtc = [DateTime]::UtcNow.ToString('O')
    $manifestState.completed = $completed.ToArray()
    Write-JsonAtomic $manifestPath $manifestState
}

function Get-Sha256([string]$Path) {
    (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToUpperInvariant()
}

function Assert-PlainPath([string]$Path) {
    $cursor = [System.IO.DirectoryInfo]::new($Path)
    while ($null -ne $cursor -and $cursor.FullName -ne 'D:\') {
        if (-not $cursor.Exists) { throw "Mounted path disappeared: $($cursor.FullName)" }
        if ($cursor.Attributes -band [System.IO.FileAttributes]::ReparsePoint) {
            throw "Mounted path has a reparse-point ancestor: $($cursor.FullName)"
        }
        $cursor = $cursor.Parent
    }
}

function Get-SourceInventory([string]$Root) {
    Assert-PlainPath $Root
    $stack = [System.Collections.Generic.Stack[string]]::new()
    $stack.Push($Root)
    $files = [System.Collections.Generic.List[object]]::new()
    $directories = [System.Collections.Generic.List[object]]::new()
    $entryCount = 0
    while ($stack.Count -gt 0) {
        $directory = $stack.Pop()
        foreach ($child in @(Get-ChildItem -LiteralPath $directory -Force -ErrorAction Stop)) {
            $entryCount++
            if ($entryCount -gt 100000) { throw "Mounted inventory exceeded 100,000 entries: $Root" }
            if ($child.Attributes -band [System.IO.FileAttributes]::ReparsePoint) {
                throw "Writable mount contains a reparse point: $($child.FullName)"
            }
            $relative = [System.IO.Path]::GetRelativePath($Root, $child.FullName).Replace('\', '/')
            $record = [pscustomobject]@{
                relative = $relative
                fullPath = $child.FullName
                length = if ($child.PSIsContainer) { $null } else { [long]$child.Length }
                lastWriteUtc = $child.LastWriteTimeUtc.ToString('O')
                attributes = [int]$child.Attributes
                sha256 = $null
            }
            if ($child.PSIsContainer) {
                $directories.Add($record)
                $stack.Push($child.FullName)
            } else {
                $files.Add($record)
            }
        }
    }
    foreach ($file in $files) {
        Assert-UnchangedSource $file
        $sourceStream = [System.IO.FileStream]::new(
            [string]$file.fullPath, [System.IO.FileMode]::Open,
            [System.IO.FileAccess]::Read, [System.IO.FileShare]::Read
        )
        try {
            if ($sourceStream.Length -ne [long]$file.length) {
                throw "Source length changed during inventory: $($file.fullPath)"
            }
            $file.sha256 = Get-StreamSha256 $sourceStream
        } finally {
            $sourceStream.Dispose()
        }
        Assert-UnchangedSource $file
    }
    return [pscustomobject]@{
        files = @($files | Sort-Object relative)
        directories = @($directories | Sort-Object relative)
    }
}

function Add-FileToZip([System.IO.Compression.ZipArchive]$Zip, [object]$File) {
    $entry = $Zip.CreateEntry([string]$File.relative, [System.IO.Compression.CompressionLevel]::Fastest)
    $inputStream = [System.IO.FileStream]::new(
        [string]$File.fullPath, [System.IO.FileMode]::Open,
        [System.IO.FileAccess]::Read, [System.IO.FileShare]::Read
    )
    $entryStream = $null
    $hash = [System.Security.Cryptography.IncrementalHash]::CreateHash(
        [System.Security.Cryptography.HashAlgorithmName]::SHA256
    )
    try {
        if ($inputStream.Length -ne [long]$File.length) {
            throw "Source length changed before archiving: $($File.fullPath)"
        }
        $entryStream = $entry.Open()
        $buffer = [byte[]]::new(1048576)
        [long]$copied = 0
        while (($count = $inputStream.Read($buffer, 0, $buffer.Length)) -gt 0) {
            $hash.AppendData($buffer, 0, $count)
            $entryStream.Write($buffer, 0, $count)
            $copied += $count
        }
        if ($copied -ne [long]$File.length) {
            throw "Source length changed while archiving: $($File.fullPath)"
        }
        $archiveSourceHash = [System.Convert]::ToHexString($hash.GetHashAndReset())
        if ($archiveSourceHash -ne [string]$File.sha256) {
            throw "Source content changed after initial inventory: $($File.fullPath)"
        }
    } finally {
        if ($null -ne $entryStream) { $entryStream.Dispose() }
        $hash.Dispose()
        $inputStream.Dispose()
    }
}

function Get-StreamSha256([System.IO.Stream]$Stream) {
    [System.Convert]::ToHexString([System.Security.Cryptography.SHA256]::HashData($Stream))
}

function Assert-UnchangedSource([object]$Record) {
    $item = Get-Item -LiteralPath ([string]$Record.fullPath) -Force -ErrorAction Stop
    if ($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) {
        throw "Source became a reparse point: $($Record.fullPath)"
    }
    if ([int]$item.Attributes -ne [int]$Record.attributes -or
        $item.LastWriteTimeUtc.ToString('O') -ne [string]$Record.lastWriteUtc) {
        throw "Source metadata changed during backup: $($Record.fullPath)"
    }
    if ($null -ne $Record.length -and [long]$item.Length -ne [long]$Record.length) {
        throw "Source length changed during backup: $($Record.fullPath)"
    }
}

function Assert-ZipMatchesSource([string]$Archive, [object]$Inventory) {
    $zip = [System.IO.Compression.ZipFile]::OpenRead($Archive)
    try {
        $expected = [System.Collections.Generic.Dictionary[string, object]]::new(
            [System.StringComparer]::Ordinal
        )
        foreach ($directory in $Inventory.directories) {
            $expected.Add(([string]$directory.relative + '/'), $directory)
        }
        foreach ($file in $Inventory.files) { $expected.Add([string]$file.relative, $file) }
        if ($zip.Entries.Count -ne $expected.Count) {
            throw "ZIP entry count mismatch: $Archive ($($zip.Entries.Count) vs $($expected.Count))"
        }
        foreach ($entry in $zip.Entries) {
            $record = $null
            if (-not $expected.TryGetValue($entry.FullName, [ref]$record)) {
                throw "Unexpected or duplicate ZIP entry: $($entry.FullName)"
            }
            [void]$expected.Remove($entry.FullName)
            Assert-UnchangedSource $record
            if ($entry.FullName.EndsWith('/')) {
                if ($entry.Length -ne 0) { throw "Nonempty ZIP directory: $($entry.FullName)" }
                continue
            }
            if ($entry.Length -ne [long]$record.length) {
                throw "ZIP length mismatch: $($entry.FullName)"
            }
            $zipStream = $entry.Open()
            try { $zipHash = Get-StreamSha256 $zipStream } finally { $zipStream.Dispose() }
            $sourceStream = [System.IO.FileStream]::new(
                [string]$record.fullPath, [System.IO.FileMode]::Open,
                [System.IO.FileAccess]::Read, [System.IO.FileShare]::Read
            )
            try { $sourceHash = Get-StreamSha256 $sourceStream } finally { $sourceStream.Dispose() }
            if ($zipHash -ne $sourceHash -or $sourceHash -ne [string]$record.sha256) {
                throw "ZIP/source SHA-256 mismatch: $($entry.FullName)"
            }
            Assert-UnchangedSource $record
        }
        if ($expected.Count -ne 0) { throw "ZIP is missing $($expected.Count) source entries: $Archive" }
    } finally {
        $zip.Dispose()
    }
}

try {
    # Publish an incomplete manifest before reading any mount. A crash after
    # this point cannot leave a complete-looking partial three-ZIP set.
    Publish-Manifest
    foreach ($mount in $mounts) {
        $source = [string]$mount.Source
        if (-not (Test-Path -LiteralPath $source -PathType Container)) {
            throw "Mounted source directory is missing: $source"
        }
        $inventory = Get-SourceInventory $source
        $sourceManifest = Join-Path $evidence ("{0}-source-manifest.json" -f $mount.Name)
        Write-Json $sourceManifest @{
            source = $source
            sourceRoot = $source
            capturedUtc = [DateTime]::UtcNow.ToString('O')
            files = $inventory.files
            directories = $inventory.directories
        }
        $initialInventories.Add([pscustomobject]@{
            mount = $mount
            inventory = $inventory
            sourceManifest = $sourceManifest
            sourceManifestSha256 = Get-Sha256 $sourceManifest
        })
    }

    foreach ($item in $initialInventories) {
        $mount = $item.mount
        $source = [string]$mount.Source
        $parent = [string]$mount.Parent
        $inventory = $item.inventory
        $archive = Join-Path $parent ("{0}-pre-live-{1}.zip" -f $mount.ArchiveStem, $tag)
        $partial = "$archive.partial"
        if ((Test-Path -LiteralPath $archive) -or (Test-Path -LiteralPath $partial)) {
            throw "Refusing to overwrite an existing archive or partial: $archive"
        }
        $activePartial = $partial
        Write-Output ("Archiving {0}: {1} files, {2} directories" -f $mount.Name, $inventory.files.Count, $inventory.directories.Count)
        $archiveStream = [System.IO.FileStream]::new(
            $partial, [System.IO.FileMode]::CreateNew,
            [System.IO.FileAccess]::ReadWrite, [System.IO.FileShare]::None
        )
        $zip = $null
        try {
            $zip = [System.IO.Compression.ZipArchive]::new(
                $archiveStream, [System.IO.Compression.ZipArchiveMode]::Create, $false
            )
            foreach ($directory in $inventory.directories) {
                [void]$zip.CreateEntry(([string]$directory.relative + '/'), [System.IO.Compression.CompressionLevel]::NoCompression)
            }
            foreach ($file in $inventory.files) { Add-FileToZip $zip $file }
        } finally {
            if ($null -ne $zip) { $zip.Dispose() } else { $archiveStream.Dispose() }
        }
        Write-Output ("Verifying {0} ZIP entries against original source" -f $mount.Name)
        Assert-ZipMatchesSource $partial $inventory
        $zipHash = Get-Sha256 $partial
        $archiveBytes = (Get-Item -LiteralPath $partial).Length
        [System.IO.File]::Move($partial, $archive)
        $activePartial = $null
        if ((Get-Sha256 $archive) -ne $zipHash) {
            throw "Archive changed after verified rename: $archive"
        }
        $sourceBytes = [long]0
        foreach ($file in $inventory.files) { $sourceBytes += [long]$file.length }
        $completed.Add([pscustomobject]@{
            name = [string]$mount.Name
            sourceRoot = $source
            archivePath = $archive
            source = $source
            archive = $archive
            archiveBytes = $archiveBytes
            archiveSha256 = $zipHash
            sourceFiles = $inventory.files.Count
            sourceDirectories = $inventory.directories.Count
            sourceBytes = $sourceBytes
            sourceManifest = [string]$item.sourceManifest
            sourceManifestSha256 = [string]$item.sourceManifestSha256
            postBackupSourceManifest = $null
            postBackupSourceManifestSha256 = $null
            postBackupInventoryVerified = $false
            zipFilesMatchedSourceBySha256 = $true
        })
        Publish-Manifest
        Write-Output ("Verified {0}: {1}" -f $mount.Name, $archive)
    }

    # The archives are sequential, so re-read every source after *all three*
    # ZIPs have passed. Compare content SHA as well as the final size/mtime
    # inventory that the later pre-spawn check will use.
    for ($index = 0; $index -lt $initialInventories.Count; $index++) {
        $item = $initialInventories[$index]
        $source = [string]$item.mount.Source
        $finalInventory = Get-SourceInventory $source
        Assert-BackupInventoryMatch -Initial $item.inventory -Final $finalInventory -SourceRoot $source
        $finalManifest = Join-Path $evidence ("{0}-post-backup-source-manifest.json" -f $item.mount.Name)
        Write-Json $finalManifest @{
            sourceRoot = $source
            capturedUtc = [DateTime]::UtcNow.ToString('O')
            files = $finalInventory.files
            directories = $finalInventory.directories
        }
        $completed[$index].postBackupSourceManifest = $finalManifest
        $completed[$index].postBackupSourceManifestSha256 = Get-Sha256 $finalManifest
        $completed[$index].postBackupInventoryVerified = $true
        Publish-Manifest
    }

    foreach ($row in $completed) {
        if ((Get-Sha256 ([string]$row.archivePath)) -ne [string]$row.archiveSha256 -or
            (Get-Sha256 ([string]$row.sourceManifest)) -ne [string]$row.sourceManifestSha256 -or
            (Get-Sha256 ([string]$row.postBackupSourceManifest)) -ne [string]$row.postBackupSourceManifestSha256) {
            throw "Backup evidence changed before final completion: $($row.name)"
        }
    }
    if ($completed.Count -ne 3 -or
        $completed[0].name -ne 'SD' -or
        $completed[1].name -ne 'ResidentFlash' -or
        $completed[2].name -ne 'EmulatorRoot' -or
        @($completed | Where-Object { -not $_.postBackupInventoryVerified }).Count -ne 0) {
        throw 'The three required backup rows are not finally verified'
    }
    $manifestState.complete = $true
    Publish-Manifest
} catch {
    $primaryError = $_.Exception
    $message = $primaryError.Message
    $recoveryErrors = [System.Collections.Generic.List[string]]::new()
    $removedPartial = $null
    $manifestState.complete = $false
    try {
        Publish-Manifest
    } catch {
        $recoveryErrors.Add("Incomplete-manifest publication failed: $($_.Exception.Message)")
    }
    if ($null -ne $activePartial -and (Test-Path -LiteralPath $activePartial -PathType Leaf)) {
        try {
            Remove-Item -LiteralPath $activePartial -Force -ErrorAction Stop
            $removedPartial = $activePartial
        } catch {
            $recoveryErrors.Add("Own partial cleanup failed: $($_.Exception.Message)")
        }
    }
    try {
        Write-Json (Join-Path $evidence 'failure.json') @{
            failedUtc = [DateTime]::UtcNow.ToString('O')
            error = $message
            recoveryErrors = $recoveryErrors.ToArray()
            completed = $completed.ToArray()
            removedOwnIncompletePartial = $removedPartial
        }
    } catch {
        $recoveryErrors.Add("Failure receipt write failed: $($_.Exception.Message)")
    }
    if ($recoveryErrors.Count -gt 0) {
        throw [System.InvalidOperationException]::new(
            "Backup failed: $message; recovery also failed: $($recoveryErrors -join '; ')",
            $primaryError
        )
    }
    throw $primaryError
}
Write-Output "All three adjacent ZIP backups and final source inventories verified. Manifest: $manifestPath"
