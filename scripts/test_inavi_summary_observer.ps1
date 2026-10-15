#requires -Version 7.0
<#
Offline controls for the pure summary reader/parser. This test does not load
the run script, open a socket, enumerate mounts, start a process, or write a
backup/approval ledger.
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'inavi_summary_observer_core.ps1')

$passed = 0
function Assert-True([bool]$Condition, [string]$Name) {
    if (-not $Condition) { throw "FAIL $Name" }
    $script:passed++
    Write-Output "PASS $Name"
}

function Assert-ParseFailure([byte[]]$Body, [string]$Name) {
    $failed = $false
    try { [void](ConvertFrom-InaviSummaryBody -Bytes $Body) } catch { $failed = $true }
    Assert-True $failed $Name
}

$utf8 = [System.Text.UTF8Encoding]::new($false)
$validText = "  publication_sequence: 7`n  publication_host_monotonic_ns: 123456789`n  publication_utc_unix_ms: 1792022400000`n  live_guest_stable_page_budget_used_bytes: 4096`n  last_stop_snapshot: none`n"
$validBytes = $utf8.GetBytes($validText)
$parsed = ConvertFrom-InaviSummaryBody -Bytes $validBytes
Assert-True (
    $parsed.publicationSequence -eq 7 -and
    $parsed.publicationHostMonotonicNs -eq 123456789 -and
    $parsed.publicationUtcUnixMs -eq 1792022400000 -and
    $parsed.liveGuestStablePageBudgetUsedBytes -eq 4096
) 'valid four-field publication body'

Assert-ParseFailure ($utf8.GetBytes($validText.Replace('publication_sequence: 7', 'publication_sequence: -7'))) 'negative sequence rejects'
Assert-ParseFailure ($utf8.GetBytes($validText.Replace('publication_sequence: 7', 'publication_sequence: 0'))) 'zero sequence rejects'
Assert-ParseFailure ($utf8.GetBytes($validText.Replace('publication_host_monotonic_ns: 123456789', 'publication_host_monotonic_ns: 18446744073709551616'))) 'overflowing monotonic clock rejects'
Assert-ParseFailure ($utf8.GetBytes($validText.Replace('publication_utc_unix_ms: 1792022400000', 'publication_utc_unix_ms: invalid'))) 'malformed UTC rejects'
Assert-ParseFailure ($utf8.GetBytes("  publication_sequence: 7`n  publication_host_monotonic_ns: 123456789`n")) 'missing publication fields reject'
Assert-ParseFailure ([byte[]]@(0xFF, 0xFE, 0xFD)) 'invalid UTF-8 rejects'

$exactBytes = [byte[]]::new(65536)
$exactStream = [System.IO.MemoryStream]::new($exactBytes, $false)
try {
    $exact = Read-InaviBoundedSummaryStream -Stream $exactStream
    Assert-True (-not $exact.oversize -and $exact.bytesRead -eq 65536 -and $exact.bytes.Length -eq 65536) 'exact 64-KiB body accepted'
} finally { $exactStream.Dispose() }

$largeBytes = [byte[]]::new(70000)
$largeStream = [System.IO.MemoryStream]::new($largeBytes, $false)
try {
    $large = Read-InaviBoundedSummaryStream -Stream $largeStream
    Assert-True ($large.oversize -and $large.bytesRead -eq 65537 -and $large.bytes.Length -eq 0 -and $largeStream.Position -eq 65537) 'oversize body rejected without buffering its tail'
} finally { $largeStream.Dispose() }

Write-Output "PASS all $passed offline summary controls; no app, mount, backup or ledger action"
