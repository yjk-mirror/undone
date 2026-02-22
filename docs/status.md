# Undone — Scaffold Status

Living document. Update after each task. Replaces the scaffold table in HANDOFF.md.

**Plan:** `docs/plans/2026-02-21-scaffold.md`
**Branch:** `master` (scaffold started before worktree workflow was established)

---

## Task Status

| # | Task | Status | Commit | Notes |
|---|---|---|---|---|
| 1 | Cargo workspace + 7 crate stubs | ✅ Done | `5561ed0` | All 7 crates compile clean |
| 2 | Engine-level enums (`undone-domain`) | ✅ Done | `154b119` | 3/3 tests pass; added `serde_json` as dev-dep |
| 3 | Content ID newtypes (`undone-domain`) | ✅ Done | `19076ac` | Clean build |
| 4 | Player struct (`undone-domain`) | 🔲 Pending | — | |
| 5 | NpcCore, MaleNpc, FemaleNpc (`undone-domain`) | 🔲 Pending | — | |
| 6 | World + GameData (`undone-world`) | 🔲 Pending | — | |
| 7 | Pack manifests + base data TOML files | 🔲 Pending | — | |
| 8 | PackRegistry with lasso interning | 🔲 Pending | — | |
| 9 | Expression lexer (`undone-expr`) | 🔲 Pending | — | |
| 10 | Expression parser — recursive descent AST | 🔲 Pending | — | |
| 11 | Expression evaluator + SceneCtx | 🔲 Pending | — | Stubs intentional — wired in scene engine session |
| 12 | Minimal eframe window (`undone-ui`) | 🔲 Pending | — | |
| 13 | Final verification (test + clippy + release) | 🔲 Pending | — | |

---

## Test Counts

| Crate | Tests |
|---|---|
| `undone-domain` | 3 |
| `undone-world` | 0 |
| `undone-packs` | 0 |
| `undone-expr` | 0 |
| **Total** | **3** |

Target at scaffold completion: ~20 (5 lexer + 7 parser + 7 eval + 1 manifest + 3 registry + 2 domain-domain)

---

## Tooling Notes (this session)

- Rust MCP tools (`mcp__rust__get_diagnostics`, `mcp__rust__format_code`) not used for
  Tasks 1–3 — workflow established mid-session. Use from Task 4 onward.
- Worktree not created — scaffold started on `master` before worktree rule was established.
  Future plans should use `superpowers:using-git-worktrees` before touching code.

---

*Last updated: 2026-02-22 — Tasks 1–3 complete*
