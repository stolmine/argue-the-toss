// Map Generation Testing Tool
// Standalone binary for rapid iteration on terrain generation parameters

use argue_the_toss::config::battlefield_config::{
    BattlefieldGenerationConfig, FortificationLevel, Side, TrenchDensity,
};
use argue_the_toss::game_logic::battlefield::{Position, TerrainType};
use argue_the_toss::game_logic::terrain_generation::BattlefieldGenerator;
use clap::{Parser, ValueEnum};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "map_test")]
#[command(about = "Test battlefield generation with custom parameters", long_about = None)]
struct Args {
    /// Generation seed (for reproducible maps)
    #[arg(short, long, default_value = "12345")]
    seed: u64,

    /// Map size (square, NxN)
    #[arg(long, default_value = "50")]
    size: usize,

    /// Use a historical preset
    #[arg(short, long, value_enum)]
    preset: Option<Preset>,

    /// Trench density level
    #[arg(long, value_enum)]
    trench_density: Option<TrenchDensityArg>,

    /// Fortification level
    #[arg(long, value_enum)]
    fortification: Option<FortificationArg>,

    /// Barbed wire coverage (0.0-1.0)
    #[arg(long)]
    barbed_wire: Option<f32>,

    /// Crater density (craters per 100 tiles)
    #[arg(long)]
    crater: Option<f32>,

    /// Mud coverage (0.0-1.0)
    #[arg(long)]
    mud: Option<f32>,

    /// Forest coverage (0.0-1.0)
    #[arg(long)]
    forest: Option<f32>,

    /// No-man's land width (tiles)
    #[arg(long)]
    nml_width: Option<usize>,

