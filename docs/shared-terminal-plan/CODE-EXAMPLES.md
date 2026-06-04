# Code Examples

## Complete SplitLayout Implementation

```rust
// crates/shared-terminal/src/layout.rs

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Configuration for the split-pane layout
pub struct SplitLayout {
    /// Whether the terminal pane is visible
    pub show_terminal: bool,
    
    /// Terminal pane width as percentage (20-80)
    pub terminal_width: u16,
    
    /// Current divider position in columns
    pub divider_position: u16,
    
    /// Minimum terminal width percentage
    pub min_width: u16,
    
    /// Maximum terminal width percentage
    pub max_width: u16,
    
    /// Step size for resize (percentage)
    pub resize_step: u16,
}

impl SplitLayout {
    /// Create a new SplitLayout with default settings
    pub fn new() -> Self {
        Self {
            show_terminal: false,
            terminal_width: 50,
            divider_position: 0,
            min_width: 20,
            max_width: 80,
            resize_step: 10,
        }
    }

    /// Toggle terminal pane visibility
    pub fn toggle(&mut self) {
        self.show_terminal = !self.show_terminal;
    }

    /// Split the given area into conversation and terminal panes
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

    /// Resize terminal pane smaller
    pub fn resize_left(&mut self) {
        self.terminal_width = self.terminal_width.saturating_sub(self.resize_step);
        if self.terminal_width < self.min_width {
            self.terminal_width = self.min_width;
        }
    }

    /// Resize terminal pane larger
    pub fn resize_right(&mut self) {
        self.terminal_width = self.terminal_width.saturating_add(self.resize_step);
        if self.terminal_width > self.max_width {
            self.terminal_width = self.max_width;
        }
    }

    /// Set terminal width to a specific percentage
    pub fn set_width(&mut self, width: u16) {
        self.terminal_width = width.clamp(self.min_width, self.max_width);
    }

    /// Get the current split ratio as a string
    pub fn ratio_string(&self) -> String {
        if !self.show_terminal {
            "100:0".to_string()
        } else {
            format!("{}:{}", 100 - self.terminal_width, self.terminal_width)
        }
    }
}

impl Default for SplitLayout {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_layout() {
        let layout = SplitLayout::new();
        assert!(!layout.show_terminal);
        assert_eq!(layout.terminal_width, 50);
    }

    #[test]
    fn test_toggle() {
        let mut layout = SplitLayout::new();
        assert!(!layout.show_terminal);
        
        layout.toggle();
        assert!(layout.show_terminal);
        
        layout.toggle();
        assert!(!layout.show_terminal);
    }

    #[test]
    fn test_split() {
        let layout = SplitLayout {
            show_terminal: true,
            terminal_width: 50,
            ..Default::default()
        };
        
        let area = Rect::new(0, 0, 100, 24);
        let (left, right) = layout.split(area);
        
        assert_eq!(left.width, 50);
        assert_eq!(right.width, 50);
    }

    #[test]
    fn test_split_hidden() {
        let layout = SplitLayout {
            show_terminal: false,
            ..Default::default()
        };
        
        let area = Rect::new(0, 0, 100, 24);
        let (left, right) = layout.split(area);
        
        assert_eq!(left.width, 100);
        assert_eq!(right.width, 0);
    }

    #[test]
    fn test_resize_bounds() {
        let mut layout = SplitLayout::new();
        layout.show_terminal = true;
        layout.terminal_width = 20;
        
        layout.resize_left();
        assert_eq!(layout.terminal_width, 20); // Can't go below min
        
        layout.terminal_width = 80;
        layout.resize_right();
        assert_eq!(layout.terminal_width, 80); // Can't go above max
    }
}
```

## Complete PTY Manager Implementation

