# eVC4 MIPSII Build Notes

These are notes, not a required runtime dependency.

## Automatic test build

The ignored fixture integration test builds missing or stale MIPSII EXEs from
these sources and runs them through the emulator:

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

The test writes EXEs, fixture DLLs, import libraries, and intermediates under
`target/wince-fixtures/mipsii/`. Runtime files are staged under
`target/wince-fixtures/sdmmc/<fixture-name>/` before the emulator run.

## Fixture DLLs

A fixture can include runtime DLLs under:

```text
tests/test_progs/<fixture>/dlls/<dll-name>/
```

Each DLL directory may contain `.cpp`, `.rc`, and one optional `.def` file.
The runner links it as `<dll-name>.dll`, stages it beside the fixture EXE, and
keeps the generated import library under the fixture output directory. Use a
`.def` file for ordinal-only exports.

Fixture DLLs are built in sorted directory-name order. Each later DLL is linked
against import libraries produced by earlier DLLs in the same fixture, so name
low-level dependency DLL directories before the DLLs that import from them.

## Important

The emulator runtime must not invoke eVC4.
These EXEs are generated test artifacts and must not be committed.

## Dynamic MIPS code fixtures

`004_delay_slot_dynamic` and `005_branch_likely_dynamic` generate small MIPS
code buffers at runtime. They are meant to test CPU execution semantics even
before there is a convenient MIPS assembler path.

If a real device fails these due to instruction-cache coherency, verify
`FlushInstructionCache` is imported/resolved.
