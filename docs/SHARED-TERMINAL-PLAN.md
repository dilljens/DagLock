# Shared Terminal Feature — Implementation Plan

## Overview

Add a split-pane terminal to Pi where both AI and human can interact with bash commands in real-time. This enables interactive CLI tools like Railway, Docker, git, etc.

## Architecture

```
┌─────────────────────┬─────────────────────┐
│   ratatui           │   portable-pty      │
│   ┌───────────────┐ │   ┌───────────────┐ │
│   │ Conversation  │ │   │ Bash/Shell    │ │
│   │ (Pi chat)     │ │   │ (interactive) │ │
│   └───────────────┘ │   └───────────────┘ │
└─────────────────────┴─────────────────────┘
```

## Components

### 1. TUI Layout (ratatui)

**Split-pane renderer:**
- Left pane: Conversation (existing Pi chat)
- Right pane: Terminal (interactive shell)
- Draggable divider between panes
- Toggle visibility with hotkey

### 2. Terminal Pane (portable-pty)

**PTY management:**
- Spawn bash/zsh/fish shell
- Handle input/output streams
- Resize terminal on window resize
- Scrollback buffer (1000+ lines)

### 3. Keyboard Handler

**Input routing:**
- When focused on conversation: normal Pi input
- When focused on terminal: all input goes to PTY
- Hotkeys work from either pane

### 4. Toggle Mechanism

**Show/hide terminal pane:**
- `ctrl+\` — Toggle terminal visibility
- Terminal remembers state when hidden
- Divider position preserved

## Hotkey Reference

| Hotkey | Action | Conflict? |
|--------|--------|-----------|
| `ctrl+\` | Toggle terminal pane | None |
| `ctrl+shift+1` | Focus conversation | None |
| `ctrl+shift+2` | Focus terminal | None |
| `ctrl+shift+3` | Resize terminal (50%) | None |
| `ctrl+shift+e` | Expand terminal (75%) | None |
| `ctrl+shift+←` | Shrink terminal | None |
| `ctrl+shift+→` | Grow terminal | None |
| `ctrl+shift+l` | Clear terminal | None |
| `ctrl+shift+s` | Save terminal scroll | None |

## Implementation Phases

### Phase 1: Basic Split Pane (2-3 days)
- [ ] Add ratatui dependency
- [ ] Create SplitLayout component
- [ ] Implement toggle hotkey (`ctrl+\`)
- [ ] Render conversation in left pane
- [ ] Render placeholder in right pane

### Phase 2: PTY Integration (3-4 days)
- [ ] Add portable-pty dependency
- [ ] Spawn bash shell on toggle
- [ ] Route keyboard input to PTY
- [ ] Render PTY output in terminal pane
- [ ] Handle resize events

### Phase 3: Polish & Features (2-3 days)
- [ ] Scrollback buffer
- [ ] Command history
- [ ] Visual indicators
- [ ] Status bar
- [ ] Mouse support for divider

### Phase 4: Pi Integration (2-3 days)
- [ ] Integrate with Pi's keybinding system
- [ ] Add to `~/.pi/agent/keybindings.json`
- [ ] Test with Railway, Docker, git
- [ ] Documentation

## Dependencies

```toml
[dependencies]
ratatui = "0.27"
portable-pty = "0.9"
tokio = { version = "1", features = ["full"] }
crossterm = "0.28"
```

## Success Metrics

- [ ] Terminal pane toggles with `ctrl+\`
- [ ] Both panes visible simultaneously
- [ ] Keyboard input routes correctly
- [ ] PTY output renders in real-time
- [ ] Scrollback works
- [ ] No hotkey conflicts with existing Pi bindings
- [ ] Works with Railway, Docker, git
- [ ] Performance: < 16ms frame time
