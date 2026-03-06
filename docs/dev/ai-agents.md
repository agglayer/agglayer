# AI agent configuration

This project uses AI coding agents (Claude Code and others) with shared
configuration checked into the repository.

## Design decisions

**`.agents/skills/` as source of truth.**
Skills live in `.agents/skills/`.
This keeps the configuration tool-agnostic.
Claude Code discovers them via the `.claude/skills` symlink.

**No `.claude/rules/`.**
Claude Code supports path-scoped rules in `.claude/rules/`, but we use
`.agents/skills/` with `user-invocable: false` for background conventions instead.
This keeps all agent configuration in one place.

**Skills over AGENTS.md for task-specific workflows.**
`AGENTS.md` contains only always-on behavioral rules and a documentation index.
Task-specific workflows (committing, PR creation, verification) are skills
that load on demand, reducing context consumption.

## Adding a new skill

1. Create `.agents/skills/<name>/SKILL.md` with YAML frontmatter and instructions.
3. Use `disable-model-invocation: true` for manual-only workflows (e.g., `/commit`).
4. Use `user-invocable: false` for background conventions that Claude should apply
   automatically but users shouldn't invoke directly.

## Updating agent configuration

Run `/update-context` at the end of a session to review the conversation
and propose improvements to skills, documentation, or `AGENTS.md`.
