#requires -Version 7.0
<#
Bounded mounted iNavi evidence run. A fresh, separately approved run manifest,
source/build seal and three completed adjacent ZIP backups are required. The
detached monitor repeats provenance and mount checks and durably reserves the
approval and ZIPs immediately before it may launch the emulator. Never call
the worker switches directly.

The NMEA stream is a NEW diagnostic input, not a replay of the older GPS run:
16 one-Hz RMC/GGA/VTG bursts at 37.5 N, 127.25 E, altitude 42 m, speed 0.
Its HTTP ACKs are not proof that guest COM7: consumed or parsed the bytes.
#>
[CmdletBinding()]
param(
    [switch]$ObserverWorker,
    [switch]$MonitorWorker,
    [string]$RunDirectory,
    [string]$ManifestPath,
    [string]$ManifestSha256,
    [string]$ApprovalReference,
    [string]$LaunchNonce,
    [ValidateRange(512, 65536)][int]$MaxWorkingSetMiB = 2048
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$utf8 = [System.Text.UTF8Encoding]::new($false)
$frequency = [System.Diagnostics.Stopwatch]::Frequency
. (Join-Path $PSScriptRoot 'inavi_summary_observer_core.ps1')
. (Join-Path $PSScriptRoot 'inavi_run_preflight_core.ps1')

function Write-JsonFile([string]$Path, [object]$Value) {
    $json = $Value | ConvertTo-Json -Depth 12
    [System.IO.File]::WriteAllText($Path, $json + "`n", $utf8)
}

function Write-AtomicJsonFile([string]$Path, [object]$Value) {
    $temporary = "$Path.tmp"
    Write-JsonFile $temporary $Value
    [System.IO.File]::Move($temporary, $Path)
}

function Add-JsonLine([string]$Path, [object]$Value) {
    $json = $Value | ConvertTo-Json -Compress -Depth 12
    [System.IO.File]::AppendAllText($Path, $json + "`n", $utf8)
}

function Get-Sha256([string]$Path) {
    (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToUpperInvariant()
}

function Get-ByteSha256([byte[]]$Bytes) {
    [System.Convert]::ToHexString([System.Security.Cryptography.SHA256]::HashData($Bytes))
}

function Test-ExpectedHash([string]$Path, [string]$Expected) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Required file is missing: $Path"
    }
    $actual = Get-Sha256 $Path
    if ($actual -ne $Expected) {
        throw "SHA-256 mismatch for $Path (expected $Expected, got $actual)"
    }
}

function Assert-NoReparseMounts([object[]]$Backups) {
    # The guest path mapper joins names lexically. A junction in any writable
    # root could move guest writes outside the three archived trees.
    $entryCount = 0
    foreach ($item in $Backups) {
        $root = [string]$item.Source
        $ancestor = [System.IO.DirectoryInfo]::new($root)
        while ($null -ne $ancestor -and $ancestor.Parent -ne $null) {
            if (-not $ancestor.Exists) { throw "Mounted path disappeared: $($ancestor.FullName)" }
            if ($ancestor.Attributes -band [System.IO.FileAttributes]::ReparsePoint) {
                throw "Mounted path contains a reparse-point ancestor: $($ancestor.FullName)"
            }
            $ancestor = $ancestor.Parent
        }
        $pending = [System.Collections.Generic.Stack[string]]::new()
        $pending.Push($root)
        while ($pending.Count -ne 0) {
            $directory = $pending.Pop()
            foreach ($child in @(Get-ChildItem -LiteralPath $directory -Force -ErrorAction Stop)) {
                $entryCount++
                if ($entryCount -gt 100000) { throw 'Mounted reparse scan exceeded its 100,000-entry safety bound.' }
                if ($child.Attributes -band [System.IO.FileAttributes]::ReparsePoint) {
                    throw "Writable mount contains a reparse point: $($child.FullName)"
                }
                if ($child.PSIsContainer) { $pending.Push($child.FullName) }
            }
        }
    }
}

function Read-InaviJson([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw "Missing JSON file: $Path" }
    return Get-Content -LiteralPath $Path -Raw -ErrorAction Stop |
        ConvertFrom-Json -AsHashtable -DateKind String -ErrorAction Stop
}

function Assert-SlowBootMarkers([string]$SdRoot) {
    foreach ($relative in @(
            'INavi\traffictest.txt', 'INavi\routetest.txt',
            'mapdata\soap.bin', 'mapdata\searchdb\oildata.dat')) {
        $marker = Join-Path $SdRoot $relative
        if (Test-Path -LiteralPath $marker) {
            throw "Slow-boot marker appeared; investigate without deleting it: $marker"
        }
    }
}

function Get-InaviWritableMountRoots([string]$Path) {
    $sections = [System.Collections.Generic.List[object]]::new()
    $section = $null
    foreach ($line in [System.IO.File]::ReadLines($Path)) {
        if ($line -match '^\s*\[root\]\s*(?:#.*)?$') {
            if ($null -ne $section) { $sections.Add($section) }
            $section = @{ kind = 'root' }
        } elseif ($line -match '^\s*\[\[mounts\]\]\s*(?:#.*)?$') {
            if ($null -ne $section) { $sections.Add($section) }
            $section = @{ kind = 'mount' }
        } elseif ($null -ne $section -and $line -match '^\s*\[') {
            $sections.Add($section); $section = $null
        } elseif ($null -ne $section -and $line -match '^\s*(host_root|name|writable)\s*=\s*(.+?)\s*(?:#.*)?$') {
            $key = $Matches[1]; $value = $Matches[2]
            if ($section.ContainsKey($key)) { throw "Duplicate mount field $key in $Path" }
            if ($key -eq 'writable') {
                if ($value -notin @('true', 'false')) { throw "Invalid writable value in $Path" }
                $section[$key] = ($value -eq 'true')
            } else {
                if ($value -cnotmatch '^"(?:[^"\\]|\\.)*"$') { throw "Unsupported TOML string in $Path" }
                $section[$key] = [string]($value | ConvertFrom-Json -ErrorAction Stop)
            }
        }
    }
    if ($null -ne $section) { $sections.Add($section) }
    $root = @($sections | Where-Object { $_.kind -eq 'root' })
    $writable = @($sections | Where-Object { $_.kind -eq 'mount' -and $_.writable -eq $true })
    if ($root.Count -ne 1 -or $writable.Count -ne 2 -or
        [string]$writable[0].name -cne 'sdmmc' -or
        [string]$writable[1].name -cne 'resident_flash' -or
        [string]::IsNullOrWhiteSpace($root[0].host_root) -or
        [string]::IsNullOrWhiteSpace($writable[0].host_root) -or
        [string]::IsNullOrWhiteSpace($writable[1].host_root)) {
        throw 'Mount configuration is not the expected three writable roots.'
    }
    return @([string]$writable[0].host_root,
        [string]$writable[1].host_root, [string]$root[0].host_root)
}

function Get-InaviExecutableRsds([string]$Path) {
    $stream = [System.IO.File]::Open($Path, [System.IO.FileMode]::Open,
        [System.IO.FileAccess]::Read, [System.IO.FileShare]::Read)
    $reader = $null
    try {
        $reader = [System.Reflection.PortableExecutable.PEReader]::new($stream)
        $matches = @($reader.ReadDebugDirectory() | Where-Object {
            $_.Type -eq [System.Reflection.PortableExecutable.DebugDirectoryEntryType]::CodeView
        })
        if ($matches.Count -ne 1) { throw "Expected exactly one executable CodeView record: $Path" }
        $record = $reader.ReadCodeViewDebugDirectoryData($matches[0])
        if ($record.Age -lt 1) { throw "Invalid executable CodeView age: $Path" }
        return '{0}:{1}' -f $record.Guid.ToString('D').ToUpperInvariant(), $record.Age
    } finally {
        if ($null -ne $reader) { $reader.Dispose() }
        $stream.Dispose()
    }
}

