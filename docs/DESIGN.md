# pcli - Project CLI

> A developer-focused project and task management CLI tool with reminders, timers, and a floating widget.

**Version:** 0.1.0  
**Created:** 2026-02-07  
**Platform:** macOS (Windows support planned)

---

## 📋 Overview

pcli is an 80% CLI / 20% GUI productivity tool for developers to manage project tasks, set reminders, and track time—all from the terminal with an optional floating timer widget.

### Key Features

- **Project Management** - Create and switch between multiple projects
- **Task Tracking** - Add, complete, and organize tasks with priorities and due dates
- **Reminders** - Set time-based reminders with system notifications
- **Timer/Pomodoro** - Track time spent on tasks with floating popup
- **TUI Board** - Interactive Kanban board in the terminal
- **Cross-Project References** - Reference tasks across projects in reminders

---

## 🖥️ Command Reference

### Navigation

```bash
pcli                              # Show dashboard (current project status)
pcli projects                     # List all projects
pcli <project-name>               # Switch to project
pcli new <project-name>           # Create new project
pcli current                      # Show current project
pcli delete <project-name>        # Delete project
```

### Tasks

```bash
pcli tasks                        # All tasks in current project
pcli tasks today                  # Tasks due today
pcli tasks pending                # Pending tasks only
pcli tasks done                   # Completed tasks

pcli add "Task name"              # Add new task
pcli add "Task" -p high           # Add with priority (low, normal, high, urgent)
pcli add "Task" -d tomorrow       # Add with due date
pcli add "Task" -p high -d friday # Add with priority and due date

pcli task <id>                    # View task details
pcli task <id> done               # Mark as complete
pcli task <id> complete           # Same as done
pcli task <id> start              # Start working (timer)
pcli task <id> stop               # Stop timer
pcli task <id> edit               # Edit task
pcli task <id> delete             # Delete task
pcli task <id> priority <level>   # Change priority
pcli task <id> due <date>         # Set due date
pcli task <id> note "text"        # Add note to task
```

### Reminders

```bash
pcli remind 10m "message"                    # Remind in 10 minutes
pcli remind 2h "message"                     # Remind in 2 hours
pcli remind 30m "calorii task 2"             # Remind about task 2 in Calorii project
pcli remind 1h "mywebsite task 5"            # Cross-project task reference

pcli reminders                               # List pending reminders
pcli reminder <id> cancel                    # Cancel reminder
pcli reminder <id> snooze 15m                # Snooze reminder
```

### Timer

```bash
pcli starttime                    # Open floating timer widget
pcli starttime 25                 # Start 25-minute pomodoro
pcli starttime task <id>          # Start timer for specific task
pcli stoptime                     # Stop current timer
pcli pausetime                    # Pause timer
pcli status                       # Show current timer status
```

### TUI / GUI

```bash
pcli board                        # Open interactive Kanban board (TUI)
pcli ui                           # Open floating window
```

---

## 🏗️ Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         pcli ecosystem                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐     IPC (Unix Socket)     ┌────────────────┐  │
│  │   pcli CLI   │◄─────────────────────────►│  pcli daemon   │  │
│  │   (Clap)     │                           │  (Background)  │  │
│  └──────────────┘                           └───────┬────────┘  │
│         │                                           │           │
│         │                                     ┌─────┴─────┐     │
│         │                                     │           │     │
│         ▼                                     ▼           ▼     │
│  ┌──────────────┐                      ┌──────────┐ ┌─────────┐ │
│  │   Ratatui    │                      │  Timer   │ │Reminder │ │
│  │  (TUI Board) │                      │  Popup   │ │ Notify  │ │
│  └──────────────┘                      │  (Tauri) │ │ (System)│ │
│                                        └──────────┘ └─────────┘ │
│                                                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    Shared Core (Rust Library)              │  │
│  │  ┌─────────┐ ┌─────────┐ ┌──────────┐ ┌────────────────┐  │  │
│  │  │ Projects│ │  Tasks  │ │ Reminders│ │  Time Tracker  │  │  │
│  │  └─────────┘ └─────────┘ └──────────┘ └────────────────┘  │  │
│  │                         │                                  │  │
│  │                    ┌────┴────┐                             │  │
│  │                    │ SQLite  │                             │  │
│  │                    └─────────┘                             │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Components

| Component | Technology | Purpose |
|-----------|------------|---------|
| **pcli-core** | Rust library | Shared business logic, models, storage |
| **pcli-cli** | Rust + Clap | Command-line interface |
| **pcli-tui** | Rust + Ratatui | Terminal UI (Kanban board) |
| **pcli-daemon** | Rust + Tokio | Background service for reminders/timer |
| **pcli-gui** | Tauri | Floating timer widget, system tray |

