# KNOWN_BUGS

## Open

- CPU execution is not available in the default build.
  - Symptom: `--run-cpu` reports that the binary was built without the `unicorn`
    feature.
  - Evidence: PE image mapping and import traps are not implemented yet.
  - Status: expected initial scaffold limitation.

- PE loading is inspection-only.
  - Symptom: `--image` validates the MZ header and reports `e_lfanew`, but does
    not map sections.
  - Evidence: `src/pe/mod.rs` only performs minimal image inspection.
  - Status: next implementation step.

- COREDLL dispatcher is not connected to guest import traps yet.
  - Symptom: subsystem APIs can be exercised from Rust tests, but guest PE calls
    do not dispatch into them.
  - Evidence: `src/ce/coredll.rs` resolves and dispatches typed calls, but no
    MIPS import thunk/trap argument decoder exists yet.
  - Status: expected until PE mapping and import trap work lands.