function Get-InaviPdbRsds([string]$Path) {
    # MSF 7.0 PDB stream 1 contains version, signature, age and the RSDS GUID.
    # Parse block bounds rather than trusting the filename or an adjacent PDB.
    $bytes = [System.IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -lt 56 -or
        -not [System.Text.Encoding]::ASCII.GetString($bytes, 0, 24).StartsWith(
            'Microsoft C/C++ MSF 7.00', [StringComparison]::Ordinal)) {
        throw "Unsupported PDB signature: $Path"
    }
    [int64]$blockSize = [BitConverter]::ToUInt32($bytes, 32)
    [int64]$directoryBytes = [BitConverter]::ToUInt32($bytes, 44)
    [int64]$blockMap = [BitConverter]::ToUInt32($bytes, 52)
    if ($blockSize -lt 512 -or $blockSize -gt 65536 -or
        ($blockSize -band ($blockSize - 1)) -ne 0 -or
        $directoryBytes -lt 12 -or $directoryBytes -gt 16MB) {
        throw "Invalid PDB superblock: $Path"
    }
    [int64]$directoryBlockCount = [Math]::Ceiling($directoryBytes / $blockSize)
    if ($directoryBlockCount * 4 -gt $blockSize -or
        (($blockMap + 1) * $blockSize) -gt $bytes.LongLength) {
        throw "Unsupported PDB directory block map: $Path"
    }
    $directory = [byte[]]::new([int]$directoryBytes)
    for ($i = 0; $i -lt $directoryBlockCount; $i++) {
        [int64]$block = [BitConverter]::ToUInt32($bytes, [int]($blockMap * $blockSize + 4 * $i))
        [int64]$source = $block * $blockSize
        [int]$count = [int][Math]::Min($blockSize, $directoryBytes - $i * $blockSize)
        if ($source + $count -gt $bytes.LongLength) { throw "PDB directory block is out of range: $Path" }
        [Array]::Copy($bytes, $source, $directory, $i * $blockSize, $count)
    }
    [int64]$streamCount = [BitConverter]::ToUInt32($directory, 0)
    if ($streamCount -lt 2 -or $streamCount -gt 65536 -or
        4 + 4 * $streamCount -gt $directoryBytes) {
        throw "PDB stream table is invalid: $Path"
    }
    [int64]$offset = 4 + 4 * $streamCount
    for ($streamIndex = 0; $streamIndex -le 1; $streamIndex++) {
        [int64]$size = [BitConverter]::ToUInt32($directory, [int](4 + 4 * $streamIndex))
        if ($size -eq 0xFFFFFFFF) { throw "PDB identity stream is absent: $Path" }
        [int64]$pageCount = [Math]::Ceiling($size / $blockSize)
        if ($offset + 4 * $pageCount -gt $directoryBytes) {
            throw "PDB stream page table is out of range: $Path"
        }
        if ($streamIndex -eq 1) {
            if ($size -lt 28) { throw "PDB identity stream is too short: $Path" }
            $identity = [byte[]]::new(28)
            for ($page = 0; $page -lt $pageCount -and $page * $blockSize -lt 28; $page++) {
                [int64]$block = [BitConverter]::ToUInt32($directory, [int]($offset + 4 * $page))
                [int64]$source = $block * $blockSize
                [int]$count = [int][Math]::Min($blockSize, 28 - $page * $blockSize)
                if ($source + $count -gt $bytes.LongLength) { throw "PDB identity page is out of range: $Path" }
                [Array]::Copy($bytes, $source, $identity, $page * $blockSize, $count)
            }
            [uint32]$age = [BitConverter]::ToUInt32($identity, 8)
            if ($age -lt 1) { throw "Invalid PDB age: $Path" }
            $guidBytes = [byte[]]::new(16)
            [Array]::Copy($identity, 12, $guidBytes, 0, 16)
            return '{0}:{1}' -f ([Guid]::new($guidBytes)).ToString('D').ToUpperInvariant(), $age
        }
        $offset += 4 * $pageCount
    }
    throw "PDB identity stream was not found: $Path"
}

function Assert-InaviBackupComplete([hashtable]$Manifest, [hashtable]$BackupManifest) {
    if ($BackupManifest.complete -isnot [bool] -or -not $BackupManifest.complete -or
        $BackupManifest.completed -isnot [array] -or $BackupManifest.completed.Count -ne 3) {
        throw 'The three-mount backup manifest is not complete.'
    }
    $expectedNames = @('SD', 'ResidentFlash', 'EmulatorRoot')
    for ($i = 0; $i -lt 3; $i++) {
        $row = $BackupManifest.completed[$i]
        if ($row -isnot [hashtable] -or
            [string]$row.name -cne $expectedNames[$i] -or
            $row.postBackupInventoryVerified -isnot [bool] -or
            -not $row.postBackupInventoryVerified -or
            [string]::IsNullOrWhiteSpace([string]$row.postBackupSourceManifest) -or
            [string]::IsNullOrWhiteSpace([string]$row.postBackupSourceManifestSha256)) {
            throw "Incomplete post-backup inventory for $($expectedNames[$i])."
        }
        Test-ExpectedHash ([string]$row.postBackupSourceManifest) ([string]$row.postBackupSourceManifestSha256)
    }
}

function Write-MountedInventory([object[]]$Backups, [string]$OutputPath) {
    # Enumerate only the three named mount roots, never following reparse points.
    $entryCount = 0
    foreach ($item in $Backups) {
        $root = [string]$item.Source
        $pending = [System.Collections.Generic.Stack[string]]::new()
        $pending.Push($root)
        while ($pending.Count -ne 0) {
            $directory = $pending.Pop()
            $children = @(Get-ChildItem -LiteralPath $directory -Force -ErrorAction Stop | Sort-Object FullName)
            foreach ($child in $children) {
                $entryCount++
                if ($entryCount -gt 100000) { throw 'Mounted inventory exceeded its 100,000-entry safety bound.' }
                $relative = [System.IO.Path]::GetRelativePath($root, $child.FullName)
                $isLink = [bool]($child.Attributes -band [System.IO.FileAttributes]::ReparsePoint)
                $kind = if ($isLink) { 'reparse-point' } elseif ($child.PSIsContainer) { 'directory' } else { 'file' }
                $length = if ($kind -eq 'file') { [long]$child.Length } else { $null }
                $sha = if ($kind -eq 'file') { Get-Sha256 $child.FullName } else { $null }
                Add-JsonLine $OutputPath @{
                    root = $root; relative = $relative; kind = $kind
                    length = $length; sha256 = $sha
                    lastWriteUtc = $child.LastWriteTimeUtc.ToString('O')
                }
                if ($kind -eq 'directory') { $pending.Push($child.FullName) }
            }
        }
    }
    return $entryCount
}

