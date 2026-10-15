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
    & $Mutation $last
    try {
        Assert-BackupInventoryMatch -Initial $first -Final $last -SourceRoot 'X:\offline'
    } catch {
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
