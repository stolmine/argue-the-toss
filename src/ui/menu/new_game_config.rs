use crate::config::battlefield_config::{
    BattlefieldGenerationConfig, FortificationLevel, TrenchDensity,
};
use crate::config::game_config::GameConfig;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

const SOLDIER_COUNT_OPTIONS: &[usize] = &[5, 10, 15, 20, 30, 50, 75, 100, 150, 200, 300, 500];
const DEFAULT_SOLDIER_COUNT_INDEX: usize = 2;
const DEFAULT_TIME_BUDGET: f32 = 12.0;
const MAP_SIZE_OPTIONS: &[usize] = &[50, 75, 100, 125, 150, 200];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigField {
    BattlefieldPreset,
    MapWidth,
    MapHeight,
    TrenchDensity,
    FortificationLevel,
    MudCoverage,
    CraterDensity,
    ForestCoverage,
    BuildingDensity,
    BarbedWireCoverage,
    Seed,
    SoldierCount,
    TimeBudget,
    StartGame,
    BackToMenu,
}

impl ConfigField {
    fn next(self) -> Self {
        match self {
            Self::BattlefieldPreset => Self::MapWidth,
            Self::MapWidth => Self::MapHeight,
            Self::MapHeight => Self::TrenchDensity,
            Self::TrenchDensity => Self::FortificationLevel,
            Self::FortificationLevel => Self::MudCoverage,
            Self::MudCoverage => Self::CraterDensity,
            Self::CraterDensity => Self::ForestCoverage,
            Self::ForestCoverage => Self::BuildingDensity,
            Self::BuildingDensity => Self::BarbedWireCoverage,
            Self::BarbedWireCoverage => Self::Seed,
            Self::Seed => Self::SoldierCount,
            Self::SoldierCount => Self::TimeBudget,
            Self::TimeBudget => Self::StartGame,
            Self::StartGame => Self::BackToMenu,
            Self::BackToMenu => Self::BattlefieldPreset,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::BattlefieldPreset => Self::BackToMenu,
            Self::MapWidth => Self::BattlefieldPreset,
            Self::MapHeight => Self::MapWidth,
            Self::TrenchDensity => Self::MapHeight,
            Self::FortificationLevel => Self::TrenchDensity,
            Self::MudCoverage => Self::FortificationLevel,
            Self::CraterDensity => Self::MudCoverage,
            Self::ForestCoverage => Self::CraterDensity,
            Self::BuildingDensity => Self::ForestCoverage,
            Self::BarbedWireCoverage => Self::BuildingDensity,
            Self::Seed => Self::BarbedWireCoverage,
            Self::SoldierCount => Self::Seed,
            Self::TimeBudget => Self::SoldierCount,
            Self::StartGame => Self::TimeBudget,
            Self::BackToMenu => Self::StartGame,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BattlefieldPreset {
    Verdun,
    Somme,
    Ypres,
    Tannenberg,
    Village,
    Urban,
    OpenField,
    Custom,
}

impl BattlefieldPreset {
    const ALL: &'static [Self] = &[
        Self::Verdun,
        Self::Somme,
        Self::Ypres,
        Self::Tannenberg,
        Self::Village,
        Self::Urban,
        Self::OpenField,
        Self::Custom,
    ];

    fn name(&self) -> &'static str {
        match self {
            Self::Verdun => "Verdun",
            Self::Somme => "Somme",
            Self::Ypres => "Ypres",
            Self::Tannenberg => "Tannenberg",
            Self::Village => "Village",
            Self::Urban => "Urban",
            Self::OpenField => "Open Field",
            Self::Custom => "Custom",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::Verdun => "Dense trenches, heavy fortifications, extensive barbed wire",
            Self::Somme => "Mixed terrain, moderate fortifications, shell craters",
            Self::Ypres => "Muddy fields, dispersed trenches, scattered ruins",
            Self::Tannenberg => "Eastern front, open terrain, light fortifications",
            Self::Village => "Urban combat, buildings, narrow streets",
            Self::Urban => "Dense buildings, rubble, close quarters",
            Self::OpenField => "Minimal cover, long sightlines, tactical movement",
            Self::Custom => "Fully customizable battlefield parameters",
        }
    }

    fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&p| p == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    fn prev(self) -> Self {
        let idx = Self::ALL.iter().position(|&p| p == self).unwrap_or(0);
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    fn to_config(&self) -> BattlefieldGenerationConfig {
        match self {
            Self::Verdun => BattlefieldGenerationConfig::verdun(),
            Self::Somme => BattlefieldGenerationConfig::somme(),
            Self::Ypres => BattlefieldGenerationConfig::ypres(),
            Self::Tannenberg => BattlefieldGenerationConfig::tannenberg(),
            Self::Village => BattlefieldGenerationConfig::village(),
            Self::Urban => BattlefieldGenerationConfig::urban(),
            Self::OpenField => BattlefieldGenerationConfig::open_field(),
            Self::Custom => BattlefieldGenerationConfig::default(),
        }
    }
}

pub struct NewGameConfigState {
    selected_preset: BattlefieldPreset,
    map_width_index: usize,
    map_height_index: usize,
    trench_density: TrenchDensity,
    fortification_level: FortificationLevel,
    mud_coverage: f32,
    crater_density: f32,
    forest_coverage: f32,
    building_density: f32,
    barbed_wire_coverage: f32,
    seed: u64,
    soldier_count_index: usize,
    time_budget: f32,
    selected_field: ConfigField,
}

impl NewGameConfigState {
    pub fn new() -> Self {
        let config = BattlefieldGenerationConfig::verdun();
        Self {
            selected_preset: BattlefieldPreset::Verdun,
            map_width_index: MAP_SIZE_OPTIONS.iter().position(|&s| s == config.width).unwrap_or(2),
            map_height_index: MAP_SIZE_OPTIONS.iter().position(|&s| s == config.height).unwrap_or(2),
            trench_density: config.trench_density,
            fortification_level: config.fortification_level,
            mud_coverage: config.mud_coverage,
            crater_density: config.crater_density,
            forest_coverage: config.forest_coverage,
            building_density: config.building_density,
            barbed_wire_coverage: config.barbed_wire_coverage,
            seed: config.seed,
            soldier_count_index: DEFAULT_SOLDIER_COUNT_INDEX,
            time_budget: DEFAULT_TIME_BUDGET,
            selected_field: ConfigField::BattlefieldPreset,
        }
    }

    fn load_preset(&mut self, preset: BattlefieldPreset) {
        let config = preset.to_config();
        self.map_width_index = MAP_SIZE_OPTIONS.iter().position(|&s| s == config.width).unwrap_or(2);
        self.map_height_index = MAP_SIZE_OPTIONS.iter().position(|&s| s == config.height).unwrap_or(2);
        self.trench_density = config.trench_density;
        self.fortification_level = config.fortification_level;
        self.mud_coverage = config.mud_coverage;
        self.crater_density = config.crater_density;
        self.forest_coverage = config.forest_coverage;
        self.building_density = config.building_density;
        self.barbed_wire_coverage = config.barbed_wire_coverage;
        self.seed = config.seed;
    }

    fn switch_to_custom_if_needed(&mut self) {
        if self.selected_preset != BattlefieldPreset::Custom {
            self.selected_preset = BattlefieldPreset::Custom;
        }
    }

    pub fn soldier_count(&self) -> usize {
        SOLDIER_COUNT_OPTIONS[self.soldier_count_index]
    }

    fn map_width(&self) -> usize {
        MAP_SIZE_OPTIONS[self.map_width_index]
    }

    fn map_height(&self) -> usize {
        MAP_SIZE_OPTIONS[self.map_height_index]
    }

    pub fn handle_up(&mut self) {
        self.selected_field = self.selected_field.prev();
    }

    pub fn handle_down(&mut self) {
        self.selected_field = self.selected_field.next();
    }

    pub fn handle_tab(&mut self) {
        self.selected_field = self.selected_field.next();
    }

    pub fn handle_left(&mut self) {
        match self.selected_field {
            ConfigField::BattlefieldPreset => {
                let new_preset = self.selected_preset.prev();
                self.selected_preset = new_preset;
                if new_preset != BattlefieldPreset::Custom {
                    self.load_preset(new_preset);
                }
            }
            ConfigField::MapWidth => {
                if self.map_width_index > 0 {
                    self.map_width_index -= 1;
                    self.switch_to_custom_if_needed();
                }
            }
            ConfigField::MapHeight => {
                if self.map_height_index > 0 {
                    self.map_height_index -= 1;
                    self.switch_to_custom_if_needed();
                }
            }
            ConfigField::TrenchDensity => {
                self.trench_density = match self.trench_density {
                    TrenchDensity::VeryDense => TrenchDensity::Dense,
                    TrenchDensity::Dense => TrenchDensity::Moderate,
                    TrenchDensity::Moderate => TrenchDensity::Sparse,
                    TrenchDensity::Sparse => TrenchDensity::None,
                    TrenchDensity::None => TrenchDensity::None,
                };
                self.switch_to_custom_if_needed();
            }
            ConfigField::FortificationLevel => {
                self.fortification_level = match self.fortification_level {
                    FortificationLevel::Fortress => FortificationLevel::Heavy,
                    FortificationLevel::Heavy => FortificationLevel::Moderate,
                    FortificationLevel::Moderate => FortificationLevel::Light,
                    FortificationLevel::Light => FortificationLevel::None,
                    FortificationLevel::None => FortificationLevel::None,
                };
                self.switch_to_custom_if_needed();
            }
            ConfigField::MudCoverage => {
                self.mud_coverage = (self.mud_coverage - 0.05).max(0.0);
                self.switch_to_custom_if_needed();
            }
            ConfigField::CraterDensity => {
                self.crater_density = (self.crater_density - 0.5).max(0.0);
                self.switch_to_custom_if_needed();
            }
            ConfigField::ForestCoverage => {
                self.forest_coverage = (self.forest_coverage - 0.05).max(0.0);
                self.switch_to_custom_if_needed();
            }
            ConfigField::BuildingDensity => {
                self.building_density = (self.building_density - 0.5).max(0.0);
                self.switch_to_custom_if_needed();
            }
            ConfigField::BarbedWireCoverage => {
                self.barbed_wire_coverage = (self.barbed_wire_coverage - 0.05).max(0.0);
                self.switch_to_custom_if_needed();
            }
            ConfigField::Seed => {
                self.seed = self.seed.saturating_sub(100);
                self.switch_to_custom_if_needed();
            }
            ConfigField::SoldierCount => {
                if self.soldier_count_index > 0 {
                    self.soldier_count_index -= 1;
                }
            }
            ConfigField::TimeBudget => {
                self.time_budget = (self.time_budget - 1.0).clamp(5.0, 30.0);
            }
            _ => {}
        }
    }

    pub fn handle_right(&mut self) {
        match self.selected_field {
            ConfigField::BattlefieldPreset => {
                let new_preset = self.selected_preset.next();
                self.selected_preset = new_preset;
                if new_preset != BattlefieldPreset::Custom {
                    self.load_preset(new_preset);
                }
            }
            ConfigField::MapWidth => {
                if self.map_width_index < MAP_SIZE_OPTIONS.len() - 1 {
                    self.map_width_index += 1;
                    self.switch_to_custom_if_needed();
                }
            }
            ConfigField::MapHeight => {
                if self.map_height_index < MAP_SIZE_OPTIONS.len() - 1 {
                    self.map_height_index += 1;
                    self.switch_to_custom_if_needed();
                }
            }
            ConfigField::TrenchDensity => {
                self.trench_density = match self.trench_density {
                    TrenchDensity::None => TrenchDensity::Sparse,
                    TrenchDensity::Sparse => TrenchDensity::Moderate,
                    TrenchDensity::Moderate => TrenchDensity::Dense,
                    TrenchDensity::Dense => TrenchDensity::VeryDense,
                    TrenchDensity::VeryDense => TrenchDensity::VeryDense,
                };
                self.switch_to_custom_if_needed();
            }
            ConfigField::FortificationLevel => {
                self.fortification_level = match self.fortification_level {
                    FortificationLevel::None => FortificationLevel::Light,
                    FortificationLevel::Light => FortificationLevel::Moderate,
                    FortificationLevel::Moderate => FortificationLevel::Heavy,
                    FortificationLevel::Heavy => FortificationLevel::Fortress,
                    FortificationLevel::Fortress => FortificationLevel::Fortress,
                };
                self.switch_to_custom_if_needed();
            }
            ConfigField::MudCoverage => {
                self.mud_coverage = (self.mud_coverage + 0.05).min(1.0);
                self.switch_to_custom_if_needed();
            }
            ConfigField::CraterDensity => {
                self.crater_density = (self.crater_density + 0.5).min(10.0);
                self.switch_to_custom_if_needed();
            }
            ConfigField::ForestCoverage => {
                self.forest_coverage = (self.forest_coverage + 0.05).min(1.0);
                self.switch_to_custom_if_needed();
            }
            ConfigField::BuildingDensity => {
                self.building_density = (self.building_density + 0.5).min(10.0);
                self.switch_to_custom_if_needed();
            }
            ConfigField::BarbedWireCoverage => {
                self.barbed_wire_coverage = (self.barbed_wire_coverage + 0.05).min(1.0);
                self.switch_to_custom_if_needed();
            }
            ConfigField::Seed => {
                self.seed = self.seed.saturating_add(100);
                self.switch_to_custom_if_needed();
            }
            ConfigField::SoldierCount => {
                if self.soldier_count_index < SOLDIER_COUNT_OPTIONS.len() - 1 {
                    self.soldier_count_index += 1;
                }
            }
            ConfigField::TimeBudget => {
                self.time_budget = (self.time_budget + 1.0).clamp(5.0, 30.0);
            }
            _ => {}
        }
    }

    pub fn is_start_selected(&self) -> bool {
        matches!(self.selected_field, ConfigField::StartGame)
    }

    pub fn is_back_selected(&self) -> bool {
        matches!(self.selected_field, ConfigField::BackToMenu)
    }

    pub fn to_game_config(&self) -> GameConfig {
        GameConfig::new().with_time_budget(self.time_budget)
    }

    pub fn to_battlefield_config(&self) -> BattlefieldGenerationConfig {
        BattlefieldGenerationConfig {
            width: self.map_width(),
            height: self.map_height(),
            trench_density: self.trench_density,
            fortification_level: self.fortification_level,
            mud_coverage: self.mud_coverage,
            crater_density: self.crater_density,
            forest_coverage: self.forest_coverage,
            building_density: self.building_density,
            barbed_wire_coverage: self.barbed_wire_coverage,
            seed: self.seed,
            ..self.selected_preset.to_config()
        }
    }
}

impl Default for NewGameConfigState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct NewGameConfigWidget<'a> {
    state: &'a NewGameConfigState,
}

impl<'a> NewGameConfigWidget<'a> {
    pub fn new(state: &'a NewGameConfigState) -> Self {
        Self { state }
    }