function Compare-MountedInventories([string]$BeforePath, [string]$AfterPath, [string]$ChangesPath) {
    [System.IO.File]::WriteAllText($ChangesPath, '', $utf8)
    $before = @{}
    foreach ($line in [System.IO.File]::ReadLines($BeforePath)) {
        $row = $line | ConvertFrom-Json
        $before["$($row.root)|$($row.relative)"] = $row
    }
    $changes = 0
    foreach ($line in [System.IO.File]::ReadLines($AfterPath)) {
        $row = $line | ConvertFrom-Json
        $key = "$($row.root)|$($row.relative)"
        if (-not $before.ContainsKey($key)) {
            Add-JsonLine $ChangesPath @{ change = 'added'; root = $row.root; relative = $row.relative; after = $row }
            $changes++
            continue
        }
        $old = $before[$key]
        [void]$before.Remove($key)
        if ($old.kind -ne $row.kind -or $old.length -ne $row.length -or $old.sha256 -ne $row.sha256 -or $old.lastWriteUtc -ne $row.lastWriteUtc) {
            Add-JsonLine $ChangesPath @{ change = 'changed'; root = $row.root; relative = $row.relative; before = $old; after = $row }
            $changes++
        }
    }
    foreach ($key in $before.Keys) {
        $old = $before[$key]
        Add-JsonLine $ChangesPath @{ change = 'removed'; root = $old.root; relative = $old.relative; before = $old }
        $changes++
    }
    return $changes
}

function Assert-WorkerDirectory([string]$Directory) {
    $prefix = 'D:\GitHub\Wince_emulation_v3\evidence\e37-live-'
    $resolved = [System.IO.Path]::GetFullPath($Directory)
    if (-not $resolved.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Worker evidence directory is outside E-37: $resolved"
    }
    if (-not (Test-Path -LiteralPath $resolved -PathType Container)) {
        throw "Worker evidence directory is missing: $resolved"
    }
}

function Assert-RemoteEndpointAvailable {
    # A stale emulator must not supply the observer's t0 or receive these POSTs.
    $other = @(Get-Process -Name 'wince_emulation_v3', 'wince_emulation_v3-op4', 'wince_emulation_v3-e37' -ErrorAction SilentlyContinue)
    if ($other.Count -ne 0) {
        throw "Another wince_emulation_v3 process is running (PID(s): $($other.Id -join ', '))."
    }
    $listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Parse('192.168.0.39'), 8765)
    try { $listener.Start() } finally { $listener.Stop() }
}

function New-HttpClient {
    # These are same-host requests to the explicitly bound emulator endpoint.
    $handler = [System.Net.Http.HttpClientHandler]::new()
    $handler.UseProxy = $false
    $client = [System.Net.Http.HttpClient]::new($handler, $true)
    $client.Timeout = [System.TimeSpan]::FromSeconds(3)
    $client.MaxResponseContentBufferSize = 8MB
    return $client
}

function Invoke-GetCapture(
    [System.Net.Http.HttpClient]$Client,
    [string]$Url,
    [string]$Name,
    [string]$LogPath,
    [string]$CaptureDirectory,
    [int]$Sequence,
    [bool]$SaveBody
) {
    $startedUtc = [DateTime]::UtcNow.ToString('O')
    $statusCode = $null
    $body = [byte[]]@()
    $errorText = $null
    try {
        $response = $Client.GetAsync($Url).GetAwaiter().GetResult()
        try {
            $statusCode = [int]$response.StatusCode
            $body = $response.Content.ReadAsByteArrayAsync().GetAwaiter().GetResult()
        } finally {
            $response.Dispose()
        }
    } catch {
        $errorText = $_.Exception.Message
    }
    $receivedTicks = [System.Diagnostics.Stopwatch]::GetTimestamp()
    $receivedUtc = [DateTime]::UtcNow.ToString('O')
    $hash = if ($null -ne $statusCode) { Get-ByteSha256 $body } else { $null }
    $saved = $null
    if ($SaveBody -and $statusCode -eq 200 -and $body.Length -le 1MB) {
        $extension = if ($Name -eq 'frame') { 'jpg' } else { 'txt' }
        $saved = Join-Path $CaptureDirectory ('{0}-{1:D4}.{2}' -f $Name, $Sequence, $extension)
        [System.IO.File]::WriteAllBytes($saved, $body)
    }
    Add-JsonLine $LogPath @{
        name = $Name; url = $Url; startedUtc = $startedUtc
        receivedUtc = $receivedUtc; receivedTicks = $receivedTicks
        status = $statusCode; bytes = $body.Length; sha256 = $hash
        savedPath = $saved; error = $errorText
    }
    return @{ Status = $statusCode; Body = $body; ReceivedTicks = $receivedTicks; ReceivedUtc = $receivedUtc; Hash = $hash }
}

function Invoke-SummaryCapture(
    [System.Net.Http.HttpClient]$Client,
    [string]$Url,
    [string]$LogPath,
    [string]$CaptureDirectory,
    [int]$Sequence,
    [int]$SavedBodies,
    [AllowNull()][object]$LastPublicationSequence
) {
    $startedTicks = [System.Diagnostics.Stopwatch]::GetTimestamp()
    $startedUtc = [DateTime]::UtcNow.ToString('O')
    $statusCode = $null
    $declaredBytes = $null
    $bytesRead = 0
    $body = [byte[]]@()
    $bodyComplete = $false
    $sampleOutcome = 'transport-error'
    $errorText = $null
    $publication = $null
    $request = $null; $response = $null; $stream = $null
    $deadline = [System.Threading.CancellationTokenSource]::new([TimeSpan]::FromSeconds(3))
    try {
        $request = [System.Net.Http.HttpRequestMessage]::new([System.Net.Http.HttpMethod]::Get, $Url)
        $response = $Client.SendAsync(
            $request, [System.Net.Http.HttpCompletionOption]::ResponseHeadersRead, $deadline.Token
        ).GetAwaiter().GetResult()
        $statusCode = [int]$response.StatusCode
        if ($null -ne $response.Content) {
            $declaredBytes = $response.Content.Headers.ContentLength
            if ($null -ne $declaredBytes -and $declaredBytes -gt 65536) {
                $sampleOutcome = 'oversize'
                $errorText = "Declared summary body exceeds 65536 bytes: $declaredBytes"
            } else {
                $stream = $response.Content.ReadAsStreamAsync($deadline.Token).GetAwaiter().GetResult()
                $bounded = Read-InaviBoundedSummaryStream -Stream $stream -MaxBytes 65536 -CancellationToken $deadline.Token
                $bytesRead = $bounded.bytesRead
                if ($bounded.oversize) {
                    $sampleOutcome = 'oversize'
                    $errorText = 'Summary body exceeds 65536 bytes.'
                } else {
                    $body = $bounded.bytes
                    $bodyComplete = $true
                }
            }
        } else {
            $bodyComplete = $true
        }
        if ($bodyComplete) {
            if ($statusCode -ne 200) {
                $sampleOutcome = 'http-error'
                $errorText = "Summary GET returned HTTP $statusCode."
            } else {
                try {
                    $publication = ConvertFrom-InaviSummaryBody -Bytes $body
                    $sampleOutcome = 'ok'
                } catch {
                    $sampleOutcome = 'parse-error'
                    $errorText = $_.Exception.Message
                }
            }
        }
    } catch {
        $errorText = $_.Exception.Message
        $sampleOutcome = 'transport-error'
    } finally {
        if ($null -ne $stream) { $stream.Dispose() }
        if ($null -ne $response) { $response.Dispose() }
        if ($null -ne $request) { $request.Dispose() }
        $deadline.Dispose()
    }
    $receivedTicks = [System.Diagnostics.Stopwatch]::GetTimestamp()
    $receivedUtc = [DateTime]::UtcNow.ToString('O')
    $hash = if ($bodyComplete) { Get-ByteSha256 $body } else { $null }
    $saved = $null
    if ($bodyComplete -and $body.Length -gt 0 -and $SavedBodies -lt 256) {
        $extension = if ($sampleOutcome -eq 'ok') { 'txt' } else { 'bin' }
        $saved = Join-Path $CaptureDirectory ('summary-{0:D4}.{1}' -f $Sequence, $extension)
        [System.IO.File]::WriteAllBytes($saved, $body)
    }
    $freshness = 'unknown'
    if ($null -ne $publication) {
        $freshness = if ($null -eq $LastPublicationSequence) {
            'initial'
        } elseif ($publication.publicationSequence -eq [UInt64]$LastPublicationSequence) {
            'repeated'
        } elseif ($publication.publicationSequence -gt [UInt64]$LastPublicationSequence) {
            'advanced'
        } else {
            'regressed'
        }
    }
    Add-JsonLine $LogPath @{
        name = 'summary'; url = $Url; sequence = $Sequence
        startedUtc = $startedUtc; startedTicks = $startedTicks
        receivedUtc = $receivedUtc; receivedTicks = $receivedTicks
        status = $statusCode; declaredBytes = $declaredBytes
        bytesRead = $bytesRead; bytes = if ($bodyComplete) { $body.Length } else { $null }
        bodyComplete = $bodyComplete; sha256 = $hash; savedPath = $saved
        outcome = $sampleOutcome; error = $errorText; freshness = $freshness
        publicationSequence = if ($null -ne $publication) { $publication.publicationSequence } else { $null }
        publicationHostMonotonicNs = if ($null -ne $publication) { $publication.publicationHostMonotonicNs } else { $null }
        publicationUtcUnixMs = if ($null -ne $publication) { $publication.publicationUtcUnixMs } else { $null }
        liveGuestStablePageBudgetUsedBytes = if ($null -ne $publication) { $publication.liveGuestStablePageBudgetUsedBytes } else { $null }
    }
    return [pscustomobject]@{
        outcome = $sampleOutcome
        savedBody = ($null -ne $saved)
        publicationSequence = if ($null -ne $publication) { $publication.publicationSequence } else { $null }
    }
}

