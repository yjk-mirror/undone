Work in `C:\Users\YJK\dev\mirror\undone`.

Continue the ironclad-engine hardening sprint from commit `72e90d4`.

This is a correctness-and-readiness sprint, not a content sprint.

First read:

- `HANDOFF.md`
- `docs/plans/2026-03-07-ironclad-engine-hardening.md`
- `docs/engine-contract.md`
- `docs/audits/2026-03-07-engine-readiness-matrix.md`
- `docs/content-schema.md`
- `src/bin/validate_pack.rs`

Current baseline already completed:

- pack/bootstrap fail-fast validation is in place
- schedule conditions/triggers and entry-scene refs are validated at startup
- char-creation trait dependencies are startup-validated
- readiness matrix and engine contract docs exist
- DeepSeek subordinate helper exists at `tools/deepseek-helper.mjs`

Primary goal for this session:

1. Make the engine more ironclad before content writing begins.
2. Focus on the remaining engineering hardening items, especially save/resume, diagnostics, and authoring validation.
3. Reconcile all work against current code, not old assumptions in older handoff notes.

Priority order:

1. Save / resume hardening
2. Runtime diagnostics hardening
3. `validate-pack` / authoring validation hardening
4. NPC context contract decision or reinforcement
5. Final docs sync

Constraints:

- do not start new real content writing beyond tiny fixtures/tests if needed
- prefer contract enforcement and tests over comments and intentions
- if a contract is unclear, update `docs/engine-contract.md`
- if readiness status changes, update `docs/audits/2026-03-07-engine-readiness-matrix.md`
- keep DeepSeek as a subordinate helper only

Expected deliverables:

- code and tests for the hardening work you complete
- updated docs reflecting the new truth
- final verification with:
  - `cargo fmt --all`
  - `cargo check --workspace`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo run --bin validate-pack`
  - `node tools/deepseek-helper.mjs --help`

When choosing where to start, bias toward the highest-risk correctness gaps that could let writers build on a false assumption.
