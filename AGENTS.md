# AGENTS.md

Entry point for any coding agent (Codex, Claude Code, others) working in this
repo. Keep it short and current; details live in the files it points to.

## What this is

A Rust emulator that runs a Windows CE 6.0 MIPS (little-endian, MIPS-II) GUI
app — iNavi SE G3 (`iNavi.exe`, MFC + COREDLL) — on a PC. The guest runs on
Unicorn; `COREDLL` and friends are implemented as host-side shims (import
traps) in Rust, with host-backed files, registry, GDI/DIB framebuffer, serial
GPS, audio and a remote HTTP control surface.

## Read first, in this order

1. `RULES.md` — hard user rules and machine paths. **Mandatory.**
2. `PROGRESS.md` (confirmed facts), `TODO.md` (active work),
   `KNOWN_BUGS.md` (open failures). Keep them short and non-stale.
3. `docs/ARCHITECTURE.md` is the common ownership/identity/time/outcome anchor;
   `docs/PLAN.md` gates implementation. Read both before architecture changes,
   then the relevant `docs/subsystems/` page. Proposed types are not implemented.
   `docs/DECISIONS.md` is the decision log (decided and open items) and
   `docs/evidence/ledger.md` holds executed evidence; cite them, do not copy them.
4. `git status` — know whether the tree is a committed fix, a diagnostic, or an
   untracked artifact before you change anything.
5. `PLAN.MD` is a long ledger of CE-parity work plus a roadmap. Do not read it
   end to end; grep it. `SOURCE_REFERENCES.md` indexes CE source evidence.

## Hard rules (from RULES.md — they win over convenience)

- This is an emulator. Do not fake app behavior to make a screenshot look right:
  no hand-painting, no forced callbacks, no guessed state, no behavior keyed on
  hardcoded strings or file names, no hardcoded iNavi fixes as "the" fix.
- Allowed: targeted tracing, guards, temporary breakpoints/watchpoints,
  known-address diagnostics, host shims that implement real API semantics.
