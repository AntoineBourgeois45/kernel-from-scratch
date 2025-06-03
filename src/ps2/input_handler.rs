use crate::{
    ps2::keyboard::{KeyEvent, KeyCode, keyboard_has_data, keyboard_read_scancode},
    vga::terminal::{terminal, LogLevel},
    kprint,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Terminal,
    Navigation,
}

pub struct InputHandler {
    pub mode: InputMode,
    help_shown: bool,
}

impl InputHandler {
    pub const fn new() -> Self {
        Self {
            mode: InputMode::Terminal,
            help_shown: false,
        }
    }

    pub fn poll_and_handle_input(&mut self, keyboard_state: &mut crate::ps2::keyboard::KeyboardState) {
        if keyboard_has_data() {
            let scancode = keyboard_read_scancode();
            
            if let Some(event) = keyboard_state.process_scancode(scancode) {
                self.handle_key_event(event);
            }
        }
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        if !event.pressed {
            return;
        }

        match self.mode {
            InputMode::Terminal => self.handle_terminal_input(event),
            InputMode::Navigation => self.handle_navigation_input(event),
        }
    }

    fn handle_terminal_input(&mut self, event: KeyEvent) {
        let terminal = terminal();
        
        match event.key {
            KeyCode::ArrowLeft => unsafe { terminal.move_cursor_left() },
            KeyCode::ArrowRight => unsafe { terminal.move_cursor_right() },
            KeyCode::ArrowUp => unsafe { terminal.move_cursor_up() },
            KeyCode::ArrowDown => unsafe { terminal.move_cursor_down() },
            
            KeyCode::Home => unsafe { terminal.move_to_line_start() },
            KeyCode::End => unsafe { terminal.move_to_line_end() },
            
            KeyCode::PageUp => unsafe { terminal.page_up() },
            KeyCode::PageDown => unsafe { terminal.page_down() },
            
            KeyCode::F1 => self.show_help(),
            KeyCode::F2 => self.toggle_input_mode(),
            KeyCode::F3 => unsafe { 
                terminal.clear_screen();
                kprint!(LogLevel::Info, "Screen cleared");
            },
            KeyCode::F4 => unsafe {
                if terminal.cursor_visible {
                    terminal.disable_cursor();
                    kprint!(LogLevel::Info, "Cursor disabled");
                } else {
                    terminal.enable_cursor();
                    kprint!(LogLevel::Info, "Cursor enabled");
                }
            },
            
            _ if event.ctrl => self.handle_ctrl_shortcuts(event),
            
            _ => {
                if let Some(ch) = event.to_char() {
                    unsafe { terminal.put_char(ch as u8) };
                } else {
                    let event_clone = event.clone();
                    match event_clone.key {
                        KeyCode::Escape => {
                            kprint!(LogLevel::Info, "[ESC pressed]");
                        },
                        KeyCode::Insert => {
                            kprint!(LogLevel::Info, "[INSERT pressed]");
                        },
                        KeyCode::Delete => {
                            unsafe {
                                let current_pos = terminal.row * 80 + terminal.column;
                                if terminal.column < 79 {
                                    for x in terminal.column..79 {
                                        let next_index = terminal.row * 80 + x + 1;
                                        let next_char = if x == 78 { 
                                            b' ' 
                                        } else { 
                                            let next_entry = core::ptr::read_volatile(terminal.buffer.add(next_index));
                                            (next_entry & 0xFF) as u8
                                        };
                                        terminal.put_entry_at(next_char, terminal.color, x, terminal.row);
                                    }
                                }
                            }
                        },
                        _ => {
                            // kprint!(LogLevel::Debug, "[{:?} pressed]", event.key);
                        }
                    }
                }
            }
        }
    }

    fn handle_navigation_input(&mut self, event: KeyEvent) {
        let terminal = terminal();
        
        match event.key {
            KeyCode::ArrowLeft => unsafe { terminal.move_cursor_left() },
            KeyCode::ArrowRight => unsafe { terminal.move_cursor_right() },
            KeyCode::ArrowUp => unsafe { terminal.move_cursor_up() },
            KeyCode::ArrowDown => unsafe { terminal.move_cursor_down() },
            
            KeyCode::Home => unsafe { terminal.move_to_line_start() },
            KeyCode::End => unsafe { terminal.move_to_line_end() },
            
            KeyCode::PageUp => unsafe { terminal.page_up() },
            KeyCode::PageDown => unsafe { terminal.page_down() },
            
            KeyCode::ArrowLeft if event.ctrl => unsafe {
                for _ in 0..5 {
                    terminal.move_cursor_left();
                }
            },
            KeyCode::ArrowRight if event.ctrl => unsafe {
                for _ in 0..5 {
                    terminal.move_cursor_right();
                }
            },
            KeyCode::ArrowUp if event.ctrl => unsafe {
                for _ in 0..3 {
                    terminal.move_cursor_up();
                }
            },
            KeyCode::ArrowDown if event.ctrl => unsafe {
                for _ in 0..3 {
                    terminal.move_cursor_down();
                }
            },
            
            KeyCode::F1 => self.show_help(),
            KeyCode::F2 => self.toggle_input_mode(),
            KeyCode::F3 => unsafe { terminal.clear_screen() },
            
            _ if event.ctrl => self.handle_ctrl_shortcuts(event),
            
            _ => {
                kprint!(LogLevel::Trace, "[{:?} ignored in navigation mode]", event.key);
            }
        }
    }

