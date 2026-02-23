# MCP Server Evaluation Guide

## Overview

This guide covers creating evaluations for MCP servers. Quality is measured by how effectively LLMs can leverage your server's tools to tackle realistic, challenging questions using only the provided capabilities.

---

## Key Evaluation Requirements

Develop 10 human-readable questions meeting these criteria:

- **Read-only operations only** — Questions MUST be independent and avoid destructive changes
- **Complex multi-step reasoning** — Each question should potentially require dozens of tool calls
- **Verifiable answers** — Single, stable values answerable through direct string comparison
- **Historical focus** — Questions based on closed concepts unlikely to change over time

---

## Question Development Standards

Questions should be realistic, clear, and concise while remaining genuinely difficult. Avoid straightforward keyword searches — use synonyms and paraphrases requiring multiple searches and analytical synthesis.

Avoid questions where answers fluctuate (e.g., counting current open issues or reactions). Focus on completed projects, archived data, or events within fixed time windows.

### Acceptable Answer Types

- Usernames, channel names, URLs, timestamps, file names
- Numeric quantities
- Multiple-choice selections

Answers must avoid complex structures or lists that could be formatted multiple ways. Prefer human-readable formats over opaque identifiers when possible.

---

## Evaluation Process Workflow

Five sequential stages:

1. **Documentation study** — Understand available endpoints and functionality
2. **Tool inspection** — Catalog available tools without calling them initially
3. **Understanding refinement** — Iterate until thoroughly understanding the system
4. **Read-only exploration** — Use tools minimally to identify specific content for questions
5. **Task generation** — Create 10 questions following all guidelines

---

## Output Format

Create an XML file with this structure:

```xml
<evaluation>
  <qa_pair>
    <question>Find discussions about AI model launches with animal codenames. One model needed a specific safety designation that uses the format ASL-X. What number X was being determined for the model named after a spotted wild cat?</question>
    <answer>3</answer>
  </qa_pair>
  <qa_pair>
    <question>Your second question here.</question>
    <answer>The answer</answer>
  </qa_pair>
  <!-- 8 more qa_pairs... -->
</evaluation>
```

---

## Running Evaluations

The evaluation harness supports three transport mechanisms:

- **STDIO** — Automatically launches the MCP server
- **SSE** — Server-Sent Events requiring a pre-started server
- **HTTP** — Streamable HTTP connections with pre-running servers

Usage:
```bash
python scripts/evaluation.py -t stdio -c python -a server.py evaluation.xml
```

The script generates reports displaying:
- Accuracy metrics
- Per-task results
- Tool call counts
- Detailed feedback about tool usability and clarity

---

## Quality Assurance

After creating evaluations, verify answers by solving questions yourself using the MCP server tools. Flag any operations requiring writes or destructive actions and remove those question pairs from the final document.

Use pagination with limited result sets to manage context windows effectively during exploration.

---

## Anti-Patterns to Avoid

- Questions answerable with a single tool call
- Questions with answers that change over time (live counts, current status)
- Questions requiring destructive or write operations to answer
- Questions with ambiguous answers that could be formatted multiple ways
- Questions testing basic functionality rather than complex reasoning
