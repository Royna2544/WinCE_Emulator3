param(
    [Parameter(Mandatory = $true)]
    [ValidateRange(1, 1000000)]
    [int]$Attempt,

    [Parameter(Mandatory = $true)]
    [string]$LogPath,

    [Parameter(Mandatory = $true)]
    [string[]]$CargoArguments,

    [string[]]$SourceFiles = @('src/emulator/unicorn.rs'),

    [string]$Executable = 'cargo',

    [string]$BaselineRef = '',

    # Only for the exact release build after a separate successful clean.
    # The target outputs must be absent before this attempt and are stamped
    # immediately after Cargo exits successfully.
    [switch]$CaptureReleaseArtifacts
)

$ErrorActionPreference = 'Stop'
$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$evidenceRoot = [System.IO.Path]::GetFullPath((Join-Path $repoRoot 'evidence'))
$rawLog = [System.IO.Path]::GetFullPath((Join-Path $repoRoot $LogPath))
if (-not $rawLog.StartsWith($evidenceRoot + [System.IO.Path]::DirectorySeparatorChar,
        [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'LogPath must be beneath this repository\evidence.'
}
if (-not (Test-Path -LiteralPath (Split-Path -Parent $rawLog) -PathType Container)) {
    throw "The raw-log directory does not exist: $(Split-Path -Parent $rawLog)"
}
if ($CargoArguments.Count -eq 0) {
    throw 'CargoArguments must not be empty.'
}
$releaseArtifacts = $null
if ($CaptureReleaseArtifacts) {
    $releaseBuildArgs = @('build', '--release', '--features',
        'unicorn,trace,win32-desktop', '-j', '1')
    if ($Executable -cne 'cargo' -or
        $CargoArguments.Count -ne $releaseBuildArgs.Count -or
        -not [string]::IsNullOrEmpty($env:CARGO_TARGET_DIR)) {
        throw 'Release artifact capture requires the exact Cargo build and this repository target directory.'
    }
    for ($i = 0; $i -lt $releaseBuildArgs.Count; $i++) {
        if ($CargoArguments[$i] -cne $releaseBuildArgs[$i]) {
            throw 'Release artifact capture requires the exact Cargo build arguments.'
        }
    }
    $releaseArtifacts = @{
        exe = [System.IO.Path]::GetFullPath(
            (Join-Path $repoRoot 'target\release\wince_emulation_v3.exe'))
        pdb = [System.IO.Path]::GetFullPath(
            (Join-Path $repoRoot 'target\release\wince_emulation_v3.pdb'))
    }
    foreach ($path in @($releaseArtifacts.exe, $releaseArtifacts.pdb)) {
        if (Test-Path -LiteralPath $path) {
            throw "Release output existed before this build attempt: $path"
        }
    }
}
$executablePath = $Executable
$executableHashBefore = $null
if ($Executable -ne 'cargo') {
    if ([System.IO.Path]::IsPathRooted($Executable)) {
        $executablePath = [System.IO.Path]::GetFullPath($Executable)
    } else {
        $executablePath = [System.IO.Path]::GetFullPath((Join-Path $repoRoot $Executable))
    }
    if (-not (Test-Path -LiteralPath $executablePath -PathType Leaf)) {
        throw "Executable does not exist: $executablePath"
    }
    $executableHashBefore = (Get-FileHash -LiteralPath $executablePath -Algorithm SHA256).Hash.ToLowerInvariant()
}

$sourceSnapshots = @()
foreach ($relativePath in $SourceFiles) {
    $absolutePath = [System.IO.Path]::GetFullPath((Join-Path $repoRoot $relativePath))
    if (-not $absolutePath.StartsWith($repoRoot + [System.IO.Path]::DirectorySeparatorChar,
            [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Source path is outside the repository: $relativePath"
    }
    if (-not (Test-Path -LiteralPath $absolutePath -PathType Leaf)) {
        throw "Source file does not exist: $relativePath"
    }
    $sourceSnapshots += [pscustomobject]@{
        Relative = $relativePath.Replace('\', '/')
        Absolute = $absolutePath
        Before = (Get-FileHash -LiteralPath $absolutePath -Algorithm SHA256).Hash.ToLowerInvariant()
    }
}

Push-Location -LiteralPath $repoRoot
$writer = $null
$commandExitCode = 125
$runnerError = $null
$capturedReleaseArtifacts = $null
try {
    $headBefore = (& git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Could not read source HEAD.' }
    $baselineHead = $null
    $baselineBlobs = @()
    if ($BaselineRef.Length -gt 0) {
        $baselineHead = (& git rev-parse $BaselineRef).Trim()
        if ($LASTEXITCODE -ne 0) { throw "Could not resolve baseline ref: $BaselineRef" }
        foreach ($snapshot in $sourceSnapshots) {
            $blobSpec = "$($baselineHead):$($snapshot.Relative)"
            $blob = (& git rev-parse $blobSpec).Trim()
            if ($LASTEXITCODE -ne 0) { throw "Could not resolve baseline source blob: $blobSpec" }
            $baselineBlobs += "$($snapshot.Relative) $blob"
        }
    }
    $commandWords = foreach ($argument in $CargoArguments) {
        if ($argument -match '[\s"`]' -or $argument.Length -eq 0) {
            '"' + $argument.Replace('"', '\"') + '"'
        } else {
            $argument
        }
    }
    $displayExecutable = if ($executablePath -match '\s') { '"' + $executablePath + '"' } else { $executablePath }
    $command = $displayExecutable + ' ' + ($commandWords -join ' ')

    # CreateNew refuses a previous attempt's path, even if it is empty.
    $stream = [System.IO.File]::Open($rawLog,
        [System.IO.FileMode]::CreateNew,
        [System.IO.FileAccess]::Write,
        [System.IO.FileShare]::Read)
    $writer = New-Object System.IO.StreamWriter($stream, (New-Object System.Text.UTF8Encoding($false)))
    $writer.AutoFlush = $true
    $writer.WriteLine("ATTEMPT=$Attempt")
    $writer.WriteLine("COMMAND=$command")
    $writer.WriteLine('COMMAND_ARGV_JSON=' + (ConvertTo-Json -InputObject @($CargoArguments) -Compress))
    $writer.WriteLine("SOURCE_HEAD_BEFORE=$headBefore")
    if ($null -ne $baselineHead) {
        $writer.WriteLine("BASELINE_SOURCE_HEAD=$baselineHead")
        foreach ($baselineBlob in $baselineBlobs) {
            $writer.WriteLine("BASELINE_SOURCE_GIT_BLOB=$baselineBlob")
        }
    }
    if ($null -ne $executableHashBefore) {
        $writer.WriteLine("EXECUTABLE_SHA256_BEFORE=$executableHashBefore")
    }
    foreach ($snapshot in $sourceSnapshots) {
        $writer.WriteLine("SOURCE_SHA256_BEFORE=$($snapshot.Relative) $($snapshot.Before)")
    }
    $writer.WriteLine('LIBCLANG_PATH=' + $env:LIBCLANG_PATH)
    $writer.WriteLine('RUST_MIN_STACK=' + $env:RUST_MIN_STACK)
    $writer.WriteLine('CARGO_TARGET_DIR=' + $env:CARGO_TARGET_DIR)
    $writer.WriteLine('RUN_START_UTC=' + [System.DateTime]::UtcNow.ToString('o'))

    $ErrorActionPreference = 'Continue'
    & $executablePath @CargoArguments 2>&1 | ForEach-Object {
        $line = $_.ToString()
        $writer.WriteLine($line)
        Write-Output $line
    }
    $commandExitCode = $LASTEXITCODE
    if ($null -eq $commandExitCode) { $commandExitCode = 125 }
    $ErrorActionPreference = 'Stop'
    if ($CaptureReleaseArtifacts -and $commandExitCode -eq 0) {
        foreach ($path in @($releaseArtifacts.exe, $releaseArtifacts.pdb)) {
            $output = Get-Item -LiteralPath $path -Force -ErrorAction Stop
            if ($output -isnot [System.IO.FileInfo] -or
                ($output.Attributes -band [System.IO.FileAttributes]::ReparsePoint)) {
                throw "Fresh release output is missing or redirected: $path"
            }
        }
        $capturedReleaseArtifacts = @{
            exe = (Get-FileHash -LiteralPath $releaseArtifacts.exe -Algorithm SHA256).Hash.ToUpperInvariant()
            pdb = (Get-FileHash -LiteralPath $releaseArtifacts.pdb -Algorithm SHA256).Hash.ToUpperInvariant()
        }
    }
} catch {
    $runnerError = $_.Exception.Message
} finally {
    if ($null -ne $writer) {
        if ($null -ne $runnerError) {
            $writer.WriteLine('RUNNER_ERROR=' + $runnerError)
        }
        $writer.WriteLine('RUN_END_UTC=' + [System.DateTime]::UtcNow.ToString('o'))
        $writer.WriteLine("EXIT_CODE=$commandExitCode")
        if ($null -ne $capturedReleaseArtifacts) {
            $writer.WriteLine("BUILD_OUTPUT_EXE_PATH=$($releaseArtifacts.exe)")
            $writer.WriteLine("BUILD_OUTPUT_PDB_PATH=$($releaseArtifacts.pdb)")
            $writer.WriteLine("BUILD_OUTPUT_EXE_SHA256=$($capturedReleaseArtifacts.exe)")
            $writer.WriteLine("BUILD_OUTPUT_PDB_SHA256=$($capturedReleaseArtifacts.pdb)")
        }
        $headAfter = (& git rev-parse HEAD).Trim()
        $writer.WriteLine("SOURCE_HEAD_AFTER=$headAfter")
        $sourceChanged = $false
        foreach ($snapshot in $sourceSnapshots) {
            $after = (Get-FileHash -LiteralPath $snapshot.Absolute -Algorithm SHA256).Hash.ToLowerInvariant()
            $writer.WriteLine("SOURCE_SHA256_AFTER=$($snapshot.Relative) $after")
            if ($after -ne $snapshot.Before) { $sourceChanged = $true }
        }
        if ($null -ne $executableHashBefore) {
            $executableHashAfter = (Get-FileHash -LiteralPath $executablePath -Algorithm SHA256).Hash.ToLowerInvariant()
            $writer.WriteLine("EXECUTABLE_SHA256_AFTER=$executableHashAfter")
            if ($executableHashAfter -ne $executableHashBefore) { $sourceChanged = $true }
        }
        $writer.WriteLine("SOURCE_CHANGED_DURING_RUN=$sourceChanged")
        $writer.Dispose()
        $logHash = (Get-FileHash -LiteralPath $rawLog -Algorithm SHA256).Hash.ToLowerInvariant()
        Write-Output "CLOSED_LOG_SHA256=$logHash"
        Write-Output "RAW_LOG=$rawLog"
        if ($sourceChanged) { $commandExitCode = 124 }
    }
    Pop-Location
}
if ($null -ne $runnerError) { throw $runnerError }
exit $commandExitCode
