#requires -Version 7.0
<#
Offline expected-red control for the two effectful scripts' atomic JSON
overwrite helpers. Parse and define only the named function bodies; never
dot-source either parent script or open an emulator mount, ZIP or ledger.
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$utf8 = [System.Text.UTF8Encoding]::new($false)

function Get-ExactFunctionDefinition([string]$SourcePath, [string]$Name) {
    $tokens = $null
    $errors = $null
    $ast = [System.Management.Automation.Language.Parser]::ParseFile(
        $SourcePath, [ref]$tokens, [ref]$errors)
    if ($errors.Count -ne 0) {
        throw "$SourcePath has $($errors.Count) parse error(s)."
    }
    $found = @($ast.FindAll({ param($node)
        $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
            $node.Name -ceq $Name
    }, $true))
    if ($found.Count -ne 1) {
        throw "Expected one exact $Name definition in $SourcePath; found $($found.Count)."
    }
    return $found[0].Extent.Text
}

$backupSource = Join-Path $PSScriptRoot 'backup_rw_mounts.ps1'
$runnerSource = Join-Path $PSScriptRoot 'inavi_bounded_e22.ps1'
foreach ($name in @('Write-Json', 'Write-JsonAtomic')) {
    . ([scriptblock]::Create((Get-ExactFunctionDefinition $backupSource $name)))
}
foreach ($name in @('Write-JsonFile', 'Write-ReplacingJsonFile')) {
    . ([scriptblock]::Create((Get-ExactFunctionDefinition $runnerSource $name)))
}

function Assert-Test([bool]$Condition, [string]$Label) {
    if (-not $Condition) { throw "Offline atomic JSON control failed: $Label" }
    Write-Output "PASS $Label"
}

$failures = [System.Collections.Generic.List[string]]::new()
function Invoke-Control([string]$Label, [scriptblock]$Action) {
    try {
        & $Action
        Write-Output "PASS $Label"
    } catch {
        $message = $_.Exception.Message
        $script:failures.Add("$Label`: $message")
        Write-Output "EXPECTED-RED $Label`: $message"
    }
}

function Assert-TargetValue([string]$Path, [string]$Expected) {
    $content = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    Assert-Test ([string]$content.value -ceq $Expected) "target value $Expected"
}

function Assert-NoScratchFiles([string]$Directory, [string]$Label) {
    $leftovers = @(Get-ChildItem -LiteralPath $Directory -File |
        Where-Object { $_.Name -like '*.tmp' })
    Assert-Test ($leftovers.Count -eq 0) "$Label leaves no temporary file"
}

function Assert-FailedOverwriteCleansScratch(
    [string]$Path,
    [scriptblock]$Action,
    [string]$Label
) {
    $before = [System.IO.File]::ReadAllBytes($Path)
    $handle = [System.IO.FileStream]::new(
        $Path,
        [System.IO.FileMode]::Open,
        [System.IO.FileAccess]::ReadWrite,
        [System.IO.FileShare]::None
    )
    $rejected = $false
    try {
        try { & $Action $Path } catch { $rejected = $true }
    } finally {
        $handle.Dispose()
    }
    Assert-Test $rejected "$Label rejects a locked existing target"
    $after = [System.IO.File]::ReadAllBytes($Path)
    Assert-Test ([Convert]::ToHexString($before) -ceq [Convert]::ToHexString($after)) `
        "$Label preserves original bytes on replace failure"
    Assert-NoScratchFiles (Split-Path -Parent $Path) "$Label failed overwrite"
}

$scratchParent = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath()).TrimEnd('\', '/')
$scratch = [System.IO.Path]::GetFullPath((Join-Path $scratchParent (
    'wce-atomic-json-test-' + [Guid]::NewGuid().ToString('N'))))
$resolvedParent = [System.IO.Path]::GetDirectoryName($scratch)
if (-not [string]::Equals($resolvedParent, $scratchParent,
        [StringComparison]::OrdinalIgnoreCase) -or
    -not ([System.IO.Path]::GetFileName($scratch) -match '^wce-atomic-json-test-[0-9a-f]{32}$')) {
    throw "Unsafe scratch target: $scratch"
}

[void][System.IO.Directory]::CreateDirectory($scratch)
try {
    $backupTarget = Join-Path $scratch 'manifest.json'
    $runnerTarget = Join-Path $scratch 'heartbeat.json'

    Write-JsonAtomic $backupTarget @{ value = 'created-backup' }
    Assert-TargetValue $backupTarget 'created-backup'
    Invoke-Control 'backup existing-file atomic replacement' {
        Write-JsonAtomic $backupTarget @{ value = 'replaced-backup' }
        Assert-TargetValue $backupTarget 'replaced-backup'
    }
    Assert-NoScratchFiles $scratch 'backup successful overwrite'

    Write-ReplacingJsonFile $runnerTarget @{ value = 'created-runner' }
    Assert-TargetValue $runnerTarget 'created-runner'
    Invoke-Control 'runner existing-file atomic replacement' {
        Write-ReplacingJsonFile $runnerTarget @{ value = 'replaced-runner' }
        Assert-TargetValue $runnerTarget 'replaced-runner'
    }
    Invoke-Control 'runner successful overwrite cleanup' {
        Assert-NoScratchFiles $scratch 'runner successful overwrite'
    }
    # Isolate the next failure-path control from the prior expected-red leak.
    $runnerTemporary = "$runnerTarget.tmp"
    if (Test-Path -LiteralPath $runnerTemporary -PathType Leaf) {
        Remove-Item -LiteralPath $runnerTemporary -Force
    }

    $lockedBackup = Join-Path $scratch 'locked-manifest.json'
    $lockedRunner = Join-Path $scratch 'locked-heartbeat.json'
    Write-JsonAtomic $lockedBackup @{ value = 'old-backup' }
    Write-ReplacingJsonFile $lockedRunner @{ value = 'old-runner' }
    Invoke-Control 'backup failed-overwrite cleanup' {
        Assert-FailedOverwriteCleansScratch $lockedBackup {
        param($path) Write-JsonAtomic $path @{ value = 'blocked-backup' }
        } 'backup helper'
    }
    Invoke-Control 'runner failed-overwrite cleanup' {
        Assert-FailedOverwriteCleansScratch $lockedRunner {
        param($path) Write-ReplacingJsonFile $path @{ value = 'blocked-runner' }
        } 'runner helper'
    }
    if ($failures.Count -ne 0) {
        throw "$($failures.Count) offline atomic JSON control(s) failed on current source."
    }
    Write-Output 'All offline atomic JSON controls passed.'
} finally {
    $resolvedParent = [System.IO.Path]::GetDirectoryName(
        [System.IO.Path]::GetFullPath($scratch))
    if (-not [string]::Equals($resolvedParent, $scratchParent,
            [StringComparison]::OrdinalIgnoreCase) -or
        -not ([System.IO.Path]::GetFileName($scratch) -match '^wce-atomic-json-test-[0-9a-f]{32}$')) {
        throw "Refusing unsafe scratch cleanup: $scratch"
    }
    if (Test-Path -LiteralPath $scratch -PathType Container) {
        Remove-Item -LiteralPath $scratch -Recurse -Force
    }
}
