# linear-cli-rs

Fast, zero-config CLI for [Linear](https://linear.app). Built for both humans and agents: readable terminal output by default, `--json` everywhere it matters.

```
cargo install linear-cli-rs
```

Installs a `linear` binary.

## Setup

```
linear auth
```

Prompts for a Linear API key (create one under Linear → Settings → Security & access → Personal API keys).

## Usage

```
linear issue list                        List my active issues
linear issue list --state 'In Progress'  What am I working on?
linear issue list --team ENG --all       All ENG issues including done
linear issue DIS-510                     View an issue (shorthand)
linear issue view DIS-510 --json         View as JSON (for agents)
linear issue search 'privacy policy'     Full-text search across teams
linear issue create --team ENG           Create issue (interactive)
linear issue update DIS-510 --state 'In Progress'
linear issue comment DIS-510 -b 'Fixed'  Add a comment
echo 'details...' | linear issue comment DIS-510
linear team list                         List teams
linear me                                Show authenticated user
linear api -q '{ viewer { id } }'        Run raw GraphQL
```

`issue view` shows full metadata (state, priority, assignee, labels, project), the parent issue, the git branch name, sub-issues with their states, and threaded comments.

## Why this over other Linear CLIs?

- **Zero config** — no per-repo config files; `issue list` defaults to your issues.
- **Triage-friendly** — parent/sub-issue relations and threaded comments in both human and JSON output.
- **Cross-team full-text search.**
- **Fast** — a single static binary; issue view round-trips in ~0.2s.
- **Agent-ready** — `--json` on `list`, `view`, and `comment`, plus a raw GraphQL escape hatch.

## License

MIT
