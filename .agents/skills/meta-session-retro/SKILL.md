---
name: session-retro
description: >
  Review the current conversation and propose structured improvements
  to skills, documentation, and agent rules.
disable-model-invocation: true
---

Review the full conversation and identify actionable improvements
to the project's AI agent configuration and documentation.

## Steps

### Step 1: Audit

- Scan the session for:
  - **Corrections**: where you were corrected or redirected.
  - **Repeated patterns**: workflows or knowledge applied multiple times.
  - **Failed approaches**: dead ends that future sessions should avoid.
  - **Discoveries**: codebase knowledge, architectural insights,
    or debugging techniques learned during the session.
  - **Missing context**: information you had to look up
    that should have been readily available.
- Produce proposals only, each with: What, Why (evidence), Where (exact file), Risk.
- Changes should follow the [Guidelines](#guidelines)

### Step 2: Approval gate

- User approves proposal IDs (or rejects).
- No edits before explicit approval

### Step 3: Apply mode

- Apply only approved items, minimally.
- No auto-commit; leave changes staged/unstaged per your normal flow.

### Step 4: Verify/report

- Run your repo verification policy and report exact commands + pass/fail.

## Guidelines

- Prefer updating existing files over creating new ones.
- Keep skills focused: one workflow per skill.
- Keep docs factual and concise.
- Do not add speculative content.
  Only propose changes backed by concrete conversation evidence.
- Use `.agents/skills/` for all agent conventions, including background knowledge
  (with `user-invocable: false`).

### Where to propose changes (priority order)

1. **Skills** (`.agents/skills/`):
   new skills for recurring workflows, or refinements to existing skills.
2. **Human-readable docs** (`docs/knowledge-base`):
   codebase knowledge, architecture guides, debugging playbooks, patterns.
   Anything useful to both humans and AI agents.
   Terms should be defined in `docs/knowledge-base/glossary.md`.
   Other topics should be organized into documents with relevant names.
   Closely related subjects should be grouped under subfolders
   with understandable names.
   When creating or updating a document, use markdown links
   to refer to the glossary.
3. **Always-on rules** (`AGENTS.md`):
   behavioral refinements to the agent interaction model.
   Keep changes minimal; this file should stay concise.
