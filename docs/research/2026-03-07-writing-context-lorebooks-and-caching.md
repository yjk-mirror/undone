# Writing Context, Lorebooks, and Low-Cost Delegation Research

Date: 2026-03-07

## Goal

Research public patterns for:

- keeping long-running writing sessions coherent
- managing lore, character profiles, and world facts without dumping everything into every prompt
- reducing API cost when delegating to subordinate writing agents, especially with DeepSeek

This note is meant to inform a future writing-agent/tooling session. It is not a design commitment yet.

## Bottom Line

The public pattern is consistent across serious roleplay/chat tooling:

1. Keep a small always-on core prompt
2. Store world facts and character profiles outside the main prompt
3. Inject only relevant entries when triggered by the current scene or recent messages
4. Cap the retrieval budget explicitly
5. Preserve a stable prompt prefix so provider-side prompt caching stays effective

Undone does not do this automatically today. The current repo relies on manual document reads and manual prompt-file assembly.

## Public Patterns

### 1. Anthropic: subagents should stay focused and context-aware

Anthropic's current Claude Code guidance matches the direction we should want:

- subagents should be focused on one task
- descriptions should be detailed enough to support correct delegation
- tool access should be limited
- subagents run in their own context windows
- large always-loaded instruction files are not ideal

Relevant implications for Undone:

- `scene-writer` and `writing-reviewer` are the right conceptual split
- the repo should avoid inflating both of them with duplicated writing law
- large specialized instruction sets should move into thinner imported/topic files or equivalent scoped memory

Anthropic also documents that `CLAUDE.md` files are loaded in full, while large memory should be split into smaller topic files and imported deliberately.

### 2. Anthropic prompt caching: stable prefix matters

Anthropic's prompt-caching docs reinforce a core principle:

- caching works on the full prefix of `tools`, `system`, and `messages` up to the cache breakpoint
- large stable prefixes are worth preserving
- prompt shape should be stable across repeated calls

For writing delegation, that means:

- the static rules should be in a stable header
- the scene-specific ask should be the changing tail
- repeatedly reformatting the whole prompt wastes cache opportunity

Even if the orchestrator is not directly calling Anthropic's API in a custom way, the principle is still the right one for any cacheable provider: keep the immutable prefix stable.

### 3. DeepSeek: caching is automatic, prefix-based, and materially cheaper

DeepSeek's current API docs are unusually relevant here.

The important points:

- context caching is enabled by default
- repeated prefixes produce cache hits
- only repeated prefix content is reused
- `usage` reports `prompt_cache_hit_tokens` and `prompt_cache_miss_tokens`
- few-shot patterns benefit because the fixed examples become a reusable prefix

DeepSeek also publishes current cache-hit vs cache-miss input pricing, and the difference is large. That means good prompt shaping is not a micro-optimization here; it is a first-order cost lever.

Practical implication for Undone:

- if subordinate writing calls keep re-sending the same guide text, examples, and schema in the same order, the cost can stay low
- if the prompt is rebuilt ad hoc every time with different ordering and extra noise, the cost advantage erodes

### 4. SillyTavern: lorebooks work because they are dynamic, scoped, and budgeted

SillyTavern is the clearest public/open implementation of lorebook-style prompting.

Its documented pattern:

- world info entries are activated when keywords appear
- only relevant entries are inserted
- entries should be standalone, because metadata is not inserted
- budgets are explicit
- scan depth is explicit
- recursion can pull in linked entries
- persona data is separate from world info
- larger document sets can be handled through vector retrieval via Data Bank

This is the closest public analogue to the behavior people often describe informally when talking about Janitor-style lore handling.

The important idea is not the exact UI. It is the architecture:

- permanent identity/profile data is separate
- lore is chunked
- retrieval is selective
- the prompt budget is capped

### 5. Public Janitor ecosystem patterns: "permanent tokens" stay small, dynamic context is added later

Official Janitor architecture docs are sparse in public. The closest visible public evidence comes from the surrounding Janny/Janitor creator ecosystem, which repeatedly emphasizes:

- keeping "permanent" prompt sections small
- treating stable persona/scenario text as scarce
- relying on dynamic activation or scripts for larger settings
- avoiding giant universal system prompts

Public Janitor script/lorebook examples also show a common pattern:

- inspect recent context
- detect mentions through regex or aliases
- append only the matching character or setting blocks

This is not a primary-source Janitor architecture document, so it should be treated as ecosystem evidence, not official product documentation. But the pattern is consistent with SillyTavern and with what low-cost roleplay setups generally optimize for.

## What This Suggests For Undone

### A. Separate "always-loaded" from "retrieved on demand"

Undone should eventually separate writing context into at least four layers:

1. Stable writer core
   - very short
   - engine contract rules that must always hold
   - format/schema rules

2. Stable style packet
   - concise voice rules
   - a few validated examples
   - no full guide dump unless needed

3. Scene-local retrieval
   - relevant arc doc
   - only the NPC docs involved in this scene
   - only the setting/lore entries involved in this scene

4. Ephemeral task payload
   - the user brief
   - the target scene slot or file
   - any temporary revision goals

That shape is much closer to how good lorebook systems work than the current "read a lot of docs every time" pattern.

### B. Introduce writer-facing "profiles" and "lore entries"

If Undone wants Janitor-like or SillyTavern-like retrieval behavior later, the raw material should be structured first.

Likely future units:

- NPC profile cards
  - aliases
  - one-paragraph identity
  - voice notes
  - role in arcs
  - hard invariants

