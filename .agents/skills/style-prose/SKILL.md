---
name: style-prose
description: >
  Text formatting conventions for markdown and prose files.
  Use when writing or editing any .md, .txt, .adoc, or .rst file,
  including AGENTS.md, SKILL.md files, and documentation in docs/.
user-invocable: false
---

When writing or editing text files (markdown, plain text, etc.),
use **Semantic Line Breaks** (https://sembr.org/).

## Rules

- **Line length**: aim for 80-100 characters per line.
  This is the primary constraint; the rules below serve it.
- Add a line break after each sentence (after `.`, `!`, or `?`).
- Break at a clause boundary when a line would otherwise exceed ~100 characters.
- Do **not** micro-split: short clauses, enumerations, and closely related
  phrases should stay on the same line when they fit within the target length.
- Break before enumerated or itemized lists.
- Do not break within hyphenated words.
- Allow longer lines for URLs, code spans, or markup.

## Why

Semantic line breaks make diffs cleaner:
editing one clause changes only one line, not the entire paragraph.
The rendered output is unaffected.

## Scope

Apply to all prose in markup files (markdown, rst, adoc, plain text).
Do **not** apply to code blocks, YAML frontmatter,
or structured data inside these files.