function Invoke-Observer([string]$Directory) {
    Assert-WorkerDirectory $Directory
    $capture = Join-Path $Directory 'observer-captures'
    [void][System.IO.Directory]::CreateDirectory($capture)
    $log = Join-Path $Directory 'observer-get.jsonl'
    $readyPath = Join-Path $Directory 'observer-worker-ready.json'
    $inputReadyPath = Join-Path $Directory 'input-ready.json'
    $stopRequestPath = Join-Path $Directory 'observer-stop-request.json'
    $client = New-HttpClient
    $startedTicks = [System.Diagnostics.Stopwatch]::GetTimestamp()
    Write-AtomicJsonFile $readyPath @{ startedUtc = [DateTime]::UtcNow.ToString('O'); startedTicks = $startedTicks; horizonSeconds = 420 }
    $base = 'http://192.168.0.39:8765'
    $nextSlot = 0
    $frameHash = $null
    $savedFrames = 0
    $summarySamples = 0
    $summaryBodiesSaved = 0
    $lastSummarySequence = $null
    $stoppedEarly = $false
    try {
        while (([System.Diagnostics.Stopwatch]::GetTimestamp() - $startedTicks) -lt (420L * $frequency)) {
            if (Test-Path -LiteralPath $stopRequestPath) { $stoppedEarly = $true; break }
            $elapsedMs = 1000.0 * ([System.Diagnostics.Stopwatch]::GetTimestamp() - $startedTicks) / $frequency
            $slot = [int][Math]::Floor($elapsedMs / 2000.0)
            if ($slot -lt $nextSlot) {
                Start-Sleep -Milliseconds 100
                continue
            }
            $nextSlot = $slot + 1
            $status = Invoke-GetCapture $client "$base/api/v1/status" 'status' $log $capture $slot $false
            if ($summarySamples -lt 256) {
                $summary = Invoke-SummaryCapture $client "$base/api/v1/debug/summary.txt" $log $capture $slot $summaryBodiesSaved $lastSummarySequence
                $summarySamples++
                if ($summary.savedBody) { $summaryBodiesSaved++ }
                if ($summary.outcome -eq 'ok') { $lastSummarySequence = $summary.publicationSequence }
            }
            if ($status.Status -eq 200 -and (Test-Path -LiteralPath (Join-Path $Directory 'process.json')) -and -not (Test-Path -LiteralPath $inputReadyPath)) {
                try {
                    $parsed = [System.Text.Encoding]::UTF8.GetString($status.Body) | ConvertFrom-Json
                    if ($parsed.running -eq $true -and $parsed.gpsTarget -eq 'COM7:') {
                        Write-AtomicJsonFile $inputReadyPath @{
                            receivedUtc = $status.ReceivedUtc; receivedTicks = $status.ReceivedTicks
                            statusSha256 = $status.Hash; gpsTarget = 'COM7:'
                        }
                    }
                } catch {
                    Add-JsonLine $log @{ name = 'status-parse-error'; utc = [DateTime]::UtcNow.ToString('O'); error = $_.Exception.Message }
                }
            }
            $frame = Invoke-GetCapture $client "$base/api/v1/frame.jpg" 'frame' $log $capture $slot $false
            if ($frame.Status -eq 200 -and $frame.Hash -ne $frameHash) {
                $frameHash = $frame.Hash
                if ($savedFrames -lt 256 -and $frame.Body.Length -le 1MB) {
                    $savedFrames++
                    $path = Join-Path $capture ('changed-frame-{0:D4}.jpg' -f $savedFrames)
                    [System.IO.File]::WriteAllBytes($path, $frame.Body)
                    Add-JsonLine $log @{ name = 'changed-frame-saved'; utc = [DateTime]::UtcNow.ToString('O'); sha256 = $frame.Hash; savedPath = $path }
                }
            }
            # The retained file ring can turn over; sample COM7: reads at the
            # same two-second cadence as the frame/status bracket.
            [void](Invoke-GetCapture $client "$base/api/v1/debug/files-full.txt" 'files-full' $log $capture $slot $true)
            [void](Invoke-GetCapture $client "$base/api/v1/debug/remote-input.txt" 'remote-input' $log $capture $slot $true)
            if ($slot % 5 -eq 0) {
                foreach ($name in @('windows', 'messages', 'active')) {
                    [void](Invoke-GetCapture $client "$base/api/v1/debug/$name.txt" $name $log $capture $slot $true)
                }
            }
        }
    } finally {
        $client.Dispose()
        Write-JsonFile (Join-Path $Directory 'observer-result.json') @{
            startedTicks = $startedTicks; endedUtc = [DateTime]::UtcNow.ToString('O')
            completedHorizon = (([System.Diagnostics.Stopwatch]::GetTimestamp() - $startedTicks) -ge (420L * $frequency))
            stoppedOnEarlyAppEnd = $stoppedEarly
            changedFramesSaved = $savedFrames
            summarySamples = $summarySamples; summaryBodiesSaved = $summaryBodiesSaved
        }
    }
}