    /// Allies spawn side
    #[arg(long, value_enum, default_value = "south")]
    allies_side: SideArg,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Preset {
    Verdun,
    Somme,
    Ypres,
    Tannenberg,
    Village,
    Urban,
    OpenField,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TrenchDensityArg {
    None,
    Sparse,
    Moderate,
    Dense,
    VeryDense,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FortificationArg {
    None,
    Light,
    Moderate,
    Heavy,
    Fortress,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SideArg {
    North,
    South,
    East,
    West,
}

impl From<TrenchDensityArg> for TrenchDensity {
    fn from(arg: TrenchDensityArg) -> Self {
        match arg {
            TrenchDensityArg::None => TrenchDensity::None,
            TrenchDensityArg::Sparse => TrenchDensity::Sparse,
            TrenchDensityArg::Moderate => TrenchDensity::Moderate,
            TrenchDensityArg::Dense => TrenchDensity::Dense,
            TrenchDensityArg::VeryDense => TrenchDensity::VeryDense,
        }
    }
}

impl From<FortificationArg> for FortificationLevel {
    fn from(arg: FortificationArg) -> Self {
        match arg {
            FortificationArg::None => FortificationLevel::None,
            FortificationArg::Light => FortificationLevel::Light,
            FortificationArg::Moderate => FortificationLevel::Moderate,
            FortificationArg::Heavy => FortificationLevel::Heavy,
            FortificationArg::Fortress => FortificationLevel::Fortress,
        }
    }
}

impl From<SideArg> for Side {
    fn from(arg: SideArg) -> Self {
        match arg {
            SideArg::North => Side::North,
            SideArg::South => Side::South,
            SideArg::East => Side::East,
            SideArg::West => Side::West,
        }
    }
}

fn main() {
    let args = Args::parse();

    println!("╔═══════════════════════════════════════════════════════════════════╗");
    println!("║          ARGUE THE TOSS - Map Generation Test Tool               ║");
    println!("╚═══════════════════════════════════════════════════════════════════╝\n");

    // Build configuration
    let mut config = if let Some(preset) = args.preset {
        println!("Using preset: {:?}", preset);
        match preset {
            Preset::Verdun => BattlefieldGenerationConfig::verdun(),
            Preset::Somme => BattlefieldGenerationConfig::somme(),
            Preset::Ypres => BattlefieldGenerationConfig::ypres(),
            Preset::Tannenberg => BattlefieldGenerationConfig::tannenberg(),
            Preset::Village => BattlefieldGenerationConfig::village(),
            Preset::Urban => BattlefieldGenerationConfig::urban(),
            Preset::OpenField => BattlefieldGenerationConfig::open_field(),
        }
    } else {
        BattlefieldGenerationConfig::default()
    };

    // Apply custom parameters (override preset if specified)
    config.seed = args.seed;
    config.width = args.size;
    config.height = args.size;
    config.allies_side = args.allies_side.into();

    if let Some(density) = args.trench_density {
        config.trench_density = density.into();
    }
    if let Some(fortification) = args.fortification {
        config.fortification_level = fortification.into();
    }
    if let Some(wire) = args.barbed_wire {
        config.barbed_wire_coverage = wire.clamp(0.0, 1.0);
    }
    if let Some(crater) = args.crater {
        config.crater_density = crater.max(0.0);
    }
    if let Some(mud) = args.mud {
        config.mud_coverage = mud.clamp(0.0, 1.0);
    }
    if let Some(forest) = args.forest {
        config.forest_coverage = forest.clamp(0.0, 1.0);
    }
    if let Some(nml_width) = args.nml_width {
        config.no_mans_land_width = nml_width;
    }

    // Print configuration
    println!("Configuration:");
    println!("  Seed: {}", config.seed);
    println!("  Size: {}x{}", config.width, config.height);
    println!("  Type: {:?}", config.battlefield_type);
    println!("  Trench Density: {:?}", config.trench_density);
    println!("  Fortifications: {:?}", config.fortification_level);
    println!("  Barbed Wire: {:.1}%", config.barbed_wire_coverage * 100.0);
    println!("  Crater Density: {:.1}/100 tiles", config.crater_density);
    println!("  Mud Coverage: {:.1}%", config.mud_coverage * 100.0);
    println!("  Forest Coverage: {:.1}%", config.forest_coverage * 100.0);
    println!("  No-Man's Land Width: {} tiles", config.no_mans_land_width);
    println!("  Allies Side: {:?}", config.allies_side);
    println!();

    // Generate battlefield
    println!("Generating battlefield...");
    let start = Instant::now();
    let mut generator = BattlefieldGenerator::new(config.clone());
    let battlefield = generator.generate();
    let duration = start.elapsed();
    println!("Generation time: {}ms\n", duration.as_millis());

    // Render ASCII map
    println!("Map Preview:");
    println!("┌{}┐", "─".repeat(config.width));
    render_battlefield(&battlefield, config.width, config.height);
    println!("└{}┘\n", "─".repeat(config.width));

    // Calculate statistics
    let stats = calculate_terrain_stats(&battlefield, config.width, config.height);
    print_statistics(&stats, config.width * config.height);

    // Print legend
    print_legend();
}

fn render_battlefield(battlefield: &argue_the_toss::game_logic::battlefield::Battlefield, width: usize, height: usize) {
    for y in 0..height {
        print!("│");
        for x in 0..width {
            let pos = Position::new(x as i32, y as i32);
            if let Some(tile) = battlefield.get_tile(&pos) {
                let ch = terrain_to_char(tile.terrain);
                print!("{}", ch);
            } else {
                print!(" ");
            }
        }
        println!("│");
    }
}

fn terrain_to_char(terrain: TerrainType) -> char {
    match terrain {
        TerrainType::NoMansLand => '.',
        TerrainType::Grass => ',',
        TerrainType::Mud => '~',
        TerrainType::Water => '≈',
        TerrainType::DeepWater => '≋',
        TerrainType::TrenchFloor => '═',
        TerrainType::TrenchParapet => '║',
        TerrainType::TrenchRamp => '╬',
        TerrainType::Trench => '╠',
        TerrainType::Sandbags => 's',
        TerrainType::Bunker => '■',
        TerrainType::MgNest => 'M',
        TerrainType::BarbedWire => 'x',
        TerrainType::Tree => '♣',
        TerrainType::Forest => '♠',
        TerrainType::Hedge => '#',
        TerrainType::Rubble => '%',
        TerrainType::ShellCrater => 'o',
        TerrainType::CraterWater => 'O',
        TerrainType::BuildingWall => '█',
        TerrainType::BuildingFloor => '·',
        TerrainType::BuildingDoor => '▒',
        TerrainType::BuildingWindow => '▓',
        TerrainType::Ruins => '▄',
        TerrainType::Road => '─',
        TerrainType::Path => '┄',
        TerrainType::CommTrench => '┼',
        TerrainType::Fortification => 'F',
        TerrainType::CivilianBuilding => 'B',
    }
}

fn calculate_terrain_stats(
    battlefield: &argue_the_toss::game_logic::battlefield::Battlefield,
    width: usize,
    height: usize,
) -> Vec<(TerrainType, usize)> {
    // Collect all terrain types
    let mut terrain_counts: Vec<(TerrainType, usize)> = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let pos = Position::new(x as i32, y as i32);
            if let Some(tile) = battlefield.get_tile(&pos) {
                // Find if terrain type already exists
                if let Some(entry) = terrain_counts.iter_mut().find(|(t, _)| *t == tile.terrain) {
                    entry.1 += 1;
                } else {
                    terrain_counts.push((tile.terrain, 1));
                }
            }
        }
    }

    // Sort by count (descending)
    terrain_counts.sort_by(|a, b| b.1.cmp(&a.1));

    terrain_counts
}

fn print_statistics(stats: &[(TerrainType, usize)], total_tiles: usize) {
    println!("Terrain Statistics:");

    for (terrain, count) in stats {
        let percentage = (*count as f32 / total_tiles as f32) * 100.0;
        let terrain_name = format!("{:?}", terrain);

        // Flag high barbed wire density
        let warning = if matches!(terrain, TerrainType::BarbedWire) && percentage > 15.0 {
            " ⚠ HIGH!"
        } else {
            ""
        };

        println!(
            "  {:20} {:5} tiles ({:5.1}%){}",
            terrain_name, count, percentage, warning
        );
    }
    println!();
}

fn print_legend() {
    println!("Legend:");
    println!("  .  = NoMansLand       ,  = Grass             ~  = Mud");
    println!("  ≈  = Water            ≋  = DeepWater        ═  = TrenchFloor");
    println!("  ║  = TrenchParapet    ╬  = TrenchRamp        ╠  = Trench (legacy)");
    println!("  s  = Sandbags         ■  = Bunker            M  = MgNest");
    println!("  x  = BarbedWire       ♣  = Tree              ♠  = Forest");
    println!("  #  = Hedge            %  = Rubble            o  = ShellCrater");
    println!("  O  = CraterWater      █  = BuildingWall      ·  = BuildingFloor");
    println!("  ▒  = BuildingDoor     ▓  = BuildingWindow    ▄  = Ruins");
    println!("  ─  = Road             ┄  = Path              ┼  = CommTrench");
    println!("  F  = Fortification    B  = CivilianBuilding");
    println!();
}
