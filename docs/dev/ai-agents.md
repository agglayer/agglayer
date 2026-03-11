# AI agent configuration

This project uses AI coding agents (Claude Code and others) with shared
configuration checked into the repository.

## Design decisions

**`.agents/skills/` as source of truth.**
This keeps the configuration tool-agnostic.
Claude Code discovers them via the `.claude/skills` symlink.

**Prefer `.agents/skills/` over `.claude/rules/`.**
Use `.agents/skills/` for most conventions.
`.claude/rules/` may be used for Claude-specific behavior
that doesn't fit the skill model (e.g., sub-agent coordination).

**Skills over AGENTS.md for task-specific workflows.**
`AGENTS.md` contains only always-on behavioral rules and a documentation index.
Task-specific workflows (committing, PR creation, verification) are skills
that load on demand, reducing context consumption.

## Skill prefixes

Skill folders are prefixed by category to keep them organized:

- **`workflow-`**: step-by-step actions with side effects;
  usually manual-only (`disable-model-invocation: true`).
- **`domain-`**: agglayer-specific invariants and safety behavior.
- **`analysis-`**: investigation and reasoning tasks (debugging, etc.);
  usually no side effects.
- **`tech-`**: stack / tools playbooks, not domain-specific.
- **`style-`**: writing and formatting conventions.
- **`meta-`**: agent governance and maintenance workflows.
- **`docs-`**: knowledge-base maintenance workflows.

## Adding a new skill

1. Pick the appropriate prefix from the list above.
2. Create `.agents/skills/<prefix><name>/SKILL.md`
   with YAML frontmatter and instructions.
3. Use `disable-model-invocation: true` for manual-only workflows (e.g., `/commit`).
4. Use `user-invocable: false` for background conventions that Claude should apply
   automatically but users shouldn't invoke directly.

## End-of-session retrospective

Run `/session-retro` at the end of a session to review the conversation
and propose improvements to skills, documentation, or `AGENTS.md`.
