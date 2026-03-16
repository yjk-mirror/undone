# UI Correctness Sweep Design

**Date:** 2026-03-16

## Goal

Establish a concrete engineering workflow for player-visible UI correctness in Undone so interaction bugs, layout bugs, and stale-state bugs are treated as first-class regressions instead of one-off fixes.

## Scope

This sweep is limited to engineering behavior in the current desktop runtime UI:

- custom title bar and tab controls
- story text and markdown rendering
- action bar and continue flow
- right sidebar
- saves, settings, and dev panels
- resize and scene-transition behavior that changes what the user can see or click

Prose quality is explicitly out of scope for this pass.

## Core Principle

If the UI behaves in a way that would feel broken to a player, it is broken, even if the underlying event wiring technically fires.

Concrete examples:

- dead space reacts as if it were a button
- a visible button has a larger or smaller hitbox than its chrome suggests
- resize changes layout in a way that breaks wrapping, clipping, or alignment
- switching scene, tab, or phase leaves stale detail text, stale continue state, or stale enabled controls

## Correctness Layers

### 1. Pure layout and input rules

Deterministic UI rules should be testable without a running app whenever possible.

This layer owns:

- hitbox ownership rules
- responsive width and row calculations
- tab enablement rules
- state-reset helper logic
- other view-logic seams that can be expressed as pure functions or narrowly testable helpers

These behaviors should be locked with focused unit tests.

### 2. Runtime UI contract

Player-visible runtime truth should be asserted through the structured runtime contract already exposed by `RuntimeSnapshot`.

This layer owns acceptance-style verification for:

- visible prose
- visible actions
- continue state
- tab and phase state
- active scene transitions
- window metrics and other runtime context needed to reason about layout-sensitive behavior

Acceptance tests should verify behavior through this contract rather than brittle screenshot-only assertions.

### 3. Live interaction audit

Some failures only fully reveal themselves in a running window. Those should be verified with targeted live checks after the lower layers are green.

This layer is the final gate for:

- clicking visible controls
- verifying dead space stays dead
- confirming resize behavior survives real scene progression
- matching live behavior against the runtime contract

Live checks validate the implementation; they do not replace lower-level tests.

## Recommended Development Approach

Use layered user-facing TDD.

For each exposed UI bug class:

1. reproduce the player-visible failure
2. write the narrowest failing regression test at the smallest stable seam
3. implement the minimal fix for the proven root cause
4. verify the behavior through a player-facing acceptance check
5. audit adjacent surfaces that share the same failure pattern and add missing coverage

This avoids both ad hoc patching and overreliance on screenshot-driven testing.

## Sweep Structure

Each engineering slice follows the same shape:

1. `Reproduce`
   Record exact user-facing steps or a runtime assertion showing the failure.
2. `Red`
   Add the smallest failing test that captures the real bug.
3. `Green`
   Patch only the root cause.
4. `Acceptance`
   Verify through the runtime contract or a targeted live interaction check.
5. `Audit`
   Expand coverage to nearby surfaces using the same pattern.

## Initial Priority Order

### Hitbox correctness

Only visible controls should react. Dead space must stay dead.

This is the first priority because it breaks the basic affordance contract of the UI. The current bottom action area is the clearest example: the `Continue` bar in `crates/undone-ui/src/left_panel.rs` attaches click handling to the full-width container instead of only the visible button chrome.

### Action-bar layout correctness

Buttons should wrap, align, and size predictably across common window widths and after scene transitions.

### State-transition correctness

Changing scene, tab, or phase must not leave stale interaction or display state behind.

### Text and rendering correctness

Markdown layout, clipping, scrolling, and spacing should remain readable and stable across common window sizes.

## Definition Of Done Per Slice

A UI-correctness slice is done only when all of the following are true:

- the original bug has a clear reproduction
- a focused failing test was added first
- the minimal root-cause fix passes the focused test
- a player-facing acceptance check passes
- nearby exposed surfaces using the same pattern now have explicit coverage

## Non-Goals

- prose rewriting
- broad visual redesign
- speculative refactors unrelated to a proven UI-correctness issue
- screenshot-diff infrastructure as the primary verification method

## Outcome

The repo should end this sweep with an explicit development model for user-facing correctness: visible affordances match real interaction, runtime behavior is covered by structured acceptance tests, and new UI regressions are caught at the smallest stable seam before they reach players.
