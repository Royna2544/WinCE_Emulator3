# RULES.md - FakeCE / iNavi SE G3 Emulator

## Project

Target app:

- iNavi SE G3
- Windows CE GUI application
- MIPS R4000 PE
- Uses COREDLL.dll, mfcce400.dll, CRT-like exports, WINSOCK.dll

Runtime DLL source for mounted iNavi runs:

- `D:\INAVI_Emulator\DUMPPLZ\Windows`

Treat this dumped Windows directory as the true source of target DLL images.
Use SDK paths below only as import-library/header/source evidence or fallback
when a dumped runtime DLL is genuinely unavailable.

Installed Windows CE 4.2 Standard SDK import libraries:

- `C:\Program Files (x86)\Windows CE Tools\wce420\STANDARDSDK_420\Lib`
- `C:\Program Files (x86)\Windows CE Tools\wce420\STANDARDSDK_420\Mfc\Lib`

For this target, use `Mipsii` as the primary SDK directory.

The Visual Studio installation is at: `C:\Program Files\Microsoft Visual Studio\18\Community`

Windows CE Core-OS source is at: `C:\WINCE600`

MFC reference source is at: `C:\Program Files (x86)\Microsoft Visual Studio 8\VC\ce\atlmfc\src\mfc`

Device sd-card firmware (see mounts.toml): `D:\INAVI_Emulator\INAVI`

Main target right now: `D:\INAVI_Emulator\INAVI\INavi\iNavi.exe`

LLVM Tools for target=MIPS: `D:\GitHub\llvm-proj\build-mips-objdump\bin`

eVC4 toolchain for testsuite: `C:\Program Files (x86)\Microsoft eMbedded C++ 4.0\EVC\wce420\bin`

---

## Hard User Rule

This is an emulator.

Do not fake custom app behavior just to make a screenshot look correct/make it work.

Prefer real-ish emulation:

Guest app behavior → MIPS CPU execution → PE imports / COREDLL shims → host-backed file, drawing, audio, registry, timer, and device behavior.

Allowed:

- Targeted tracing
- Targeted guards
- Known-address diagnostics
- Temporary breakpoints/watchpoints
- Host-backed shims for real API semantics
- Referencing sources for diagnostics are very welcome.

Dangerous unless explicitly justified:

- Manually painting to match app behavior
- Forcing app-specific callbacks
- Replacing app state with guessed values
- Hardcoding iNavi behavior as the final fix
- Pretending success while bypassing the real app path
- Specifying special behavior based on hardcoded strings or file names.

Forbidden:
- Running recursive find over too broad directory, e.g. /mnt/c/. Will you pay for my disk TBW?
- Inventing custom behavior
- Quitting investigation unless the user permits, do not think you own.

IMPORTANT: DO. NOT. INVENT BEHAVIOR, ADD HARDCODED SPECIFIC FIX, QUIT INVESTIGATION UNLESS THE USER ASKS TO. KEEP RUNNING AUTO TESTS

---

## Development Discipline

Before making changes:

1. Read this file.
2. Read `PROGRESS.md`, `TODO.md`, and `KNOWN_BUGS.md`.
3. Check current git status.
4. Understand whether the current state is a committed fix, diagnostic experiment, broken regression, or untracked artifact.

After meaningful changes:

1. Build.
2. Run a bounded test.
3. Inspect logs/output.
4. Commit real fixes separately from diagnostics. Use git with proper commit message.
5. Clean up / Update `PROGRESS.md`, `TODO.md`, and `KNOWN_BUGS.md`. So there will be no stale entries.
6. Update `README.md` if it contains any stale entries after the change. Always verify against the latest sources.

Do not mix these in one commit:

- Real emulator fix
- Diagnostic hook
- Temporary fallback
- Speculative app-specific workaround
- Formatting-only cleanup

Good commit examples:

- `fix: implement MIPS branch-likely delay-slot annulment`
- `fix: decode indexed DIB byte-plane lower splash slice`
- `trace: add PNG cleanup saved-register watch`
- `diag: log MFC CFile ordinal arguments around mapinfo.bin`
- `revert: remove broad DIB fallback experiment`

---

## Persistent Project Memory Files

Maintain these files as durable memory across Codex sessions:

- `RULES.md`
- `PROGRESS.md`
- `TODO.md`
- `KNOWN_BUGS.md`

`PROGRESS.md` is for confirmed facts only. Include what works, what was fixed, what was a false lead, what regressed, current last-known state, and important commit hashes.

`TODO.md` is for active next steps. Keep sections like Immediate, Next, Later, Parked.

`KNOWN_BUGS.md` is for reproducible failures. Include symptom, current hypothesis, evidence, relevant addresses/ordinals/logs, and status.

---

## MIPS / CPU Context

If PC becomes `0x0`, it is not a normal guest exit. Treat it as an
emulator control-flow/resume/return-address bug unless proven otherwise, and
make runtime code fail the run instead of reporting success.
