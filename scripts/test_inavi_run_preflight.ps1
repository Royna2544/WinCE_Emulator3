#requires -Version 7.5
<#
Offline acceptance controls for a future bounded-run preflight. This file never
dot-sources the old runner or backup script, creates fixture files, enumerates
mounts, launches a process, or writes a use ledger. All evidence artifact paths
are synthetic strings under this repository's evidence directory. The source
root string names the requested D: checkout; no source or mount path is opened.

Proposed pure contract in scripts/inavi_run_preflight_core.ps1:
  Test-InaviRunManifestCore -Manifest <hashtable-or-null> -Observed <hashtable>
    -LocalLedger <array> -TrackedLedger <array>
returns { accepted: bool, codes: string[] }. Observed values are independently
collected by the future monitor worker; the production worker must recheck its
own observations immediately before spawn. Core validation cannot itself
grant or spend approval. The exclusive dual-ledger reservation is separate.

-FixturesOnly checks the in-memory fixture inventory. Until the core exists,
normal execution produces a dependency red, not a behavioral validator red.
#>
[CmdletBinding()]
param([switch]$FixturesOnly)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Copy-Fixture([object]$Value) {
    return ($Value | ConvertTo-Json -Depth 24 |
        ConvertFrom-Json -AsHashtable -DateKind String)
}

function New-SyntheticHash([string]$Label) {
    $bytes = [System.Text.Encoding]::UTF8.GetBytes("offline-only fixture: $Label")
    return [Convert]::ToHexString([System.Security.Cryptography.SHA256]::HashData($bytes))
}

