# Pure comparison used by the writable-mount backup script and offline tests.
# A matching size/mtime is insufficient: compare the final content hash too.
function Assert-BackupStartedUtcContract {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)] [hashtable]$Manifest,
        [Parameter(Mandatory)] [string]$ExpectedStartedUtc
    )

    if (-not $Manifest.ContainsKey('startedUtc') -or
        [string]$Manifest.startedUtc -cne $ExpectedStartedUtc) {
        throw 'Backup startedUtc is missing or changed during publication'
    }
    if ($ExpectedStartedUtc -cnotmatch '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{7}Z$') {
        throw 'Backup startedUtc must be an exact UTC round-trip timestamp'
    }
    $parsed = [DateTimeOffset]::MinValue
    if (-not [DateTimeOffset]::TryParseExact(
        $ExpectedStartedUtc, 'O', [System.Globalization.CultureInfo]::InvariantCulture,
        [System.Globalization.DateTimeStyles]::RoundtripKind, [ref]$parsed
    )) {
        throw 'Backup startedUtc is not a valid UTC timestamp'
    }
}

function Assert-BackupInventoryMatch {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)] [object]$Initial,
        [Parameter(Mandatory)] [object]$Final,
        [Parameter(Mandatory)] [string]$SourceRoot
    )

    foreach ($kind in @('files', 'directories')) {
        $before = @($Initial.$kind)
        $after = @($Final.$kind)
        if ($before.Count -ne $after.Count) {
            throw "Post-backup $kind count changed: $SourceRoot ($($before.Count) -> $($after.Count))"
        }

        $byPath = [System.Collections.Generic.Dictionary[string, object]]::new(
            [System.StringComparer]::Ordinal
        )
        foreach ($record in $before) {
            $relative = [string]$record.relative
            if ([string]::IsNullOrWhiteSpace($relative) -or
                -not $byPath.TryAdd($relative, $record)) {
                throw "Invalid or duplicate initial $kind entry: $SourceRoot/$relative"
            }
            if ($kind -eq 'files' -and
                [string]$record.sha256 -cnotmatch '^[0-9A-F]{64}$') {
                throw "Initial file hash is missing or invalid: $SourceRoot/$relative"
            }
        }

        foreach ($record in $after) {
            $relative = [string]$record.relative
            $original = $null
            if (-not $byPath.TryGetValue($relative, [ref]$original)) {
                throw "Post-backup $kind entry was added or renamed: $SourceRoot/$relative"
            }
            [void]$byPath.Remove($relative)
            foreach ($field in @('length', 'lastWriteUtc', 'attributes', 'sha256')) {
                if ([string]$record.$field -cne [string]$original.$field) {
                    throw "Post-backup $kind $field changed: $SourceRoot/$relative"
                }
            }
        }
        if ($byPath.Count -ne 0) {
            throw "Post-backup $kind entry was removed: $SourceRoot"
        }
    }
}