```rust
// crates/shared-terminal/src/pty.rs

use portable_pty::{CommandBuilder, MasterPty, native_pty_system, PtySize};
use std::io::{Read, Write};
use tokio::sync::mpsc;

/// Manages a pseudo-terminal (PTY) for interactive shell sessions
pub struct PtyManager {
    master: Box<dyn MasterPty>,
    writer: Box<dyn Write + Send>,
    output_rx: mpsc::Receiver<String>,
    child: Option<Box<dyn portable_pty::Child + Send>>,
}

impl PtyManager {
    /// Create a new PTY manager with the specified shell
    pub fn new(shell: &str) -> Result<Self, anyhow::Error> {
        let pty_system = native_pty_system();
        
        // Open PTY pair
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Spawn shell process
        let cmd = CommandBuilder::new(shell);
        let child = pair.slave.spawn_command(cmd)?;

        // Get reader/writer handles
        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        // Create output channel
        let (tx, rx) = mpsc::channel(1000);

        // Spawn async reader task
        let mut async_reader = pair.master.try_clone_reader()?;
        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                match async_reader.read(&mut buf) {
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
            master: pair.master,
            writer,
            output_rx: rx,
            child: Some(child),
        })
    }

    /// Write input to the PTY
    pub fn write(&mut self, data: &[u8]) -> Result<(), anyhow::Error> {
        self.writer.write_all(data)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Write a string to the PTY
    pub fn write_str(&mut self, s: &str) -> Result<(), anyhow::Error> {
        self.write(s.as_bytes())
    }

    /// Send a key sequence to the PTY
    pub fn send_key(&mut self, key: &str) -> Result<(), anyhow::Error> {
        let bytes = match key {
            "enter" => b"\r".to_vec(),
            "backspace" => b"\x7f".to_vec(),
            "tab" => b"\t".to_vec(),
            "ctrl+c" => b"\x03".to_vec(),
            "ctrl+d" => b"\x04".to_vec(),
            "ctrl+z" => b"\x1a".to_vec(),
            "ctrl+l" => b"\x0c".to_vec(),
            "up" => b"\x1b[A".to_vec(),
            "down" => b"\x1b[B".to_vec(),
            "left" => b"\x1b[D".to_vec(),
            "right" => b"\x1b[C".to_vec(),
            "page_up" => b"\x1b[5~".to_vec(),
            "page_down" => b"\x1b[6~".to_vec(),
            "home" => b"\x1b[H".to_vec(),
            "end" => b"\x1b[F".to_vec(),
            _ => key.as_bytes().to_vec(),
        };
        self.write(&bytes)
    }

    /// Resize the PTY
    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<(), anyhow::Error> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }

    /// Receive next output from PTY (async)
    pub async fn read_output(&mut self) -> Option<String> {
        self.output_rx.recv().await
    }

    /// Try to receive output without blocking
    pub fn try_read_output(&mut self) -> Option<String> {
        self.output_rx.try_recv().ok()
    }

    /// Kill the PTY process
    pub fn kill(&mut self) {
        if let Some(mut child) = self.child.take() {
            child.kill().ok();
        }
    }

    /// Check if the PTY process is still running
    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }
}

impl Drop for PtyManager {
    fn drop(&mut self) {
        self.kill();
    }
}

/// Detect the user's default shell
pub fn detect_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(target_os = "macos") {
            "/bin/zsh".to_string()
        } else {
            "/bin/bash".to_string()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_pty_spawn() {
        let mut pty = PtyManager::new("bash").unwrap();
        assert!(pty.is_running());
        
        // Wait for shell prompt
        timeout(Duration::from_secs(1), async {
            while let Some(output) = pty.read_output().await {
                if output.contains("$") || output.contains("#") {
                    break;
                }
            }
        }).await.ok();
        
        pty.kill();
        assert!(!pty.is_running());
    }

    #[tokio::test]
    async fn test_pty_write() {
        let mut pty = PtyManager::new("bash").unwrap();
        
        // Wait for shell
        timeout(Duration::from_secs(1), async {
            while let Some(output) = pty.read_output().await {
                if output.contains("$") || output.contains("#") {
                    break;
                }
            }
        }).await.ok();
        
        // Write a command
        pty.write_str("echo hello\r").unwrap();
        
        // Read output
        let mut found_hello = false;
        timeout(Duration::from_secs(1), async {
            while let Some(output) = pty.read_output().await {
                if output.contains("hello") {
                    found_hello = true;
                    break;
                }
            }
        }).await.ok();
        
        assert!(found_hello);
        pty.kill();
    }
}
```

## Complete Keyboard Router Implementation

