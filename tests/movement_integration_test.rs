// Integration test for movement execution
// Tests that movement actions are properly executed and positions update

use argue_the_toss::{
    components::{
        action::{ActionType, QueuedAction},
        facing::{Direction8, Facing},
        health::Health,
        position::Position,
        soldier::{Faction, Rank, Soldier},
        soldier_stats::SoldierStats,
        time_budget::TimeBudget,
        vision::Vision,
        weapon::Weapon,
    },
    game_logic::{
        battlefield::Battlefield,
        turn_state::{TurnOrderMode, TurnPhase, TurnState},
    },
    systems::{
        action_execution::ActionExecutionSystem,
        turn_manager::TurnManagerSystem,
    },
    utils::event_log::EventLog,
};
use specs::{Builder, DispatcherBuilder, World, WorldExt};

#[test]
fn test_movement_execution() {
    // Setup world
    let mut world = World::new();

    // Register components
    world.register::<Position>();
    world.register::<Facing>();
    world.register::<Health>();
    world.register::<Soldier>();
    world.register::<SoldierStats>();
    world.register::<Vision>();
    world.register::<Weapon>();
    world.register::<QueuedAction>();
    world.register::<TimeBudget>();

    // Add resources
    world.insert(TurnState::new_with_mode(TurnOrderMode::Simultaneous));
    world.insert(EventLog::new());
    world.insert(Battlefield::new(100, 100));

    // Create test entity with movement action
    let entity = world
        .create_entity()
        .with(Position::new(10, 10))
        .with(Facing::new(Direction8::N))
        .with(Health::new(100))
        .with(Soldier {
            name: "Test Soldier".to_string(),
            faction: Faction::Allies,
            rank: Rank::Private,
        })
        .with(SoldierStats::new(0.0, 1.0, 0, 100))
        .with(Vision::new(10))
        .with(Weapon::rifle())
        .with(TimeBudget::new(10.0))
        .with(QueuedAction::new(ActionType::Move {
            dx: 1,
            dy: 0,
            terrain_cost: 1.0,
        }))
        .build();

    // Create dispatcher with CORRECT system order (TurnManager before ActionExecution)
    let mut dispatcher = DispatcherBuilder::new()
        .with(TurnManagerSystem, "turn_manager", &[])
        .with(ActionExecutionSystem, "action_execution", &["turn_manager"])
        .build();

    dispatcher.setup(&mut world);

    // Mark entity as ready to trigger execution
    {
        let mut turn_state = world.write_resource::<TurnState>();
        turn_state.mark_entity_ready(entity);
    }

    // Get initial position
    let initial_pos = {
        let positions = world.read_storage::<Position>();
        let pos = positions.get(entity).unwrap();
        (pos.x(), pos.y())
    };

    assert_eq!(initial_pos, (10, 10), "Initial position should be (10, 10)");

    // Run one dispatch cycle - should transition to Execution and execute action
    dispatcher.dispatch(&world);
    world.maintain();

    // Check that position was updated
    let new_pos = {
        let positions = world.read_storage::<Position>();
        let pos = positions.get(entity).unwrap();
        (pos.x(), pos.y())
    };

    assert_eq!(
        new_pos,
        (11, 10),
        "Position should have moved from (10, 10) to (11, 10)"
    );

    // Verify phase progressed correctly
    let turn_state = world.read_resource::<TurnState>();
    assert!(
        matches!(turn_state.phase, TurnPhase::Resolution | TurnPhase::Execution),
        "Phase should have progressed to Execution or Resolution, got {:?}",
        turn_state.phase
    );
}

#[test]
#[should_panic(expected = "Position should have moved")]
fn test_movement_fails_with_wrong_system_order() {
    // This test demonstrates the bug when systems are in wrong order

    let mut world = World::new();

    // Register components
    world.register::<Position>();
    world.register::<Facing>();
    world.register::<Health>();
    world.register::<Soldier>();
    world.register::<SoldierStats>();
    world.register::<Vision>();
    world.register::<Weapon>();
    world.register::<QueuedAction>();
    world.register::<TimeBudget>();

    // Add resources
    world.insert(TurnState::new_with_mode(TurnOrderMode::Simultaneous));
    world.insert(EventLog::new());
    world.insert(Battlefield::new(100, 100));

    // Create test entity
    let entity = world
        .create_entity()
        .with(Position::new(10, 10))
        .with(Facing::new(Direction8::N))
        .with(Health::new(100))
        .with(Soldier {
            name: "Test Soldier".to_string(),
            faction: Faction::Allies,
            rank: Rank::Private,
        })
        .with(SoldierStats::new(0.0, 1.0, 0, 100))
        .with(Vision::new(10))
        .with(Weapon::rifle())
        .with(TimeBudget::new(10.0))
        .with(QueuedAction::new(ActionType::Move {
            dx: 1,
            dy: 0,
            terrain_cost: 1.0,
        }))
        .build();

    // Create dispatcher with WRONG system order (ActionExecution before TurnManager)
    let mut dispatcher = DispatcherBuilder::new()
        .with(ActionExecutionSystem, "action_execution", &[])
        .with(TurnManagerSystem, "turn_manager", &["action_execution"])
        .build();

    dispatcher.setup(&mut world);

    // Mark entity as ready
    {
        let mut turn_state = world.write_resource::<TurnState>();
        turn_state.mark_entity_ready(entity);
    }

    // Run dispatch
    dispatcher.dispatch(&world);
    world.maintain();

    // This should fail because of wrong system order
    let new_pos = {
        let positions = world.read_storage::<Position>();
        let pos = positions.get(entity).unwrap();
        (pos.x(), pos.y())
    };

    assert_eq!(
        new_pos,
        (11, 10),
        "Position should have moved from (10, 10) to (11, 10)"
    );
}