function New-Baseline {
    $root = 'D:\GitHub\Wince_emulation_v3\evidence\stage3-live-evidence-20261015\offline-only'
    $sourceRoot = 'D:\GitHub\Wince_emulation_v3'
    $backupTag = '20261015T020000Z'
    $sourceFiles = @{}
    foreach ($relative in @(
        'src/main.rs', 'src/remote_server.rs', 'src/ce/coredll.rs',
        'src/ce/kernel.rs', 'src/ce/remote.rs', 'src/ce/gwe.rs',
        'tests/basic_subsystems.rs', 'src/ce/memory.rs',
        'src/emulator/unicorn.rs', 'Cargo.toml', 'Cargo.lock',
        '.gitattributes',
        'scripts/inavi_bounded_e22.ps1', 'scripts/inavi_run_preflight_core.ps1',
        'scripts/backup_rw_mounts.ps1', 'scripts/test_inavi_run_preflight.ps1',
        'scripts/backup_rw_mounts_inventory_core.ps1',
        'scripts/test_backup_rw_mounts_inventory.ps1',
        'scripts/inavi_summary_observer_core.ps1',
        'scripts/test_inavi_summary_observer.ps1',
        'scripts/test_inavi_runner_guard.ps1'
    )) { $sourceFiles[$relative] = New-SyntheticHash "source $relative" }
    $hash = @{
        sd = (New-SyntheticHash 'SD archive')
        resident = (New-SyntheticHash 'ResidentFlash archive')
        emulator = (New-SyntheticHash 'EmulatorRoot archive')
        patch = (New-SyntheticHash 'fresh source patch')
        seal = (New-SyntheticHash 'external source seal file')
        build = (New-SyntheticHash 'build receipt')
        exe = (New-SyntheticHash 'fresh executable')
        pdb = (New-SyntheticHash 'fresh PDB')
        registry = (New-SyntheticHash 'registry configuration')
        devices = (New-SyntheticHash 'serial device configuration')
        mounts = (New-SyntheticHash 'mount configuration')
        app = (New-SyntheticHash 'target application')
        backupManifest = (New-SyntheticHash 'backup manifest')
    }
    $paths = @{
        exe = "$root\fresh.exe"; pdb = "$root\fresh.pdb"
        build = "$root\build-receipt.json"; seal = "$root\source-seal.json"
        backups = "$root\backup-manifest.json"
        registry = "$root\registry.reg"; devices = "$root\serial_devices.json"
        mounts = "$root\mounts.toml"; app = "$root\app.exe"
    }
    $roots = @("$root\SD", "$root\ResidentFlash", "$root\EmulatorRoot")
    $backups = @(
        @{ name = 'SD'; sourceRoot = $roots[0]; archivePath = "$root\INAVI-pre-live-$backupTag.zip"; archiveSha256 = $hash.sd },
        @{ name = 'ResidentFlash'; sourceRoot = $roots[1]; archivePath = "$root\ResidentFlash-pre-live-$backupTag.zip"; archiveSha256 = $hash.resident },
        @{ name = 'EmulatorRoot'; sourceRoot = $roots[2]; archivePath = "$root\EmulatorRoot-pre-live-$backupTag.zip"; archiveSha256 = $hash.emulator }
    )
    $seal = @{
        head = '9C302DB35583669FAB435449A5F4515B712583D6'
        parentHead = '0e06e2c5c662ad224c9b472e7798f4604240a6b6'
        sourceRoot = $sourceRoot; sourceFiles = $sourceFiles
        patchSha256 = $hash.patch
        kernelSha256 = $sourceFiles['src/ce/kernel.rs']
        remoteSha256 = $sourceFiles['src/ce/remote.rs']
        gweSha256 = $sourceFiles['src/ce/gwe.rs']
        basicSubsystemsSha256 = $sourceFiles['tests/basic_subsystems.rs']
    }
    $receipt = @{
        path = $paths.build; sha256 = $hash.build; sourceHead = $seal.head
        sourcePatchSha256 = $seal.patchSha256; wB1cParentHead = $seal.parentHead
        sourceSealSha256 = $hash.seal
        focusedPassed = $true; releasePassed = $true
        noFeaturePassed = $true; alternateFeaturesPassed = $true
        executableSha256 = $hash.exe; pdbSha256 = $hash.pdb
    }
    # The receipt file cannot contain its own digest. `sha256` is only the
    # external manifest/observation digest; parsed content omits it.
    $parsedReceipt = Copy-Fixture $receipt
    $parsedReceipt.Remove('sha256')
    $manifest = @{
        schemaVersion = 1; runTag = 'offline-fixture-fresh'
        approvalReference = 'call_offline_fresh_approval'
        approvalObservedUtc = '2026-10-15T01:59:00.0000000Z'
        sourceSeal = $seal
        sourceSealFile = @{ path = $paths.seal; sha256 = $hash.seal }
        buildReceipt = $receipt
        executable = @{ path = $paths.exe; sha256 = $hash.exe; rsds = 'E072B53A-6F13-4C91-B379-2C5E166A419B:1' }
        pdb = @{ path = $paths.pdb; sha256 = $hash.pdb; rsds = 'E072B53A-6F13-4C91-B379-2C5E166A419B:1' }
        config = @{
            registry = @{ path = $paths.registry; sha256 = $hash.registry }
            devices = @{ path = $paths.devices; sha256 = $hash.devices }
            mounts = @{ path = $paths.mounts; sha256 = $hash.mounts }
            app = @{ path = $paths.app; sha256 = $hash.app }
        }
        backupManifest = @{ path = $paths.backups; sha256 = $hash.backupManifest }
        backups = $backups
        caps = @{ appSeconds = 330; watchdogSeconds = 390; rssMiB = 2048; rawLogMiB = 512 }
    }
    $files = @{}
    foreach ($item in @(
        @($paths.exe, $hash.exe), @($paths.pdb, $hash.pdb),
        @($paths.seal, $hash.seal),
        @($paths.build, $hash.build), @($paths.backups, $hash.backupManifest),
        @($paths.registry, $hash.registry), @($paths.devices, $hash.devices),
        @($paths.mounts, $hash.mounts), @($paths.app, $hash.app)
    )) { $files[$item[0]] = $item[1] }
    foreach ($item in $backups) { $files[$item.archivePath] = $item.archiveSha256 }
    $completed = Copy-Fixture $backups
    foreach ($row in $completed) {
        $row.postBackupInventoryVerified = $true
        $row.postBackupSourceManifest = "$root\$($row.name)-post-backup-source-manifest.json"
        $row.postBackupSourceManifestSha256 = New-SyntheticHash "post-backup $($row.name) manifest"
    }
    $observed = @{
        currentSourceSeal = Copy-Fixture $seal
        parsedSourceSeal = Copy-Fixture $seal
        currentSourceRoot = $sourceRoot
        sourceFileSha256 = Copy-Fixture $sourceFiles
        currentManifestSha256 = New-SyntheticHash 'current external run manifest'
        parsedBuildReceipt = $parsedReceipt
        fileSha256 = $files
        executableRsds = $manifest.executable.rsds
        pdbRsds = $manifest.pdb.rsds
        writableMountRoots = $roots
        verifiedBackupManifest = @{
            sha256 = $hash.backupManifest; complete = $true; completed = $completed
            schemaVersion = 2; tag = $backupTag
            startedUtc = '2026-10-15T02:00:00.0000000Z'
            updatedUtc = '2026-10-15T02:10:00.0000000Z'
            scriptSha256 = $sourceFiles['scripts/backup_rw_mounts.ps1']
        }
        existingEvidenceTags = @()
    }
    return @{ manifest = $manifest; observed = $observed; localLedger = @(); trackedLedger = @() }
}

