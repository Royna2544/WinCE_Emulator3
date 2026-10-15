#requires -Version 7.0
# Pure synthetic controls. This script never opens the mounted trees or ZIPs.
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'backup_rw_mounts_inventory_core.ps1')

function New-Inventory {
    [pscustomobject]@{
        files = @(
            [pscustomobject]@{
                relative = 'INavi/map.dat'; fullPath = 'X:\offline\INavi\map.dat'
                length = [long]4096; lastWriteUtc = '2026-10-15T00:00:00.0000000Z'
                attributes = 32; sha256 = ('A' * 64)
            }
        )
        directories = @(
            [pscustomobject]@{
                relative = 'INavi'; fullPath = 'X:\offline\INavi'
                length = $null; lastWriteUtc = '2026-10-15T00:00:00.0000000Z'
                attributes = 16; sha256 = $null
            }
        )
    }
}

function Assert-Rejects([string]$Label, [scriptblock]$Mutation) {
    $first = New-Inventory
    $last = New-Inventory
    try { & $Mutation $last } catch { throw "Invalid synthetic mutation '$Label': $_" }
    try {
        Assert-BackupInventoryMatch -Initial $first -Final $last -SourceRoot 'X:\offline'
    } catch {
        if ($_.Exception.Message -notmatch '^(Post-backup|Initial|Invalid or duplicate)') {
            throw "Unexpected failure for '$Label': $_"
        }
        Write-Output "PASS reject: $Label"
        return
    }
    throw "Expected rejection did not occur: $Label"
}

Assert-BackupInventoryMatch -Initial (New-Inventory) -Final (New-Inventory) -SourceRoot 'X:\offline'
Write-Output 'PASS unchanged content and metadata'
Assert-Rejects 'same-size same-mtime changed content' { param($item) $item.files[0].sha256 = 'B' * 64 }
Assert-Rejects 'file length changed' { param($item) $item.files[0].length = 4097 }
Assert-Rejects 'file mtime changed' { param($item) $item.files[0].lastWriteUtc = '2026-10-15T00:00:01.0000000Z' }
Assert-Rejects 'file added' { param($item) $item.files += [pscustomobject]@{ relative='other'; length=0; lastWriteUtc='x'; attributes=32; sha256=('C'*64) } }
Assert-Rejects 'file removed' { param($item) $item.files = @() }
Assert-Rejects 'directory removed' { param($item) $item.directories = @() }
Assert-Rejects 'directory metadata changed' { param($item) $item.directories[0].attributes = 17 }
Assert-Rejects 'missing final hash' { param($item) $item.files[0].sha256 = $null }
Write-Output 'All nine offline backup inventory controls passed.'

# Inspect the effectful script as syntax only; do not invoke it. The one UTC
# capture must precede both source-inventory calls and be wired into the
# manifest state and every publication.
$backupScript = Join-Path $PSScriptRoot 'backup_rw_mounts.ps1'
$tokens = $null
$parseErrors = $null
$ast = [System.Management.Automation.Language.Parser]::ParseFile(
    $backupScript, [ref]$tokens, [ref]$parseErrors
)
if ($parseErrors.Count -ne 0) { throw 'Backup script has a PowerShell parse error' }
$startAssignments = @($ast.FindAll({
    param($node)
    $node -is [System.Management.Automation.Language.AssignmentStatementAst] -and
        $node.Left.Extent.Text -ceq '$backupStartedUtc'
}, $true))
$sourceReads = @($ast.FindAll({
    param($node)
    $node -is [System.Management.Automation.Language.CommandAst] -and
        $node.GetCommandName() -eq 'Get-SourceInventory'
}, $true))
if ($startAssignments.Count -ne 1 -or $sourceReads.Count -ne 2 -or
    $startAssignments[0].Extent.StartOffset -ge $sourceReads[0].Extent.StartOffset -or
    $ast.Extent.Text -notmatch '(?m)^\s*startedUtc\s*=\s*\$backupStartedUtc\s*$' -or
    $ast.Extent.Text -notmatch '(?m)^\s*\$tag\s*=\s*\$backupStarted\.ToString\(''yyyyMMddTHHmmssZ''\)\s*$' -or
    $ast.Extent.Text -notmatch 'Assert-BackupStartedUtcContract\s+-Manifest\s+\$manifestState\s+-ExpectedStartedUtc\s+\$backupStartedUtc') {
    throw 'Backup script does not preserve one pre-inventory startedUtc capture through publication'
}
Write-Output 'PASS parser: one start capture precedes inventory and is published unchanged'

$expectedStart = '2026-10-15T01:02:03.1234567Z'
$expectedTag = '20261015T010203Z'
$fixture = @{ tag = $expectedTag; startedUtc = $expectedStart; updatedUtc = $expectedStart; complete = $false }
if (([DateTime]::ParseExact(
    $expectedStart, 'O', [System.Globalization.CultureInfo]::InvariantCulture,
    [System.Globalization.DateTimeStyles]::RoundtripKind
)).ToString('yyyyMMddTHHmmssZ') -ne $expectedTag) {
    throw 'Synthetic backup tag does not match the start timestamp second'
}
Assert-BackupStartedUtcContract -Manifest $fixture -ExpectedStartedUtc $expectedStart
$fixture.updatedUtc = '2026-10-15T01:05:06.1234567Z'
$fixture.complete = $true
Assert-BackupStartedUtcContract -Manifest $fixture -ExpectedStartedUtc $expectedStart
Write-Output 'PASS fixture: initial and final publications retain the same start time'

function Assert-StartRejects([string]$Label, [hashtable]$Fixture, [string]$Expected) {
    try {
        Assert-BackupStartedUtcContract -Manifest $Fixture -ExpectedStartedUtc $Expected
    } catch {
        if ($_.Exception.Message -notmatch '^Backup startedUtc') {
            throw "Unexpected start-time failure for '$Label': $_"
        }
        Write-Output "PASS reject: $Label"
        return
    }
    throw "Expected start-time rejection did not occur: $Label"
}

$missingStart = $fixture.Clone()
[void]$missingStart.Remove('startedUtc')
Assert-StartRejects 'missing startedUtc' $missingStart $expectedStart
$changedStart = $fixture.Clone()
$changedStart.startedUtc = '2026-10-15T01:02:04.1234567Z'
Assert-StartRejects 'changed startedUtc across publication' $changedStart $expectedStart
$malformedStart = $fixture.Clone()
$malformedStart.startedUtc = '2026-99-99T01:02:03.1234567Z'
Assert-StartRejects 'invalid UTC startedUtc' $malformedStart $malformedStart.startedUtc
Write-Output 'All five offline backup freshness controls passed.'
