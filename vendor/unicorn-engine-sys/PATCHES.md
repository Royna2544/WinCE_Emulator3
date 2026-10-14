# Local patches to `unicorn-engine-sys` 2.1.5

The repository uses this vendored native crate through the root
`[patch.crates-io]` entry. The separate `unicorn-engine` Rust wrapper remains a
registry dependency. These changes are local to this repository; no upstream
submission or merge is recorded here.

| Patch | Files | Purpose and status |
| --- | --- | --- |
| Windows CMake alias copy (D-007; introduced with vendoring in `ca477366`) | `cmake/bundle_static.cmake` | Its post-build command uses `cmake -E copy` to create the static-library alias instead of requiring symlink privilege on Windows. Local; not recorded as submitted upstream. |
| Additive native count-stop query (O-P1; `05eed768`) | `include/unicorn/unicorn.h`, `include/uc_priv.h`, `uc.c` | Adds `UC_QUERY_COUNT_STOP` and per-start native state. The count hook marks a successful stop request, start resets the observation, stop reversal clears it, and a same-handle nested start makes the query return `UC_ERR_RESOURCE` rather than a guessed result. This is a stop-request witness, not an exclusive cause or retired-instruction count. Local; not recorded as submitted upstream. |
| Windows TCG commit-tail clamp (E-5/E-6/E-32) | `qemu/accel/tcg/translate-all.c` | Caps each VEH commit at the remaining reserved code arena. Small pre-init MIPS tests reproduced the pristine access violation and pass with this local fix. This does not establish the E-32 app crash cause or cover concurrent VEH teardown. Local; not recorded as submitted upstream. |

The count query is intentionally additive to Unicorn's C API. The vendored
build generates the Rust `Query` binding from the header; generated bindings
and the registry Rust wrapper are not hand-patched here. Validation and any
future upstream submission need separate evidence.
