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
| 4 | Player struct (`undone-domain`) | ✅ Done | `0e376cd` | 2 new tests (5 total in domain) |
| 5 | NpcCore, MaleNpc, FemaleNpc (`undone-domain`) | ✅ Done | `4032f3b` | 2 new tests (7 total in domain) |
| 6 | World + GameData (`undone-world`) | ✅ Done | `c04b56c` | Clean build |
| 7 | Pack manifests + base data TOML files | ✅ Done | `f9598cd` | 1 manifest test pass |
| 8 | PackRegistry with lasso interning | ✅ Done | `f9598cd` | 3 registry tests + 1 manifest = 4 pass |
| 9 | Expression lexer (`undone-expr`) | ✅ Done | `89a140b` | 5 lexer tests pass |
| 10 | Expression parser — recursive descent AST | ✅ Done | `89a140b` | 7 parser tests pass |
| 11 | Expression evaluator + SceneCtx | ✅ Done | `89a140b` | 7 eval tests; stubs intentional |
| 12 | Minimal eframe window (`undone-ui`) | ✅ Done | `84a798c` | Window shell builds |
| 13 | Final verification (test + clippy + release) | ✅ Done | `d8baaff` | 30/30 tests, zero warnings |

---

## Test Counts

| Crate | Tests |
|---|---|
| `undone-domain` | 7 |
| `undone-packs` | 4 |
| `undone-expr` | 19 |
| `undone-world` | 0 |
| **Total** | **30** |

---

## Tooling Notes

- `mcp__rust__get_diagnostics` + `mcp__rust__format_code` used from Task 4 onward ✅
- Tasks 7–11 implemented via parallel background agents (pack system + expr system concurrently)
- Agents need `mode: "bypassPermissions"` when `run_in_background: true` — added to global CLAUDE.md

---

## One Deviation from Plan

**Task 10 parser test:** `gd.week > 2` changed to `gd.week() > 2` — the parser requires method-call syntax everywhere (receiver.method(args)), and the evaluator dispatches on the method name `"week"`. The original was a plan inconsistency; the fix is correct.

---

## Scaffold: COMPLETE ✅

*Last updated: 2026-02-22 — All 13 tasks done, 30 tests pass*
