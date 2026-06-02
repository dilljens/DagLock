# Codebase Wiki

AI-optimized codebase map for DagLock.

## For humans
1. `_index.md` — quick reference + architecture (keep open)
2. `_standards.md` § Rules — what never to do
3. `_standards.md` § Practices — how to write new code
4. `features/<domain>.md` — module deep dive

## For AI agents

### Cold start (zero context)
1. `_glossary.md` — learn project vocabulary
2. `_index.md` — architecture topology + domain one-liners
3. `_standards.md` § Rules — what never to do
4. `_standards.md` § Practices — how to write new code
5. `features/<domain>.md` — the domain you're working on
6. `_standards.md` § Patterns — match conventions during generation

### Task-specific

### Adding a feature
_index (Navigation) → _standards (Rules + Practices) → domain doc → _standards (Patterns)

### Debugging
_index (Navigation) → domain doc (edge cases) → _standards (Rules)

### Refactoring
domain docs (deps + consumers) → _index (topology) → _standards (Patterns)

### Unfamiliar code
_index (domain one-liners) → domain doc

### Writing a test
domain doc → _standards (Practices: Testing + Patterns: Test patterns)

## Commands
- `/wiki:make` — initialize wiki (interactive)
- `/wiki:onboard` — cold-start walkthrough
- `/wiki:update` — refresh after code changes
- `/wiki:sync` — upgrade wiki after skill changes
- `/wiki:check` — verify internal consistency

## Preventing drift
Run `/wiki:check` before PRs to catch stale cross-references.

## Stale docs
Read source. If doc is wrong, propose update. Don't silently ignore.

## Plans
`docs/wiki/plans/` — architecture proposals and migration plans. Justify decisions (not living docs).