- Do not quit an investigation on your own; keep running bounded auto tests.
- Never run recursive `find`/glob over a broad root (`/mnt/c`, `C:\`, `D:\`).
- Guest `PC == 0` is an emulator control-flow bug, never a normal exit. Fail
  the run; do not report success.
- Real behavior comes from CE evidence: the dumped device binaries, the CE
  source under `C:\WINCE600`, MFC source, and disassembly — not from guessing.

## Environment (details in RULES.md)

- Target app: `D:\INAVI_Emulator\INAVI\INavi\iNavi.exe` (SD image
  `D:\INAVI_Emulator\INAVI`, mounted via `mounts.toml`).
- Real device DLL/EXE dump (CE 6.0 ROM, includes `coredll.dll`, `gwes.dll`,
  `filesys.dll`, `device.dll`, `kernel.dll`, `nk.exe`):
  `D:\INAVI_Emulator\DUMPPLZ\Windows`. Treat it as the source of truth for
  target binaries.
- MIPS disassembly: `D:\GitHub\llvm-proj\build\bin\llvm-objdump.exe`
  (`-d --mattr=+mips2`, `-p` for imports/exports). ImageMagick is on PATH.
- Windows host, PowerShell/`-NoProfile`; Rust edition 2024.

## Build, test, run

Normal feature set is `unicorn,trace,win32-desktop`. Check `--features unicorn`
alone and CI's `unicorn,win32-desktop` profile as well when touching feature
gates; a prior copy-based validation passed them, but the current D: source
must be checked independently.

```bash
cargo check --features unicorn,trace,win32-desktop
cargo test  --features unicorn,trace,win32-desktop --test coredll_raw_kernel
cargo test  --features unicorn,trace,win32-desktop --test coredll_raw_gwe
cargo test  --features unicorn,trace,win32-desktop --test coredll_raw_memory_file
cargo test  --features unicorn,trace,win32-desktop --test basic_subsystems
cargo test -j 1          # no-feature stub/raw smoke profile
```

Run `cargo fmt`/targeted `rustfmt` after Rust edits and `git diff --check`
before handoff (CRLF warnings on old files are pre-existing noise).
For each new focused or broad validation attempt, refuse an existing raw-log
path, stamp `ATTEMPT=`, exact `COMMAND=`, source HEAD/file hash and
`EXIT_CODE=` into that attempt's own log, then hash the closed log in its
receipt. Never repair missing stamps by rewriting historical raw output.

Build-environment traps (see also the repo-external notes in your memory):

- `unicorn-engine-sys` (cmake + bindgen) rebuilds from scratch after any
  toolchain change. bindgen needs `LIBCLANG_PATH=C:\Program Files\LLVM\bin`
  with that directory on `PATH`; its `libclang.dll` was verified present and
  used for the 2026-10-16 broad source suites. The user-supplied build copy
  at `D:\GitHub\llvm-proj\build\bin\libclang.dll` previously failed to load;
  do not rely on that build-output path without checking it afresh.
  The upstream cmake post-build step uses
  `create_symlink`, which fails without symlink privilege. Do not change system
  settings. Per D-007, the in-repo `vendor/unicorn-engine-sys` patch now copies
  instead of linking and has built in this checkout; do not create a source
  mirror or restore that symlink step.
- Use only this source checkout and the repo `target/` as the single disposable
  Cargo build directory. If MSVC C1041 proves that path too long, select one
  fixed short `CARGO_TARGET_DIR`, record it here, and retire the old build
  location after preserving evidence. Do not create per-experiment targets.
- Tests load `registry.reg`, `mounts.toml`, `serial_devices.json` and
  `host_progs/` relative to this repository's working directory.
- `target/` is disposable build output, not evidence storage. Move retained
  frames, logs and receipts into a separate evidence directory with a manifest
  and hash check before ever removing old target contents.

Mounted live run (the only meaningful launch; single-shot exits in ~1 s):

```powershell
cargo run --release --features unicorn,trace,win32-desktop -- `
  --registry registry.reg --devices serial_devices.json --mount-config mounts.toml `
  --image "D:\INAVI_Emulator\INAVI\INavi\iNavi.exe" `
  --dll-search-dir "D:\INAVI_Emulator\DUMPPLZ\Windows" `
  --desktop virtual --run-cpu --remote-server 192.168.0.39:8765
```

- Keep `--remote-server 192.168.0.39:8765` unless the user overrides it.
- Remote API: `GET /api/v1/frame.jpg`, `/api/v1/status`,
  `/api/v1/debug/{summary,windows,messages,imports,wndproc}.txt`; touch is
  `POST` with `{"type":"tap","x":400,"y":240}` (`type` is required). Debug
  snapshots mix live main-loop publication and retained CPU import/file
  snapshots. Capture timestamps/body/frame hashes and trust executed slices;
  alternating cached process snapshots do not establish guest progress.
- `--tracefile counts|milestones OUT.txt` dumps import counts/milestones.
- Logging: `RUST_LOG=ce.slice_profile=debug` (per-slice timing),
  `ce.quantum`, `ce.gwe`, `ce.imports`, `ce.scheduler`, `ce.devices`,
  `ce.file`. Many `WINCE_EMU_*` env switches exist (trace and experiment
  gates, e.g. `WINCE_EMU_SCOPED_HOOKS`, `WINCE_EMU_LIVE_PUMP_NATIVE_TIMEOUT_MS`);
  grep `src/emulator/unicorn.rs` before inventing a new one.
- Guest fixtures (eVC4 MIPS-II sources, one invariant each) are in
  `tests/test_progs/`; run them before blaming the real app.

## Code map

| Path | Role |
|---|---|
| `src/main.rs` | CLI, run orchestration, `DesktopRuntime` enum/implementation and desktop selection/wiring |
| `src/emulator/unicorn.rs` | Unicorn engine glue **and** the scheduler/slice loop (`run_with_unicorn`), hooks, parked processes, blocked/pending state. ~51k lines incl. tests |
| `src/emulator/{pe_loader,imports,dll_search,cpu_mips,...}.rs` | PE loading, import-trap table, DLL search, MIPS helpers |
| `src/ce/coredll.rs`, `coredll_ordinals.rs` | COREDLL ordinal dispatch and raw handlers (~69k + 14k lines; ~40% GWES/GDI/UI) |
| `src/ce/kernel.rs`, `scheduler.rs`, `object.rs`, `memory.rs`, `thread.rs`, `timer.rs` | CE kernel state, handles, waits, scheduler records |
| `src/ce/gwe.rs`, `resource.rs`, `shell.rs`, `framebuffer.rs` | windows/messages, resources, shell and framebuffer |
| `src/ce/win32_desktop.rs` | Win32 host input and presenter used by `DesktopRuntime` in `src/main.rs`; no `src/emulator/win32_desktop.rs` exists |
| `src/ce/file.rs`, `registry.rs`, `devices.rs`, `devices/` | host-backed storage, registry (`registry.reg`), virtual devices (`COM7:` GPS, `MFS1:`, `SMB1:`) |
| `src/winsock.rs`, `src/remote_server.rs`, `src/ce/audio.rs` | sockets, HTTP/WS control, waveOut |
| `tests/` | raw-API and subsystem tests, PE fixtures (`tests/test_progs`) |
| `host_progs/` | host-side helper programs (not part of the emulator build) |

## Current architecture status (refreshed 2026-10-17)

The user authorized the scheduler-core rewrite with plan-first/common-anchor
dependency gates. The reviewed docs describe target contracts and staged
gates, not completed architecture. The historical persistent-owner 650 trial
passed 1,618 tests but host-crashed at 129.108 s with `0xC0000005` and
1,662.59 MiB RSS. D-003 keeps native crash/cache work in a diagnostic lane:
fresh engine per slice is now the source-gated default, while persistent
engines are deferred to Stage 8. BED2 is the hash-preserved, measured
splash-only fallback, not the current source's release executable. E-38 is
the only bounded fresh-default iNavi run; the combined current source has
not been live-tested. Do not mistake unit success, native handle reuse or a
decorative splash for runtime acceptance.

Thirteen Task/handle controls formed the historical expected-red Stage 3
baseline (12 failures, one close-source preservation pass; ledger E-8/E-11).
The current D: source has source-gated canonical thread/process aliases,
checked creation ownership, exact-birth child completion, a bounded
`TerminateProcess` execution-stop handshake, and O-K3 execution/registered
Thread/Process wait pins
([receipt](evidence/stage3-pins-20261015/validation.receipt.md)). Opaque
non-task wait slots remain unpinned. The bounded
[stale-TID/native-stamp gate](evidence/stage3-stale-tid-20261015/validation.receipt.md)
birth-verifies pending returns, suspended contexts, native slice-end staging,
and the GetMessage bridge. The bounded
[Sleep/object parked-wait gate](evidence/stage3-parked-wait-20261015/validation.receipt.md)
verifies captured wait-owner birth before publication and wake consumption;
[parked MsgWait gate](evidence/stage3-msgwait-birth-20261015/validation.receipt.md)
extends that rule before timer pumping or message-edge consumption. The bounded
[parked queue/socket gate](evidence/stage3-queue-socket-birth-20261015/validation.receipt.md)
applies captured-birth checks to native MessageQueueIo/WinsockRead publication
and paired resume. The bounded
[parked serial gate](evidence/stage3-serial-birth-20261015/validation.receipt.md)
checks native SerialRead/SerialCommEvent register ownership before host serial
polling or wait publication and before paired wake/timeout completion. The
bounded [parked SendMessage gate](evidence/stage3-send-birth-20261015/validation.receipt.md)
checks native sender birth before queueing and the exact scheduler/native pair
before result consumption or register restoration; its tested source is sealed
by a Git HEAD and binary patch. The separate bounded
[kernel sender-exit gate](evidence/stage3-send-exit-20261015/validation.receipt.md)
removes queued sends for an exact verified exiting birth and detaches active
sends on its first successful live exit.
The bounded [native exit gate](evidence/stage3-native-exit-birth-20261015/validation.receipt.md)
checks captured thread birth and process lease before native `ExitThread` or
guest return-stub no-exit-candidate decisions; the return-stub hook itself has
no end-to-end test, and pending-return KData PID still follows the mutable
selected process.
The bounded [native SMT gate](evidence/stage3-smt-birth-20261015/validation.receipt.md)
checks eligible parked `BlockedSendMessageTimeout` publication, re-entry and
resume against captured birth and the exact wait/send pair; stale cleanup
abandons only the exact GWE send ID. The bounded
[occupied-slot gate](evidence/stage3-smt-occupied-20261015/validation.receipt.md)
stops an eligible second sender with a backend error instead of raw fallback;
it does not support concurrent A+B sends. Global timeout may appear as
ready/success, and receiver-side `DestroyWindow` result writes remain ungated.
Raw synchronous/nonblocking/overlapped serial paths, raw queue/socket and
MsgWait fallbacks, aliasless Sleep-suspend, other SMT semantics, receiver-side
delivery, and other parked-wait families remain open. These
slices do not complete the machine-owned Task/continuation/scheduler core,
Stage 3, or live-app acceptance. See `PROGRESS.md` and `docs/PLAN.md` for
current gates.

The [A10 ordinary Send/GetMessage](evidence/scheduler-source-cut-a10/validation.receipt.md)
and [A12 strict reciprocal Send](evidence/scheduler-source-cut-a12/validation.production.receipt.md)
source/test gates pass their bounded shapes. A11b's next-lease import replay
is historical for A12's one birth-owned, same-process plain S→T→S pair; timed,
result-pointer, third-sender, deeper-pending and cross-process cases are not
supported and can fail closed with terminal `Backend`. No A12 iNavi run or
writable-mount backup was made. Do not treat these slices as full Stage 3.

The bounded [decoded pseudo self-exit gate](evidence/stage3-raw-decoded-pseudo-exit-20261016/decoded-pseudo-exit.receipt.md)
requires an attempt-local executed JALR/delay-slot witness and exact stopped
caller/owner birth before the shared D-011-preflight finalizer. Its sealed
source passed 18-block release-feature **2,107/0/9 ignored** and no-default
**1,511/0/3 ignored**
([receipt](evidence/stage3-raw-decoded-pseudo-exit-20261016/broad-validation.receipt.md)).
Raw direct pseudo compatibility remains fixture-only and fail-closed; this is
not full Task/VM authority or live-app proof. No app or mount run was made.
The bounded [D-011 modal-presence refusal gate](evidence/stage3-modal-exit-failclosed-20261016/validation.receipt.md)
now preserves modal-bearing parked CPU subtrees and fails closed before
Kernel effects for three red-to-green stopped-exit shapes. Its sealed source
passed 19-block release-feature **2,137/0/10 ignored** and no-default
**1,524/0/3 ignored**; checked modal-pixel restoration and live-app proof
remain open. The later bounded
[parked-survivor/saved-CPU-owner gate](evidence/stage3-survivor-resume-green-20261016/validation.receipt.md)
preserves tested nonmodal foreign CPU trees on natural-parked and
requested-active stopped exits and refuses stale saved CPU thread ownership
before effects. Its combined sealed source passed release **2,144/0/10
ignored** and no-default **1,524/0/3 ignored**, each over 19 blocks. The
later [first-slice `Exited` helper gate](evidence/stage3-first-slice-survivor-green-20261016/validation.receipt.md)
synthetically preserves one A→B survivor shape; it does not execute guest
MIPS or prove classifier reachability. Its held-parent-queue conflict returns
typed terminal error but loses local A+B CPU ownership, not recovery. The
final source passed release **2,147/0/10 ignored** and no-default
**1,524/0/3 ignored** across 19 blocks each. A later
[real-MIPS non-live expected-red](evidence/stage3-real-mips-first-slice-20261016/validation.receipt.md)
at its earlier source seal executed guest `CreateProcessW`: A exited `0x63`,
B stayed Kernel-live, but its original CPU was dropped by the already-exited A
skip after rotation. That fixture was ignored at its earlier seal and later
retired after missing the post-deferral route; this does not exercise the
live-pump route. The separate
[post-admission native-fault red](evidence/stage3-post-admission-native-fault-20261016/validation.receipt.md)
at its earlier seal executed a synthetic child into `READ_UNMAPPED`; Kernel
left it live but the ordinary first-slice error dropped its CPU. That red is
historical; an ordinary real-MIPS Cut 3a replacement now passes focused tests.
The
[pre-CPU pending-launch retention gate](evidence/stage3-pre-cpu-launch-retention-green-20261016/validation.receipt.md)
now passes same-birth retry and queued-tail controls after file-view/path/PE
setup errors; it does not retain an admitted CPU after a native failure. The
[live-checkpoint lineage gate](evidence/stage3-checkpoint-orphan-frame-green-20261016/validation.receipt.md)
now refuses save/restore for retained canonical Kernel Task/launch authority
or CPU custody. A real monitor session cannot currently rewind; D-011
recovery requires repairing the cause and restarting until whole-machine
relineage exists. The gate is not general checkpoint acceptance. The
[counted checkpoint validation](evidence/stage3-checkpoint-unignored-20261017/validation.receipt.md)
passes 19-block no-default **1,532/0/3 ignored** and release-feature
**2,157/0/13 ignored** with all seven former ignores now ordinary green tests.
The [three pre-deferral offline reds](evidence/stage3-completed-reentry-red-20261017/validation.receipt.md)
reproduced `Completed` borrow/panic, already-exited A dropping B, and
native-fault child CPU loss at that earlier seal. The later
[deferred `CreateProcessW` source/test gate](evidence/stage3-createprocess-deferred-20261017/validation.receipt.md)
stages a nonzero parent return PC and launches the child post-native in both
modes; non-live rotation may select the child before the parent resumes.
Its 19-block release-feature **2,161/0/13 ignored** and no-default
**1,532/0/3 ignored** suites passed. Ignored current-source reruns 16/17
missed their old fixture preconditions, so they neither reproduced nor fixed
those failures; their stale ignored fixtures have since been retired. A new
direct-MIPS first-slice `Completed` control reaches window restoration with an
empty descendant queue and a held parent-queue read borrow; the old source
panicked there. The current source skips that unnecessary borrow and uses
fallible borrow/reservation for nonempty descendants, where failure remains
terminal after committed exit effects and can lose descendant CPU custody.
Its [bounded receipt](evidence/stage3-completed-borrow-20261017/validation.receipt.md)
seals `unicorn.rs` SHA-256 `0dcf5c5d04d0e632cbe405554bfc7029b7eea654070d4ca1fbdc3070ac136d06`:
focused **1/0**, 19-block release-feature **2,162/0/11 ignored**, and
no-default **1,532/0/3 ignored**. An ignored native-fault rerun at that
earlier seal lost an admitted child's CPU (exit 101).
This is not a CPU slot/Task port, Stage 5 scheduling gate or live result. The
later [test-only live-pump fixed-stack witness](evidence/stage3-live-pump-fixed-stack-hook-20261017/validation.receipt.md)
seals `unicorn.rs` SHA-256
`ff0bce3c3976c5f71bbcddbb2e35d182dd7d221552223529ed49b9eac720abd0`:
real-MIPS first-touch `SW`/`LW` passed explicit-PC/global-hook and default
hookless/count-limited modes (focused **2/0**, 19-block release-feature
**2,164/0/11 ignored**, no-default **1,532/0/3 ignored**). Source exclusivity
attributes the production memory-fault hook; no independent callback counter,
raw Kernel-pointer port, reentry safety or CPU custody is claimed. The
shipped PE [REA check](evidence/ce6-rea-inavi-20261016/findings.md) separately
identifies `happyway_win.exe` and `iSearch.exe` launch branches; static
analysis does not establish their runtime frequency. The
dumped COREDLL [REA wait/exit-code follow-up](evidence/ce6-rea-exit-20261016/findings.md#2026-10-16-bounded-waitchild-lifetime-follow-up)
and local CE6 source further distinguish loader completion, process waits,
termination requests and accepted exit; the ROM kernel exit body is unknown.
The [canonical-primary query gate](evidence/stage3-primary-canonical-green-20261017/validation.receipt.md)
now keeps the exact primary thread birth on the canonical process instead of
the drained launch queue/128-entry trace ring (release **2,169/0/10 ignored**,
no-default **1,537/0/2 ignored**). The dumped COREDLL
[REA check](evidence/ce6-rea-primary-thread-20261017/findings.md) supports
durable main-thread identity, not full exit semantics. The bounded
[exact-primary exit gate](evidence/stage3-primary-exit-birth-v4-20261017/validation.receipt.md)
authenticates tested stopped child/boot CPU ownership and refuses a missing
boot birth before effects (release **2,175/0/10 ignored**, no-default
**1,539/0/2 ignored**). Its first broad attempt failed 16 controls: 15 stale
synthetic fixture preconditions and one boot selected-owner regression. It was
corrected and superseded; no live iNavi run followed. Current-route
reachability of a nonempty `Completed` descendant publication conflict,
general exit reconstruction, later-slice survivor behavior, other framebuffer
reentry and exit paths remain open. The
[bounded scheduler-block self-reentry fence](evidence/stage3-scheduler-block-fence-20261017/validation.receipt.md)
passes a real-MIPS preseeded-busy control and 19-block release/no-default
suites (**2,176/0/10** and **1,539/0/2 ignored**), but has no cross-hook
exclusion or whole-attempt port. The
[bounded native-entry precondition](evidence/stage3-native-entry-precondition-20261017/validation.receipt.md)
refuses same-host-thread nested native entry before private CPU setup; a
preseeded-busy real-MIPS control retries on the same CPU. Sealed 19-block
release/no-default suites passed **2,178/0/10 ignored** and **1,539/0/2
ignored**. This is not a machine-owned leaf permit or a Kernel/framebuffer
port. The [latest admitted-child native-fault red](evidence/stage3-cpu-custody-current-red-20261017/validation.receipt.md)
is historical. Cut 3a now retains the original faulted B CPU in a machine-owned,
exact-birth, non-runnable slot after A's staged `CreateProcessW` return and
native sync/finish. Earlier focused debug/release fault controls passed; after
a producer-side pre-entry seal, same-source a08 synthetic pre-entry and a09
real-MIPS fault debug controls passed (`unicorn.rs` SHA-256 `1AAE8A66…33090`).
The superseded producer-only a10 focused release also passed. Current source
moves the terminal seal into `CpuCustody::quarantine_reserved` under Unicorn
on `Reserved`→`Quarantined`; final-source focused/broad release and no-default
counts and receipt are **pending**. A
`TerminateProcess` request does not retire B; checked accepted exit retires
the slot and its fixed-stack page in the focused native-fault shape. A
a07 expected red reproduced a post-load/pre-native-entry unpoisoned CPU that
the exit consumer refused to retire; the superseded producer-side a08 made
that synthetic precondition green. Custody now enforces the seal itself, with
the exit consumer's poison check retained. Natural reachability and general closure
remain unproved. Status/debug report retained mapped-blob
bytes, not measured RSS. The accepted-exit witness relies on the same Kernel
used by the production caller; it is not independently machine-lineaged.
Runnable/parked custody, public-run bypass removal, general private VM/view
cleanup, the callback-scoped Kernel/framebuffer port and live-app proof remain
open. The
[v2 offline preflight](evidence/stage3-preflight-v2-review-20261017/validation.receipt.md)
passes 171 source/build-chain controls and 83 runner guards; it requires a
future fresh output-hash-stamped rebuild and does not authorize a run.
No app or mount run was made.
Opt-in synthetic writable-main-image VM/pin and mutable-code source gates now
pass [bounded suites](evidence/stage2-mutable-code-fallback-20261016/broad-validation.receipt.md),
but normal SD-app loading remains copy-backed. D-012 keeps production EXE/DLL
images on that path until O-M3 physical-page accounting is known; synthetic
pinned-image ownership tests do not change the loader. The later bounded
[native-map failure](evidence/stage2-writable-image-native-failure-20261016/validation.receipt.md)
and [legacy file-view overlap](evidence/stage2-writable-image-file-view-20261016/broad-validation.receipt.md)
and [VM-allocation overlap](evidence/stage2-writable-image-vm-allocation-20261016/validation.receipt.md)
and [exact-process-exit/retained-pin](evidence/stage2-writable-image-exit-retire-20261016/validation.receipt.md)
synthetic gates pass; the intermittent WINDOWPOS test-thread stack overflow
and a separate full-suite access violation remain unexplained. Other direct
host-write/restore/rebind and module-retirement fences, preloaded/runtime DLL
ownership and O-M3 `PrivatePageBudget` treatment remain Stage 2 priorities
ahead of live acceptance or the next MsgWait gate.

The scheduler, message delivery, and CPU slicing are the weak spine. Measured
facts an agent must not re-derive or contradict (numbers and run conditions in
`PROGRESS.md`, 2026-10-12 section):

- Guest throughput was hook-bound, not instruction-bound. iNavi ran at ~1 MIPS;
  a bare Unicorn MIPS loop on the same host does ~3000 MIPS unhooked and ~400
  with a trivial per-instruction hook. The per-basic-block scheduler hook
  (`uc.add_block_hook` after `scheduler_timeslice_counter` in
  `run_with_unicorn`) used to query kernel/window state on **every block**;
  polling is now amortized (service point every 1024 blocks, at most once per
  ms) and hookless trampoline mode is the default
  (`WINCE_EMU_GLOBAL_CODE_HOOK=1` restores the global code hook): iNavi busy
  slices run at 22–34 MIPS. `ce.slice_profile` logs `guest_insns`/`mips`.
- Live-pump slices are still bounded by `emu_start(count = 1_000_000)`, which
  installs Unicorn's per-instruction counting hook and acts as the round-robin
  tick. `WINCE_EMU_LIVE_PUMP_RUN_SLICE_INSTRUCTIONS=0` is ~2x faster but
  changes scheduling dynamics (helper starvation, KNOWN_BUGS.md), so
  it stays opt-in until the scheduler has a real time-based quantum.
- Historical fresh-per-slice runs rebuilt Unicorn and lost the translation
  cache: fixed per-slice cost was 3.6% at ~300 ms slices but 15–25% at the
  65–140 ms slices fair round-robin produced. The current source uses a fresh
  native owner per slice; persistent-owner cache benefits remain a Stage 8
  measurement, not an accepted performance claim.
- TimerSystem mixes boot wall time with a sleep-driven virtual offset;
  there is no coherent centralized virtual scheduler or idle fast-forward.
  The former growing sensor-init retries are corrected by packed samples;
  current startup/profile facts must be taken from the latest bounded runs.
- Read-only files now use bounded 64 KiB windows, not whole-file retention.
  Matched 330 s live profiles reduced peak host RSS from ~1,300 to 562 MiB.
  The preserved BED2 run observed the first splash by +4.358 s, sampled peak
  RSS 555.633 MiB and a single FIFO tap's first callback after 0.397 s; these
  are not current-source runtime results or controlled improvements. All its
  sampled and terminal frames remained splash. Do not call splash timing full
  CE boot, or conflate guest heap, host RSS and virtual reservation.
- Blocking/callbacks are hand-built per API (`pending_*_returns` x16,
  `blocked_*`/`suspended_*`/`parked_*`), duplicated against
  `scheduler.rs::SchedulerBlockedWaitKind`.
- Scheduling is bimodal run to run (`happyway_win` fair vs starved) for the
  same build; judge changes by busy-slice MIPS and total guest instructions,
  not single-run milestone times.

Decided direction (phase 0 done except the slice-count item; the user explicitly
authorized beginning phase 1 on 2026-10-13 after discussion):

0. **Hot-path hygiene** — done: amortized block hook, hookless default,
   permanent MIPS logging. Open: replace the 1M instruction count with a
   time-based quantum (needs the scheduler work in phase 1).
1. **Machine-owned core** — one `Task` model (state, wait set, deadline,
   continuation frames) replacing the ad hoc vectors, CE priority/round-robin
   in virtual time with opt-in fast-idle. Persistent per-process engines are
   deferred to Stage 8 behind the CPU interface.
   The source-gated fresh-engine-per-slice lifetime (D-003) carries bounded
   canonical task identity and registered-wait pin slices; persistent engines
   are a later Stage 8 swap. Run mode is live-first with opt-in fast-idle
   (D-002). Full task state, continuations and virtual time are not yet
   implemented. Keep instruction count and current scheduling policy until
   their separate gates. Removing/reinstalling global hooks invalidates hooked
   translations, so persistence alone is not a cache/performance claim.
2. **Move the HLE boundary to the CE 6 syscall layer** (spike first): run the
   real `coredll.dll` over a Rust kernel (trap addresses `0xFFFFxxxx`,
   `FIRST_METHOD=0xFFFFFC02` in `nkmips.h`), serve each API set from Rust or
   from the real DLL. Real `gwes.dll` is kernel-mode at `0xC0330000` and would
   need TLB/kseg2 support — separate go/no-go.

Until phase 1 lands:

- Keep scheduler fixes minimal and evidence-based. Do not add new
  "visible window"/priority/rotation heuristics, new `pending_*`/`blocked_*`
  record types, or page/region-scoped code-hook variants.
- Any performance claim must come with guest instructions per slice (sum of
  block `size/4`) and MIPS, not just time-to-milestone. Compare on a clean SD
  image, `--release`, same boot phase.
- Prefer pull-based API work: implement what iNavi, MFC, or the dumped DLLs
  actually import (use `--tracefile counts`), not CE parity for its own sake.

## Working conventions

- Verify bounded: build → bounded test/run → read logs → then commit.
- Commit real fixes separately from diagnostics, temporary fallbacks,
  speculative workarounds, and formatting-only changes. Prefixes in use:
  `fix:`, `perf:`, `trace:`, `diag:`, `docs:`, `revert:`. Do not commit or push
  unless the user asks.
- After meaningful changes update `PROGRESS.md`, `TODO.md`, `KNOWN_BUGS.md`
  (no stale entries) and `README.md` if it went stale. `PROGRESS.md` holds
  confirmed facts only; reproducible failures go to `KNOWN_BUGS.md`.
- A false lead is worth recording in `PROGRESS.md` with how it was disproved.

## Gotchas

- **SD image markers**: zero-byte `INavi\traffictest.txt`, `INavi\routetest.txt`,
  `mapdata\soap.bin`, `mapdata\searchdb\oildata.dat` enable `happyway_win.exe`
  test modes and slow boot ~30x. Check for them before any boot-time bisect; if
  they reappear, an emulator path is materializing files on existence probes.
  Originals: `D:\INAVI_Emulator\_zero_byte_probe_artifacts_backup`.
- Stops requested from a per-instruction hook can land in a branch delay slot
  or inside a trampoline and lose the branch. Voluntary scheduler block
  stops skip possible page-crossing delay slots, but asynchronous native
  timeout branch-state preservation is not proven; keep count=0 opt-in.
- `happyway_win.exe` polls `FindWindowW(NULL, "utis_parser")` every second and
  keeps a WM_ACTIVATE queued forever (same on real hardware). Any "pending
  message = runnable priority" rule will starve iNavi.
- Recovery/cleanup heuristics that compare against stop snapshots must require
  evidence created after the state they clean up (see the escaped-callout fix,
  `2fcc365f`).
- The guest memory layout is CE 5-style (KData page at `0x5800`, import-trap
  page at `0x7fff0000`, DLLs near `0x6xxxxxxx`), whereas the real device is
  CE 6 (KData `0xFFFFC800`, user DLLs from `0x40010000`). Keep that in mind when
  comparing against dumped binaries.
