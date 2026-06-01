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
