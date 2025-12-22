// Custom ratatui widgets for battlefield rendering

use crate::game_logic::battlefield::{Battlefield, Position};
use crate::rendering::viewport::Camera;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use std::collections::HashMap;

/// Widget that renders the battlefield viewport
pub struct BattlefieldWidget<'a> {
    battlefield: &'a Battlefield,
    camera: &'a Camera,
    show_fog_of_war: bool,
    peripheral_tiles: Option<&'a HashMap<Position, bool>>,
}

impl<'a> BattlefieldWidget<'a> {
    pub fn new(battlefield: &'a Battlefield, camera: &'a Camera) -> Self {
        Self {
            battlefield,
            camera,
            show_fog_of_war: true,
            peripheral_tiles: None,
        }
    }

    pub fn show_fog_of_war(mut self, show: bool) -> Self {
        self.show_fog_of_war = show;
        self
    }

    pub fn with_peripheral_tiles(mut self, peripheral: &'a HashMap<Position, bool>) -> Self {
        self.peripheral_tiles = Some(peripheral);
        self
    }
}

impl<'a> Widget for BattlefieldWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let top_left = self.camera.top_left();

        // Render each cell in the viewport
        for screen_y in 0..area.height {
            for screen_x in 0..area.width {
                let world_pos = Position::new(
                    top_left.x + screen_x as i32,
                    top_left.y + screen_y as i32,
                );

                // Get the tile at this position
                if let Some(tile) = self.battlefield.get_tile(&world_pos) {
                    let (ch, style) = if self.show_fog_of_war {
                        if tile.visible {
                            // Check if this is peripheral vision (dimmed)
                            let is_peripheral = self.peripheral_tiles
                                .and_then(|map| map.get(&world_pos))
                                .copied()
                                .unwrap_or(false);

                            if is_peripheral {
                                // Peripheral vision: dimmed (50% brightness via gray color)
                                (tile.terrain.to_char(), Style::default().fg(Color::Gray))
                            } else {
                                // Main vision: full brightness
                                (tile.terrain.to_char(), Style::default().fg(Color::White))
                            }
                        } else if tile.explored {
                            // Explored but not currently visible (dark gray)
                            (tile.terrain.to_char(), Style::default().fg(Color::DarkGray))
                        } else {
                            // Unexplored (black/hidden)
                            (' ', Style::default())
                        }
                    } else {
                        // No fog of war, always visible
                        (tile.terrain.to_char(), Style::default().fg(Color::White))
                    };

                    // Calculate buffer position
                    let buf_x = area.x + screen_x;
                    let buf_y = area.y + screen_y;

                    if buf_x < area.right() && buf_y < area.bottom() {
                        buf[(buf_x, buf_y)].set_char(ch).set_style(style);
                    }
                } else {
                    // Out of bounds - render as empty
                    let buf_x = area.x + screen_x;
                    let buf_y = area.y + screen_y;

                    if buf_x < area.right() && buf_y < area.bottom() {
                        buf[(buf_x, buf_y)]
                            .set_char(' ')
                            .set_style(Style::default());
                    }
                }
            }
        }
    }
}