function New-UseEntry([string]$Approval, [string]$State, [string[]]$ZipHashes) {
    $allZipHashes = @($ZipHashes)
    while ($allZipHashes.Count -lt 3) {
        $allZipHashes += New-SyntheticHash "prior ZIP $Approval $State $($allZipHashes.Count)"
    }
    $entry = @{
        eventId = "offline-$State-$Approval"
        attemptId = "offline-attempt-$Approval"
        manifestSha256 = New-SyntheticHash "prior manifest $Approval"
        approvalReference = $Approval; zipSha256 = $allZipHashes
        state = $State; utc = '2026-10-15T00:00:00.0000000Z'
    }
    if ($State -eq 'spent') {
        $entry.pid = [long]4242
        $entry.processStartUtc = '2026-10-15T00:00:01.0000000Z'
    }
    return $entry
}

function New-LegacyDenyEntry([string[]]$ZipHashes) {
    $entry = New-UseEntry 'legacy-fixture-only' 'legacy-deny' $ZipHashes
    $entry.approvalReference = $null
    $entry.provenance = 'evidence\e22-live-20261013T154435Z\manifest.json'
    $entry.reason = 'Historical writable-mount launch; approval identifier not recorded.'
    return $entry
}

$base = New-Baseline
$cases = [System.Collections.Generic.List[object]]::new()
$cases.Add(@{ name = 'fresh_baseline'; data = Copy-Fixture $base; accepted = $true; code = $null })

$sample = Copy-Fixture $base
$sample.manifest = $null
$cases.Add(@{ name = 'missing_manifest'; data = $sample; accepted = $false; code = 'SCHEMA_MISSING' })

$sample = Copy-Fixture $base
$sample.manifest = @{
    runTag = 'e37-live-20261014T051228Z'
    executable = 'D:\GitHub\Wince_emulation_v3\evidence\op4-fresh-view-readback-20261014\wince_emulation_v3-e37.exe'
    executableSha256 = '7A55C049D2BBD0110D30D587BBB1877BA80C50CEE1D33E453434C0366A871A1A'
    approvalQuestionId = 'call_93cb71e3f4dc432c84262f0a340f3091'
    backupProvenanceManifest = 'D:\GitHub\Wince_emulation_v3\evidence\pre-live-mount-backup-20261014T050927Z\manifest.json'
}
$cases.Add(@{ name = 'legacy_e37_manifest_shape'; data = $sample; accepted = $false; code = 'SCHEMA_MISSING' })

$sample = Copy-Fixture $base
$sample.manifest.sourceSeal.head = 'A764019D25F3C2D64BFD926103FEAC8B50890142'
$sample.manifest.executable.path = 'D:\GitHub\Wince_emulation_v3\evidence\op4-fresh-view-readback-20261014\wince_emulation_v3-e37.exe'
$cases.Add(@{ name = 'stale_e37_binary_and_source'; data = $sample; accepted = $false; code = 'SOURCE_SEAL_MISMATCH' })

$sample = Copy-Fixture $base
$oldApproval = 'call_93cb71e3f4dc432c84262f0a340f3091'
$sample.manifest.approvalReference = $oldApproval
$entry = New-UseEntry $oldApproval 'spent' @((New-SyntheticHash 'old E-37 SD archive'), (New-SyntheticHash 'old E-37 ResidentFlash archive'), (New-SyntheticHash 'old E-37 EmulatorRoot archive'))
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'stale_e37_spent_approval'; data = $sample; accepted = $false; code = 'APPROVAL_REUSED' })

$sample = Copy-Fixture $base
$entry = New-UseEntry $sample.manifest.approvalReference 'spent' @((New-SyntheticHash 'unrelated spent archive'))
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'already_spent_approval'; data = $sample; accepted = $false; code = 'APPROVAL_REUSED' })

$sample = Copy-Fixture $base
$entry = New-UseEntry $sample.manifest.approvalReference 'reserved' @((New-SyntheticHash 'unresolved reserved archive'))
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'interrupted_reservation'; data = $sample; accepted = $false; code = 'APPROVAL_RESERVED' })

$sample = Copy-Fixture $base
$entry = New-UseEntry 'call_other_offline_approval' 'reserved' @((New-SyntheticHash 'unrelated reserved archive'))
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'unrelated_unresolved_reservation'; data = $sample; accepted = $false; code = 'LEDGER_UNRESOLVED_RESERVATION' })

