# Golden corpus

Recorded outputs of the `dac` CLI run against the shared fixtures under
`../fixtures/`. The corpus is the long-term drift gate for B2.9 (NFR-9):
`cargo xtask golden check` re-runs every case and fails CI if any byte
of any captured sidecar differs from what is recorded here.

## Layout

```
tests/golden/
  <case-name>/
    listing.txt       # the dac `--output` file
    manifest.json     # the `<output>.manifest.json` sidecar
    report.txt        # the `<output>.report.txt` sidecar (--emit-report)
    cfg.dot           # the `<output>.cfg.dot` sidecar (--emit-cfg)
    source.c          # the `<output>.c` sidecar (--target c at -O1+)
```

Each case is one row in the `CASES` array in `xtask/src/golden.rs` —
fixture name + CLI flag set + the sidecars to capture.

## Refreshing the goldens

After an intentional change to a renderer (listing, manifest, report,
DOT, or the C backend), regenerate the recorded bytes:

```
cargo xtask golden update
```

The xtask invokes `dac` once per case under workspace-relative paths
(so the `input.path` field in the manifest does not depend on the
developer's home directory), then writes each captured sidecar into
its slot under `tests/golden/<case>/`. A diff in `git status` is the
intended outcome — review the change, commit it alongside the code
change that produced it, and reference the batch in the commit
message.

## Adding a case

1. Add a fixture under `../fixtures/` if you need a new input.
2. Add a `Case { … }` entry to `CASES` in `xtask/src/golden.rs`.
3. Run `cargo xtask golden update`.
4. `cargo xtask ci` to confirm the harness round-trips cleanly.

## CI gate

`cargo xtask ci` calls `golden::run(Mode::Check)` after the test
suite, so drift fails the same canonical command developers use
locally. Failures point to the first differing line, the byte
counts, and a hint to re-run `cargo xtask golden update`.
