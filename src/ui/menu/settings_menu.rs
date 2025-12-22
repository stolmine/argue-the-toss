use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::config::game_config::GameConfig;
use crate::game_logic::turn_state::TurnOrderMode;

#[derive(Debug, Clone)]
pub struct SettingsMenuState {
    pub turn_order_mode: TurnOrderMode,
    pub default_time_budget: f32,
    pub selected_index: usize,
}

impl SettingsMenuState {
    pub fn new() -> Self {
        Self {
            turn_order_mode: TurnOrderMode::PlayerFirst,
            default_time_budget: 12.0,
            selected_index: 0,
        }
    }

    pub fn from_game_config(config: &GameConfig) -> Self {
        Self {
            turn_order_mode: config.turn_order_mode,
            default_time_budget: config.time_budget_seconds,
            selected_index: 0,
        }
    }

    pub fn to_game_config(&self) -> GameConfig {
        GameConfig::new()
            .with_turn_order_mode(self.turn_order_mode)
            .with_time_budget(self.default_time_budget)
    }

    pub fn select_next(&mut self) {
        if self.selected_index < 3 {
            self.selected_index += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn handle_left(&mut self) {
        match self.selected_index {
            0 => {
                self.turn_order_mode = match self.turn_order_mode {
                    TurnOrderMode::PlayerFirst => TurnOrderMode::InitiativeBased,
                    TurnOrderMode::Simultaneous => TurnOrderMode::PlayerFirst,
                    TurnOrderMode::InitiativeBased => TurnOrderMode::Simultaneous,
                };
            }
            1 => {
                self.default_time_budget = (self.default_time_budget - 1.0).clamp(5.0, 30.0);
            }
            _ => {}
        }
    }

    pub fn handle_right(&mut self) {
        match self.selected_index {
            0 => {
                self.turn_order_mode = match self.turn_order_mode {
                    TurnOrderMode::PlayerFirst => TurnOrderMode::Simultaneous,
                    TurnOrderMode::Simultaneous => TurnOrderMode::InitiativeBased,
                    TurnOrderMode::InitiativeBased => TurnOrderMode::PlayerFirst,
                };
            }
            1 => {
                self.default_time_budget = (self.default_time_budget + 1.0).clamp(5.0, 30.0);
            }
            _ => {}
        }
    }
}

impl Default for SettingsMenuState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SettingsMenuWidget<'a> {
    state: &'a SettingsMenuState,
}

impl<'a> SettingsMenuWidget<'a> {
    pub fn new(state: &'a SettingsMenuState) -> Self {
        Self { state }
    }

    fn render_category_header(&self, title: &str, y: u16, area: Rect, buf: &mut Buffer) {
        let line = Line::from(vec![
            Span::styled(title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]);

        let paragraph = Paragraph::new(line);
        let line_area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        };
        paragraph.render(line_area, buf);
    }

    fn render_choice_item(
        &self,
        label: &str,
        value: &str,
        is_selected: bool,
        y: u16,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let line = if is_selected {
            Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Yellow)),
                Span::styled(label, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw(": "),
                Span::styled(value, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ])
        } else {
            Line::from(vec![
                Span::raw("  "),
                Span::styled(label, Style::default().fg(Color::Gray)),
                Span::raw(": "),
                Span::styled(value, Style::default().fg(Color::Green)),
            ])
        };

        let paragraph = Paragraph::new(line);
        let line_area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        };
        paragraph.render(line_area, buf);
    }

    fn render_slider_item(
        &self,
        label: &str,
        value: f32,
        min: f32,
        max: f32,
        is_selected: bool,
        y: u16,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let line = if is_selected {
            Line::from(format!("> {}: {:.1} ({:.1}-{:.1})", label, value, min, max))
                .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        } else {
            Line::from(format!("  {}: {:.1} ({:.1}-{:.1})", label, value, min, max))
                .style(Style::default().fg(Color::Gray))
        };

        let paragraph = Paragraph::new(line);
        let line_area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        };
        paragraph.render(line_area, buf);
    }

    fn render_button(
        &self,
        label: &str,
        is_selected: bool,
        y: u16,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let line = if is_selected {
            Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Yellow)),
                Span::styled(label, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ])
        } else {
            Line::from(vec![
                Span::raw("  "),
                Span::styled(label, Style::default().fg(Color::Gray)),
            ])
        };

        let paragraph = Paragraph::new(line);
        let line_area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        };
        paragraph.render(line_area, buf);
    }

    fn render_text_line(&self, text: &str, y: u16, area: Rect, buf: &mut Buffer) {
        let line = Line::from(Span::styled(text, Style::default().fg(Color::DarkGray)));

        let paragraph = Paragraph::new(line);
        let line_area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        };
        paragraph.render(line_area, buf);
    }
}

impl<'a> Widget for SettingsMenuWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Settings")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        let mut y = inner.y;

        self.render_category_header("Gameplay", y, inner, buf);
        y += 1;

        let turn_order_text = match self.state.turn_order_mode {
            TurnOrderMode::PlayerFirst => "PlayerFirst",
            TurnOrderMode::Simultaneous => "Simultaneous",
            TurnOrderMode::InitiativeBased => "InitiativeBased",
        };
        self.render_choice_item(
            "Turn Order",
            turn_order_text,
            self.state.selected_index == 0,
            y,
            inner,
            buf,
        );
        y += 1;

        self.render_slider_item(
            "Default Time Budget",
            self.state.default_time_budget,
            5.0,
            30.0,
            self.state.selected_index == 1,
            y,
            inner,
            buf,
        );
        y += 2;

        self.render_category_header("Controls", y, inner, buf);
        y += 1;

        self.render_text_line("  Movement: qweasdzxc (8-direction)", y, inner, buf);
        y += 1;

        self.render_text_line("  Look Mode: l", y, inner, buf);
        y += 1;

        self.render_text_line("  Fire: f", y, inner, buf);
        y += 1;

        self.render_text_line("  Reload: r", y, inner, buf);
        y += 2;

        self.render_button("[Save & Return]", self.state.selected_index == 2, y, inner, buf);
        y += 1;

        self.render_button("[Cancel]", self.state.selected_index == 3, y, inner, buf);
    }
}