function Get-MatchingProcess([int]$TargetProcessId, [long]$StartUtcTicks) {
    try {
        $candidate = Get-Process -Id $TargetProcessId -ErrorAction Stop
    } catch {
        if ($_.CategoryInfo.Category -eq [System.Management.Automation.ErrorCategory]::ObjectNotFound) { return $null }
        throw
    }
    if ($candidate.StartTime.ToUniversalTime().Ticks -ne $StartUtcTicks) { return $null }
    return $candidate
}

function Stop-ExactApp([int]$TargetProcessId, [long]$StartUtcTicks) {
    $candidate = Get-MatchingProcess $TargetProcessId $StartUtcTicks
    if ($null -eq $candidate) { return }
    $candidate.Kill()
    if (-not $candidate.WaitForExit(10000)) {
        throw "Exact emulator PID $TargetProcessId did not terminate within 10 seconds."
    }
}

function Invoke-Monitor([string]$Directory, [int]$LimitMiB) {
    Assert-WorkerDirectory $Directory
    $requestPath = Join-Path $Directory 'launch-request.json'
    $descriptorPath = Join-Path $Directory 'process.json'
    $readyPath = Join-Path $Directory 'monitor-worker-ready.json'
    $abortPath = Join-Path $Directory 'abort-request.json'
    $samples = Join-Path $Directory 'memory.jsonl'
    $resultPath = Join-Path $Directory 'monitor-result.json'
    $stdoutPath = Join-Path $Directory 'app.stdout.raw.txt'
    $stderrPath = Join-Path $Directory 'app.stderr.raw.txt'
    $ppmPath = Join-Path $Directory 'terminal-frame.ppm'
    $self = Get-Process -Id $PID -ErrorAction Stop
    Write-AtomicJsonFile $readyPath @{
        readyUtc = [DateTime]::UtcNow.ToString('O'); monitorPid = $PID
        monitorStartUtcTicks = $self.StartTime.ToUniversalTime().Ticks
        watchdogSeconds = 390; rssLimitMiB = $LimitMiB
    }
    $waitStarted = [System.Diagnostics.Stopwatch]::GetTimestamp()
    while (-not (Test-Path -LiteralPath $requestPath)) {
        if (([System.Diagnostics.Stopwatch]::GetTimestamp() - $waitStarted) -ge (30L * $frequency)) {
            throw 'No launch request arrived within 30 seconds of monitor readiness.'
        }
        Start-Sleep -Milliseconds 100
    }
    $request = Get-Content -LiteralPath $requestPath -Raw | ConvertFrom-Json
    if (-not (Test-Path -LiteralPath (Join-Path $Directory 'observer-worker-ready.json'))) {
        throw 'Observer was not ready at monitor-owned launch.'
    }
    if ((Get-Sha256 ([string]$request.executable)) -ne [string]$request.executableSha256) {
        throw 'Executable hash changed between build and monitor-owned launch.'
    }
    if ([string]$request.repository -ne 'D:\GitHub\Wince_emulation_v3') {
        throw 'Monitor launch request has the wrong checkout.'
    }
    $arguments = @($request.arguments | ForEach-Object { [string]$_ })
    $launchTicks = [System.Diagnostics.Stopwatch]::GetTimestamp()
    $app = $null
    $nextSample = $launchTicks
    $reason = 'launch-failure'
    $fired = $false
    $monitorError = $null
    $appExitCode = $null
    $terminationConfirmed = $false
    try {
        $app = Start-Process -FilePath ([string]$request.executable) -ArgumentList $arguments `
            -WorkingDirectory ([string]$request.repository) -WindowStyle Hidden -PassThru `
            -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath
        Write-AtomicJsonFile $descriptorPath @{
            processId = $app.Id; startUtcTicks = $app.StartTime.ToUniversalTime().Ticks
            launchTicks = $launchTicks; launchedUtc = [DateTime]::UtcNow.ToString('O')
            executableSha256 = [string]$request.executableSha256
        }
        $reason = 'normal-exit'
        while ($true) {
            $app.Refresh()
            if ($app.HasExited) { break }
            $nowTicks = [System.Diagnostics.Stopwatch]::GetTimestamp()
            if (Test-Path -LiteralPath $abortPath) {
                $reason = 'driver-abort-request'; $fired = $true
                $app.Kill()
                break
            }
            if ($nowTicks -ge $nextSample) {
                Add-JsonLine $samples @{
                    utc = [DateTime]::UtcNow.ToString('O'); elapsedSeconds = ($nowTicks - $launchTicks) / [double]$frequency
                    processId = $app.Id; workingSetBytes = $app.WorkingSet64
                    privateBytes = $app.PrivateMemorySize64; virtualBytes = $app.VirtualMemorySize64
                    cpuMilliseconds = $app.TotalProcessorTime.TotalMilliseconds
                }
                $nextSample = $nowTicks + (2L * $frequency)
                if ($app.WorkingSet64 -gt ([long]$LimitMiB * 1MB)) {
                    $reason = 'rss-limit'; $fired = $true
                    $app.Kill()
                    break
                }
                foreach ($streamPath in @($stdoutPath, $stderrPath)) {
                    if ((Test-Path -LiteralPath $streamPath -PathType Leaf) -and
                        (Get-Item -LiteralPath $streamPath).Length -gt 512MB) {
                        $reason = 'raw-log-limit'; $fired = $true
                        $app.Kill()
                        break
                    }
                }
                if ($fired) { break }
            }
            if ($nowTicks -ge ($launchTicks + (390L * $frequency))) {
                $reason = '390-second-watchdog'; $fired = $true
                $app.Kill()
                break
            }
            Start-Sleep -Milliseconds 200
        }
    } catch {
        $monitorError = $_.Exception.Message
        $reason = 'monitor-error'; $fired = $true
    } finally {
        if ($null -ne $app) {
            $app.Refresh()
            if (-not $app.HasExited) {
                try { $app.Kill() } catch { $monitorError = "kill failed: $($_.Exception.Message); $monitorError" }
            }
            $terminationConfirmed = $app.WaitForExit(10000)
            if ($terminationConfirmed) { $appExitCode = $app.ExitCode }
        }
        $endedTicks = [System.Diagnostics.Stopwatch]::GetTimestamp()
        Write-JsonFile $resultPath @{
            endedUtc = [DateTime]::UtcNow.ToString('O'); endedTicks = $endedTicks
            appElapsedSeconds = ($endedTicks - $launchTicks) / [double]$frequency
            processId = if ($null -ne $app) { $app.Id } else { $null }
            fired = $fired; reason = $reason; rssLimitMiB = $LimitMiB
            appExitCode = $appExitCode; terminationConfirmed = $terminationConfirmed
            error = $monitorError; stdout = $stdoutPath; stderr = $stderrPath; terminalPpm = $ppmPath
        }
    }
    if ($null -ne $monitorError -or -not $terminationConfirmed) { throw "Monitor failed to confirm app termination: $monitorError" }
}

function Get-NmeaLine([string]$Body) {
    [byte]$checksum = 0
    foreach ($octet in [System.Text.Encoding]::ASCII.GetBytes($Body)) { $checksum = $checksum -bxor $octet }
    return ('$' + ('{0}*{1:X2}' -f $Body, $checksum))
}