```rust
// crates/shared-terminal/src/keyboard.rs

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

/// Represents which pane should receive input
#[derive(Debug, Clone, PartialEq)]
pub enum InputTarget {
    /// Pi's conversation pane
    Conversation,
    
    /// Terminal pane
    Terminal,
    
    /// Global action (handled by router)
    Global,
}

/// Routes keyboard input to the appropriate pane
pub struct KeyboardRouter {
    active_pane: InputTarget,
    terminal_visible: bool,
}

impl KeyboardRouter {
    /// Create a new KeyboardRouter
    pub fn new() -> Self {
        Self {
            active_pane: InputTarget::Conversation,
            terminal_visible: false,
        }
    }

    /// Route a key event to the appropriate target
    pub fn route(&mut self, key: KeyEvent) -> InputTarget {
        // Only handle key press events
        if key.kind != KeyEventKind::Press {
            return InputTarget::Global;
        }

        // Check global hotkeys first
        match (key.code, key.modifiers) {
            // Toggle terminal: ctrl+\
            (KeyCode::Char('\\'), KeyModifiers::CONTROL) => {
                self.terminal_visible = !self.terminal_visible;
                return InputTarget::Global;
            }
            
            // Focus conversation: ctrl+shift+1
            (KeyCode::Char('1'), KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                self.active_pane = InputTarget::Conversation;
                return InputTarget::Global;
            }
            
            // Focus terminal: ctrl+shift+2
            (KeyCode::Char('2'), KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                self.active_pane = InputTarget::Terminal;
                return InputTarget::Global;
            }
            
            // Resize left: ctrl+shift+left
            (KeyCode::Left, KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                return InputTarget::Global;
            }
            
            // Resize right: ctrl+shift+right
            (KeyCode::Right, KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                return InputTarget::Global;
            }
            
            // Clear terminal: ctrl+shift+l
            (KeyCode::Char('l'), KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                return InputTarget::Global;
            }
            
            _ => {}
        }

        // If terminal is not visible, route to conversation
        if !self.terminal_visible {
            return InputTarget::Conversation;
        }

        // Route to active pane
        self.active_pane.clone()
    }

    /// Get the currently active pane
    pub fn active_pane(&self) -> &InputTarget {
        &self.active_pane
    }

    /// Check if terminal is visible
    pub fn terminal_visible(&self) -> bool {
        self.terminal_visible
    }

    /// Focus the conversation pane
    pub fn focus_conversation(&mut self) {
        self.active_pane = InputTarget::Conversation;
    }

    /// Focus the terminal pane
    pub fn focus_terminal(&mut self) {
        self.active_pane = InputTarget::Terminal;
    }

    /// Toggle terminal visibility
    pub fn toggle_terminal(&mut self) {
        self.terminal_visible = !self.terminal_visible;
    }
}

impl Default for KeyboardRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_toggle_terminal() {
        let mut router = KeyboardRouter::new();
        assert!(!router.terminal_visible());
        
        let key = make_key(KeyCode::Char('\\'), KeyModifiers::CONTROL);
        router.route(key);
        
        assert!(router.terminal_visible());
    }

    #[test]
    fn test_focus_terminal() {
        let mut router = KeyboardRouter::new();
        assert_eq!(router.active_pane(), &InputTarget::Conversation);
        
        let key = make_key(KeyCode::Char('2'), KeyModifiers::CONTROL | KeyModifiers::SHIFT);
        router.route(key);
        
        assert_eq!(router.active_pane(), &InputTarget::Terminal);
    }

    #[test]
    fn test_route_to_conversation_when_hidden() {
        let mut router = KeyboardRouter::new();
        router.terminal_visible = false;
        
        let key = make_key(KeyCode::Char('a'), KeyModifiers::empty());
        let target = router.route(key);
        
        assert_eq!(target, InputTarget::Conversation);
    }
}
```

## Complete TerminalPane Implementation

