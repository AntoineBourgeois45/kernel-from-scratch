use crate::vga::terminal::line_editor::LineEditor;

pub struct TerminalShell {
    line_editor: LineEditor,
    prompt: &'static str,
    current_row: usize,
    current_col: usize,
}

impl TerminalShell {
    pub const fn new() -> Self {
        Self {
            line_editor: LineEditor::new(),
            prompt: "kernel> ",
            current_row: 0,
            current_col: 0,
        }
    }

    pub unsafe fn initialize(&mut self) {
        self.show_prompt();
    }
}