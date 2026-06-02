param(
    [string]$MapPath = "coredll.map",
    [string]$OutputPath = "src/ce/coredll_ordinals.rs"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Escape-RustString {
    param([string]$Value)
    $Value.Replace("\", "\\").Replace('"', '\"')
}

function Read-MapExports {
    param([string]$Path)

    $seen = @{}
    $exports = New-Object System.Collections.Generic.List[object]
    foreach ($raw in [System.IO.File]::ReadLines((Resolve-Path $Path))) {
        $parts = @($raw -split "\s+" | Where-Object { $_ -ne "" })
        if ($parts.Count -lt 2) {
            continue
        }

        $ordinal = 0
        if (-not [int]::TryParse($parts[0], [ref]$ordinal)) {
            continue
        }

        $name = $parts[1]
        $key = "$name`0$ordinal"
        if ($seen.ContainsKey($key)) {
            continue
        }

        $entry = [pscustomobject]@{
            Name = $name
            Ordinal = $ordinal
            Key = $key
        }
        $seen[$key] = $entry
        $exports.Add($entry) | Out-Null
    }

    return $exports
}

function New-MapSet {
    param($Exports)

    $set = @{}
    foreach ($export in $Exports) {
        $set[$export.Key] = $export
    }
    return $set
}

function Get-Section {
    param(
        [string]$Text,
        [string]$StartMarker,
        [string]$EndMarker
    )

    $start = $Text.IndexOf($StartMarker)
    if ($start -lt 0) {
        throw "missing marker: $StartMarker"
    }

    $end = $Text.IndexOf($EndMarker, $start)
    if ($end -lt 0) {
        throw "missing marker after ${StartMarker}: $EndMarker"
    }

    return $Text.Substring($start, $end - $start).TrimEnd()
}

function Read-ExistingExportIndex {
    param(
        [string]$Text,
        $MapSet
    )

    $startMarker = "pub const COREDLL_EXPORT_INDEX"
    $start = $Text.IndexOf($startMarker)
    if ($start -lt 0) {
        throw "missing COREDLL_EXPORT_INDEX; restore the checked-in generated file before regenerating"
    }

    $end = $Text.IndexOf("pub fn is_current_map_export", $start)
    if ($end -lt 0) {
        throw "missing is_current_map_export after COREDLL_EXPORT_INDEX"
    }

    $block = $Text.Substring($start, $end - $start)
    $entryPattern = 'None,|Some\(CoredllOrdinalDef\s*\{\s*name:\s*"((?:\\.|[^"])*)",\s*ordinal:\s*(\d+),\s*\}\),'
    $entries = New-Object System.Collections.Generic.List[object]
    foreach ($match in [regex]::Matches($block, $entryPattern, [System.Text.RegularExpressions.RegexOptions]::Singleline)) {
        if ($match.Value.StartsWith("None")) {
            $entries.Add($null) | Out-Null
            continue
        }

        $name = $match.Groups[1].Value
        $ordinal = [int]$match.Groups[2].Value
        $key = "$name`0$ordinal"
        if ($MapSet.ContainsKey($key)) {
            $entries.Add($MapSet[$key]) | Out-Null
        } else {
            $entries.Add($null) | Out-Null
        }
    }

    if ($entries.Count -eq 0) {
        throw "COREDLL_EXPORT_INDEX parsed zero entries"
    }

    return $entries
}

function New-ExportsBlock {
    param($Exports)

    $lines = New-Object System.Collections.Generic.List[string]
    $lines.Add("pub const COREDLL_EXPORTS: &[CoredllOrdinalDef; $($Exports.Count)] = &[") | Out-Null
    foreach ($export in $Exports) {
        $name = Escape-RustString $export.Name
        $lines.Add("    CoredllOrdinalDef { name: `"$name`", ordinal: $($export.Ordinal) },") | Out-Null
    }
    $lines.Add("];") | Out-Null
    return [string]::Join("`n", $lines)
}

function New-ExportIndexBlock {
    param($Entries)

    $lines = New-Object System.Collections.Generic.List[string]
    $lines.Add("pub const COREDLL_EXPORT_INDEX: &[Option<CoredllOrdinalDef>; $($Entries.Count)] = &[") | Out-Null
    foreach ($entry in $Entries) {
        if ($null -eq $entry) {
            $lines.Add("    None,") | Out-Null
            continue
        }

        $name = Escape-RustString $entry.Name
        $lines.Add("    Some(CoredllOrdinalDef { name: `"$name`", ordinal: $($entry.Ordinal) }),") | Out-Null
    }
    $lines.Add("];") | Out-Null
    return [string]::Join("`n", $lines)
}

$mapExports = Read-MapExports $MapPath
$mapSet = New-MapSet $mapExports
$existing = [System.IO.File]::ReadAllText((Resolve-Path $OutputPath))

$constantsBlock = Get-Section $existing "pub const ORD_SYSTEM_MEMORY_LOW" "pub const SDK_ORDINALS"
$sdkBlock = Get-Section $existing "pub const SDK_ORDINALS" "pub fn lookup"
$exportIndex = Read-ExistingExportIndex $existing $mapSet

$newText = @(
    "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
    "pub struct CoredllOrdinalDef {",
    "    pub name: &'static str,",
    "    pub ordinal: u32,",
    "}",
    "",
    (New-ExportsBlock $mapExports),
    "",
    (New-ExportIndexBlock $exportIndex),
    "",
    "pub fn is_current_map_export(export: &CoredllOrdinalDef) -> bool {",
    "    COREDLL_EXPORTS",
    "        .iter()",
    "        .any(|current| current.name == export.name && current.ordinal == export.ordinal)",
    "}",
    "",
    "pub fn current_static_export_count() -> usize {",
    "    COREDLL_EXPORTS.len()",
    "}",
    "",
    $constantsBlock,
    "",
    $sdkBlock,
    "",
    "pub fn lookup(ordinal: u32) -> Option<&'static CoredllOrdinalDef> {",
    "    COREDLL_EXPORTS.iter().find(|export| export.ordinal == ordinal)",
    "}",
    "",
    "pub fn lookup_export_index(index: u32) -> Option<&'static CoredllOrdinalDef> {",
    "    COREDLL_EXPORT_INDEX",
    "        .get(index as usize)",
    "        .and_then(Option::as_ref)",
    "}",
    ""
) -join "`n"

$resolvedOutput = Resolve-Path $OutputPath
[System.IO.File]::WriteAllText($resolvedOutput.Path, $newText, [System.Text.UTF8Encoding]::new($false))

$rustfmt = Get-Command rustfmt -ErrorAction SilentlyContinue
if ($null -ne $rustfmt) {
    & $rustfmt.Source --edition 2021 $resolvedOutput.Path
    if ($LASTEXITCODE -ne 0) {
        throw "rustfmt failed for $OutputPath"
    }
} else {
    Write-Warning "rustfmt was not found; run cargo fmt before committing $OutputPath"
}
Write-Output "generated $OutputPath from ${MapPath}: exports=$($mapExports.Count), export_index_slots=$($exportIndex.Count)"
