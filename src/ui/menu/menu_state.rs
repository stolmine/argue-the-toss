pub struct MenuState {
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub input_buffer: String,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            scroll_offset: 0,
            input_buffer: String::new(),
        }
    }

    pub fn select_next(&mut self, max_index: usize) {
        if self.selected_index < max_index {
            self.selected_index += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn reset_selection(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
    }
}

impl Default for MenuState {
    fn default() -> Self {
        Self::new()
    }
}
