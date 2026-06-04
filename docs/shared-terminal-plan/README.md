# Shared Terminal Feature — Complete Implementation Plan

**Project**: Pi Coding Agent  
**Feature**: Split-pane terminal for interactive CLI tools  
**Author**: AI Assistant  
**Date**: 2026-06-04  
**Status**: Ready for implementation

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Prerequisites](#prerequisites)
4. [Dependencies](#dependencies)
5. [File Structure](#file-structure)
6. [Implementation Guide](#implementation-guide)
7. [Hotkey System](#hotkey-system)
8. [PTY Integration](#pty-integration)
9. [Testing Strategy](#testing-strategy)
10. [Success Criteria](#success-criteria)
11. [Troubleshooting](#troubleshooting)

---

## Overview

### Problem
Pi's bash tool runs commands non-interactively — it can't accept user input. This makes interactive CLI tools (Railway, Docker, git rebase, etc.) impossible to use through Pi.

### Solution
Add a split-pane terminal to Pi where both AI and human can interact with bash commands in real-time.

### User Experience

**Default state (conversation only):**
```
┌─────────────────────────────────────────┐
│                                         │
│   Conversation (full width)             │
│                                         │
│   You: deploy the bot                   │
│   Pi: Running railway commands...       │
│                                         │
└─────────────────────────────────────────┘
```

**Toggle ON (split pane):**
```
┌─────────────────────┬───────────────────┐
│                     │                   │
│   Conversation      │   Terminal        │
│                     │   (shared)        │
│   You: deploy bot   │   $ railway login │
│   Pi: running...    │   > open URL...   │
│                     │                   │
└─────────────────────┴───────────────────┘
```

---

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                      Pi TUI Application                     │
├─────────────────────────┬───────────────────────────────────┤
│                         │                                   │
│   ConversationPane      │   TerminalPane                    │
│   ┌───────────────────┐ │   ┌───────────────────────────┐  │
│   │                   │ │   │                           │  │
│   │  - Message list   │ │   │  - PTY output buffer      │  │
│   │  - Input editor   │ │   │  - Command history        │  │
│   │  - Tool output    │ │   │  - Scrollback             │  │
│   │                   │ │   │                           │  │
│   └───────────────────┘ │   └───────────────────────────┘  │
│                         │                                   │
│   Uses: ratatui         │   Uses: portable-pty + ratatui   │
│   Input: Pi keybindings │   Input: PTY stdin              │
│   Output: Pi render     │   Output: PTY stdout            │
│                         │                                   │
├─────────────────────────┴───────────────────────────────────┤
│                      Shared Components                      │
│   - SplitLayout (divider, resize)                           │
│   - KeyboardRouter (input dispatch)                         │
│   - StatusBar (active pane, shell info)                     │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

```
User Input
    │
    ▼
KeyboardRouter
    │
    ├──► ConversationPane (if focused)
    │       │
    │       ▼
    │    Pi's existing keybinding handler
    │
    └──► TerminalPane (if focused)
            │
            ▼
         PTY stdin
            │
            ▼
         Bash/Shell process
            │
            ▼
         PTY stdout
            │
            ▼
         TerminalPane buffer
            │
            ▼
         ratatui render
```

---

## Prerequisites

### Rust Version
- Minimum: Rust 1.70.0 (for `portable-pty` compatibility)
- Recommended: Latest stable

### Pi Version
- Pi must be built from source or have extensible TUI
- Access to `~/.pi/agent/keybindings.json` for custom hotkeys

### System Requirements
- Unix-like OS (Linux, macOS, WSL)
- Terminal with 256-color support
- Minimum terminal size: 80x24

---

## Dependencies

### Cargo.toml additions

```toml
[dependencies]
# TUI rendering
ratatui = "0.27"
crossterm = "0.28"

# PTY management
portable-pty = "0.9"

# Async runtime (if not already using)
tokio = { version = "1", features = ["full"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"
```

### Feature flags

```toml
[features]
default = ["shared-terminal"]
shared-terminal = ["ratatui", "portable-pty", "crossterm"]
```

---

## File Structure

```
pi/
├── crates/
│   └── shared-terminal/
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs              # Public API
│       │   ├── layout.rs           # SplitPaneLayout
│       │   ├── terminal.rs         # TerminalPane (PTY wrapper)
│       │   ├── conversation.rs     # ConversationPane (existing Pi chat)
│       │   ├── keyboard.rs         # KeyboardRouter
│       │   ├── pty.rs              # PTY management
│       │   ├── buffer.rs           # Terminal buffer/scrollback
│       │   ├── status.rs           # StatusBar component
│       │   └── hotkeys.rs          # Hotkey definitions
│       └── tests/
│           ├── layout_test.rs
│           ├── terminal_test.rs
│           └── integration_test.rs
│
├── crates/pi-tui/
│   └── src/
│       └── app.rs                  # Add toggle_terminal() method
│
└── ~/.pi/agent/
    └── keybindings.json            # Add terminal hotkeys
```

---

## Implementation Guide

### Phase 1: Basic Split Pane (Days 1-3)

#### Step 1.1: Create shared-terminal crate

```bash
cd pi/crates
cargo new shared-terminal
```

#### Step 1.2: Implement SplitLayout

```rust
// crates/shared-terminal/src/layout.rs

use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct SplitLayout {
    pub show_terminal: bool,
    pub terminal_width: u16,  // Percentage (33, 50, 66)
    pub divider_position: u16,
}

impl SplitLayout {
    pub fn new() -> Self {
        Self {
            show_terminal: false,
            terminal_width: 50,
            divider_position: 0,
        }
    }

    pub fn toggle(&mut self) {
        self.show_terminal = !self.show_terminal;
    }

    pub fn split(&self, area: Rect) -> (Rect, Rect) {
        if !self.show_terminal {
            return (area, Rect::default());
        }

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(100 - self.terminal_width),
                Constraint::Percentage(self.terminal_width),
            ])
            .split(area);

        (chunks[0], chunks[1])
    }

    pub fn resize_left(&mut self) {
        self.terminal_width = (self.terminal_width - 10).max(20);
    }

    pub fn resize_right(&mut self) {
        self.terminal_width = (self.terminal_width + 10).min(80);
    }
}
```

#### Step 1.3: Implement toggle hotkey

```rust
// In Pi's keybinding handler

pub fn handle_global_key(key: Key) -> Option<Action> {
    match key {
        Key::Ctrl('\\') => Some(Action::ToggleTerminal),
        Key::Ctrl('1') => Some(Action::FocusConversation),
        Key::Ctrl('2') => Some(Action::FocusTerminal),
        _ => None,
    }
}
```

#### Step 1.4: Add to Pi's keybindings.json

```json
{
  "app.terminal.toggle": ["ctrl+\\"],
  "app.terminal.focusConversation": ["ctrl+shift+1"],
  "app.terminal.focusTerminal": ["ctrl+shift+2"],
  "app.terminal.resizeLeft": ["ctrl+shift+left"],
  "app.terminal.resizeRight": ["ctrl+shift+right"],
  "app.terminal.clear": ["ctrl+shift+l"]
}
```

---

### Phase 2: PTY Integration (Days 4-7)

#### Step 2.1: Implement PTY manager

```rust
// crates/shared-terminal/src/pty.rs

use portable_pty::{CommandBuilder, native_pty_system, PtySize, MasterPty};
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct PtyManager {
    pty: Box<dyn MasterPty>,
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
    output_rx: mpsc::Receiver<String>,
}

impl PtyManager {
    pub fn new(shell: &str) -> Result<Self, anyhow::Error> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let cmd = CommandBuilder::new(shell);
        let child = pair.slave.spawn_command(cmd)?;

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let (tx, rx) = mpsc::channel(100);

        // Spawn reader thread
        let mut reader_clone = pair.master.try_clone_reader()?;
        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                match reader_clone.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let output = String::from_utf8_lossy(&buf[..n]).to_string();
                        if tx.send(output).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            pty: pair.master,
            reader,
            writer,
            output_rx: rx,
        })
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), anyhow::Error> {
        self.writer.write_all(data)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<(), anyhow::Error> {
        self.pty.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }

    pub async fn read_output(&mut self) -> Option<String> {
        self.output_rx.recv().await
    }
}
```

#### Step 2.2: Implement TerminalPane

```rust
// crates/shared-terminal/src/terminal.rs

use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::collections::VecDeque;

pub struct TerminalPane {
    buffer: VecDeque<String>,
    max_lines: usize,
    scroll_position: usize,
    shell: String,
}

impl TerminalPane {
    pub fn new(shell: &str) -> Self {
        Self {
            buffer: VecDeque::new(),
            max_lines: 1000,
            scroll_position: 0,
            shell: shell.to_string(),
        }
    }

    pub fn push_line(&mut self, line: String) {
        self.buffer.push_back(line);
        if self.buffer.len() > self.max_lines {
            self.buffer.pop_front();
        }
    }

    pub fn render(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let block = Block::default()
            .title(format!("Terminal ({})", self.shell))
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Green));

        let text: Vec<String> = self.buffer.iter().cloned().collect();
        let paragraph = Paragraph::new(text.join("\n"))
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_position = self.scroll_position.saturating_add(lines);
    }

    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_position = self.scroll_position.saturating_sub(lines);
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.scroll_position = 0;
    }
}
```

#### Step 2.3: Implement KeyboardRouter

```rust
// crates/shared-terminal/src/keyboard.rs

use crossterm::event::{KeyEvent, KeyModifiers, KeyCode};

pub enum InputTarget {
    Conversation,
    Terminal,
    Global,
}

pub struct KeyboardRouter {
    active_pane: InputTarget,
}

impl KeyboardRouter {
    pub fn new() -> Self {
        Self {
            active_pane: InputTarget::Conversation,
        }
    }

    pub fn route(&mut self, key: KeyEvent) -> InputTarget {
        // Global hotkeys (work in both panes)
        match (key.code, key.modifiers) {
            (KeyCode::Char('\\'), KeyModifiers::CONTROL) => {
                return InputTarget::Global;
            }
            (KeyCode::Char('1'), KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                self.active_pane = InputTarget::Conversation;
                return InputTarget::Global;
            }
            (KeyCode::Char('2'), KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                self.active_pane = InputTarget::Terminal;
                return InputTarget::Global;
            }
            _ => {}
        }

        self.active_pane.clone()
    }

    pub fn active_pane(&self) -> &InputTarget {
        &self.active_pane
    }

    pub fn focus_conversation(&mut self) {
        self.active_pane = InputTarget::Conversation;
    }

    pub fn focus_terminal(&mut self) {
        self.active_pane = InputTarget::Terminal;
    }
}
```

---

### Phase 3: Polish & Features (Days 8-10)

#### Step 3.1: StatusBar component

```rust
// crates/shared-terminal/src/status.rs

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct StatusBar {
    shell: String,
    lines: usize,
    scroll: usize,
    total: usize,
    active_pane: String,
}

impl StatusBar {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let status = format!(
            "[{}] {} | Lines: {}/{} | Scroll: {} | Focus: {}",
            if self.active_pane == "Terminal" { "●" } else { "○" },
            self.shell,
            self.lines,
            self.total,
            self.scroll,
            self.active_pane
        );

        let paragraph = Paragraph::new(status)
            .style(Style::default().fg(Color::DarkGray));

        frame.render_widget(paragraph, area);
    }
}
```

#### Step 3.2: Command history

```rust
// Save/load command history

use std::fs;
use std::path::PathBuf;

pub struct CommandHistory {
    commands: Vec<String>,
    position: usize,
    path: PathBuf,
}

impl CommandHistory {
    pub fn load() -> Self {
        let path = dirs::home_dir()
            .unwrap_or_default()
            .join(".pi/terminal-history.json");

        let commands = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        Self {
            commands,
            position: 0,
            path,
        }
    }

    pub fn save(&self) {
        let json = serde_json::to_string_pretty(&self.commands).unwrap();
        fs::write(&self.path, json).ok();
    }

    pub fn push(&mut self, cmd: String) {
        self.commands.push(cmd);
        if self.commands.len() > 1000 {
            self.commands.remove(0);
        }
        self.save();
    }

    pub fn previous(&mut self) -> Option<&str> {
        if self.position > 0 {
            self.position -= 1;
            self.commands.get(self.position).map(|s| s.as_str())
        } else {
            None
        }
    }

    pub fn next(&mut self) -> Option<&str> {
        if self.position < self.commands.len() - 1 {
            self.position += 1;
            self.commands.get(self.position).map(|s| s.as_str())
        } else {
            None
        }
    }
}
```

---

## Hotkey System

### Global Hotkeys (work in both panes)

| Hotkey | Action | Description |
|--------|--------|-------------|
| `ctrl+\` | Toggle terminal | Show/hide terminal pane |
| `ctrl+shift+1` | Focus conversation | Switch to conversation pane |
| `ctrl+shift+2` | Focus terminal | Switch to terminal pane |
| `ctrl+shift+←` | Shrink terminal | Reduce terminal width by 10% |
| `ctrl+shift+→` | Grow terminal | Increase terminal width by 10% |
| `ctrl+shift+l` | Clear terminal | Clear terminal buffer |
| `ctrl+shift+s` | Save scroll | Save scroll position |

### Conversation Pane Hotkeys

| Hotkey | Action | Description |
|--------|--------|-------------|
| All existing Pi hotkeys | As defined | No changes needed |

### Terminal Pane Hotkeys

| Hotkey | Action | Description |
|--------|--------|-------------|
| `ctrl+c` | Send SIGINT | Interrupt current process |
| `ctrl+d` | Send EOF | End of input |
| `ctrl+z` | Send SIGTSTP | Suspend process |
| `ctrl+l` | Clear screen | Clear terminal display |
| `ctrl+r` | History search | Search command history |
| `↑` | Previous command | Navigate history up |
| `↓` | Next command | Navigate history down |
| `pageUp` | Scroll up | Scroll terminal buffer |
| `pageDown` | Scroll down | Scroll terminal buffer |

### Hotkey Configuration

```json
// ~/.pi/agent/keybindings.json
{
  "app.terminal.toggle": ["ctrl+\\"],
  "app.terminal.focusConversation": ["ctrl+shift+1"],
  "app.terminal.focusTerminal": ["ctrl+shift+2"],
  "app.terminal.resizeLeft": ["ctrl+shift+left"],
  "app.terminal.resizeRight": ["ctrl+shift+right"],
  "app.terminal.clear": ["ctrl+shift+l"],
  "app.terminal.scrollUp": ["pageUp"],
  "app.terminal.scrollDown": ["pageDown"],
  "app.terminal.historyUp": ["up"],
  "app.terminal.historyDown": ["down"],
  "app.terminal.interrupt": ["ctrl+c"],
  "app.terminal.eof": ["ctrl+d"],
  "app.terminal.suspend": ["ctrl+z"]
}
```

---

## PTY Integration

### Shell Selection

```rust
fn detect_shell() -> String {
    std::env::var("SHELL")
        .unwrap_or_else(|_| {
            if cfg!(target_os = "macos") {
                "/bin/zsh".to_string()
            } else {
                "/bin/bash".to_string()
            }
        })
}
```

### PTY I/O Handling

```rust
// Async PTY reader
async fn pty_reader(
    mut pty: Box<dyn MasterPty>,
    tx: mpsc::Sender<String>,
) {
    let mut reader = pty.try_clone_reader().unwrap();
    let mut buf = [0u8; 4096];

    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let output = String::from_utf8_lossy(&buf[..n]).to_string();
                if tx.send(output).await.is_err() {
                    break;
                }
            }
            Err(e) => {
                eprintln!("PTY read error: {}", e);
                break;
            }
        }
    }
}

// Async PTY writer
async fn pty_writer(
    mut pty: Box<dyn MasterPty>,
    mut rx: mpsc::Receiver<Vec<u8>>,
) {
    while let Some(input) = rx.recv().await {
        if pty.write_all(&input).is_err() {
            break;
        }
    }
}
```

### Resize Handling

```rust
// Handle terminal resize events
async fn handle_resize(pty: Arc<Mutex<PtyManager>>) {
    let mut signal = tokio::signal::windows::ctrl_c();
    
    loop {
        // Detect terminal size changes
        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        
        let mut pty = pty.lock().await;
        pty.resize(rows, cols).ok();
        
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

---

## Testing Strategy

### Unit Tests

```rust
// crates/shared-terminal/tests/layout_test.rs

#[test]
fn test_split_layout_toggle() {
    let mut layout = SplitLayout::new();
    assert!(!layout.show_terminal);
    
    layout.toggle();
    assert!(layout.show_terminal);
    
    layout.toggle();
    assert!(!layout.show_terminal);
}

#[test]
fn test_split_layout_resize() {
    let mut layout = SplitLayout::new();
    layout.show_terminal = true;
    layout.terminal_width = 50;
    
    layout.resize_left();
    assert_eq!(layout.terminal_width, 40);
    
    layout.resize_right();
    assert_eq!(layout.terminal_width, 50);
}

#[test]
fn test_split_layout_bounds() {
    let mut layout = SplitLayout::new();
    layout.show_terminal = true;
    layout.terminal_width = 20;
    
    layout.resize_left();
    assert_eq!(layout.terminal_width, 20); // Can't go below 20
    
    layout.terminal_width = 80;
    layout.resize_right();
    assert_eq!(layout.terminal_width, 80); // Can't go above 80
}
```

### Integration Tests

```rust
// crates/shared-terminal/tests/integration_test.rs

#[tokio::test]
async fn test_terminal_pane_output() {
    let mut pane = TerminalPane::new("bash");
    
    // Simulate PTY output
    pane.push_line("$ ls".to_string());
    pane.push_line("file1.txt".to_string());
    pane.push_line("file2.txt".to_string());
    
    assert_eq!(pane.buffer.len(), 3);
}

#[tokio::test]
async fn test_keyboard_routing() {
    let mut router = KeyboardRouter::new();
    assert!(matches!(router.active_pane(), InputTarget::Conversation));
    
    // Simulate ctrl+shift+2
    let key = KeyEvent {
        code: KeyCode::Char('2'),
        modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    
    router.route(key);
    assert!(matches!(router.active_pane(), InputTarget::Terminal));
}
```

### E2E Tests

```bash
# Test with Railway CLI
$ railway login
# User clicks URL in browser
$ railway link
# User selects project
$ railway deploy
# User sees deployment progress
```

---

## Success Criteria

### Functional Requirements

- [ ] Terminal pane toggles with `ctrl+\`
- [ ] Both panes visible simultaneously
- [ ] Keyboard input routes correctly based on active pane
- [ ] PTY output renders in real-time
- [ ] Scrollback buffer works (1000+ lines)
- [ ] Command history persists across sessions
- [ ] Resize works with `ctrl+shift+←/→`
- [ ] Clear works with `ctrl+shift+l`

### Non-Functional Requirements

- [ ] No hotkey conflicts with existing Pi bindings
- [ ] Works with Railway, Docker, git, npm
- [ ] Performance: < 16ms frame time
- [ ] Memory: < 50MB for terminal pane
- [ ] Works on Linux, macOS, WSL

### Compatibility

- [ ] Existing Pi hotkeys still work
- [ ] Pi's conversation pane unchanged
- [ ] Pi's keybinding system extended (not replaced)
- [ ] No breaking changes to Pi's API

---

## Troubleshooting

### Common Issues

**PTY not spawning:**
- Check shell path exists: `which bash`
- Check permissions: `ls -la /bin/bash`
- Try explicit shell: `SHELL=/bin/bash pi`

**Terminal not rendering:**
- Check terminal color support: `echo $TERM`
- Try different terminal emulator
- Check ratatui version compatibility

**Hotkeys not working:**
- Check `~/.pi/agent/keybindings.json`
- Run `/reload` in Pi
- Check for conflicts with other tools

**Performance issues:**
- Reduce scrollback buffer size
- Disable mouse support
- Use simpler shell (ash instead of zsh)

---

## Future Enhancements

### v2.0
- Multiple terminal tabs
- Split terminal vertically/horizontally
- Copy/paste between panes
- Terminal themes

### v3.0
- Terminal recording/replay
- Custom shell configs
- Plugin system for terminal extensions
- Web-based terminal access

---

## References

- [ratatui documentation](https://ratatui.rs/)
- [portable-pty documentation](https://docs.rs/portable-pty/)
- [Pi keybindings](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/keybindings.md)
- [crossterm documentation](https://docs.rs/crossterm/)
