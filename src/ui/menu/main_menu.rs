use super::menu_state::MenuState;
use super::widgets::MenuAction;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

#[derive(Debug, Clone)]
pub struct MainMenuItem {
    pub label: String,
    pub action: MenuAction,
    pub enabled: bool,
}

impl MainMenuItem {
    pub fn new(label: impl Into<String>, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            action,
            enabled: true,
        }
    }

    pub fn disabled(label: impl Into<String>, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            action,
            enabled: false,
        }
    }
}

pub struct MainMenuWidget<'a> {
    items: &'a [MainMenuItem],
    selected_index: usize,
}

impl<'a> MainMenuWidget<'a> {
    pub fn new(items: &'a [MainMenuItem], selected_index: usize) -> Self {
        Self {
            items,
            selected_index,
        }
    }
}

impl<'a> Widget for MainMenuWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));

        let inner = block.inner(area);
        block.render(area, buf);

        let title = "ARGUE THE TOSS - WWI Trench Warfare Roguelike";
        let title_line = Line::from(vec![Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]);

        let title_paragraph = Paragraph::new(title_line).alignment(Alignment::Center);
        let title_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 1,
        };
        title_paragraph.render(title_area, buf);

        let menu_start_y = inner.y + 3;
        let mut y = menu_start_y;

        for (idx, item) in self.items.iter().enumerate() {
            if y >= inner.bottom() - 3 {
                break;
            }

            let is_selected = idx == self.selected_index;

            let line = if !item.enabled {
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        &item.label,
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        " (Coming Soon)",
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            } else if is_selected {
                Line::from(vec![
                    Span::styled("> ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        &item.label,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(&item.label, Style::default().fg(Color::Gray)),
                ])
            };

            let paragraph = Paragraph::new(line).alignment(Alignment::Center);
            let line_area = Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            };
            paragraph.render(line_area, buf);

            y += 2;
        }

        let controls_y = inner.bottom().saturating_sub(2);
        if controls_y < inner.bottom() {
            let controls_line = Line::from(vec![
                Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
                Span::raw(" or "),
                Span::styled("k/j", Style::default().fg(Color::Yellow)),
                Span::raw(": Navigate  "),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(": Select  "),
                Span::styled("ESC/q", Style::default().fg(Color::Red)),
                Span::raw(": Quit"),
            ]);

            let controls_paragraph = Paragraph::new(controls_line).alignment(Alignment::Center);
            let controls_area = Rect {
                x: inner.x,
                y: controls_y,
                width: inner.width,
                height: 1,
            };
            controls_paragraph.render(controls_area, buf);
        }
    }
}

pub struct MainMenuState {
    menu_state: MenuState,
    items: Vec<MainMenuItem>,
}

impl MainMenuState {
    pub fn new() -> Self {
        let items = vec![
            MainMenuItem::new("New Game", MenuAction::StartGame),
            MainMenuItem::disabled("Load Game", MenuAction::MainMenu),
            MainMenuItem::new("Settings", MenuAction::Settings),
            MainMenuItem::new("Quit", MenuAction::Quit),
        ];

        Self {
            menu_state: MenuState::new(),
            items,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<MenuAction> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_prev();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                None
            }
            KeyCode::Enter => {
                let selected_item = &self.items[self.menu_state.selected_index];
                if selected_item.enabled {
                    Some(selected_item.action)
                } else {
                    None
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => Some(MenuAction::Quit),
            _ => None,
        }
    }

    pub fn selected_index(&self) -> usize {
        self.menu_state.selected_index
    }

    pub fn items(&self) -> &[MainMenuItem] {
        &self.items
    }

    fn select_next(&mut self) {
        let max_index = self.items.len().saturating_sub(1);
        if self.menu_state.selected_index < max_index {
            self.menu_state.selected_index += 1;
        } else {
            self.menu_state.selected_index = 0;
        }
    }

    fn select_prev(&mut self) {
        if self.menu_state.selected_index > 0 {
            self.menu_state.selected_index -= 1;
        } else {
            self.menu_state.selected_index = self.items.len().saturating_sub(1);
        }
    }
}

impl Default for MainMenuState {
    fn default() -> Self {
        Self::new()
    }
}
