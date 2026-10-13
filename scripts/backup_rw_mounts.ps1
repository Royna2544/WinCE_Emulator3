#requires -Version 7.0
<#
Create point-in-time ZIP backups beside the three exact writable iNavi mounts.
The original trees and earlier archives are never modified. Every file in a
new archive is hashed against the source after archive creation; a ZIP is
published only after all entries have passed verification.
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$utf8 = [System.Text.UTF8Encoding]::new($false)
$repository = Split-Path -Parent $PSScriptRoot
if ($repository -ne 'D:\GitHub\Wince_emulation_v3') {
    throw "Unexpected checkout: $repository"
}

$mounts = @(
    @{ Name = 'INAVI'; Source = 'D:\INAVI_Emulator\INAVI'; Parent = 'D:\INAVI_Emulator' },
    @{ Name = 'ResidentFlash'; Source = 'D:\INAVI_Emulator\DUMPPLZ\ResidentFlash'; Parent = 'D:\INAVI_Emulator\DUMPPLZ' },
    @{ Name = 'EmulatorRoot'; Source = 'D:\INAVI_Emulator\EmulatorRoot'; Parent = 'D:\INAVI_Emulator' }
)
$tag = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ')
$evidence = Join-Path $repository "evidence\pre-live-mount-backup-$tag"
if (Test-Path -LiteralPath $evidence) { throw "Evidence directory already exists: $evidence" }
[void][System.IO.Directory]::CreateDirectory($evidence)
$activePartial = $null
$completed = [System.Collections.Generic.List[object]]::new()

function Write-Json([string]$Path, [object]$Value) {
    $json = $Value | ConvertTo-Json -Depth 12
    [System.IO.File]::WriteAllText($Path, $json + "`n", $utf8)
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
        $File.sha256 = [System.Convert]::ToHexString($hash.GetHashAndReset())
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
    foreach ($mount in $mounts) {
        $source = [string]$mount.Source
        $parent = [string]$mount.Parent
        if (-not (Test-Path -LiteralPath $source -PathType Container)) {
            throw "Mounted source directory is missing: $source"
        }
        $inventory = Get-SourceInventory $source
        $archive = Join-Path $parent ("{0}-pre-live-{1}.zip" -f $mount.Name, $tag)
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
        $sourceManifest = Join-Path $evidence ("{0}-source-manifest.json" -f $mount.Name)
        Write-Json $sourceManifest @{
            source = $source
            files = $inventory.files
            directories = $inventory.directories
        }
        $sourceBytes = [long]0
        foreach ($file in $inventory.files) { $sourceBytes += [long]$file.length }
        $completed.Add([pscustomobject]@{
            source = $source
            archive = $archive
            archiveBytes = $archiveBytes
            archiveSha256 = $zipHash
            sourceFiles = $inventory.files.Count
            sourceDirectories = $inventory.directories.Count
            sourceBytes = $sourceBytes
            sourceManifest = $sourceManifest
            sourceManifestSha256 = Get-Sha256 $sourceManifest
            zipFilesMatchedSourceBySha256 = $true
        })
        Write-Json (Join-Path $evidence 'manifest.json') @{
            tag = $tag; updatedUtc = [DateTime]::UtcNow.ToString('O')
            repository = $repository; scriptSha256 = Get-Sha256 $PSCommandPath
            completed = $completed.ToArray()
        }
        Write-Output ("Verified {0}: {1}" -f $mount.Name, $archive)
    }
    Write-Output "All three adjacent ZIP backups verified. Manifest: $(Join-Path $evidence 'manifest.json')"
} catch {
    $message = $_.Exception.Message
    if ($null -ne $activePartial -and (Test-Path -LiteralPath $activePartial -PathType Leaf)) {
        Remove-Item -LiteralPath $activePartial -Force -ErrorAction Stop
    }
    Write-Json (Join-Path $evidence 'failure.json') @{
        failedUtc = [DateTime]::UtcNow.ToString('O')
        error = $message; completed = $completed.ToArray()
        removedOwnIncompletePartial = $activePartial
    }
    throw
}