    fn render_field(
        &self,
        label: &str,
        value: String,
        is_selected: bool,
        y: u16,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let (prefix, label_style, value_style) = if is_selected {
            (
                "> ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (
                "  ",
                Style::default().fg(Color::Gray),
                Style::default().fg(Color::DarkGray),
            )
        };

        let line = Line::from(vec![
            Span::raw(prefix),
            Span::styled(label, label_style),
            Span::raw(": "),
            Span::styled(value, value_style),
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

    fn render_section_header(&self, title: &str, y: u16, area: Rect, buf: &mut Buffer) {
        let line = Line::from(vec![
            Span::raw("─── "),
            Span::styled(title, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" ───"),
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

    fn render_percentage_bar(
        &self,
        label: &str,
        value: f32,
        is_selected: bool,
        y: u16,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let (prefix, label_style, value_style) = if is_selected {
            (
                "> ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (
                "  ",
                Style::default().fg(Color::Gray),
                Style::default().fg(Color::DarkGray),
            )
        };

        let percentage = (value * 100.0) as u8;
        let bar_width = 10;
        let filled = ((value * bar_width as f32) as usize).min(bar_width);
        let bar: String = (0..bar_width)
            .map(|i| if i < filled { '■' } else { '□' })
            .collect();

        let line = Line::from(vec![
            Span::raw(prefix),
            Span::styled(label, label_style),
            Span::raw(": "),
            Span::styled(bar, value_style.clone()),
            Span::raw(" "),
            Span::styled(format!("{}%", percentage), value_style),
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

    fn render_density_bar(
        &self,
        label: &str,
        value: f32,
        max: f32,
        is_selected: bool,
        y: u16,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let (prefix, label_style, value_style) = if is_selected {
            (
                "> ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (
                "  ",
                Style::default().fg(Color::Gray),
                Style::default().fg(Color::DarkGray),
            )
        };

        let normalized = (value / max).min(1.0);
        let bar_width = 10;
        let filled = ((normalized * bar_width as f32) as usize).min(bar_width);
        let bar: String = (0..bar_width)
            .map(|i| if i < filled { '■' } else { '□' })
            .collect();

        let line = Line::from(vec![
            Span::raw(prefix),
            Span::styled(label, label_style),
            Span::raw(": "),
            Span::styled(bar, value_style.clone()),
            Span::raw(" "),
            Span::styled(format!("{:.1}", value), value_style),
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

    fn render_button(
        &self,
        label: &str,
        is_selected: bool,
        y: u16,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let (prefix, style) = if is_selected {
            (
                "> ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            ("  ", Style::default().fg(Color::Gray))
        };

        let line = Line::from(vec![
            Span::raw(prefix),
            Span::styled(format!("[{}]", label), style),
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

    fn render_slider(&self, y: u16, area: Rect, buf: &mut Buffer) {
        let is_selected = matches!(self.state.selected_field, ConfigField::TimeBudget);

        let (label_style, bar_color) = if is_selected {
            (
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
                Color::Green,
            )
        } else {
            (Style::default().fg(Color::Gray), Color::DarkGray)
        };

        let label_line = Line::from(vec![
            Span::raw(if is_selected { "> " } else { "  " }),
            Span::styled("Time Budget", label_style),
            Span::raw(": "),
            Span::styled(
                format!("{:.1}s", self.state.time_budget),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(if is_selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
        ]);

        let label_area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        };
        Paragraph::new(label_line).render(label_area, buf);

        let bar_y = y + 1;
        if bar_y < area.bottom() {
            let bar_width = 40.min(area.width.saturating_sub(4));
            let normalized = (self.state.time_budget - 5.0) / (30.0 - 5.0);
            let filled_width = (normalized * bar_width as f32) as u16;

            for x in 0..bar_width {
                let buf_x = area.x + 2 + x;
                let buf_y = bar_y;

                if buf_x < area.right() && buf_y < area.bottom() {
                    let ch = if x < filled_width { '█' } else { '░' };
                    let color = if x < filled_width {
                        bar_color
                    } else {
                        Color::DarkGray
                    };

                    buf[(buf_x, buf_y)].set_char(ch).set_style(Style::default().fg(color));
                }
            }
        }
    }

    fn trench_density_name(density: TrenchDensity) -> &'static str {
        match density {
            TrenchDensity::None => "None",
            TrenchDensity::Sparse => "Sparse",
            TrenchDensity::Moderate => "Moderate",
            TrenchDensity::Dense => "Dense",
            TrenchDensity::VeryDense => "Very Dense",
        }
    }

    fn fortification_level_name(level: FortificationLevel) -> &'static str {
        match level {
            FortificationLevel::None => "None",
            FortificationLevel::Light => "Light",
            FortificationLevel::Moderate => "Moderate",
            FortificationLevel::Heavy => "Heavy",
            FortificationLevel::Fortress => "Fortress",
        }
    }
}

impl<'a> Widget for NewGameConfigWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("New Game - Configuration")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        let mut y = inner.y;

        self.render_field(
            "Preset",
            self.state.selected_preset.name().to_string(),
            matches!(self.state.selected_field, ConfigField::BattlefieldPreset),
            y,
            inner,
            buf,
        );
        y += 1;

        let description = Paragraph::new(Line::from(Span::styled(
            format!("  {}", self.state.selected_preset.description()),
            Style::default().fg(Color::DarkGray),
        )))
        .alignment(Alignment::Left);
        let desc_area = Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: 2,
        };
        description.render(desc_area, buf);
        y += 3;

        self.render_section_header("Map Settings", y, inner, buf);
        y += 1;

        self.render_field(
            "Dimensions",
            format!("{} x {}", self.state.map_width(), self.state.map_height()),
            matches!(self.state.selected_field, ConfigField::MapWidth),
            y,
            inner,
            buf,
        );
        y += 1;

        self.render_field(
            "Width",
            format!("{}", self.state.map_width()),
            matches!(self.state.selected_field, ConfigField::MapWidth),
            y,
            inner,
            buf,
        );
        y += 1;

        self.render_field(
            "Height",
            format!("{}", self.state.map_height()),
            matches!(self.state.selected_field, ConfigField::MapHeight),
            y,
            inner,
            buf,
        );
        y += 1;

        self.render_field(
            "Seed",
            format!("{}", self.state.seed),
            matches!(self.state.selected_field, ConfigField::Seed),
            y,
            inner,
            buf,
        );
        y += 2;

        self.render_section_header("Terrain Density", y, inner, buf);
        y += 1;

        self.render_field(
            "Trenches",
            Self::trench_density_name(self.state.trench_density).to_string(),
            matches!(self.state.selected_field, ConfigField::TrenchDensity),
            y,
            inner,
            buf,
        );
        y += 1;

        self.render_field(
            "Fortifications",
            Self::fortification_level_name(self.state.fortification_level).to_string(),
            matches!(self.state.selected_field, ConfigField::FortificationLevel),
            y,
            inner,
            buf,
        );
        y += 1;

        self.render_percentage_bar(
            "Mud Coverage",
            self.state.mud_coverage,
            matches!(self.state.selected_field, ConfigField::MudCoverage),
            y,
            inner,
            buf,
        );
        y += 1;

        self.render_density_bar(
            "Craters",
            self.state.crater_density,
            10.0,
            matches!(self.state.selected_field, ConfigField::CraterDensity),
            y,
            inner,
            buf,
        );
        y += 1;

        self.render_percentage_bar(
            "Forest Coverage",
            self.state.forest_coverage,
            matches!(self.state.selected_field, ConfigField::ForestCoverage),
            y,
            inner,
            buf,
        );
        y += 1;

        self.render_density_bar(
            "Buildings",
            self.state.building_density,
            10.0,
            matches!(self.state.selected_field, ConfigField::BuildingDensity),
            y,
            inner,
            buf,
        );
        y += 1;

        self.render_percentage_bar(
            "Barbed Wire",
            self.state.barbed_wire_coverage,
            matches!(self.state.selected_field, ConfigField::BarbedWireCoverage),
            y,
            inner,
            buf,
        );
        y += 2;

        self.render_section_header("Game Settings", y, inner, buf);
        y += 1;

        self.render_field(
            "Soldiers/Team",
            self.state.soldier_count().to_string(),
            matches!(self.state.selected_field, ConfigField::SoldierCount),
            y,
            inner,
            buf,
        );
        y += 2;

        self.render_slider(y, inner, buf);
        y += 3;

        self.render_button(
            "Start Game",
            matches!(self.state.selected_field, ConfigField::StartGame),
            y,
            inner,
            buf,
        );
        y += 1;

        self.render_button(
            "Back to Main Menu",
            matches!(self.state.selected_field, ConfigField::BackToMenu),
            y,
            inner,
            buf,
        );

        let help_y = inner.bottom().saturating_sub(2);
        if help_y > y {
            let help_text = Line::from(vec![
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::raw(" Navigate  "),
                Span::styled("←→", Style::default().fg(Color::Yellow)),
                Span::raw(" Change  "),
                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                Span::raw(" Next  "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" Confirm  "),
                Span::styled("ESC", Style::default().fg(Color::Yellow)),
                Span::raw(" Back"),
            ]);

            let help_area = Rect {
                x: inner.x,
                y: help_y,
                width: inner.width,
                height: 1,
            };
            Paragraph::new(help_text)
                .alignment(Alignment::Center)
                .render(help_area, buf);
        }
    }
}
