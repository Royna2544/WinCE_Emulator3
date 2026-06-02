# eVC4 MIPSII Build Notes

These are notes, not a required runtime dependency.

## Automatic test build

The ignored fixture integration test builds missing or stale MIPSII EXEs from these sources and runs them through the emulator:

```powershell
cargo test --features "unicorn evc4-fixtures" --test fixture_exes -- --ignored
```

Configure the toolchain with environment variables:

- `WINCE_EVC4_MIPSII_CC`: eVC4 MIPSII C/C++ compiler.
- `WINCE_EVC4_MIPSII_LINK`: eVC4 linker.
- `WINCE_EVC4_MIPSII_INCLUDE`: SDK/eVC4 include directories, separated with the platform path separator.
- `WINCE_EVC4_MIPSII_LIB`: SDK/eVC4 library directories, separated with the platform path separator.
- `WINCE_EVC4_RC`: resource compiler, required only for fixtures with `.rc` files.
- `WINCE_EVC4_MIPSII_CFLAGS`, `WINCE_EVC4_MIPSII_LFLAGS`: optional extra flags.
- `WINCE_EVC4_FORCE_REBUILD=1`: optional rebuild override.

The test writes EXEs and intermediates under `target/wince-fixtures/mipsii/`.

## Important

The emulator runtime must not invoke eVC4.
These EXEs are generated test artifacts and must not be committed.

## Dynamic MIPS code fixtures

`004_delay_slot_dynamic` and `005_branch_likely_dynamic` generate small MIPS code buffers at runtime.
They are meant to test CPU execution semantics even before there is a convenient MIPS assembler path.

If a real device fails these due to instruction-cache coherency, verify `FlushInstructionCache` is imported/resolved.