- location lore entries
  - where this place is
  - what makes it specific
  - recurring sensory and social facts

- arc state packets
  - what must already be true
  - what scenes have happened
  - open tensions

- style/example packets
  - tiny curated examples by scene type

Without chunking the source material first, no retrieval layer will stay clean.

### C. Prefer retrieval over stuffing more into agent prompts

The wrong future shape would be:

- make `scene-writer.md` enormous
- paste more lore into every prompt
- depend on brute-force context length

The better future shape is:

- thin agent prompt
- structured local docs
- retrieval/build step that assembles the smallest relevant authoring packet

### D. Preserve stable prefixes for DeepSeek

For subordinate `draft` and `review` calls, the future helper should preserve a stable order:

1. system prompt
2. stable writer law / review law
3. stable schema/format notes
4. retrieved lore/profile packet
5. scene-specific ask

That gives DeepSeek the best chance to produce repeated-prefix cache hits.

### E. Track cost with real cache metrics

Right now the helper prints only:

- `prompt_tokens`
- `completion_tokens`

Before a writing sprint, it should also print:

- `prompt_cache_hit_tokens`
- `prompt_cache_miss_tokens`

Without that, there is no way to know whether the prompt shape is actually exploiting DeepSeek's caching model.

## A Practical Future Shape For Delegated Writing

Not an implementation commitment yet, but this is the direction the research supports.

### Step 1: build a small authoring packet

Given a target scene, assemble only:

- short writer core
- scene schema checklist
- one or two style examples
- the relevant arc summary
- the involved NPC profile(s)
- the user brief

Not:

- the entire writing guide
- unrelated NPC docs
- unrelated arcs
- large generic world dumps

### Step 2: keep a stable prompt header

The authoring packet should have a canonical order and formatting so repeated calls keep the same prefix.

### Step 3: use retrieval gates

Simple gates would already help:

- by route
- by pack
- by arc
- by named NPC alias
- by location id

This does not need full embeddings on day one. Keyword/alias retrieval is enough to get most of the value.

### Step 4: reserve vectors for large background corpora

If Undone later has:

- long setting docs
- long character dossiers
- multiple packs with deep lore

then a vector-backed retrieval layer becomes useful. Until then, explicit structured chunks and alias-based selection are likely enough.

## Recommended Future Experiments

These are the highest-value experiments before another heavy writing phase.

### Experiment 1: thin prompt vs fat prompt

Compare:

- current manual large-context prompting
- a thin stable writer packet plus only relevant retrieved docs

Measure:

- output quality
- prompt tokens
- cache-hit tokens
- revision burden

### Experiment 2: static profile cards

Create a few structured NPC profile files and use only those, instead of free-form doc reads, for a trial writing task.

### Experiment 3: helper instrumentation only

Do not change writing behavior yet. Only add:

- cache-hit reporting
- miss reporting
- prompt-size reporting

Then learn from real usage before building more tooling.

### Experiment 4: keyword retrieval before embeddings

Try a very simple local packer:

- scene id
- NPC aliases
- route id
- location id

Use those to pull only the right docs into a prompt file.

This should be much cheaper and easier to validate than jumping straight to vector retrieval.

## Applicability To Undone Right Now

What is immediately applicable:

- thinner prompts
- more stable prompt ordering
- cache metric reporting
- smaller writer packets
- structured profile/lore documents

What is not yet justified:

- large retrieval infrastructure
- embedding/vector infra
- automatic multi-source lore orchestration without first cleaning up the source documents

## Notes On Janitor AI Specifically

Janitor AI was part of the motivation for this research, but the public official documentation around its internal lorebook/memory architecture is limited.

What is public and high-signal:

- community emphasis on low permanent-token budgets
- public script/lorebook templates that activate character blocks from recent mentions
- widespread use of external lorebook/proxy tooling in the Janitor ecosystem

So the safest conclusion is:

- the behavior people like in that ecosystem is real
- the most public, inspectable implementation model is closer to SillyTavern's documented world-info and retrieval system than to an official Janitor technical spec

## Sources

- Anthropic Claude Code subagents: <https://docs.anthropic.com/en/docs/claude-code/sub-agents>
- Anthropic Claude Code memory/imports: <https://docs.anthropic.com/en/docs/claude-code/memory>
- Anthropic prompt caching: <https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching>
- DeepSeek context caching: <https://api-docs.deepseek.com/guides/kv_cache>
- DeepSeek chat completion usage fields: <https://api-docs.deepseek.com/api/create-chat-completion/>
- DeepSeek current pricing: <https://api-docs.deepseek.com/quick_start/pricing/>
- SillyTavern World Info: <https://docs.sillytavern.app/usage/core-concepts/worldinfo/>
- SillyTavern Personas: <https://docs.sillytavern.app/usage/core-concepts/personas/>
- SillyTavern Data Bank: <https://docs.sillytavern.app/usage/core-concepts/data-bank/>
- SillyTavern Chat Vectorization: <https://docs.sillytavern.app/extensions/chat-vectorization/>
- Public Janitor ecosystem script example: <https://jannyai.com/characters/596dc3a1-6b62-4774-98db-6d3e9c05d7e2_character-multiple-character-drop-in-drop-out-lorebook-template>
- Public Janitor ecosystem token-budget guide example: <https://jannyai.com/characters/2856e071-a071-4b28-a2e2-8fbbccd79be5_character-bot-maker>
