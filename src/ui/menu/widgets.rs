use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

#[derive(Debug, Clone)]
pub enum MenuItem {
    Button { label: String, action: MenuAction },
    Toggle { label: String, value: bool, action: MenuAction },
    Slider { label: String, value: f32, min: f32, max: f32, step: f32, action: MenuAction },
    Choice { label: String, options: Vec<String>, selected: usize, action: MenuAction },
    TextInput { label: String, value: String, action: MenuAction },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    StartGame,
    ConfigureGame,
    Settings,
    Quit,
    Resume,
    MainMenu,
    UpdateBattlefieldSize,
    UpdateTimeBudget,
    UpdateFaction,
    UpdateDifficulty,
    ConfirmConfig,
    CancelConfig,
}

pub struct MenuWidget<'a> {
    items: &'a [MenuItem],
    selected_index: usize,
    title: String,
}

impl<'a> MenuWidget<'a> {
    pub fn new(items: &'a [MenuItem], selected_index: usize, title: String) -> Self {
        Self {
            items,
            selected_index,
            title,
        }
    }
}

impl<'a> Widget for MenuWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        let mut y = inner.y;
        for (idx, item) in self.items.iter().enumerate() {
            if y >= inner.bottom() {
                break;
            }

            let is_selected = idx == self.selected_index;
            let line = match item {
                MenuItem::Button { label, .. } => {
                    if is_selected {
                        Line::from(vec![
                            Span::styled("> ", Style::default().fg(Color::Yellow)),
                            Span::styled(label, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        ])
                    } else {
                        Line::from(vec![
                            Span::raw("  "),
                            Span::styled(label, Style::default().fg(Color::Gray)),
                        ])
                    }
                }
                MenuItem::Toggle { label, value, .. } => {
                    let toggle_text = if *value { "[ON]" } else { "[OFF]" };
                    let toggle_color = if *value { Color::Green } else { Color::Red };

                    if is_selected {
                        Line::from(vec![
                            Span::styled("> ", Style::default().fg(Color::Yellow)),
                            Span::styled(label, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                            Span::raw(": "),
                            Span::styled(toggle_text, Style::default().fg(toggle_color).add_modifier(Modifier::BOLD)),
                        ])
                    } else {
                        Line::from(vec![
                            Span::raw("  "),
                            Span::styled(label, Style::default().fg(Color::Gray)),
                            Span::raw(": "),
                            Span::styled(toggle_text, Style::default().fg(toggle_color)),
                        ])
                    }
                }
                MenuItem::Slider { label, value, min, max, .. } => {
                    if is_selected {
                        Line::from(format!("> {}: {:.1} ({:.1}-{:.1})", label, value, min, max))
                            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                    } else {
                        Line::from(format!("  {}: {:.1} ({:.1}-{:.1})", label, value, min, max))
                            .style(Style::default().fg(Color::Gray))
                    }
                }
                MenuItem::Choice { label, options, selected, .. } => {
                    let choice_text = options.get(*selected).map(|s| s.as_str()).unwrap_or("N/A");

                    if is_selected {
                        Line::from(vec![
                            Span::styled("> ", Style::default().fg(Color::Yellow)),
                            Span::styled(label, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                            Span::raw(": "),
                            Span::styled(choice_text, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                        ])
                    } else {
                        Line::from(vec![
                            Span::raw("  "),
                            Span::styled(label, Style::default().fg(Color::Gray)),
                            Span::raw(": "),
                            Span::styled(choice_text, Style::default().fg(Color::Green)),
                        ])
                    }
                }
                MenuItem::TextInput { label, value, .. } => {
                    let display_value = if value.is_empty() { "_" } else { value.as_str() };

                    if is_selected {
                        Line::from(vec![
                            Span::styled("> ", Style::default().fg(Color::Yellow)),
                            Span::styled(label, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                            Span::raw(": "),
                            Span::styled(display_value, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                        ])
                    } else {
                        Line::from(vec![
                            Span::raw("  "),
                            Span::styled(label, Style::default().fg(Color::Gray)),
                            Span::raw(": "),
                            Span::styled(display_value, Style::default().fg(Color::Magenta)),
                        ])
                    }
                }
            };

            let paragraph = Paragraph::new(line);
            let line_area = Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            };
            paragraph.render(line_area, buf);

            y += 1;
        }
    }
}

pub struct ConfigSliderWidget {
    label: String,
    value: f32,
    min: f32,
    max: f32,
    width: u16,
}

impl ConfigSliderWidget {
    pub fn new(label: String, value: f32, min: f32, max: f32, width: u16) -> Self {
        Self {
            label,
            value,
            min,
            max,
            width,
        }
    }
}

impl Widget for ConfigSliderWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let normalized = ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0);
        let filled_width = (normalized * (self.width as f32)) as u16;

        let label_line = Line::from(vec![
            Span::styled(&self.label, Style::default().fg(Color::White)),
            Span::raw(": "),
            Span::styled(format!("{:.1}", self.value), Style::default().fg(Color::Cyan)),
        ]);

        let label_paragraph = Paragraph::new(label_line);
        let label_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        label_paragraph.render(label_area, buf);

        let bar_y = area.y + 1;
        if bar_y < area.bottom() {
            for x in 0..self.width.min(area.width) {
                let buf_x = area.x + x;
                let buf_y = bar_y;

                if buf_x < area.right() && buf_y < area.bottom() {
                    let ch = if x < filled_width { '█' } else { '░' };
                    let color = if x < filled_width { Color::Green } else { Color::DarkGray };

                    buf[(buf_x, buf_y)]
                        .set_char(ch)
                        .set_style(Style::default().fg(color));
                }
            }
        }
    }
}