### IPC Protocol

CLI communicates with daemon via Unix socket (`~/.pcli/daemon.sock`):

```rust
enum DaemonCommand {
    SetReminder { message: String, duration_secs: u64, task_ref: Option<TaskRef> },
    CancelReminder { id: i32 },
    StartTimer { task_id: Option<i32>, duration_secs: Option<u64> },
    PauseTimer,
    ResumeTimer,
    StopTimer,
    GetStatus,
    ShowPopup,
    HidePopup,
    Shutdown,
}

struct TaskRef {
    project: String,
    task_id: i32,
}

enum DaemonResponse {
    Ok,
    Error { message: String },
    Status { timer: Option<TimerStatus>, reminders: Vec<ReminderStatus> },
}
```

---

## 📁 Project Structure

```
pcli/
├── Cargo.toml                    # Workspace root
├── README.md
├── docs/
│   └── DESIGN.md                 # This document
│
├── crates/
│   ├── pcli-core/                # Shared business logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── project.rs
│   │       ├── task.rs
│   │       ├── reminder.rs
│   │       ├── timer.rs
│   │       ├── storage/
│   │       │   ├── mod.rs
│   │       │   ├── sqlite.rs
│   │       │   └── schema.sql
│   │       └── config.rs
│   │
│   ├── pcli-cli/                 # CLI binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── commands/
│   │       │   ├── mod.rs
│   │       │   ├── projects.rs
│   │       │   ├── tasks.rs
│   │       │   ├── remind.rs
│   │       │   └── timer.rs
│   │       ├── output.rs
│   │       └── ipc.rs
│   │
│   ├── pcli-tui/                 # Terminal UI
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── app.rs
│   │       ├── board.rs
│   │       └── widgets/
│   │
│   └── pcli-daemon/              # Background service
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── scheduler.rs
│           ├── notifier.rs
│           └── ipc_server.rs
│
├── src-tauri/                    # Tauri GUI
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs
│       ├── timer.rs
│       └── tray.rs
│
├── ui/                           # Floating window frontend
│   ├── index.html
│   ├── main.js
│   └── styles.css
│
└── .github/
    └── workflows/
        ├── ci.yml
        └── release.yml
```

---

## 🗄️ Database Schema

```sql
-- Projects
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Tasks
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT DEFAULT 'todo' CHECK(status IN ('todo', 'in_progress', 'done', 'cancelled')),
    priority TEXT DEFAULT 'normal' CHECK(priority IN ('low', 'normal', 'high', 'urgent')),
    due_date DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Task Notes
CREATE TABLE task_notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

-- Reminders
CREATE TABLE reminders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message TEXT NOT NULL,
    project_id TEXT,
    task_id INTEGER,
    remind_at DATETIME NOT NULL,
    status TEXT DEFAULT 'pending' CHECK(status IN ('pending', 'fired', 'dismissed', 'snoozed')),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE SET NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE SET NULL
);

-- Time Logs
CREATE TABLE time_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    project_id TEXT NOT NULL,
    started_at DATETIME NOT NULL,
    ended_at DATETIME,
    duration_seconds INTEGER,
    is_pomodoro BOOLEAN DEFAULT FALSE,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- App State
CREATE TABLE app_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Indexes
CREATE INDEX idx_tasks_project ON tasks(project_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_due_date ON tasks(due_date);
CREATE INDEX idx_reminders_remind_at ON reminders(remind_at);
CREATE INDEX idx_reminders_status ON reminders(status);
CREATE INDEX idx_time_logs_task ON time_logs(task_id);
```

---

## 📦 Data Storage

```
~/.pcli/
├── config.toml              # Global configuration
├── pcli.db                  # SQLite database (all data)
├── daemon.sock              # Unix socket for IPC
└── daemon.pid               # Daemon process ID
```

### Config File (`~/.pcli/config.toml`)

```toml
[general]
default_priority = "normal"
date_format = "%Y-%m-%d"
time_format = "%H:%M"

[timer]
default_pomodoro_minutes = 25
break_minutes = 5
long_break_minutes = 15
pomodoros_until_long_break = 4

[notifications]
enabled = true
sound = true

[ui]
theme = "dark"
show_completed_tasks = true
```

---

## 🎨 UI Components

### Dashboard (`pcli`)