    fn handle_ctrl_shortcuts(&mut self, event: KeyEvent) {
        match event.key {
            KeyCode::C => {
                kprint!(LogLevel::Info, "^C (interrupt signal)");
            },
            KeyCode::L => unsafe {
                terminal().clear_screen();
                kprint!(LogLevel::Info, "Screen cleared (Ctrl+L)");
            },
            KeyCode::A => unsafe {
                terminal().move_to_line_start();
            },
            KeyCode::E => unsafe {
                terminal().move_to_line_end();
            },
            KeyCode::D => {
                kprint!(LogLevel::Info, "^D (EOF signal)");
            },
            KeyCode::Z => {
                kprint!(LogLevel::Info, "^Z (suspend signal)");
            },
            KeyCode::Q => {
                kprint!(LogLevel::Info, "^Q (resume signal)");
            },
            KeyCode::S => {
                kprint!(LogLevel::Info, "^S (pause signal)");
            },
            _ => {
                kprint!(LogLevel::Debug, "Ctrl+{:?}", event.key);
            }
        }
    }

    fn show_help(&mut self) {
        if !self.help_shown {
            kprint!(LogLevel::Info, "=== KFS Keyboard Help ===");
            kprint!(LogLevel::Info, "");
            kprint!(LogLevel::Info, "Function Keys:");
            kprint!(LogLevel::Info, "  F1 - Show/hide this help");
            kprint!(LogLevel::Info, "  F2 - Toggle input mode (Terminal/Navigation)");
            kprint!(LogLevel::Info, "  F3 - Clear screen");
            kprint!(LogLevel::Info, "  F4 - Toggle cursor visibility");
            kprint!(LogLevel::Info, "");
            kprint!(LogLevel::Info, "Navigation:");
            kprint!(LogLevel::Info, "  Arrow keys - Move cursor");
            kprint!(LogLevel::Info, "  Ctrl+Arrows - Fast movement");
            kprint!(LogLevel::Info, "  Home/End - Start/end of line");
            kprint!(LogLevel::Info, "  Page Up/Down - Scroll screen");
            kprint!(LogLevel::Info, "");
            kprint!(LogLevel::Info, "Control Keys:");
            kprint!(LogLevel::Info, "  Ctrl+A - Move to line start");
            kprint!(LogLevel::Info, "  Ctrl+E - Move to line end");
            kprint!(LogLevel::Info, "  Ctrl+L - Clear screen");
            kprint!(LogLevel::Info, "  Ctrl+C - Interrupt signal");
            kprint!(LogLevel::Info, "");
            kprint!(LogLevel::Info, "Editing (Terminal mode):");
            kprint!(LogLevel::Info, "  Backspace - Delete previous char");
            kprint!(LogLevel::Info, "  Delete - Delete char under cursor");
            kprint!(LogLevel::Info, "  Tab - Insert tab (4 spaces)");
            kprint!(LogLevel::Info, "  Enter - New line");
            kprint!(LogLevel::Info, "");
            kprint!(LogLevel::Info, "Current mode: {:?}", self.mode);
            kprint!(LogLevel::Info, "=========================");
            self.help_shown = true;
        } else {
            kprint!(LogLevel::Info, "Help hidden");
            self.help_shown = false;
        }
    }

    fn toggle_input_mode(&mut self) {
        self.mode = match self.mode {
            InputMode::Terminal => {
                kprint!(LogLevel::Info, "Switched to Navigation mode");
                kprint!(LogLevel::Info, "  - Cursor movement only");
                kprint!(LogLevel::Info, "  - No text input");
                kprint!(LogLevel::Info, "  - Use F2 to switch back");
                InputMode::Navigation
            },
            InputMode::Navigation => {
                kprint!(LogLevel::Info, "Switched to Terminal mode");
                kprint!(LogLevel::Info, "  - Full text input");
                kprint!(LogLevel::Info, "  - Cursor movement + editing");
                InputMode::Terminal
            },
        };
    }

    pub fn get_current_mode(&self) -> InputMode {
        self.mode
    }
}

static mut INPUT_HANDLER: InputHandler = InputHandler::new();

pub fn get_input_handler() -> &'static mut InputHandler {
    unsafe { &mut INPUT_HANDLER }
}