$sample = Copy-Fixture $base
$reserved = New-UseEntry 'call_other_offline_approval' 'reserved' @((New-SyntheticHash 'unrelated reserved archive'))
$spent = New-UseEntry 'call_other_offline_approval' 'spent' @($reserved.zipSha256)
$sample.localLedger = @($reserved, $spent)
$sample.trackedLedger = Copy-Fixture $sample.localLedger
$cases.Add(@{ name = 'reservation_followed_by_matching_spent'; data = $sample; accepted = $true; code = $null })

$sample = Copy-Fixture $base
$reserved = New-UseEntry 'call_other_offline_approval' 'reserved' @((New-SyntheticHash 'unrelated reserved archive'))
$spent = New-UseEntry 'call_other_offline_approval' 'spent' @($reserved.zipSha256)
$spent.attemptId = 'offline-attempt-other-not-matching'
$sample.localLedger = @($reserved, $spent)
$sample.trackedLedger = Copy-Fixture $sample.localLedger
$cases.Add(@{ name = 'reservation_followed_by_mismatched_spent'; data = $sample; accepted = $false; code = 'LEDGER_UNRESOLVED_RESERVATION' })

$sample = Copy-Fixture $base
$reserved = New-UseEntry 'call_other_offline_approval' 'reserved' @((New-SyntheticHash 'unrelated reserved archive'))
$spent = New-UseEntry 'call_other_offline_approval' 'spent' @($reserved.zipSha256)
$sample.localLedger = @($spent, $reserved)
$sample.trackedLedger = Copy-Fixture $sample.localLedger
$cases.Add(@{ name = 'spent_before_reservation_does_not_resolve'; data = $sample; accepted = $false; code = 'LEDGER_UNRESOLVED_RESERVATION' })

$sample = Copy-Fixture $base
$reserved = New-UseEntry 'call_other_offline_approval' 'reserved' @((New-SyntheticHash 'unrelated reserved archive'))
$reconciled = New-UseEntry 'call_other_offline_approval' 'reconciled' @($reserved.zipSha256)
$sample.localLedger = @($reserved, $reconciled)
$sample.trackedLedger = Copy-Fixture $sample.localLedger
$cases.Add(@{ name = 'reconciled_without_process_identity_does_not_resolve'; data = $sample; accepted = $false; code = 'LEDGER_UNRESOLVED_RESERVATION' })

$sample = Copy-Fixture $base
$legacy = New-LegacyDenyEntry @((New-SyntheticHash 'old SD archive'), (New-SyntheticHash 'old ResidentFlash archive'), (New-SyntheticHash 'old EmulatorRoot archive'))
$sample.localLedger = @($legacy)
$sample.trackedLedger = @(Copy-Fixture $legacy)
$cases.Add(@{ name = 'legacy_deny_with_unknown_approval_and_unrelated_zips'; data = $sample; accepted = $true; code = $null })

$sample = Copy-Fixture $base
$oldZip = New-SyntheticHash 'old SD archive'
$sample.manifest.backups[0].archiveSha256 = $oldZip
$sample.observed.verifiedBackupManifest.completed[0].archiveSha256 = $oldZip
$sample.observed.fileSha256[$sample.manifest.backups[0].archivePath] = $oldZip
$legacy = New-LegacyDenyEntry @($oldZip, (New-SyntheticHash 'old ResidentFlash archive'), (New-SyntheticHash 'old EmulatorRoot archive'))
$sample.localLedger = @($legacy)
$sample.trackedLedger = @(Copy-Fixture $legacy)
$cases.Add(@{ name = 'legacy_deny_blocks_prior_zip'; data = $sample; accepted = $false; code = 'ZIP_REUSED' })

$sample = Copy-Fixture $base
$legacy = New-LegacyDenyEntry @((New-SyntheticHash 'old SD archive'))
$legacy.Remove('approvalReference')
$sample.localLedger = @($legacy)
$sample.trackedLedger = @(Copy-Fixture $legacy)
$cases.Add(@{ name = 'legacy_deny_missing_approval_field'; data = $sample; accepted = $true; code = $null })

$sample = Copy-Fixture $base
$legacy = New-LegacyDenyEntry @((New-SyntheticHash 'old SD archive'))
$legacy.approvalReference = 'call_fabricated_legacy_approval'
$sample.localLedger = @($legacy)
$sample.trackedLedger = @(Copy-Fixture $legacy)
$cases.Add(@{ name = 'legacy_deny_fake_approval_rejected'; data = $sample; accepted = $false; code = 'LEDGER_INVALID' })

