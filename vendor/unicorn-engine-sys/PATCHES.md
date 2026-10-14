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

## O-P4 native owner lifetime audit (2026-10-14)

Fresh-per-slice closure relies on this exact vendored 2.1.5 source, not on an
upstream guarantee: `uc.c:497-570` shows `uc_close` freeing the owner and
returning `UC_ERR_OK` on both its pre-init and initialized paths. A successful
`uc_hook_add` initializes the engine (`uc.c:1914-1934`, `:304`); subsequent
`uc_hook_del` can return an error only through `UC_INIT` (`uc.c:2042-2066`).
Deleted hooks are marked `to_delete`, skipped by dispatch, and native close
clears deferred and remaining hook lists (`uc.c:560-564`). The Rust wrapper's
`Drop` does not expose the C return, so this is a pinned-source proof, not a
general checked-close API. Fresh leases close their owning wrapper before
dropping the nonowning run wrapper, callback boxes or pointer backing pins.

Before changing Unicorn C, its Rust wrapper, crate version, or build options,
re-audit those exits and hook/owner lifetimes and rerun the O-P4 closed-owner
and churn gates. If `uc_close` can fail with an uncertain side effect, the
fresh path must stop until a consuming checked-close/quarantine design is
implemented; ordinary wrapper `Drop` would not be sufficient.
