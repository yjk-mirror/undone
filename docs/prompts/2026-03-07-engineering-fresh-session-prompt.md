Work in `C:\Users\YJK\dev\mirror\undone`.

This is a fresh engineering session after the ironclad-engine hardening sprint completed and after documentation-only research on writing-agent tooling.

First read:

- `HANDOFF.md`
- `docs/engine-contract.md`
- `docs/audits/2026-03-07-engine-readiness-matrix.md`
- `docs/audits/2026-03-07-writing-agent-tooling-audit.md`
- `docs/research/2026-03-07-writing-context-lorebooks-and-caching.md`

Current state to assume:

- the engine hardening sprint is complete
- working tree should be clean
- no new code was added in the writing-agent research pass
- writing-agent/tooling findings are documented, not implemented

Session goal:

1. Continue engineering work from current repo reality, not stale assumptions.
2. Prefer correctness, enforceable contracts, and tests over convenience.
3. Do not start real content writing.

How to choose the next task:

- Start by reconciling `HANDOFF.md` against the current code and docs.
- If there is no blocking engine correctness issue, pick the highest-value remaining engineering task that reduces false assumptions for future writers or tool builders.
- Treat writing-agent findings as input for future tooling work, not as permission to start content work.

Good candidate engineering directions:

- refine `validate-pack` warning-vs-error policy where current warnings are too weak or too noisy
- clean up remaining writer-facing contract mismatches if they create false assumptions about runtime behavior
- audit remaining hardcoded content-ID assumptions in engine/runtime code
- prepare lightweight authoring-context infrastructure only if it can be done as engineering/tooling, not content writing

Rules:

- prefer code/tests/contracts to comments
- if a contract changes, update `docs/engine-contract.md`
- if readiness status changes, update `docs/audits/2026-03-07-engine-readiness-matrix.md`
- if writing-agent/tooling contracts change, keep `docs/audits/2026-03-07-writing-agent-tooling-audit.md` and `docs/research/2026-03-07-writing-context-lorebooks-and-caching.md` in sync
- keep DeepSeek subordinate only

Required final verification if code changes are made:

- `cargo fmt --all`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run --bin validate-pack`
- `node tools/deepseek-helper.mjs --help`

Bias toward tasks that make future writing or tooling sessions safer by removing ambiguity, invalid assumptions, or silent failure paths.
