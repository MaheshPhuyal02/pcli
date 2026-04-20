---
name: pcli
description: Use this skill to track your own tasks across the user's projects via the `pcli` CLI. TRIGGER when starting non-trivial multi-step work you may need to resume later, when the user asks what's open / in-progress / on your plate, when the user mentions tasks or TODOs, or at the start of a coding session in a directory that has a matching pcli project. Tasks persist in `~/.pcli/pcli.db` across all Claude sessions, so a task you create now is visible to a future session.
---

# pcli skill

`pcli` is a lightweight CLI task manager you (Claude) use to track your own work across sessions. Its SQLite database at `~/.pcli/pcli.db` is shared — anything you write is readable by a future session in this or any other directory.

Use it to give yourself continuity: instead of relying on `auto memory` for in-progress work, write it as a task.

## When to use

- Starting a non-trivial task: create a pcli task first so the next session can resume from your description
- During work: flip status to `start` when picking up, leave notes in the description as context grows
- End of session (or before compaction): ensure each in-progress task's description captures where you left off — file paths, what you tried, what's next
- User asks "what's open?", "what were we doing?", "any tasks pending?": run `pcli tasks` in the relevant project

## Project selection

Each task belongs to a project. Use one project per user repo or working directory. Project name should match the repo/directory name for findability.

```bash
pcli projects              # list all projects
pcli current               # show currently selected project
pcli new <name>            # create a new project
```

**Switching projects:** there is no `pcli switch` subcommand at the time of writing. Options:
- Enter REPL with bare `pcli`, type the project name, then `exit`.
- Pipe it: `printf '%s\nexit\n' <project-name> | pcli`

Always verify the current project with `pcli current` before adding tasks — writing to the wrong project is silent.

## Tasks

```bash
pcli add "short title" --description "longer context" -p high -d tomorrow
pcli tasks                 # list tasks in current project
pcli tasks today           # due today
pcli tasks active          # in-progress only
pcli tasks pending         # todo only
pcli tasks done            # completed
pcli task <id>             # full detail — shows description
pcli task <id> start       # mark in_progress
pcli task <id> done        # mark complete
pcli task <id> stop        # revert to todo
pcli task <id> delete      # remove
```

The list view truncates long titles with `…` and marks rows with descriptions using `✎`. To read full context, always run `pcli task <id>`.

## Reminders

```bash
pcli remind 30m "check CI on PR #42"
pcli remind 2h "review migration before deploy"
pcli reminders             # list pending
```

Durations: `10m`, `2h`, `1d`. Reminders fire as macOS system notifications from the `pcli-daemon` background process.

## How to write tasks for your future self

- **Title**: short, scannable — what it *is*, not what you did. Max ~60 chars before it truncates.
- **Description**: what the next session needs to pick up cold. Include file paths with line numbers (`src/foo.rs:42`), what you tried, what's blocking, what's next. Don't reference ephemeral session details.
- **Priority**:
  - `urgent` — actively blocking something
  - `high` — do this session
  - `normal` — default, later this week
  - `low` — nice-to-have
- **Due date**: `today`, `tomorrow`, `week`, or `YYYY-MM-DD`. Only use when there's a real external deadline.

### Good

```bash
pcli add "Wire CLI↔daemon IPC socket" \
  --description "DESIGN.md §3 specifies Unix socket at ~/.pcli/daemon.sock. Currently pcli-cli/src/daemon_helper.rs only checks PID. Need: socket server in pcli-daemon main loop + client helper in pcli-cli. Protocol TBD — start with JSON line-delimited." \
  -p high
```

### Bad

```bash
pcli add "fix the bug we were talking about"   # no context for future-you
pcli add "finish IPC stuff tomorrow" -p urgent  # urgent is for blockers, not self-imposed deadlines
```

## Workflow pattern

1. **Pick up a session**: `pcli tasks active` → pick one → `pcli task <id>` to read description → `pcli task <id> start` if not already started.
2. **Mid-work growth**: if scope expands, create a new task rather than stuffing it into the current description.
3. **Hand-off**: before ending a session, `pcli task <id>` each in-progress task and confirm the description would let a cold reader resume. Update if not.
4. **Completion**: `pcli task <id> done` — do not leave stale `in_progress` tasks lying around.

## When NOT to use pcli

- Ephemeral steps within a single response — use the TodoWrite tool for that.
- Permanent facts about the user or the project — that's `auto memory`.
- Code comments or documentation — that belongs in the repo.

Rule of thumb: *would I want a future session to see this?* If yes, pcli task. If only this turn, TodoWrite. If forever, memory.
