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

## Agent Workflow

`scene-writer.md` and `writing-reviewer.md` are wired to call the helper through shell commands when a subordinate pass is useful.

Expected workflow:

1. Build a minimal prompt file with only the needed fictional context.
2. Run `node tools/deepseek-helper.mjs draft ...` or `review ...`.
3. Read the result critically.
4. Apply local writing rules.
5. Validate minijinja and scene structure before any content is accepted.
