---
name: update-context
description: >
  Review the current conversation and propose structured improvements
  to skills, documentation, and agent rules.
disable-model-invocation: true
---

# Context update protocol

Review the full conversation and identify actionable improvements
to the project's AI agent configuration and documentation.

## What to look for

Scan the conversation for:
- **Corrections**: where you were corrected or redirected.
- **Repeated patterns**: workflows or knowledge applied multiple times.
- **Failed approaches**: dead ends that future sessions should avoid.
- **Discoveries**: codebase knowledge, architectural insights,
  or debugging techniques learned during the session.
- **Missing context**: information you had to look up
  that should have been readily available.

## Where to propose changes (priority order)

1. **Skills** (`.agents/skills/`):
   new skills for recurring workflows, or refinements to existing skills.
2. **Human-readable docs** (`docs/`):
   codebase knowledge, architecture guides, debugging playbooks, patterns.
   Anything useful to both humans and AI agents.
3. **Always-on rules** (`AGENTS.md`):
   behavioral refinements to the agent interaction model.
   Keep changes minimal; this file should stay concise.

## Protocol

1. Review the full conversation history.
2. Categorize each finding by target (skill / docs / AGENTS.md).
3. For each proposed change, explain:
   - **What**: the specific change.
   - **Why**: the conversation evidence that motivates it.
   - **Where**: the exact file and section.
4. Present all proposed changes as a summary for user approval.
5. After approval, apply the changes.
6. Do **not** commit automatically.
   Stage the changes and let the user decide when to commit.

## Guidelines

- Prefer updating existing files over creating new ones.
- Keep skills focused: one workflow per skill.
- Keep docs factual and concise.
- Do not add speculative content.
  Only propose changes backed by concrete conversation evidence.
- Use `.agents/skills/` for all agent conventions, including background knowledge
  (with `user-invocable: false`). Do not use `.claude/rules/`.
