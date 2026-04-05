# claude-rules

Shared Claude Code rules for all shuozeli repositories. Added as a git submodule at `.claude/rules/shared/` in each project.

## Structure

```
common/              Universal coding standards
  code-standards.md    Code style, testing, database, tech stack
  rust-quality.md      Rust-specific quality rules (clippy, traits, imports)
  dependency-management.md  Cargo dependency rules (no cross-repo path deps)
  large-refactor.md    Strategy for large file rewrites
  ci-verification.md   CI checks, phased launches, circular dependency avoidance
  spanner-schemas.md   Spanner schema design patterns (keys, interleaving, foreign keys)
api/                 API design standards
  aipdev.md            Google AIP standards (comprehensive)
  docsguide.md         Documentation format, freshness rules, MANIFEST.md
workflows/           Agent workflow patterns
  beu-workflow.md      beu session memory CLI reference
  agent-driven-learning.md  Structured agent learning pipeline
  session-management.md  Claude session tracking across conversations
```

## Usage

### Add to a repo (first time)

```bash
git submodule add https://github.com/shuozeli/claude-rules.git .claude/rules/shared
git commit -m "Add shared claude-rules submodule"
```

### Clone a repo that has this submodule

```bash
git clone --recurse-submodules <repo-url>
# or if already cloned:
git submodule update --init --recursive
```

### Pull latest rules into a repo

```bash
git submodule update --remote --merge
git add .claude/rules/shared
git commit -m "Update shared claude-rules"
```

## What belongs here vs. elsewhere

| Location | What goes there |
|----------|----------------|
| **This repo** | Universal rules shared across all projects (no PII, no secrets, no repo-specific config) |
| **~/.claude/rules/** | Personal/infrastructure defaults (IPs, passwords, endpoints) |
| **repo/.claude/rules/*.md** | Repo-specific rules (project architecture, deployment, local conventions) |
| **repo/CLAUDE.md** | Repo-specific context (tech stack, build commands, project description) |

## Adding a new rule

1. Create the `.md` file in the appropriate directory
2. Add `trigger: always_on` frontmatter if the rule should always be active
3. Push to main
4. In each repo: `git submodule update --remote --merge`
