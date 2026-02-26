# AGENTS.md

Rust node implementing the AggLayer — an aggregation layer providing secure, atomic interoperability among heterogeneous ZK chains.

## Context map

Human-readable project docs:
- `README.md` — project overview, prerequisites, build and test commands.
- `CONTRIBUTING.md` — PR process, commit signing, conventional commits, force-push policy.
- `docs/dev/` — domain knowledge and architecture docs (also useful to AI).

AI agent rules (`docs/ai-agents/`). Load only the file(s) relevant to the current task:

| File | Contents | Load when |
|---|---|---|
| `behavior.md` | Response style, clarification rules, collaboration norms | Any task |
| `debugging.md` | Evidence-based debugging and communication | Debugging or root-cause analysis |
| `domain.md` | Agglayer-specific behavioral framing | Agglayer domain question or advice |
| `quality.md` | Definition of done and verification steps | Any code change |
| `git.md` | Commit and PR conventions | Commit or PR |
| `interaction-preferences.md` | Clarification style and knowledge persistence | Clarification needed or ambiguous task |
| `context-update-protocol.md` | Procedure for `update context` requests | `update context` request |

## Creating new context files

If no relevant file exists for a task, create one:
- `docs/ai-agents/<name>.md` for AI-specific procedural rules.
- `docs/dev/<name>.md` for domain or architectural knowledge useful to both humans and AI.
- Add the new file to the table above and to the context map.
- Keep each file focused on one area.