$sample = Copy-Fixture $base
$legacy = New-LegacyDenyEntry @((New-SyntheticHash 'old SD archive'))
$legacy.Remove('provenance')
$sample.localLedger = @($legacy)
$sample.trackedLedger = @(Copy-Fixture $legacy)
$cases.Add(@{ name = 'legacy_deny_without_provenance_rejected'; data = $sample; accepted = $false; code = 'LEDGER_INVALID' })

$sample = Copy-Fixture $base
$legacy = New-LegacyDenyEntry @((New-SyntheticHash 'old SD archive'))
$legacy.Remove('reason')
$sample.localLedger = @($legacy)
$sample.trackedLedger = @(Copy-Fixture $legacy)
$cases.Add(@{ name = 'legacy_deny_without_reason_rejected'; data = $sample; accepted = $false; code = 'LEDGER_INVALID' })

$sample = Copy-Fixture $base
$entry = New-UseEntry 'call_other_offline_approval' 'reserved' @((New-SyntheticHash 'unrelated reserved archive'))
$entry.approvalReference = $null
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'reservation_with_unknown_approval_rejected'; data = $sample; accepted = $false; code = 'LEDGER_INVALID' })

$sample = Copy-Fixture $base
$entry = New-UseEntry 'call_other_offline_approval' 'spent' @($sample.manifest.backups[0].archiveSha256)
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'reused_zip_hash'; data = $sample; accepted = $false; code = 'ZIP_REUSED' })

$sample = Copy-Fixture $base
$oldZip = '4766C2E3241028902DEF32B85173165F4CB34528396399E9149E669497C02AF4'
$sample.manifest.backups[0].archiveSha256 = $oldZip
$sample.observed.verifiedBackupManifest.completed[0].archiveSha256 = $oldZip
$sample.observed.fileSha256[$sample.manifest.backups[0].archivePath] = $oldZip
$entry = New-UseEntry $oldApproval 'spent' @($oldZip)
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'stale_e37_zip_hash'; data = $sample; accepted = $false; code = 'ZIP_REUSED' })

$sample = Copy-Fixture $base
$sample.manifest.backups[0].archiveSha256 = New-SyntheticHash 'incorrect claimed SD archive'
$sample.observed.verifiedBackupManifest.completed[0].archiveSha256 = $sample.manifest.backups[0].archiveSha256
$cases.Add(@{ name = 'wrong_zip_hash'; data = $sample; accepted = $false; code = 'HASH_MISMATCH' })

$sample = Copy-Fixture $base
$sample.observed.existingEvidenceTags = @($sample.manifest.runTag)
$cases.Add(@{ name = 'duplicate_evidence_tag'; data = $sample; accepted = $false; code = 'EVIDENCE_TAG_EXISTS' })

$sample = Copy-Fixture $base
$sample.manifest.backups[1].sourceRoot = 'D:\GitHub\Wince_emulation_v3\evidence\stage3-live-evidence-20261015\offline-only\WrongRoot'
$sample.observed.verifiedBackupManifest.completed[1].sourceRoot = $sample.manifest.backups[1].sourceRoot
$cases.Add(@{ name = 'mismatched_writable_root'; data = $sample; accepted = $false; code = 'MOUNT_ROOT_MISMATCH' })

$sample = Copy-Fixture $base
$sample.localLedger = @(New-UseEntry 'call_other_offline_approval' 'spent' @((New-SyntheticHash 'local-only spent archive')))
$cases.Add(@{ name = 'local_tracked_ledger_divergence'; data = $sample; accepted = $false; code = 'LEDGER_DIVERGENCE' })

$sample = Copy-Fixture $base
$sample.localLedger = $null
$cases.Add(@{ name = 'missing_local_ledger'; data = $sample; accepted = $false; code = 'LEDGER_UNAVAILABLE' })

$sample = Copy-Fixture $base
$sample.manifest.Remove('approvalReference')
$cases.Add(@{ name = 'missing_approval_reference'; data = $sample; accepted = $false; code = 'APPROVAL_MISSING' })

$sample = Copy-Fixture $base
$sample.manifest.approvalReference = '   '
$cases.Add(@{ name = 'blank_approval_reference'; data = $sample; accepted = $false; code = 'APPROVAL_MISSING' })

$sample = Copy-Fixture $base
$sample.manifest.approvalReference = $oldApproval
$cases.Add(@{ name = 'spent_e37_approval_without_ledger_history'; data = $sample; accepted = $false; code = 'APPROVAL_REUSED' })

