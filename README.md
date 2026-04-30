# pcli

A lightweight, developer-focused task manager for the terminal. Built in Rust. Designed to be fast enough that you actually use it — and simple enough that an AI assistant can manage it as persistent memory.

```
┌────┬──────────────────────────────────────┬──────────────┬──────────┬──────────┐
│ #  │ Title                                │ Status       │ Priority │ Due      │
├────┼──────────────────────────────────────┼──────────────┼──────────┼──────────┤
│ 1  │ ✎ Wire CLI↔daemon IPC socket         │ ▶ in_progress│ 🟡 high   │ Apr 22   │
│ 2  │   Fix title truncation for unicode   │ ✓ done       │ 🟡 high   │ -        │
│ 3  │ ✎ Implement Kanban board (ratatui)   │ ○ todo       │ ⚪ normal │ -        │
└────┴──────────────────────────────────────┴──────────────┴──────────┴──────────┘
  ✎ = has description · run `pcli task <id>` for full details
```

## Features

- **Projects** — group tasks by repo or context, switch between them
- **Tasks** — title, longer description, priority, due date, status
- **Reminders** — duration-based with native macOS notifications (via background daemon)
- **REPL** — interactive mode with tab completion and command history
- **Adaptive tables** — terminal-width-aware, unicode-safe truncation
- **AI-friendly** — ships with a [Claude Code skill](skill/skill.md) so assistants can use pcli as persistent task memory across sessions

### Roadmap

- Interactive Kanban board (ratatui)
- Time tracking / pomodoro
- CLI↔daemon IPC for live updates
- Floating widget (Tauri)
- Cross-platform notifications

## Installation

### From source (GitHub)

```bash
git clone https://github.com/MaheshPhuyal02/pcli.git
cd pcli
cargo install --path crates/pcli-cli
cargo install --path crates/pcli-daemon
```

### From a local checkout

```bash
cargo build --release
# binaries land in target/release/pcli and target/release/pcli-daemon
```

### Requirements

- Rust 1.75+ (edition 2021)
- macOS for native notifications (daemon works on Linux but notification backend is macOS-tuned)
- SQLite is bundled via `rusqlite` — no system dependency

The `pcli` binary auto-starts `pcli-daemon` in the background on first run. State lives in `~/.pcli/` (config + SQLite database + daemon PID).

## Quick start

```bash
pcli new myapp                             # create a project
pcli                                       # REPL — type 'myapp' to switch, then 'exit'
pcli add "Set up CI" -p high -d tomorrow
pcli add "Write auth middleware" \
    --description "Use jsonwebtoken crate. See notes in docs/AUTH.md." \
    -p high
pcli tasks                                 # list
pcli task 1                                # full detail (shows description)
pcli task 1 start                          # mark in-progress
pcli task 1 done                           # complete
pcli remind 30m "check PR #42"
```

## Commands

### Projects

| Command | Description |
|---------|-------------|
| `pcli projects` | List all projects |
| `pcli new <name>` | Create a new project |
| `pcli current` | Show currently selected project |
| `pcli delete <name>` | Delete a project and its tasks |

Switching: enter the REPL with bare `pcli`, then type the project name.

### Tasks

| Command | Description |
|---------|-------------|
| `pcli tasks` | List tasks in current project |
| `pcli tasks today` | Due today |
| `pcli tasks tomorrow` | Due tomorrow |
| `pcli tasks week` | Due within the next 7 days |
| `pcli tasks overdue` | Past due tasks |
| `pcli tasks active` | In-progress only |
| `pcli tasks pending` | Todo only |
| `pcli tasks done` | Completed |
| `pcli tasks cancelled` | Cancelled tasks only |
| `pcli add "title"` | Add a task |
| `pcli add "title" --description "..." -p high -d tomorrow` | Full form |
| `pcli task <id>` | Show full detail including description |
| `pcli task <id> start` | Mark in-progress |
| `pcli task <id> done` | Mark complete |
| `pcli task <id> stop` | Revert to todo |
| `pcli task <id> edit "new title"` | Rename task |
| `pcli task <id> description "notes"` | Update task notes |
| `pcli task <id> due tomorrow` | Set due date |
| `pcli task <id> priority high` | Change task priority |
| `pcli task <id> delete` | Remove |

**Priority:** `low`, `normal` (default), `high`, `urgent` (aliases: `l`/`n`/`h`/`u`, `1`–`4`).
**Due:** `today`, `tomorrow`, `week`, or `YYYY-MM-DD`.

### Reminders

| Command | Description |
|---------|-------------|
| `pcli remind 10m "msg"` | Remind in 10 minutes |
| `pcli remind 2h "msg"` | Remind in 2 hours |
| `pcli remind 1d "msg"` | Remind in 1 day |
| `pcli reminders` | List pending |

The daemon checks every 10s and fires system notifications when due.

### Utilities

| Command | Description |
|---------|-------------|
| `pcli stop --port 3000` | Kill whatever is bound to a port |
| `pcli open --url https://... --chrome` | Open a URL in a specific browser |
| `pcli status` | Current project + active task |

## Interactive REPL

Running `pcli` with no arguments drops you into a REPL with tab completion:

```
pcli > tasks
pcli > add "refactor storage" -p high
pcli > task 3 done
pcli > myapp                     # switch project
pcli > exit
```

## Using pcli with Claude Code (or any AI assistant)

`pcli` doubles as persistent task memory for AI assistants. A task you create in one session is visible to any future session. This project ships a ready-made Claude Code skill at [skill/skill.md](skill/skill.md).

To wire it up as a user-level skill:

```bash
mkdir -p ~/.claude/skills/pcli
cp skill/skill.md ~/.claude/skills/pcli/SKILL.md
```

Claude will then use pcli to:
- Create tasks when starting non-trivial work
- Write descriptions detailed enough that a future session can resume cold
- Check `pcli tasks active` when you ask "what were we working on?"
- Mark tasks done rather than leaving stale in-progress state

See [skill/skill.md](skill/skill.md) for the full behavioral spec — it's also good reading if you want to use pcli yourself with the same discipline.

## Configuration

Config lives at `~/.pcli/config.toml`. The file is created with defaults on first run. Keys are documented inline in [crates/pcli-core/src/config.rs](crates/pcli-core/src/config.rs).

Database: `~/.pcli/pcli.db` (SQLite). Back it up if you care about your history.

## Architecture

```
pcli (CLI) ─┐
            ├── pcli-core ── SQLite (~/.pcli/pcli.db)
pcli-daemon ┘
```

- **`pcli-core`** — models (Project, Task, Reminder, TimeLog), storage layer, config, error types
- **`pcli-cli`** — clap CLI + rustyline REPL; spawns daemon on startup
- **`pcli-daemon`** — tokio background service for reminder scheduling and notifications
- **`pcli-tui`** — ratatui integration (Kanban board, in progress)

Deeper design notes: [docs/DESIGN.md](docs/DESIGN.md).

## Development

```bash
cargo build             # build all crates
cargo test              # run tests
cargo run -p pcli       # run CLI from source
cargo run -p pcli-daemon  # run daemon manually
cargo clippy --all-targets
cargo fmt
```

## Contributing

Issues and PRs welcome at <https://github.com/MaheshPhuyal02/pcli>. Keep changes small and focused; match the existing style.

## License

MIT © Mahesh Khatri
