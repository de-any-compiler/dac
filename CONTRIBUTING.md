# Contributing to dac

Thanks for looking at dac. This document is the short version of how to send
a patch and what we expect from it. Read it together with
[PLAN.md](./PLAN.md) (what work is open) and
[ARCHITECTURE.md](./ARCHITECTURE.md) (how the codebase is shaped).

## Before you start

- **License.** Not yet selected — see [ADR-0001](./DECISIONS.md). Until it
  closes, external contributions are limited to issues and design discussion.
  Once the license lands, every PR must include a developer-certificate-of-
  origin sign-off (`git commit -s`).
- **Read the spec.** The numbered FR/NFR/I IDs are how we track what is and
  isn't done. Cite them in PRs (see "PR checklist" below).
- **Pick a batch from PLAN.md.** Work outside the plan is welcome but should
  open an issue first so we can fold it in.

## Development workflow

1. Open an issue for the batch (or comment on an existing one) so reviewers
   know who's holding it.
2. Branch off `main`. Branch names: `b<milestone>.<n>-<slug>`, e.g.
   `b1.3-x86-decoder`.
3. Land the batch in one PR when possible. If it has to split, each split
   must still be independently mergeable and tested.
4. Open the PR against `main`. CI must be green before review.

## Local checks

We use an `xtask` crate as the canonical entrypoint:

```bash
cargo xtask ci          # fmt + clippy + test + deny — what CI runs
cargo xtask test        # tests only
cargo xtask fuzz <name> # run a fuzz target for 60s
cargo xtask bench       # micro-benchmarks
cargo xtask golden update    # regenerate golden files (review the diff)
```

Direct cargo commands work too; `xtask ci` exists to make "did I run the
right things?" trivial.

## Code style

- `rustfmt` is enforced.
- `clippy -- -D warnings` is enforced.
- No `unwrap()` or `expect()` in non-test code outside of `dac-cli`'s
  startup path. Anything that touches input bytes returns `Result`.
- Comments: explain *why*, not *what*. The code already says what.

## Tests we expect with a batch

| If your batch touches…       | Add at least…                                                |
| ---------------------------- | ------------------------------------------------------------ |
| A parser or decoder          | A `cargo-fuzz` target in `fuzz/` and a corpus seed           |
| An analysis pass             | Unit tests + golden tests for the pass output                |
| A backend                    | Round-trip: emitted source compiles, with a CI gate          |
| The CLI                      | A snapshot test of `--help` and the affected subcommand      |
| Anything declared `Pure`     | A determinism test: run twice, diff the manifest             |
| AI plumbing                  | A test using the `null` provider so CI never calls the network |

Tests live next to the code they cover (`mod tests` for units, `tests/` for
integration, `tests/golden/` for goldens, `fuzz/` for fuzz targets).

## When to add an ADR

If your batch makes a choice that future-you would want to *justify*, write
an ADR in [DECISIONS.md](./DECISIONS.md). Rule of thumb: if the answer to
"why this and not the obvious alternative?" is more than two sentences, it's
an ADR.

## PR checklist

Copy this into your PR description and tick it off:

```
- [ ] Closes batch B<milestone>.<n>
- [ ] Closes spec items: FR-…, NFR-…, I-…
- [ ] `cargo xtask ci` passes
- [ ] Added/updated tests per "Tests we expect with a batch"
- [ ] Added an ADR if the change involved a non-obvious choice
- [ ] CHANGELOG.md updated — moved the batch's entry from PLAN.md
- [ ] Updated docs touching anything user-visible (CLI flags, public API)
```

## CHANGELOG hygiene

When a batch lands, move its entry from `PLAN.md` to `CHANGELOG.md` under
`Unreleased`. PRs that change behavior but do not finish a batch get a
sub-bullet under the closest batch. PRs that are pure refactors or test
additions can skip the changelog if nothing user-visible changed.

## Reviews

- One reviewer's approval is enough for cross-cutting infra changes
  (`xtask`, CI, docs).
- Two reviewers' approvals for changes inside `dac-core`, `dac-ir`, or
  `dac-verify` — these are the load-bearing parts of the invariants.
- A `Determinism::NonDeterministic` pass landing requires explicit
  reviewer acknowledgment that the pass is opt-in and `--deterministic`
  rejects it.

## Be kind, be specific

dac is built around the idea that *every claim it makes about a binary
should be traceable to evidence*. We try to hold ourselves to the same
standard in review: feedback should point at a concrete line, an invariant,
or a measurable thing — not vibes.