$sample = Copy-Fixture $base
$sample.manifest.sourceSeal.patchSha256 = 'D' * 64
$sample.observed.currentSourceSeal.patchSha256 = 'D' * 64
$sample.manifest.buildReceipt.sourcePatchSha256 = 'D' * 64
$sample.observed.parsedBuildReceipt.sourcePatchSha256 = 'D' * 64
$cases.Add(@{ name = 'placeholder_source_patch_hash'; data = $sample; accepted = $false; code = 'SOURCE_SEAL_MISMATCH' })

$sample = Copy-Fixture $base
$sample.observed.pdbRsds = 'AB72C323-E441-4824-9F24-FA4B14761680:1'
$cases.Add(@{ name = 'pdb_rsds_identity_mismatch'; data = $sample; accepted = $false; code = 'RSDS_MISMATCH' })

$sample = Copy-Fixture $base
$sample.observed.fileSha256[$sample.manifest.executable.path] = New-SyntheticHash 'different executable bytes'
$cases.Add(@{ name = 'executable_hash_mismatch'; data = $sample; accepted = $false; code = 'HASH_MISMATCH' })

$sample = Copy-Fixture $base
$sample.observed.fileSha256[$sample.manifest.pdb.path] = New-SyntheticHash 'different PDB bytes'
$cases.Add(@{ name = 'pdb_hash_mismatch'; data = $sample; accepted = $false; code = 'HASH_MISMATCH' })

$sample = Copy-Fixture $base
$sample.observed.fileSha256[$sample.manifest.buildReceipt.path] = New-SyntheticHash 'different build receipt bytes'
$cases.Add(@{ name = 'external_build_receipt_digest_mismatch'; data = $sample; accepted = $false; code = 'HASH_MISMATCH' })

$sample = Copy-Fixture $base
$sample.manifest.buildReceipt.releasePassed = $false
$sample.observed.parsedBuildReceipt.releasePassed = $false
$cases.Add(@{ name = 'failed_release_build_gate'; data = $sample; accepted = $false; code = 'BUILD_RECEIPT_MISMATCH' })

$sample = Copy-Fixture $base
$sample.manifest.caps.appSeconds = 600
$cases.Add(@{ name = 'loosened_app_cap'; data = $sample; accepted = $false; code = 'CAP_MISMATCH' })

$sample = Copy-Fixture $base
$sample.observed.verifiedBackupManifest.completed[2].archivePath = 'D:\GitHub\Wince_emulation_v3\evidence\stage3-live-evidence-20261015\offline-only\other.zip'
$cases.Add(@{ name = 'backup_manifest_row_mismatch'; data = $sample; accepted = $false; code = 'BACKUP_MANIFEST_MISMATCH' })

$sample = Copy-Fixture $base
$sample.observed.verifiedBackupManifest.complete = $false
$cases.Add(@{ name = 'backup_manifest_not_complete'; data = $sample; accepted = $false; code = 'BACKUP_MANIFEST_MISMATCH' })

$sample = Copy-Fixture $base
$sample.observed.verifiedBackupManifest.completed[1].postBackupInventoryVerified = $false
$cases.Add(@{ name = 'backup_post_inventory_not_verified'; data = $sample; accepted = $false; code = 'BACKUP_MANIFEST_MISMATCH' })

$sample = Copy-Fixture $base
$sample.observed.verifiedBackupManifest.schemaVersion = 1
$cases.Add(@{ name = 'legacy_backup_schema'; data = $sample; accepted = $false; code = 'BACKUP_MANIFEST_MISMATCH' })

$sample = Copy-Fixture $base
$sample.observed.verifiedBackupManifest.startedUtc = '2026-10-15T01:58:00.0000000Z'
$sample.observed.verifiedBackupManifest.tag = '20261015T015800Z'
$cases.Add(@{ name = 'backup_started_before_approval'; data = $sample; accepted = $false; code = 'BACKUP_NOT_FRESH' })

$sample = Copy-Fixture $base
$sample.observed.verifiedBackupManifest.scriptSha256 = New-SyntheticHash 'other backup script'
$cases.Add(@{ name = 'backup_script_not_sealed'; data = $sample; accepted = $false; code = 'BACKUP_MANIFEST_MISMATCH' })

$sample = Copy-Fixture $base
$sample.manifest.approvalObservedUtc = '2026-10-15T02:01:00.0000000Z'
$cases.Add(@{ name = 'approval_observed_after_backup_start'; data = $sample; accepted = $false; code = 'BACKUP_NOT_FRESH' })

