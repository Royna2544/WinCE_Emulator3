# KNOWN_BUGS

## Open

- CPU execution is not available in the default build.
  - Symptom: `--run-cpu` reports that the binary was built without the `unicorn`
    feature.
  - Evidence: PE image mapping and import traps are not implemented yet.
  - Status: expected initial scaffold limitation.

- Parsed PE images are not mapped into Unicorn yet.
  - Symptom: `--image` parses PE32 headers, sections, imports, exports, and
    relocations, but CPU execution still does not load the mapped image.
  - Evidence: `src/pe/mod.rs` can produce mapped image bytes; no Unicorn loader
    consumes them yet.
  - Status: next implementation step.

- COREDLL dispatcher is not connected to guest import traps yet.
  - Symptom: subsystem APIs can be exercised from Rust tests, but guest PE calls
    do not dispatch into them.
  - Evidence: `src/ce/coredll.rs` resolves and dispatches typed calls, but no
    MIPS import thunk/trap argument decoder exists yet.
  - Status: expected until PE mapping and import trap work lands.

- Most COREDLL ordinals are still subsystem stubs.
  - Symptom: every parsed ordinal has subsystem ownership and raw dispatch
    metadata, but only the implemented virtual Win32/CE facade, waveOut, and
    `cemath` paths have real semantics.
  - Evidence: `src/ce/coredll.rs` reports implemented-vs-stubbed ordinal plan
    entries and returns subsystem stub policies for remaining exports.
  - Status: active ordinal-by-ordinal implementation work.

- Remote API has no Rust socket transport yet.
  - Symptom: remote touch/key/GPS/audio/status behavior exists as emulator API
    state, but there is no HTTP/WebSocket listener serving `/api/v1/...`.
  - Evidence: `src/ce/remote.rs` implements state and control dispatch only.
  - Status: expected until host transport work lands.
