# DeepSeek Writing Helper

This repo uses DeepSeek as a subordinate writing tool, not as the orchestrating model.
Codex, Claude/Opus, or any other primary agent stays responsible for:

- deciding what context is safe to send
- applying `docs/writing-guide.md`
- validating templates and scene structure
- accepting, revising, or discarding the helper output

## Tool

Repo-local helper:

```text
tools/deepseek-helper.mjs
```

It calls the DeepSeek chat completions API and supports two workflows:

- `draft`: first-pass drafting, expansion, alternative phrasings
- `review`: second-opinion critique before the orchestrating model writes the final review

## Auth

The helper reads `DEEPSEEK_API_KEY` from:

1. the process environment
2. the repo-local `.env`

It does not print the key.

## Usage

Draft from a prompt file:

```bash
node tools/deepseek-helper.mjs draft --prompt-file tmp/scene-spec.md
```

Review into a file:

```bash
node tools/deepseek-helper.mjs review --prompt-file tmp/review-target.md --output-file tmp/deepseek-review.md
```

Read the prompt from stdin and emit JSON:

```powershell
Get-Content tmp/scene-spec.md | node tools/deepseek-helper.mjs draft --json
```

Optional flags:

- `--system-file <path>`
- `--model <name>`
- `--temperature <num>`
- `--max-tokens <num>`
- `--output-file <path>`
- `--json`

CLI help:

```bash
node tools/deepseek-helper.mjs --help
```

## Safe Input Boundary

Only send fictional content-work context:

- scene specs
- writing-guide rules
- trait/skill/stat IDs
- sample prose
- review targets

Do not send:

- secrets
- personal data
- local machine details unrelated to content work
- unrelated repository code or docs

## Prompt Packer

`tools/pack-prompt.mjs` assembles optimal prompt files from scene specs.

### Usage

```bash
node tools/pack-prompt.mjs --spec-file tmp/spec-scene.json
```

See the script's `--help` for full options and spec format.

### Full Pipeline

```bash
# 1. Pack the prompt
node tools/pack-prompt.mjs --spec-file tmp/spec.json

# 2. Generate draft
node tools/deepseek-helper.mjs draft --system-file docs/writer-core.md \
  --prompt-file tmp/prompt-scene.md --output-file tmp/draft-scene.toml

# 3. Review
node tools/deepseek-helper.mjs review --system-file docs/review-core.md \
  --prompt-file tmp/draft-scene.toml --output-file tmp/review-scene.md
```

### Reference Docs

- `docs/writer-core.md` — compact writing rules for DeepSeek system prompt (~12KB, ~3K tokens)
- `docs/review-core.md` — compact review criteria for DeepSeek review mode (~4KB, ~1K tokens)

Both are designed for prompt cache efficiency — stable prefix that stays cached across calls.

## Agent Workflow

`scene-writer.md` and `writing-reviewer.md` orchestrate the full pipeline:

1. Write a JSON scene spec to `tmp/spec-<name>.json`
2. Run `pack-prompt.mjs` to assemble the prompt
3. Run `deepseek-helper.mjs draft` with `writer-core.md` as system prompt
4. Run `deepseek-helper.mjs review` with `review-core.md` as system prompt
5. Read review findings, fix Critical/Important issues
6. Validate templates with minijinja MCP
7. Write final TOML to `packs/base/scenes/`
