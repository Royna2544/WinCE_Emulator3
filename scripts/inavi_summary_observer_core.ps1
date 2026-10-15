#requires -Version 7.0
<#
Pure bounded-body and publication-field helpers for the mounted-run observer.
This file has no top-level filesystem, network, process or mount action. The
offline controls dot-source only this file, never the effectful run script.
#>

function Read-InaviBoundedSummaryStream {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][System.IO.Stream]$Stream,
        [ValidateRange(1, 65536)][int]$MaxBytes = 65536,
        [System.Threading.CancellationToken]$CancellationToken = [System.Threading.CancellationToken]::None
    )

    $buffer = [byte[]]::new(8192)
    $body = [System.IO.MemoryStream]::new()
    try {
        while ($true) {
            # Read one byte beyond the cap to distinguish an exact-cap body
            # from an oversized one without buffering an unbounded response.
            $remaining = $MaxBytes + 1 - [int]$body.Length
            $read = $Stream.ReadAsync(
                $buffer, 0, [Math]::Min($buffer.Length, $remaining), $CancellationToken
            ).GetAwaiter().GetResult()
            if ($read -eq 0) {
                return [pscustomobject]@{
                    oversize = $false
                    bytesRead = [int]$body.Length
                    bytes = $body.ToArray()
                }
            }
            $body.Write($buffer, 0, $read)
            if ($body.Length -gt $MaxBytes) {
                return [pscustomobject]@{
                    oversize = $true
                    bytesRead = [int]$body.Length
                    bytes = [byte[]]@()
                }
            }
        }
    } finally {
        $body.Dispose()
    }
}

function ConvertFrom-InaviSummaryBody {
    [CmdletBinding()]
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $strictUtf8 = [System.Text.UTF8Encoding]::new($false, $true)
    $text = $strictUtf8.GetString($Bytes)
    $lines = $text -split "`n", 5
    if ($lines.Count -lt 4) {
        throw 'Summary is missing its four publication fields.'
    }
    $names = @(
        'publication_sequence',
        'publication_host_monotonic_ns',
        'publication_utc_unix_ms',
        'live_guest_stable_page_budget_used_bytes'
    )
    $values = [System.Collections.Generic.List[UInt64]]::new()
    for ($i = 0; $i -lt $names.Count; $i++) {
        $line = $lines[$i].TrimEnd("`r")
        $pattern = '^  ' + [regex]::Escape($names[$i]) + ': ([0-9]+)$'
        if ($line -cnotmatch $pattern) {
            throw "Summary publication field $($names[$i]) is missing or malformed."
        }
        [UInt64]$value = 0
        if (-not [UInt64]::TryParse(
                $Matches[1],
                [System.Globalization.NumberStyles]::None,
                [System.Globalization.CultureInfo]::InvariantCulture,
                [ref]$value
            )) {
            throw "Summary publication field $($names[$i]) is out of range."
        }
        $values.Add($value)
    }
    if ($values[0] -eq 0) {
        throw 'Summary publication sequence must be nonzero.'
    }
    return [pscustomobject]@{
        publicationSequence = $values[0]
        publicationHostMonotonicNs = $values[1]
        publicationUtcUnixMs = $values[2]
        liveGuestStablePageBudgetUsedBytes = $values[3]
    }
}