function Get-RawNmeaFixture([DateTime]$Utc) {
    $clock = $Utc.ToString('HHmmss.000', [Globalization.CultureInfo]::InvariantCulture)
    $date = $Utc.ToString('ddMMyy', [Globalization.CultureInfo]::InvariantCulture)
    return @(
        (Get-NmeaLine "GPRMC,$clock,A,3730.0000,N,12715.0000,E,0.0,0.0,$date,,,A"),
        (Get-NmeaLine "GPGGA,$clock,3730.0000,N,12715.0000,E,1,08,0.9,42.0,M,19.5,M,,"),
        (Get-NmeaLine 'GPVTG,0.0,T,,M,0.0,N,0.0,K,A')
    )
}

function Invoke-InputPost([System.Net.Http.HttpClient]$Client, [string]$Url, [object]$Payload, [string]$Log, [string]$Kind, [int]$Index, [long]$TargetTicks, [long]$T0Ticks) {
    $body = $Payload | ConvertTo-Json -Compress -Depth 8
    $bodyBytes = [System.Text.Encoding]::UTF8.GetBytes($body)
    $startedTicks = [System.Diagnostics.Stopwatch]::GetTimestamp()
    $startedUtc = [DateTime]::UtcNow.ToString('O')
    $status = $null; $responseBody = [byte[]]@(); $errorText = $null
    try {
        $content = [System.Net.Http.StringContent]::new($body, [System.Text.Encoding]::UTF8, 'application/json')
        try {
            $response = $Client.PostAsync($Url, $content).GetAwaiter().GetResult()
            try {
                $status = [int]$response.StatusCode
                $responseBody = $response.Content.ReadAsByteArrayAsync().GetAwaiter().GetResult()
            } finally { $response.Dispose() }
        } finally { $content.Dispose() }
    } catch { $errorText = $_.Exception.Message }
    Add-JsonLine $Log @{
        kind = $Kind; index = $Index; url = $Url
        scheduledOffsetSeconds = ($TargetTicks - $T0Ticks) / [double]$frequency
        latenessMilliseconds = 1000.0 * ($startedTicks - $TargetTicks) / $frequency
        startedUtc = $startedUtc; endedUtc = [DateTime]::UtcNow.ToString('O')
        status = $status; payload = $Payload; payloadSha256 = (Get-ByteSha256 $bodyBytes)
        responseText = [System.Text.Encoding]::UTF8.GetString($responseBody)
        responseSha256 = if ($null -ne $status) { Get-ByteSha256 $responseBody } else { $null }
        error = $errorText; guestConsumed = $null
    }
}

if ($ObserverWorker) { Invoke-Observer $RunDirectory; return }
if ($MonitorWorker) { Invoke-Monitor $RunDirectory $MaxWorkingSetMiB; return }

$repository = Split-Path -Parent $PSScriptRoot
if ($repository -ne 'D:\GitHub\Wince_emulation_v3') { throw "Unexpected checkout: $repository" }
$backups = Test-OriginalMountPreflight $repository
$experiments = @(Get-ChildItem Env:WINCE_EMU_* -ErrorAction SilentlyContinue)
if ($experiments.Count -gt 0) { throw "Unset WINCE_EMU_* experiment variables before a matched run: $($experiments.Name -join ', ')" }
# Capture the fresh-owner slice cost and guest MIPS for this one-run comparison.
$env:RUST_LOG = 'warn,ce.slice_profile=debug'