```rust
// crates/shared-terminal/src/terminal.rs

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::collections::VecDeque;

/// Terminal pane that displays PTY output
pub struct TerminalPane {
    /// Output buffer (scrollback)
    buffer: VecDeque<String>,
    
    /// Maximum lines to keep in buffer
    max_lines: usize,
    
    /// Current scroll position
    scroll_position: usize,
    
    /// Shell being used
    shell: String,
    
    /// Whether the pane is focused
    focused: bool,
}

impl TerminalPane {
    /// Create a new TerminalPane
    pub fn new(shell: &str) -> Self {
        Self {
            buffer: VecDeque::new(),
            max_lines: 1000,
            scroll_position: 0,
            shell: shell.to_string(),
            focused: false,
        }
    }

    /// Add a line to the buffer
    pub fn push_line(&mut self, line: String) {
        // Split multiline output into individual lines
        for l in line.lines() {
            self.buffer.push_back(l.to_string());
        }
        
        // Trim buffer if too large
        while self.buffer.len() > self.max_lines {
            self.buffer.pop_front();
        }
        
        // Auto-scroll to bottom
        self.scroll_position = 0;
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.scroll_position = 0;
    }

    /// Scroll up by lines
    pub fn scroll_up(&mut self, lines: usize) {
        let max_scroll = self.buffer.len().saturating_sub(1);
        self.scroll_position = (self.scroll_position + lines).min(max_scroll);
    }

    /// Scroll down by lines
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_position = self.scroll_position.saturating_sub(lines);
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll_position = self.buffer.len().saturating_sub(1);
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_position = 0;
    }

    /// Get visible lines based on scroll position
    pub fn visible_lines(&self, height: usize) -> Vec<&str> {
        let end = self.buffer.len().saturating_sub(self.scroll_position);
        let start = end.saturating_sub(height);
        
        self.buffer.iter()
            .skip(start)
            .take(height)
            .map(|s| s.as_str())
            .collect()
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Render the terminal pane
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let border_style = if self.focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .title(format!(" Terminal ({}) ", self.shell))
            .borders(Borders::ALL)
            .style(border_style);

        // Get visible lines
        let height = area.height.saturating_sub(2) as usize; // Account for borders
        let lines: Vec<Line> = self.visible_lines(height)
            .iter()
            .map(|l| Line::from(Span::raw(*l)))
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);

        // Render scroll indicator
        if self.scroll_position > 0 {
            let indicator = format!(" ↑{} ", self.scroll_position);
            let indicator_area = Rect {
                x: area.x + area.width - 6,
                y: area.y,
                width: 6,
                height: 1,
            };
            let indicator_widget = Paragraph::new(indicator)
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(indicator_widget, indicator_area);
        }
    }

    /// Get buffer statistics
    pub fn stats(&self) -> TerminalStats {
        TerminalStats {
            total_lines: self.buffer.len(),
            visible_lines: self.scroll_position,
            scroll_position: self.scroll_position,
        }
    }
}

/// Statistics about the terminal buffer
pub struct TerminalStats {
    pub total_lines: usize,
    pub visible_lines: usize,
    pub scroll_position: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_line() {
        let mut pane = TerminalPane::new("bash");
        pane.push_line("$ ls".to_string());
        pane.push_line("file1.txt".to_string());
        
        assert_eq!(pane.buffer.len(), 2);
    }

    #[test]
    fn test_push_multiline() {
        let mut pane = TerminalPane::new("bash");
        pane.push_line("line1\nline2\nline3".to_string());
        
        assert_eq!(pane.buffer.len(), 3);
    }

    #[test]
    fn test_scroll() {
        let mut pane = TerminalPane::new("bash");
        
        // Add 100 lines
        for i in 0..100 {
            pane.push_line(format!("line {}", i));
        }
        
        // Scroll up
        pane.scroll_up(10);
        assert_eq!(pane.scroll_position, 10);
        
        // Scroll down
        pane.scroll_down(5);
        assert_eq!(pane.scroll_position, 5);
    }

    #[test]
    fn test_scroll_bounds() {
        let mut pane = TerminalPane::new("bash");
        
        for i in 0..10 {
            pane.push_line(format!("line {}", i));
        }
        
        // Can't scroll past top
        pane.scroll_up(100);
        assert_eq!(pane.scroll_position, 9);
        
        // Can't scroll past bottom
        pane.scroll_down(100);
        assert_eq!(pane.scroll_position, 0);
    }

    #[test]
    fn test_clear() {
        let mut pane = TerminalPane::new("bash");
        pane.push_line("test".to_string());
        
        pane.clear();
        assert!(pane.buffer.is_empty());
    }
}
```

## Integration Example