$sample = Copy-Fixture $base
$entry = New-UseEntry 'call_unrelated_offline_approval' 'unknown' @((New-SyntheticHash 'unrelated archive'))
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'malformed_ledger_entry'; data = $sample; accepted = $false; code = 'LEDGER_INVALID' })

$sample = Copy-Fixture $base
$sample.trackedLedger = $null
$cases.Add(@{ name = 'missing_tracked_ledger'; data = $sample; accepted = $false; code = 'LEDGER_UNAVAILABLE' })

$sample = Copy-Fixture $base
$sample.manifest.backups[2].archivePath = $sample.manifest.backups[0].archivePath
$sample.manifest.backups[2].archiveSha256 = $sample.manifest.backups[0].archiveSha256
$sample.observed.verifiedBackupManifest.completed[2].archivePath = $sample.manifest.backups[2].archivePath
$sample.observed.verifiedBackupManifest.completed[2].archiveSha256 = $sample.manifest.backups[2].archiveSha256
$cases.Add(@{ name = 'duplicate_backup_archive'; data = $sample; accepted = $false; code = 'BACKUP_MANIFEST_MISMATCH' })

$sample = Copy-Fixture $base
$sample.manifest.sourceSeal.sourceFiles.Remove('src/main.rs')
$cases.Add(@{ name = 'missing_main_source_hash'; data = $sample; accepted = $false; code = 'SOURCE_SEAL_MISMATCH' })

$sample = Copy-Fixture $base
$sample.observed.sourceFileSha256['src/main.rs'] = New-SyntheticHash 'changed telemetry main source'
$cases.Add(@{ name = 'changed_main_source_bytes'; data = $sample; accepted = $false; code = 'SOURCE_SEAL_MISMATCH' })

$sample = Copy-Fixture $base
$sample.observed.sourceFileSha256['scripts/inavi_bounded_e22.ps1'] = New-SyntheticHash 'changed effectful runner'
$cases.Add(@{ name = 'changed_effectful_runner_bytes'; data = $sample; accepted = $false; code = 'SOURCE_SEAL_MISMATCH' })

$sample = Copy-Fixture $base
$sample.observed.fileSha256[$sample.manifest.sourceSealFile.path] = New-SyntheticHash 'different sealed source file'
$cases.Add(@{ name = 'external_source_seal_digest_mismatch'; data = $sample; accepted = $false; code = 'HASH_MISMATCH' })

$sample = Copy-Fixture $base
$sample.observed.parsedSourceSeal.head = 'A764019D25F3C2D64BFD926103FEAC8B50890142'
$cases.Add(@{ name = 'parsed_source_seal_content_mismatch'; data = $sample; accepted = $false; code = 'SOURCE_SEAL_MISMATCH' })

$sample = Copy-Fixture $base
$entry = New-UseEntry 'call_unrelated_offline_approval' 'reserved' @((New-SyntheticHash 'unrelated archive'))
$entry.Remove('attemptId')
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'ledger_missing_attempt_id'; data = $sample; accepted = $false; code = 'LEDGER_INVALID' })

$sample = Copy-Fixture $base
$entry = New-UseEntry 'call_unrelated_offline_approval' 'reserved' @((New-SyntheticHash 'unrelated archive'))
$entry.Remove('manifestSha256')
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'ledger_missing_manifest_digest'; data = $sample; accepted = $false; code = 'LEDGER_INVALID' })

$sample = Copy-Fixture $base
$entry = New-UseEntry 'call_unrelated_offline_approval' 'reserved' @((New-SyntheticHash 'unrelated archive'))
$entry.zipSha256 = @($entry.zipSha256[0], $entry.zipSha256[1])
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'ledger_not_three_zip_hashes'; data = $sample; accepted = $false; code = 'LEDGER_INVALID' })

$sample = Copy-Fixture $base
$entry = New-UseEntry 'call_unrelated_offline_approval' 'spent' @((New-SyntheticHash 'unrelated archive'))
$entry.Remove('pid')
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'spent_without_pid'; data = $sample; accepted = $false; code = 'LEDGER_INVALID' })

$sample = Copy-Fixture $base
$entry = New-UseEntry 'call_unrelated_offline_approval' 'spent' @((New-SyntheticHash 'unrelated archive'))
$entry.Remove('processStartUtc')
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'spent_without_start_identity'; data = $sample; accepted = $false; code = 'LEDGER_INVALID' })