$tag = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ')
$runDirectory = Join-Path $repository "evidence\e37-live-$tag"
if (Test-Path -LiteralPath $runDirectory) { throw "Evidence path already exists: $runDirectory" }
[void][System.IO.Directory]::CreateDirectory($runDirectory)
$previousDirectory = Get-Location
$observerJob = $null; $monitorProcess = $null; $appDescriptor = $null; $client = $null
$endedByScript = $null; $driveError = $null; $exitCode = $null; $exitStatus = 1
try {
    Set-Location -LiteralPath $repository
    $exe = Join-Path $repository 'evidence\op4-fresh-view-readback-20261014\wince_emulation_v3-e37.exe'
    $pdb = Join-Path $repository 'evidence\op4-fresh-view-readback-20261014\wince_emulation_v3-e37.pdb'
    Test-ExpectedHash $exe '7A55C049D2BBD0110D30D587BBB1877BA80C50CEE1D33E453434C0366A871A1A'
    Test-ExpectedHash $pdb '959D508F8694E3DD232281B79A7A46610EA87BDFB08542D916ABDF36ED06BEB4'
    $manifest = @{
        runTag = $tag; createdUtc = [DateTime]::UtcNow.ToString('O'); repository = $repository
        executable = $exe; executableSha256 = Get-Sha256 $exe
        pdb = $pdb; pdbSha256 = Get-Sha256 $pdb
        executableProvenanceReceipt = Join-Path $repository 'evidence\op4-fresh-view-readback-20261014\validation.receipt.md'
        backupProvenanceManifest = Join-Path $repository 'evidence\pre-live-mount-backup-20261014T050927Z\manifest.json'
        approvalQuestionId = 'call_93cb71e3f4dc432c84262f0a340f3091'
        approvalAnswer = 'Yes—back up and run once'
        earlierConsumedApprovalQuestionId = 'call_bf29d00a922d453992feac6a62e5e56e'
        app = 'D:\INAVI_Emulator\INAVI\INavi\iNavi.exe'; appSha256 = Get-Sha256 'D:\INAVI_Emulator\INAVI\INavi\iNavi.exe'
        scriptSha256 = Get-Sha256 $PSCommandPath
        registrySha256 = Get-Sha256 (Join-Path $repository 'registry.reg')
        devicesSha256 = Get-Sha256 (Join-Path $repository 'serial_devices.json')
        mountsSha256 = Get-Sha256 (Join-Path $repository 'mounts.toml')
        backups = $backups; originalWritableMounts = $true
        logFilter = $env:RUST_LOG
        runSeconds = 330; instructionLimit = 1000000; remoteUrl = 'http://192.168.0.39:8765'
        observerSeconds = 420; watchdogSeconds = 390; memorySampleSeconds = 2
        nmeaAssumption = 'NEW, not matched: 16 1-Hz RMC/GGA/VTG bursts at 37.5N 127.25E, altitude 42m, zero speed, from input-ready+93s'
        tapAssumption = 'input-ready+153s; x=400,y=240'
    }
    Write-JsonFile (Join-Path $runDirectory 'manifest.json') $manifest
    $beforeCount = Write-MountedInventory $backups (Join-Path $runDirectory 'mounted-before.jsonl')
    Assert-RemoteEndpointAvailable
    # The observer is GET-only. The detached monitor, not this driver, owns
    # the app launch and enforces the 390-second/2-GiB bounds if this driver dies.
    $observerJob = Start-Job -Name "e37-observer-$tag" -ScriptBlock {
        param($scriptPath, $directory)
        & $scriptPath -ObserverWorker -RunDirectory $directory
    } -ArgumentList $PSCommandPath, $runDirectory
    $monitorProcess = Start-Process -FilePath (Join-Path $PSHOME 'pwsh.exe') `
        -ArgumentList @('-NoProfile', '-File', $PSCommandPath, '-MonitorWorker', '-RunDirectory', $runDirectory, '-MaxWorkingSetMiB', [string]$MaxWorkingSetMiB) `
        -WorkingDirectory $repository -WindowStyle Hidden -PassThru `
        -RedirectStandardOutput (Join-Path $runDirectory 'monitor-worker.stdout.raw.txt') `
        -RedirectStandardError (Join-Path $runDirectory 'monitor-worker.stderr.raw.txt')
    $jobWait = [System.Diagnostics.Stopwatch]::GetTimestamp()
    while (-not ((Test-Path -LiteralPath (Join-Path $runDirectory 'observer-worker-ready.json')) -and (Test-Path -LiteralPath (Join-Path $runDirectory 'monitor-worker-ready.json')))) {
        if ($observerJob.State -eq 'Failed' -or $monitorProcess.HasExited) { throw 'Observer or detached monitor failed before launch.' }
        if (([System.Diagnostics.Stopwatch]::GetTimestamp() - $jobWait) -ge (15L * $frequency)) { throw 'Observer/monitor did not become ready in 15 seconds.' }
        Start-Sleep -Milliseconds 100
    }
    $monitorReady = Get-Content -LiteralPath (Join-Path $runDirectory 'monitor-worker-ready.json') -Raw | ConvertFrom-Json
    if ([int]$monitorReady.monitorPid -ne $monitorProcess.Id -or
        [long]$monitorReady.monitorStartUtcTicks -ne $monitorProcess.StartTime.ToUniversalTime().Ticks) {
        throw 'Detached monitor readiness descriptor did not match the launched process.'
    }
    $stdout = Join-Path $runDirectory 'app.stdout.raw.txt'
    $stderr = Join-Path $runDirectory 'app.stderr.raw.txt'
    $ppm = Join-Path $runDirectory 'terminal-frame.ppm'
    $arguments = @(
        '--registry', 'registry.reg', '--devices', 'serial_devices.json', '--mount-config', 'mounts.toml',
        '--image', 'D:\INAVI_Emulator\INAVI\INavi\iNavi.exe',
        '--dll-search-dir', 'D:\INAVI_Emulator\DUMPPLZ\Windows',
        '--desktop', 'virtual', '--run-cpu', '--remote-server', '192.168.0.39:8765',
        '--cpu-wall-clock-limit-ms', '330000', '--cpu-instruction-limit', '1000000',
        '--framebuffer-dump', $ppm
    )
    # The full archive/config/hash preflight ran before setup. Recheck junctions
    # immediately before launch without consuming the observer's 420 s window
    # rehashing multi-GB ZIPs. This cannot prevent a concurrent later change.
    Assert-NoReparseMounts $backups
    Write-AtomicJsonFile (Join-Path $runDirectory 'launch-request.json') @{
        executable = $exe; executableSha256 = $manifest.executableSha256
        repository = $repository; arguments = $arguments
    }
    $launchWait = [System.Diagnostics.Stopwatch]::GetTimestamp()
    while (-not (Test-Path -LiteralPath (Join-Path $runDirectory 'process.json'))) {
        if ($monitorProcess.HasExited) { throw 'Detached monitor exited without an app process descriptor.' }
        if (([System.Diagnostics.Stopwatch]::GetTimestamp() - $launchWait) -ge (20L * $frequency)) {
            throw 'Detached monitor did not publish the app descriptor within 20 seconds.'
        }
        Start-Sleep -Milliseconds 100
    }
    $appDescriptor = Get-Content -LiteralPath (Join-Path $runDirectory 'process.json') -Raw | ConvertFrom-Json
    if ([string]$appDescriptor.executableSha256 -ne [string]$manifest.executableSha256) {
        throw 'Launched app descriptor does not match the built executable.'
    }
    $launchTicks = [long]$appDescriptor.launchTicks
    $client = New-HttpClient
    $inputLog = Join-Path $runDirectory 'input-post.jsonl'
    $ready = $null; $nextGps = 0; $lastGpsTicks = 0L; $tapSent = $false
    while ($null -ne (Get-MatchingProcess ([int]$appDescriptor.processId) ([long]$appDescriptor.startUtcTicks))) {
        if ($observerJob.State -eq 'Failed' -or $monitorProcess.HasExited) {
            $endedByScript = 'required-worker-failed'; break
        }
        if ($null -eq $ready -and (Test-Path -LiteralPath (Join-Path $runDirectory 'input-ready.json'))) {
            try { $ready = Get-Content -LiteralPath (Join-Path $runDirectory 'input-ready.json') -Raw | ConvertFrom-Json } catch { }
        }
        $nowTicks = [System.Diagnostics.Stopwatch]::GetTimestamp()
        if ($null -eq $ready -and ($nowTicks - $launchTicks) -ge (30L * $frequency)) {
            $endedByScript = 'no-configured-COM7-status-in-30-seconds'; break
        }
        if ($null -ne $ready) {
            $t0 = [long]$ready.receivedTicks
            if ($nextGps -lt 16) {
                $ideal = $t0 + ((93L + $nextGps) * $frequency)
                $due = if ($lastGpsTicks -ne 0) { [Math]::Max($ideal, $lastGpsTicks + $frequency) } else { $ideal }
                if ($nowTicks -ge $due) {
                    $sentUtc = [DateTime]::UtcNow
                    $payload = @{ sentences = @(Get-RawNmeaFixture $sentUtc) }
                    Invoke-InputPost $client 'http://192.168.0.39:8765/api/v1/sensors/nmea' $payload $inputLog 'raw-nmea' $nextGps $ideal $t0
                    $lastGpsTicks = [System.Diagnostics.Stopwatch]::GetTimestamp()
                    $nextGps++
                }
            }
            $tapDue = $t0 + (153L * $frequency)
            if (-not $tapSent -and $nowTicks -ge $tapDue) {
                Invoke-InputPost $client 'http://192.168.0.39:8765/api/v1/input/touch' @{ type = 'tap'; x = 400; y = 240 } $inputLog 'tap' 0 $tapDue $t0
                $tapSent = $true
            }
        }
        Start-Sleep -Milliseconds 100
    }
    if ($null -ne $endedByScript) {
        Write-AtomicJsonFile (Join-Path $runDirectory 'abort-request.json') @{
            utc = [DateTime]::UtcNow.ToString('O'); reason = $endedByScript
        }
        if ($monitorProcess.HasExited) {
            # A dead monitor cannot honor the abort request. The driver is
            # still alive here, so terminate only the verified app identity.
            Stop-ExactApp ([int]$appDescriptor.processId) ([long]$appDescriptor.startUtcTicks)
        }
    }
    if (-not $monitorProcess.WaitForExit(15000)) {
        # A live app is never left to an unresponsive monitor. Preserve exact
        # PID/start-time identity before the driver performs a fallback kill.
        Stop-ExactApp ([int]$appDescriptor.processId) ([long]$appDescriptor.startUtcTicks)
        $monitorProcess.Kill()
        if (-not $monitorProcess.WaitForExit(10000)) { throw 'Detached monitor did not terminate after fallback app kill.' }
        $endedByScript = 'detached-monitor-timeout-fallback-kill'
    }
    $monitor = if (Test-Path -LiteralPath (Join-Path $runDirectory 'monitor-result.json')) {
        Get-Content -LiteralPath (Join-Path $runDirectory 'monitor-result.json') -Raw | ConvertFrom-Json
    } else { $null }
    $appElapsedSeconds = if ($null -ne $monitor) {
        [double]$monitor.appElapsedSeconds
    } else {
        ([System.Diagnostics.Stopwatch]::GetTimestamp() - $launchTicks) / [double]$frequency
    }
    if ($null -ne $monitor) { $exitCode = $monitor.appExitCode }
    $earlyFailure = ($exitCode -ne 0 -or $null -ne $endedByScript -or $appElapsedSeconds -lt 300 -or
        $null -eq $monitor -or $monitor.fired -or -not $monitor.terminationConfirmed -or
        $null -eq $ready -or $nextGps -ne 16 -or -not $tapSent)
    if ($earlyFailure) {
        Write-AtomicJsonFile (Join-Path $runDirectory 'observer-stop-request.json') @{
            utc = [DateTime]::UtcNow.ToString('O'); reason = 'app-failed-or-ended-before-requested-horizon'
            appElapsedSeconds = $appElapsedSeconds; exitCode = $exitCode; scriptStop = $endedByScript
        }
    }
    $afterCount = $null; $mountChanges = $null; $inventoryError = $null
    try {
        $afterCount = Write-MountedInventory $backups (Join-Path $runDirectory 'mounted-after.jsonl')
        $mountChanges = Compare-MountedInventories (Join-Path $runDirectory 'mounted-before.jsonl') (Join-Path $runDirectory 'mounted-after.jsonl') (Join-Path $runDirectory 'mounted-changes.jsonl')
    } catch {
        $inventoryError = $_.Exception.Message
        Write-JsonFile (Join-Path $runDirectory 'mounted-inventory-error.json') @{ utc = [DateTime]::UtcNow.ToString('O'); error = $inventoryError }
    }
    # Only a normal full run keeps the observer to its fixed 420 s horizon.
    $observerReady = Get-Content -LiteralPath (Join-Path $runDirectory 'observer-worker-ready.json') -Raw | ConvertFrom-Json
    $observerDeadline = if ($earlyFailure) {
        [System.Diagnostics.Stopwatch]::GetTimestamp() + (10L * $frequency)
    } else {
        [long]$observerReady.startedTicks + (435L * $frequency)
    }
    while ($observerJob.State -in @('NotStarted', 'Running') -and [System.Diagnostics.Stopwatch]::GetTimestamp() -lt $observerDeadline) {
        [void](Wait-Job -Job $observerJob -Timeout 2)
    }
    if ($observerJob.State -in @('NotStarted', 'Running')) {
        Stop-Job -Job $observerJob
        if ($null -eq $endedByScript) {
            $endedByScript = if ($earlyFailure) { 'observer-forced-after-early-stop-request' } else { 'observer-forced-after-435-seconds' }
        }
    }
    # These are classifications of this run's stderr, never kill decisions
    # made from cached debug GET text.
    $guestCpuExceptionText = [bool](Select-String -LiteralPath $stderr -Pattern 'Unicorn run stopped on guest CPU exception' -Quiet)
    $zeroPcText = [bool](Select-String -LiteralPath $stderr -Pattern 'pc=0x00000000|invalid_pc=0x00000000' -Quiet)
    $ppmProduced = Test-Path -LiteralPath $ppm -PathType Leaf
    $observerResultPresent = Test-Path -LiteralPath (Join-Path $runDirectory 'observer-result.json') -PathType Leaf
    $observerResult = if ($observerResultPresent) {
        Get-Content -LiteralPath (Join-Path $runDirectory 'observer-result.json') -Raw | ConvertFrom-Json
    } else { $null }
    $earlyFailure = ($earlyFailure -or $guestCpuExceptionText -or $zeroPcText -or
        -not $ppmProduced -or -not $observerResultPresent -or
        $observerJob.State -ne 'Completed' -or -not $observerResult.completedHorizon -or
        $null -ne $inventoryError)
    Write-JsonFile (Join-Path $runDirectory 'result.json') @{
        endedUtc = [DateTime]::UtcNow.ToString('O'); processId = $appDescriptor.processId; exitCode = $exitCode
        appElapsedSeconds = $appElapsedSeconds; earlyFailure = $earlyFailure
        guestCpuExceptionTextInStderr = $guestCpuExceptionText; zeroPcTextInStderr = $zeroPcText
        scriptStop = $endedByScript; monitor = $monitor
        inputReady = $ready; nmeaPostsAttempted = $nextGps; tapAttempted = $tapSent
        stdoutSha256 = Get-Sha256 $stdout; stderrSha256 = Get-Sha256 $stderr
        terminalPpmProduced = $ppmProduced
        terminalPpmSha256 = if ($ppmProduced) { Get-Sha256 $ppm } else { $null }
        observerResultPresent = $observerResultPresent
        mountedBeforeEntries = $beforeCount; mountedAfterEntries = $afterCount
        mountedChangedEntries = $mountChanges; mountedInventoryError = $inventoryError
        observationAndAckDoNotProveGuestConsumption = $true
    }
    Write-Output "Retained run evidence: $runDirectory"
    Write-Output "App exit=$exitCode; monitor=$($monitor.reason); scriptStop=$endedByScript; NMEA attempts=$nextGps; tap attempted=$tapSent"
    $exitStatus = if ($earlyFailure) { 1 } else { 0 }
} catch {
    $driveError = $_.Exception.Message
    Write-JsonFile (Join-Path $runDirectory 'harness-error.json') @{ utc = [DateTime]::UtcNow.ToString('O'); error = $driveError }
    throw
} finally {
    if ($null -ne $client) { $client.Dispose() }
    if ($null -ne $appDescriptor) {
        try {
            if ($null -ne (Get-MatchingProcess ([int]$appDescriptor.processId) ([long]$appDescriptor.startUtcTicks))) {
                if (-not (Test-Path -LiteralPath (Join-Path $runDirectory 'abort-request.json'))) {
                    Write-AtomicJsonFile (Join-Path $runDirectory 'abort-request.json') @{
                        utc = [DateTime]::UtcNow.ToString('O'); reason = 'driver-finally-cleanup'
                    }
                }
                if ($null -ne $monitorProcess) { [void]$monitorProcess.WaitForExit(10000) }
                Stop-ExactApp ([int]$appDescriptor.processId) ([long]$appDescriptor.startUtcTicks)
            }
        } catch {
            $exitStatus = 1
            Write-JsonFile (Join-Path $runDirectory 'cleanup-error.json') @{
                utc = [DateTime]::UtcNow.ToString('O'); error = $_.Exception.Message
            }
        }
    }
    if ($null -ne $monitorProcess -and -not $monitorProcess.HasExited) {
        if ($null -ne $appDescriptor) {
            # The exact app identity has been checked/killed above; a live
            # monitor is now only an orphaned helper and may be retired.
            if ($null -eq (Get-MatchingProcess ([int]$appDescriptor.processId) ([long]$appDescriptor.startUtcTicks))) {
                $monitorProcess.Kill()
                [void]$monitorProcess.WaitForExit(10000)
            }
        } else {
            # A launch request can race with descriptor publication. Never
            # kill the sole independent watchdog without an app identity;
            # it will self-timeout prelaunch or stop the app at its bound.
            Write-JsonFile (Join-Path $runDirectory 'monitor-left-detached.json') @{
                utc = [DateTime]::UtcNow.ToString('O'); monitorPid = $monitorProcess.Id
                reason = 'driver-has-no-verified-app-descriptor'
            }
        }
    }
    if ($null -ne $observerJob) {
        if ($observerJob.State -in @('NotStarted', 'Running')) { Stop-Job -Job $observerJob }
        Receive-Job -Job $observerJob -Keep *> (Join-Path $runDirectory "$($observerJob.Name).worker.raw.txt")
        Remove-Job -Job $observerJob -Force
    }
    Set-Location $previousDirectory
}
exit $exitStatus