```
╭───────────────────────────────────────╮
│  📁 Calorii                           │
├───────────────────────────────────────┤
│  ✓ 3 done  ○ 5 pending  ▶ 1 active   │
│  ⏱️ 2h 15m logged today              │
├───────────────────────────────────────┤
│  Active:                              │
│   ▶ #4 Implement authentication       │
│     ⏱️ 00:23:45                       │
├───────────────────────────────────────┤
│  Pending:                             │
│   ○ #5 Add database migrations        │
│   ○ #6 Setup CI/CD          [Feb 10] │
│   ○ #7 Write documentation            │
├───────────────────────────────────────┤
│  ⏰ Reminders:                        │
│   • "standup meeting" in 15m          │
│   • "calorii task 5" in 2h            │
╰───────────────────────────────────────╯
```

### Task List (`pcli tasks`)

```
  Calorii - Tasks
┌────┬───────────────────────────────┬────────────┬──────────┬───────────┐
│ #  │ Title                         │ Status     │ Priority │ Due       │
├────┼───────────────────────────────┼────────────┼──────────┼───────────┤
│ 1  │ Setup project structure       │ ✓ Done     │ 🔴 High  │ -         │
│ 2  │ Implement database            │ ✓ Done     │ ⚪ Normal│ -         │
│ 3  │ Add CLI commands              │ ✓ Done     │ ⚪ Normal│ -         │
│ 4  │ Implement authentication      │ ▶ Active   │ 🔴 High  │ Feb 8     │
│ 5  │ Add database migrations       │ ○ Todo     │ ⚪ Normal│ Feb 10    │
│ 6  │ Setup CI/CD                   │ ○ Todo     │ 🟡 Low   │ Feb 12    │
└────┴───────────────────────────────┴────────────┴──────────┴───────────┘
```

### Floating Timer Widget

```
╭─────────────────────────╮
│  ⏱️ 00:23:45            │
│  ━━━━━━━━━━━░░░░░ 25m   │
├─────────────────────────┤
│  📁 Calorii             │
│  #4 Implement auth      │
├─────────────────────────┤
│  [⏸ Pause] [⏹ Stop]    │
╰─────────────────────────╯
```

### Kanban Board (`pcli board`)

```
┌─ Todo ─────────────┐┌─ In Progress ──────┐┌─ Done ─────────────┐
│                    ││                    ││                    │
│ #5 Add migrations  ││ #4 Implement auth  ││ #1 Setup project   │
│ ⚪ Normal          ││ 🔴 High            ││ ✓ 45m              │
│ Due: Feb 10        ││ ⏱️ 00:23:45        ││                    │
│                    ││                    ││ #2 Implement DB    │
│ #6 Setup CI/CD     ││                    ││ ✓ 1h 20m           │
│ 🟡 Low             ││                    ││                    │
│ Due: Feb 12        ││                    ││ #3 Add CLI         │
│                    ││                    ││ ✓ 2h 5m            │
│                    ││                    ││                    │
└────────────────────┘└────────────────────┘└────────────────────┘

 [a]dd  [m]ove  [e]dit  [d]elete  [/]search  [q]uit
```

---

## 🚀 Development Phases

### Phase 1: Core Foundation (Week 1-2)
- [ ] Set up Cargo workspace
- [ ] Implement pcli-core library
  - [ ] Project model and CRUD
  - [ ] Task model and CRUD
  - [ ] SQLite storage layer
  - [ ] Database migrations
- [ ] Basic CLI structure with Clap

### Phase 2: CLI Complete (Week 3-4)
- [ ] All project commands
- [ ] All task commands
- [ ] Task filtering and search
- [ ] Pretty output formatting
- [ ] Config file support

### Phase 3: TUI Board (Week 5)
- [ ] Ratatui integration
- [ ] Kanban board view
- [ ] Keyboard navigation
- [ ] Task quick actions

### Phase 4: Daemon & Reminders (Week 6-7)
- [ ] Background daemon setup
- [ ] IPC protocol implementation
- [ ] Reminder scheduler
- [ ] System notifications (notify-rust)
- [ ] CLI reminder commands

### Phase 5: Timer & GUI (Week 8-9)
- [ ] Tauri project setup
- [ ] Floating timer window
- [ ] System tray integration
- [ ] Timer controls
- [ ] Global hotkeys

### Phase 6: Polish & Release (Week 10)
- [ ] Cross-platform testing
- [ ] Installer/packaging (dmg, exe)
- [ ] Documentation
- [ ] Release v0.1.0

---

## 🛠️ Tech Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Language | Rust | 1.75+ |
| CLI Framework | clap | 4.x |
| Terminal UI | ratatui | 0.25+ |
| Async Runtime | tokio | 1.x |
| Database | rusqlite | 0.30+ |
| Notifications | notify-rust | 4.x |
| GUI Framework | tauri | 2.x |
| Serialization | serde | 1.x |
| Date/Time | chrono | 0.4+ |
| Config | toml | 0.8+ |

---

## 📝 License

MIT License

---

## 🤝 Contributing

TBD