$sample = Copy-Fixture $base
$entry = New-UseEntry 'call_other_offline_approval' 'reserved' @($sample.manifest.backups[0].archiveSha256)
$sample.localLedger = @($entry)
$sample.trackedLedger = @(Copy-Fixture $entry)
$cases.Add(@{ name = 'zip_reused_after_reservation'; data = $sample; accepted = $false; code = 'ZIP_REUSED' })

$sample = Copy-Fixture $base
$entry = New-UseEntry 'call_unrelated_offline_approval' 'reserved' @((New-SyntheticHash 'unrelated archive'))
$sample.localLedger = @($entry, (Copy-Fixture $entry))
$sample.trackedLedger = Copy-Fixture $sample.localLedger
$cases.Add(@{ name = 'duplicate_ledger_event_id'; data = $sample; accepted = $false; code = 'LEDGER_INVALID' })

$sample = Copy-Fixture $base
$oldPath = $sample.manifest.backupManifest.path
$sample.manifest.backupManifest.path = 'D:\GitHub\Wince_emulation_v3\evidence\pre-live-mount-backup-20261014T050927Z\manifest.json'
$sample.observed.fileSha256.Remove($oldPath)
$sample.observed.fileSha256[$sample.manifest.backupManifest.path] = $sample.manifest.backupManifest.sha256
$cases.Add(@{ name = 'legacy_e37_backup_manifest_path'; data = $sample; accepted = $false; code = 'LEGACY_E37_ARTIFACT' })

if ($cases.Count -ne 59) { throw "Unexpected fixture count: $($cases.Count)" }
$baselineJson = $cases[0].data | ConvertTo-Json -Depth 24 -Compress
foreach ($test in $cases) {
    $fixtureJson = $test.data | ConvertTo-Json -Depth 24 -Compress
    if ($fixtureJson -match 'D:\\INAVI_Emulator') {
        throw "Actual writable-mount path leaked into $($test.name)"
    }
    if (-not $test.accepted -and $fixtureJson -eq $baselineJson) {
        throw "Negative fixture is identical to the baseline: $($test.name)"
    }
    if ($null -ne $test.data.manifest -and $test.name -ne 'legacy_e37_manifest_shape' -and
        $test.data.manifest.runTag -notlike 'offline-fixture-*') {
        throw "Non-synthetic run tag in $($test.name)"
    }
    if ($null -ne $test.data.manifest -and $test.name -ne 'legacy_e37_manifest_shape' -and
        $test.data.manifest.backups.Count -ne 3) {
        throw "Fixture lacks exactly three backup rows in $($test.name)"
    }
}
Write-Output "FIXTURES_OK count=$($cases.Count); no files, mounts, ledgers or processes touched"
if ($FixturesOnly) { exit 0 }

$corePath = Join-Path $PSScriptRoot 'inavi_run_preflight_core.ps1'
if (-not (Test-Path -LiteralPath $corePath -PathType Leaf)) {
    [Console]::Error.WriteLine("DEPENDENCY_RED: pure preflight core is not implemented at $corePath; no behavioral validator result is claimed")
    exit 2
}
. $corePath
if ($null -eq (Get-Command Test-InaviRunManifestCore -CommandType Function -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('DEPENDENCY_RED: Test-InaviRunManifestCore is absent; no behavioral validator result is claimed')
    exit 2
}

$failures = [System.Collections.Generic.List[string]]::new()
foreach ($test in $cases) {
    try {
        $result = Test-InaviRunManifestCore -Manifest $test.data.manifest -Observed $test.data.observed -LocalLedger $test.data.localLedger -TrackedLedger $test.data.trackedLedger
        if ($null -eq $result -or $null -eq $result.accepted -or $null -eq $result.codes) {
            throw 'Core did not return accepted and codes fields'
        }
        $accepted = [bool]$result.accepted
        $codes = @($result.codes)
        if ($accepted -ne [bool]$test.accepted) {
            $failures.Add("$($test.name): accepted=$accepted, expected=$($test.accepted)")
        } elseif (-not $accepted -and $test.code -notin $codes) {
            $failures.Add("$($test.name): missing $($test.code), got $($codes -join ',')")
        } else {
            Write-Output "PASS $($test.name) accepted=$accepted codes=$($codes -join ',')"
        }
    } catch {
        $failures.Add("$($test.name): $($_.Exception.Message)")
    }
}
if ($failures.Count -gt 0) {
    foreach ($failure in $failures) { [Console]::Error.WriteLine("FAIL $failure") }
    exit 1
}
Write-Output "PASS all $($cases.Count) offline preflight controls"
exit 0
