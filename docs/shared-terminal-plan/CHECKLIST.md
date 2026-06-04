# Implementation Checklist

## Pre-Implementation

- [ ] Review Pi's existing TUI code
- [ ] Understand Pi's keybinding system
- [ ] Set up development environment
- [ ] Create shared-terminal crate

## Phase 1: Basic Split Pane (Days 1-3)

### Day 1: Project Setup
- [ ] Create `crates/shared-terminal/Cargo.toml`
- [ ] Add dependencies (ratatui, portable-pty, crossterm)
- [ ] Create `src/lib.rs` with public API
- [ ] Create `src/layout.rs` with SplitLayout

### Day 2: Layout Implementation
- [ ] Implement `SplitLayout::new()`
- [ ] Implement `SplitLayout::toggle()`
- [ ] Implement `SplitLayout::split()`
- [ ] Implement `SplitLayout::resize_left/right()`
- [ ] Write unit tests for layout

### Day 3: Pi Integration
- [ ] Add `toggle_terminal()` to Pi's TUI
- [ ] Add `ctrl+\` hotkey binding
- [ ] Test toggle functionality
- [ ] Verify no conflicts with existing hotkeys

## Phase 2: PTY Integration (Days 4-7)

### Day 4: PTY Manager
- [ ] Create `src/pty.rs`
- [ ] Implement `PtyManager::new()`
- [ ] Implement `PtyManager::write()`
- [ ] Implement `PtyManager::resize()`
- [ ] Spawn bash shell

### Day 5: Terminal Pane
- [ ] Create `src/terminal.rs`
- [ ] Implement `TerminalPane::new()`
- [ ] Implement `TerminalPane::push_line()`
- [ ] Implement `TerminalPane::render()`
- [ ] Test PTY output rendering

### Day 6: Keyboard Router
- [ ] Create `src/keyboard.rs`
- [ ] Implement `KeyboardRouter::route()`
- [ ] Route input to PTY when terminal focused
- [ ] Handle global hotkeys

### Day 7: Integration
- [ ] Connect PTY to TerminalPane
- [ ] Wire keyboard input to PTY
- [ ] Test Railway CLI interaction
- [ ] Test Docker commands

## Phase 3: Polish & Features (Days 8-10)

### Day 8: Scrollback & History
- [ ] Create `src/buffer.rs`
- [ ] Implement scrollback buffer (1000 lines)
- [ ] Create `src/history.rs`
- [ ] Implement command history persistence

### Day 9: Visual Indicators
- [ ] Create `src/status.rs`
- [ ] Implement StatusBar
- [ ] Add active pane highlight
- [ ] Add resize handles

### Day 10: Polish
- [ ] Add mouse support for divider
- [ ] Add visual feedback for hotkeys
- [ ] Test edge cases
- [ ] Write documentation

## Phase 4: Pi Integration (Days 11-13)

### Day 11: Keybinding Integration
- [ ] Add terminal hotkeys to Pi's keybinding system
- [ ] Update `~/.pi/agent/keybindings.json`
- [ ] Test hotkey conflicts
- [ ] Verify existing hotkeys still work

### Day 12: Testing
- [ ] Unit tests for all components
- [ ] Integration tests for PTY
- [ ] E2E tests with Railway
- [ ] E2E tests with Docker
- [ ] E2E tests with git

### Day 13: Documentation
- [ ] Update Pi documentation
- [ ] Add hotkey reference
- [ ] Add troubleshooting guide
- [ ] Create user guide

## Post-Implementation

- [ ] Code review
- [ ] Performance testing
- [ ] Security audit
- [ ] Release preparation
- [ ] User feedback collection

## Dependencies

| Task | Blocked By | Blocks |
|------|------------|--------|
| Phase 1 | None | Phase 2, 3, 4 |
| Phase 2 | Phase 1 | Phase 3, 4 |
| Phase 3 | Phase 2 | Phase 4 |
| Phase 4 | Phase 1, 2, 3 | None |

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| PTY compatibility issues | Medium | High | Test on multiple platforms |
| Hotkey conflicts | Low | Medium | Verify against existing bindings |
| Performance issues | Low | Medium | Profile and optimize |
| Memory leaks | Low | High | Use RAII and drop checks |

## Success Metrics

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Toggle latency | < 100ms | Measure time from keypress to render |
| PTY output latency | < 50ms | Measure time from PTY output to render |
| Memory usage | < 50MB | Monitor memory in test environment |
| CPU usage | < 10% idle | Monitor CPU in test environment |
| Hotkey conflicts | 0 | Verify against existing bindings |
| Test coverage | > 80% | Run cargo-tarpaulin |
