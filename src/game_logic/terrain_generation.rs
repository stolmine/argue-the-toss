// Procedural Battlefield Generation
// Generates realistic WWI battlefields with trenches, fortifications, and terrain features

use super::battlefield::{Battlefield, Position, TerrainType};
use crate::config::battlefield_config::{
    BattlefieldGenerationConfig, BattlefieldType, FortificationLevel, Side, TrenchDensity,
};
use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Main battlefield generator
pub struct BattlefieldGenerator {
    config: BattlefieldGenerationConfig,
    rng: ChaCha8Rng,
    perlin: Perlin,
}

impl BattlefieldGenerator {
    /// Create a new generator with the given configuration
    pub fn new(config: BattlefieldGenerationConfig) -> Self {
        let rng = ChaCha8Rng::seed_from_u64(config.seed);
        let perlin = Perlin::new(config.seed as u32);

        Self {
            config,
            rng,
            perlin,
        }
    }

    /// Generate a complete battlefield
    pub fn generate(&mut self) -> Battlefield {
        let mut battlefield = Battlefield::new(self.config.width, self.config.height);

        // Phase 1: Base layout (terrain base layer)
        self.generate_base_layout(&mut battlefield);

        // Phase 2: Trench networks
        self.generate_trench_networks(&mut battlefield);

        // Phase 3: Fortifications
        self.generate_fortifications(&mut battlefield);

        // Phase 4: Environmental features
        self.generate_environmental_features(&mut battlefield);

        // Phase 5: Buildings (if applicable)
        if matches!(
            self.config.battlefield_type,
            BattlefieldType::Village | BattlefieldType::Urban
        ) {
            self.generate_buildings(&mut battlefield);
        }

        // Phase 6: Tactical balancing
        self.balance_tactical_features(&mut battlefield);

        // Phase 7: Spawn zones
        self.create_spawn_zones(&mut battlefield);

        battlefield
    }

    /// Get spawn positions for allies and enemies
    pub fn get_spawn_positions(&self) -> (Vec<Position>, Vec<Position>) {
        let (allies_positions, enemies_positions) = match self.config.allies_side {
            Side::South => (
                self.get_south_spawn_positions(),
                self.get_north_spawn_positions(),
            ),
            Side::North => (
                self.get_north_spawn_positions(),
                self.get_south_spawn_positions(),
            ),
            Side::East => (
                self.get_east_spawn_positions(),
                self.get_west_spawn_positions(),
            ),
            Side::West => (
                self.get_west_spawn_positions(),
                self.get_east_spawn_positions(),
            ),
        };

        (allies_positions, enemies_positions)
    }

    // ========================================================================
    // PHASE 1: Base Layout (Terrain Base Layer)
    // ========================================================================

