# Writing Agent Tooling Audit

Date: 2026-03-07

## Scope

This audit covers the current repo-local writing-agent path:

- `.claude/agents/scene-writer.md`
- `.claude/agents/writing-reviewer.md`
- `CLAUDE.md`
- `docs/writing-guide.md`
- `docs/deepseek-writing-tool.md`
- `tools/deepseek-helper.mjs`

This is a documentation-only audit. No writing-agent behavior changes were made in this session.

## Executive Summary

The current writing-agent setup is usable for Claude Code and is directionally sound:

- dedicated `scene-writer` and `writing-reviewer` agents exist
- roles are separated cleanly
- reviewer is read-only
- agent MCP exposure is restricted to `minijinja`
- DeepSeek is correctly treated as a subordinate helper

However, it is not yet in a best-practice shape for low-friction, low-token, cache-friendly long writing sessions.

The main gaps are:

1. One writer-facing contract mismatch around `m` / `f` in prose templates
2. Large duplicated prompt mass across `CLAUDE.md`, the agent files, and `docs/writing-guide.md`
3. No repo-neutral dispatch doc for non-Claude orchestrators
4. No cache observability or prompt-budget guardrails in `tools/deepseek-helper.mjs`

## Current State

### What currently works

- `CLAUDE.md` documents how Claude agents should dispatch `scene-writer` and `writing-reviewer`.
- Both custom agents are checked into the repo and have explicit frontmatter.
- `scene-writer` can write files and run template validation.
- `writing-reviewer` is read-only and focused on reporting findings.
- `docs/deepseek-writing-tool.md` keeps the DeepSeek boundary subordinate and safety-conscious.
- `tools/deepseek-helper.mjs` is intentionally simple and stable: one system message, one user prompt, one completion.

### What currently does not exist

- No repo-level `AGENTS.md` or other orchestrator-neutral dispatch spec for these writing agents
- No automatic lorebook, retrieval, or context-pruning system for writing delegation
- No local cache/replay layer for the DeepSeek helper
- No helper-level reporting of DeepSeek cache hit/miss usage

## Findings

### 1. Contract mismatch: `m` / `f` in prose templates

This is the highest-signal correctness problem in the current writing-agent docs.

- `docs/writing-guide.md` says `m` and `f` are not available in prose templates and should be used only in `condition` fields.
- `.claude/agents/scene-writer.md` lists `m.` and `f.` as prose-template objects.

Given the current engine contract and recent hardening work, the writing guide is the safer statement to trust. Writers should not be taught to assume intro/template-time NPC bindings that are not guaranteed.

Status: documented, not fixed in this session.

## 2. Prompt surface is too heavy

The repo repeats the same writing law in several places:

- `docs/writing-guide.md`
- `.claude/agents/scene-writer.md`
- `.claude/agents/writing-reviewer.md`
- `CLAUDE.md`

This has two costs:

- higher startup/context cost for fresh subagents
- worse prompt-cache efficiency because more static text is duplicated across multiple layers

The current shape is workable, but not minimal-friction.

## 3. Claude-specific dispatch is clear; generic dispatch is not

For Claude Code specifically, the path is clear:

- `CLAUDE.md` names the agents
- `CLAUDE.md` explains when to use them
- `CLAUDE.md` explains the per-scene workflow

For Codex or another orchestrator, that workflow is discoverable only by reading Claude-specific docs. That is acceptable for now, but it is not ideal if writing delegation is meant to be a repo capability rather than a Claude-only convention.

## 4. DeepSeek helper is stable but under-instrumented

`tools/deepseek-helper.mjs` currently does a good job of staying simple:

- minimal request shape
- no hidden repo scraping
- no secret printing
- usage reporting

But it lacks several things that would matter before a serious writing sprint:

- prompt cache hit/miss reporting
- oversized prompt warnings
- retry/backoff behavior
- a standard prompt skeleton that keeps stable prefixes stable
- any local dedupe layer for identical prompt files

None of these are correctness blockers today, but they are real workflow-quality and cost-control opportunities.

## Recommended Future Changes

These are intentionally ordered by leverage, not by implementation difficulty.

### Priority 1: fix the contract mismatch

- Align `.claude/agents/scene-writer.md` with `docs/writing-guide.md` and `docs/engine-contract.md`
- Make the `m` / `f` intro/prose limitation explicit and hard to miss

### Priority 2: thin the agent prompts

- Make `scene-writer` and `writing-reviewer` thinner wrappers around `docs/writing-guide.md`
- Keep only agent-specific workflow and refusal rules in the agent files
- Remove duplicated long-form rulebooks from the agent prompt bodies

### Priority 3: add a repo-neutral writing delegation doc

Add one stable doc that explains:

- what the writer agent needs as input
- what the reviewer agent needs as input
- what must never be sent to DeepSeek
- what validations must run after draft generation

That doc should not assume Claude-specific `subagent_type` mechanics.

### Priority 4: instrument the DeepSeek helper

Before the next writing sprint, strongly consider:

- printing `prompt_cache_hit_tokens`
- printing `prompt_cache_miss_tokens`
- warning on prompt files above a configurable size threshold
- optional canonical prompt wrappers for `draft` and `review`

### Priority 5: move toward retrieval-style context packing

The current workflow is doc-heavy and manual. The future shape should be:

- stable, cache-friendly instruction prefix
- tiny scene-specific task payload
- only the relevant arc / NPC / style material for this scene
- no full writing-guide dump unless needed

That is the best path to low cost and high consistency.

## Bottom Line

The writing-agent path is currently good enough to use carefully, but not yet optimized.

If a fresh session is dedicated to writing-agent/tooling cleanup, the right target is:

- correct contracts first
- then prompt slimming
- then cache and retrieval ergonomics

Until then, the safe operating rule is:

- keep DeepSeek subordinate
- build minimal prompt files manually
- avoid teaching agents assumptions that are not guaranteed by the current engine contract
