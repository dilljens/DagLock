# Quick Start Guide

## For AI Implementers

This plan is designed for an AI to implement the shared terminal feature in Pi. Here's what you need to know:

## Key Concepts

1. **Split-pane TUI**: Pi's screen splits into conversation (left) and terminal (right)
2. **PTY integration**: Interactive bash shell runs in the terminal pane
3. **Hotkey system**: `ctrl+\` toggles terminal, `ctrl+shift+1/2` switches focus
4. **No conflicts**: All hotkeys verified against existing Pi bindings

## Implementation Order

1. **Phase 1 (Days 1-3)**: Basic split pane with ratatui
2. **Phase 2 (Days 4-7)**: PTY integration with portable-pty
3. **Phase 3 (Days 8-10)**: Polish and features
4. **Phase 4 (Days 11-13)**: Pi integration and testing

## Critical Files

| File | Purpose |
|------|---------|
| `crates/shared-terminal/src/layout.rs` | Split pane layout |
| `crates/shared-terminal/src/terminal.rs` | Terminal pane rendering |
| `crates/shared-terminal/src/pty.rs` | PTY management |
| `crates/shared-terminal/src/keyboard.rs` | Input routing |
| `~/.pi/agent/keybindings.json` | Hotkey configuration |

## Quick Commands

```bash
# Create crate
cd pi/crates && cargo new shared-terminal

# Add to Pi workspace
# Add to pi/Cargo.toml workspace members

# Test locally
cargo test -p shared-terminal

# Run Pi with feature
cargo run --features shared-terminal
```

## Hotkey Reference

| Hotkey | Action |
|--------|--------|
| `ctrl+\` | Toggle terminal |
| `ctrl+shift+1` | Focus conversation |
| `ctrl+shift+2` | Focus terminal |
| `ctrl+shift+←` | Shrink terminal |
| `ctrl+shift+→` | Grow terminal |
| `ctrl+shift+l` | Clear terminal |

## Common Pitfalls

1. **Don't use existing hotkeys** - Check `~/.pi/agent/keybindings.json`
2. **Handle PTY cleanup** - Use `Drop` trait to kill shell on exit
3. **Buffer overflow** - Limit scrollback to 1000 lines
4. **Resize events** - Handle terminal resize with `SIGWINCH`
5. **Async I/O** - Use Tokio for non-blocking PTY reads

## Testing Checklist

- [ ] Toggle works with `ctrl+\`
- [ ] Both panes visible
- [ ] Input routes to correct pane
- [ ] PTY output renders
- [ ] Scrollback works
- [ ] No hotkey conflicts
- [ ] Works with Railway
- [ ] Works with Docker
- [ ] Works with git

## Resources

- [ratatui docs](https://ratatui.rs/)
- [portable-pty docs](https://docs.rs/portable-pty/)
- [Pi keybindings](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/keybindings.md)