    fn generate_base_layout(&mut self, battlefield: &mut Battlefield) {
        // First pass: Use Perlin noise for natural variation
        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let pos = Position::new(x as i32, y as i32);

                // Sample Perlin noise at this position
                let noise_value = self.sample_perlin(x as f64, y as f64, 0.05);

                // Determine base terrain based on noise and config
                let terrain = self.determine_base_terrain(noise_value, &pos);

                battlefield.set_terrain(pos, terrain);
            }
        }

        // Define no-man's land zone (will be filled with appropriate terrain)
        self.mark_no_mans_land(battlefield);
    }

    fn sample_perlin(&self, x: f64, y: f64, scale: f64) -> f64 {
        self.perlin.get([x * scale, y * scale])
    }

    fn determine_base_terrain(&mut self, noise_value: f64, _pos: &Position) -> TerrainType {
        // Normalize noise value from [-1, 1] to [0, 1]
        let normalized = (noise_value + 1.0) / 2.0;

        // Apply mud coverage
        if normalized < self.config.mud_coverage as f64 * 0.5 {
            return TerrainType::Mud;
        }

        // Apply water features
        if self.config.water_features && normalized < 0.05 {
            return TerrainType::Water;
        }

        // Default to appropriate base terrain based on battlefield type
        match self.config.battlefield_type {
            BattlefieldType::WesternFront => {
                if normalized < 0.3 {
                    TerrainType::Mud
                } else if normalized < 0.5 {
                    TerrainType::NoMansLand
                } else {
                    TerrainType::Grass
                }
            }
            BattlefieldType::EasternFront | BattlefieldType::OpenField => TerrainType::Grass,
            BattlefieldType::Village | BattlefieldType::Urban => TerrainType::Road,
        }
    }

    fn mark_no_mans_land(&mut self, battlefield: &mut Battlefield) {
        let (start, end) = self.get_no_mans_land_bounds();

        for y in start..end {
            for x in 0..self.config.width {
                let pos = Position::new(x as i32, y as i32);

                // Override with no-man's land terrain unless it's water
                if let Some(tile) = battlefield.get_tile(&pos) {
                    if tile.terrain != TerrainType::Water
                        && tile.terrain != TerrainType::DeepWater
                    {
                        battlefield.set_terrain(pos, TerrainType::NoMansLand);
                    }
                }
            }
        }
    }

    fn get_no_mans_land_bounds(&self) -> (usize, usize) {
        let center = match self.config.allies_side {
            Side::South | Side::North => self.config.height / 2,
            Side::East | Side::West => self.config.width / 2,
        };

        let half_width = self.config.no_mans_land_width / 2;
        let start = center.saturating_sub(half_width);
        let end = (center + half_width).min(self.config.height);

        (start, end)
    }

    // ========================================================================
    // PHASE 2: Trench Networks (Multi-tile structures)
    // ========================================================================

    fn generate_trench_networks(&mut self, battlefield: &mut Battlefield) {
        if matches!(self.config.trench_density, TrenchDensity::None) {
            return;
        }

        // Generate allied trench line
        self.generate_trench_line(battlefield, true);

        // Generate enemy trench line
        self.generate_trench_line(battlefield, false);

        // Generate communication trenches
        self.generate_communication_trenches(battlefield);
    }

    fn generate_trench_line(&mut self, battlefield: &mut Battlefield, is_allies: bool) {
        let y_position = self.get_trench_line_position(is_allies);
        let coverage = self.config.trench_density.coverage_percentage();

        // Generate main trench line with multi-tile structure
        let mut x = 5; // Start with margin
        while x < self.config.width - 5 {
            // Decide if we should place a trench segment here
            if self.rng.random::<f32>() < coverage {
                // Generate a trench segment (3 tiles wide minimum)
                let segment_length = self.rng.random_range(15..40);
                self.generate_trench_segment(battlefield, x, y_position, segment_length, is_allies);
                x += segment_length;
            } else {
                x += self.rng.random_range(5..15);
            }
        }
    }

    fn generate_trench_segment(
        &mut self,
        battlefield: &mut Battlefield,
        start_x: usize,
        y: usize,
        length: usize,
        is_allies: bool,
    ) {
        // Direction enemy is facing (determines which side gets the parapet)
        let enemy_direction = if is_allies { -1 } else { 1 };

        for i in 0..length {
            let x = start_x + i;
            if x >= self.config.width {
                break;
            }

            // Place ramp every 20-30 tiles
            let is_ramp = i % self.rng.random_range(20..30) == 0;

            // Multi-tile trench structure:
            // - Center: Floor
            // - Enemy-facing side: Parapet (or Ramp)
            // - Friendly side: Parapet

            let center_y = y as i32;
            let enemy_y = center_y + enemy_direction;
            let friendly_y = center_y - enemy_direction;

            // Place floor
            battlefield.set_terrain(Position::new(x as i32, center_y), TerrainType::TrenchFloor);

            // Place enemy-facing side (parapet or ramp)
            if is_ramp {
                battlefield.set_terrain(Position::new(x as i32, enemy_y), TerrainType::TrenchRamp);
            } else {
                battlefield.set_terrain(
                    Position::new(x as i32, enemy_y),
                    TerrainType::TrenchParapet,
                );
            }

            // Place friendly-facing side (always parapet)
            battlefield.set_terrain(
                Position::new(x as i32, friendly_y),
                TerrainType::TrenchParapet,
            );
        }
    }

    fn get_trench_line_position(&self, is_allies: bool) -> usize {
        let (nml_start, nml_end) = self.get_no_mans_land_bounds();

        match self.config.allies_side {
            Side::South | Side::North => {
                if is_allies == matches!(self.config.allies_side, Side::South) {
                    nml_end + 5 // South of no-man's land
                } else {
                    nml_start.saturating_sub(5) // North of no-man's land
                }
            }
            Side::East | Side::West => {
                if is_allies == matches!(self.config.allies_side, Side::East) {
                    nml_end + 5
                } else {
                    nml_start.saturating_sub(5)
                }
            }
        }
    }

    fn generate_communication_trenches(&mut self, _battlefield: &mut Battlefield) {
        // TODO: Generate perpendicular communication trenches
        // These connect the front line to support trenches
        // For now, we'll skip this to keep the implementation focused
    }

    // ========================================================================
    // PHASE 3: Fortifications
    // ========================================================================

    fn generate_fortifications(&mut self, battlefield: &mut Battlefield) {
        if matches!(self.config.fortification_level, FortificationLevel::None) {
            return;
        }

        let (sandbag_count, bunker_count, mg_nest_count) =
            self.config.fortification_level.get_counts();

        // Place fortifications for both sides
        self.place_fortifications_for_side(
            battlefield,
            true,
            sandbag_count,
            bunker_count,
            mg_nest_count,
        );
        self.place_fortifications_for_side(
            battlefield,
            false,
            sandbag_count,
            bunker_count,
            mg_nest_count,
        );

        // Place barbed wire in no-man's land
        self.place_barbed_wire(battlefield);
    }

    fn place_fortifications_for_side(
        &mut self,
        battlefield: &mut Battlefield,
        is_allies: bool,
        sandbag_count: usize,
        bunker_count: usize,
        mg_nest_count: usize,
    ) {
        let zone = self.get_fortification_zone(is_allies);

        // Place bunkers
        for _ in 0..bunker_count {
            if let Some(pos) = self.find_valid_fortification_position(battlefield, &zone) {
                self.place_bunker(battlefield, pos);
            }
        }

        // Place MG nests
        for _ in 0..mg_nest_count {
            if let Some(pos) = self.find_valid_fortification_position(battlefield, &zone) {
                battlefield.set_terrain(pos, TerrainType::MgNest);
            }
        }

        // Place sandbag positions
        for _ in 0..sandbag_count {
            if let Some(pos) = self.find_valid_fortification_position(battlefield, &zone) {
                battlefield.set_terrain(pos, TerrainType::Sandbags);
            }
        }
    }

    fn get_fortification_zone(&self, is_allies: bool) -> (usize, usize, usize, usize) {
        let (nml_start, nml_end) = self.get_no_mans_land_bounds();

        match self.config.allies_side {
            Side::South | Side::North => {
                if is_allies == matches!(self.config.allies_side, Side::South) {
                    // Allied zone (south of no-man's land)
                    (0, self.config.width, nml_end, self.config.height)
                } else {
                    // Enemy zone (north of no-man's land)
                    (0, self.config.width, 0, nml_start)
                }
            }
            _ => {
                // For East/West, use similar logic
                (0, self.config.width, 0, self.config.height)
            }
        }
    }

    fn find_valid_fortification_position(
        &mut self,
        battlefield: &Battlefield,
        zone: &(usize, usize, usize, usize),
    ) -> Option<Position> {
        let (x_min, x_max, y_min, y_max) = *zone;

        for _ in 0..50 {
            // Try 50 times
            let x = self.rng.random_range(x_min..x_max) as i32;
            let y = self.rng.random_range(y_min..y_max) as i32;
            let pos = Position::new(x, y);

            if let Some(tile) = battlefield.get_tile(&pos) {
                // Check if tile is suitable (grass or no-man's land, not water or existing structure)
                if matches!(
                    tile.terrain,
                    TerrainType::Grass | TerrainType::NoMansLand | TerrainType::Mud
                ) {
                    return Some(pos);
                }
            }
        }

        None
    }

    fn place_bunker(&mut self, battlefield: &mut Battlefield, center: Position) {
        // Bunkers are 3x3 structures
        for dy in -1..=1 {
            for dx in -1..=1 {
                let pos = Position::new(center.x + dx, center.y + dy);
                if battlefield.in_bounds(&pos) {
                    battlefield.set_terrain(pos, TerrainType::Bunker);
                }
            }
        }
    }

    fn place_barbed_wire(&mut self, battlefield: &mut Battlefield) {
        let (nml_start, nml_end) = self.get_no_mans_land_bounds();
        let wire_coverage = self.config.barbed_wire_coverage;

        for y in nml_start..nml_end {
            for x in 0..self.config.width {
                if self.rng.random::<f32>() < wire_coverage {
                    let pos = Position::new(x as i32, y as i32);
                    if let Some(tile) = battlefield.get_tile(&pos) {
                        if tile.terrain == TerrainType::NoMansLand {
                            battlefield.set_terrain(pos, TerrainType::BarbedWire);
                        }
                    }
                }
            }
        }
    }

    // ========================================================================
    // PHASE 4: Environmental Features
    // ========================================================================

    fn generate_environmental_features(&mut self, battlefield: &mut Battlefield) {
        self.place_shell_craters(battlefield);
        self.place_forests(battlefield);
    }

    fn place_shell_craters(&mut self, battlefield: &mut Battlefield) {
        let total_tiles = (self.config.width * self.config.height) as f32;
        let crater_count = (total_tiles / 100.0 * self.config.crater_density) as usize;

        for _ in 0..crater_count {
            let x = self.rng.random_range(0..self.config.width) as i32;
            let y = self.rng.random_range(0..self.config.height) as i32;
            let pos = Position::new(x, y);

            if battlefield.in_bounds(&pos) {
                // Randomly choose water-filled or dry crater
                let terrain = if self.config.water_features && self.rng.random::<f32>() < 0.3 {
                    TerrainType::CraterWater
                } else {
                    TerrainType::ShellCrater
                };

                battlefield.set_terrain(pos, terrain);
            }
        }
    }

    fn place_forests(&mut self, battlefield: &mut Battlefield) {
        let total_tiles = (self.config.width * self.config.height) as f32;
        let forest_tiles = (total_tiles * self.config.forest_coverage) as usize;
        let num_forests = self.rng.random_range(3..8);
        let tiles_per_forest = forest_tiles / num_forests.max(1);

        for _ in 0..num_forests {
            self.place_forest_cluster(battlefield, tiles_per_forest);
        }
    }

    fn place_forest_cluster(&mut self, battlefield: &mut Battlefield, size: usize) {
        // Pick a random center point
        let center_x = self.rng.random_range(0..self.config.width) as i32;
        let center_y = self.rng.random_range(0..self.config.height) as i32;

        let mut placed = 0;
        let radius = (size as f32).sqrt() as i32;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if placed >= size {
                    break;
                }

                // Create organic cluster shape
                if self.rng.random::<f32>() < 0.6 {
                    let pos = Position::new(center_x + dx, center_y + dy);

                    if battlefield.in_bounds(&pos) {
                        if let Some(tile) = battlefield.get_tile(&pos) {
                            if matches!(tile.terrain, TerrainType::Grass | TerrainType::Mud) {
                                battlefield.set_terrain(pos, TerrainType::Tree);
                                placed += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    // ========================================================================
    // PHASE 5: Buildings
    // ========================================================================

    fn generate_buildings(&mut self, battlefield: &mut Battlefield) {
        let total_tiles = (self.config.width * self.config.height) as f32;
        let building_count = (total_tiles / 100.0 * self.config.building_density) as usize;

        for _ in 0..building_count {
            self.place_building(battlefield);
        }
    }

    fn place_building(&mut self, battlefield: &mut Battlefield) {
        // Random building size (small to medium)
        let width = self.rng.random_range(4..10);
        let height = self.rng.random_range(4..10);

        // Random position
        let x = self.rng.random_range(0..(self.config.width.saturating_sub(width))) as i32;
        let y = self.rng.random_range(0..(self.config.height.saturating_sub(height))) as i32;

        // Place building structure
        for dy in 0..height {
            for dx in 0..width {
                let pos = Position::new(x + dx as i32, y + dy as i32);

                if battlefield.in_bounds(&pos) {
                    // Walls on perimeter, floor inside
                    let is_wall = dx == 0 || dx == width - 1 || dy == 0 || dy == height - 1;
                    let is_door = (dx == width / 2 && dy == 0) || (dx == width / 2 && dy == height - 1);

                    let terrain = if is_door {
                        TerrainType::BuildingDoor
                    } else if is_wall {
                        TerrainType::BuildingWall
                    } else {
                        TerrainType::BuildingFloor
                    };

                    battlefield.set_terrain(pos, terrain);
                }
            }
        }
    }

    // ========================================================================
    // PHASE 6: Tactical Balancing
    // ========================================================================

    fn balance_tactical_features(&mut self, _battlefield: &mut Battlefield) {
        // TODO: Analyze cover density, ensure balanced flanking routes
        // For now, basic generation is sufficient
    }

    // ========================================================================
    // PHASE 7: Spawn Point Helpers
    // ========================================================================

    fn get_south_spawn_positions(&self) -> Vec<Position> {
        let mut positions = Vec::new();
        let y_start = (self.config.height * 3 / 4) as i32;
        let y_end = (self.config.height - 5) as i32;

        for y in y_start..=y_end {
            for x in 5..(self.config.width - 5) as i32 {
                positions.push(Position::new(x, y));
            }
        }

        positions
    }

    fn get_north_spawn_positions(&self) -> Vec<Position> {
        let mut positions = Vec::new();
        let y_start = 5;
        let y_end = (self.config.height / 4) as i32;

        for y in y_start..=y_end {
            for x in 5..(self.config.width - 5) as i32 {
                positions.push(Position::new(x, y));
            }
        }

        positions
    }

    fn get_east_spawn_positions(&self) -> Vec<Position> {
        let mut positions = Vec::new();
        let x_start = (self.config.width * 3 / 4) as i32;
        let x_end = (self.config.width - 5) as i32;

        for x in x_start..=x_end {
            for y in 5..(self.config.height - 5) as i32 {
                positions.push(Position::new(x, y));
            }
        }

        positions
    }

    fn get_west_spawn_positions(&self) -> Vec<Position> {
        let mut positions = Vec::new();
        let x_start = 5;
        let x_end = (self.config.width / 4) as i32;

        for x in x_start..=x_end {
            for y in 5..(self.config.height - 5) as i32 {
                positions.push(Position::new(x, y));
            }
        }

        positions
    }

    fn create_spawn_zones(&mut self, battlefield: &mut Battlefield) {
        use super::battlefield::SpawnZone;

        let spawn_radius = self.calculate_spawn_radius();

        let (ally_center, enemy_center) = match self.config.allies_side {
            Side::South => (
                self.get_south_spawn_center(),
                self.get_north_spawn_center(),
            ),
            Side::North => (
                self.get_north_spawn_center(),
                self.get_south_spawn_center(),
            ),
            Side::East => (
                self.get_east_spawn_center(),
                self.get_west_spawn_center(),
            ),
            Side::West => (
                self.get_west_spawn_center(),
                self.get_east_spawn_center(),
            ),
        };

        let ally_spawn = SpawnZone::new(ally_center, spawn_radius);
        let enemy_spawn = SpawnZone::new(enemy_center, spawn_radius);

        battlefield.set_spawn_zones(ally_spawn, enemy_spawn);
    }

    fn calculate_spawn_radius(&self) -> usize {
        let map_size = self.config.width.min(self.config.height);
        (map_size / 8).max(10).min(20)
    }

    fn get_south_spawn_center(&self) -> Position {
        let x = (self.config.width / 2) as i32;
        let y = (self.config.height * 7 / 8) as i32;
        Position::new(x, y)
    }

    fn get_north_spawn_center(&self) -> Position {
        let x = (self.config.width / 2) as i32;
        let y = (self.config.height / 8) as i32;
        Position::new(x, y)
    }

    fn get_east_spawn_center(&self) -> Position {
        let x = (self.config.width * 7 / 8) as i32;
        let y = (self.config.height / 2) as i32;
        Position::new(x, y)
    }

    fn get_west_spawn_center(&self) -> Position {
        let x = (self.config.width / 8) as i32;
        let y = (self.config.height / 2) as i32;
        Position::new(x, y)
    }
}