```rust
// crates/shared-terminal/src/lib.rs

use ratatui::Frame;
use ratatui::layout::Rect;
use tokio::sync::mpsc;

pub mod layout;
pub mod terminal;
pub mod keyboard;
pub mod pty;

pub use layout::SplitLayout;
pub use terminal::TerminalPane;
pub use keyboard::{KeyboardRouter, InputTarget};
pub use pty::{PtyManager, detect_shell};

/// Main shared terminal component
pub struct SharedTerminal {
    layout: SplitLayout,
    terminal: TerminalPane,
    keyboard: KeyboardRouter,
    pty: Option<PtyManager>,
    shell: String,
}

impl SharedTerminal {
    /// Create a new SharedTerminal
    pub fn new() -> Self {
        let shell = detect_shell();
        
        Self {
            layout: SplitLayout::new(),
            terminal: TerminalPane::new(&shell),
            keyboard: KeyboardRouter::new(),
            pty: None,
            shell,
        }
    }

    /// Toggle terminal pane visibility
    pub fn toggle(&mut self) {
        self.layout.toggle();
        self.keyboard.toggle_terminal();
        
        // Spawn PTY when terminal becomes visible
        if self.layout.show_terminal && self.pty.is_none() {
            self.spawn_pty();
        }
    }

    /// Spawn a new PTY session
    fn spawn_pty(&mut self) {
        match PtyManager::new(&self.shell) {
            Ok(pty) => {
                self.pty = Some(pty);
                self.terminal.push_line(format!("Terminal ready ({})", self.shell));
            }
            Err(e) => {
                self.terminal.push_line(format!("Failed to start terminal: {}", e));
            }
        }
    }

    /// Handle a key event
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        let target = self.keyboard.route(key);
        
        match target {
            InputTarget::Global => {
                // Handle global actions
                match (key.code, key.modifiers) {
                    (crossterm::event::KeyCode::Char('\\'), crossterm::event::KeyModifiers::CONTROL) => {
                        self.toggle();
                    }
                    _ => {}
                }
            }
            InputTarget::Terminal => {
                // Send to PTY
                if let Some(pty) = &mut self.pty {
                    let key_str = match key.code {
                        crossterm::event::KeyCode::Enter => "enter",
                        crossterm::event::KeyCode::Backspace => "backspace",
                        crossterm::event::KeyCode::Tab => "tab",
                        crossterm::event::KeyCode::Up => "up",
                        crossterm::event::KeyCode::Down => "down",
                        crossterm::event::KeyCode::Left => "left",
                        crossterm::event::KeyCode::Right => "right",
                        crossterm::event::KeyCode::PageUp => "page_up",
                        crossterm::event::KeyCode::PageDown => "page_down",
                        crossterm::event::KeyCode::Char(c) => {
                            let mut s = String::new();
                            if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                                s.push('\x00');
                            }
                            s.push(c);
                            if pty.write_str(&s).is_err() {
                                self.terminal.push_line("Failed to write to terminal".to_string());
                            }
                            return;
                        }
                        _ => return,
                    };
                    
                    if pty.send_key(key_str).is_err() {
                        self.terminal.push_line("Failed to send key to terminal".to_string());
                    }
                }
            }
            InputTarget::Conversation => {
                // Let Pi handle the key
                // (handled by Pi's existing keybinding system)
            }
        }
    }

    /// Update terminal with PTY output
    pub fn update_terminal(&mut self) {
        if let Some(pty) = &mut self.pty {
            while let Some(output) = pty.try_read_output() {
                self.terminal.push_line(output);
            }
        }
    }

    /// Render the component
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let (conversation_area, terminal_area) = self.layout.split(area);
        
        // Render terminal if visible
        if self.layout.show_terminal {
            let mut terminal = TerminalPane::new(&self.shell);
            terminal.set_focused(
                *self.keyboard.active_pane() == InputTarget::Terminal
            );
            terminal.render(frame, terminal_area);
        }
        
        // Conversation pane is rendered by Pi's existing code
        // (passed conversation_area to Pi's render function)
    }

    /// Resize terminal PTY
    pub fn resize(&mut self, rows: u16, cols: u16) {
        if let Some(pty) = &mut self.pty {
            pty.resize(rows, cols).ok();
        }
    }
}

impl Default for SharedTerminal {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SharedTerminal {
    fn drop(&mut self) {
        if let Some(pty) = self.pty.take() {
            pty.kill();
        }
    }
}
```
