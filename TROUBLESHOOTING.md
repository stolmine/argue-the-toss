# Troubleshooting Guide

This document catalogs known issues and their solutions to help developers (and AI agents) quickly diagnose and fix common problems.

## The Movement Bug

**Symptoms:**
- Event log shows movement messages (e.g., "Soldier moved from (5,5) to (6,5)")
- Soldiers don't actually move on screen
- Positions appear frozen even though actions are being logged

**Root Cause:**
This bug occurs when `ActionExecutionSystem` runs **before** `TurnManagerSystem` in the ECS dispatcher. The turn state machine has three phases:
- **Planning**: Entities queue actions
- **Execution**: `ActionExecutionSystem` executes actions
- **Resolution**: Cleanup and turn advancement

Phase transitions are handled by `TurnManagerSystem` by mutating `TurnState.phase`. All systems in a single `dispatcher.dispatch()` call share the same world resources, so systems see mutations from systems that ran earlier in the same frame.

**What Goes Wrong:**

**Incorrect System Order (BUG):**
```
Frame N (Planning phase):
1. ActionExecutionSystem runs
   - Checks phase → sees Planning
   - Returns early without executing actions
2. TurnManagerSystem runs
   - Checks if all ready → YES
   - Transitions Planning → Execution (too late!)

Frame N+1 (Execution phase):
1. ActionExecutionSystem runs
   - Checks phase → sees Execution
   - Executes actions → removes them
2. TurnManagerSystem runs
   - No committed actions remain
   - Transitions Execution → Resolution

Frame N+2 (Resolution phase):
1. ActionExecutionSystem runs
   - Checks phase → sees Resolution
   - Returns early
2. TurnManagerSystem runs
   - Clears actions
   - Transitions Resolution → Planning
```

Result: Actions are queued but never executed because `ActionExecutionSystem` sees the wrong phase.

**Correct System Order (FIXED):**
```
Frame N (Planning phase):
1. TurnManagerSystem runs
   - Checks if all ready → YES
   - Transitions Planning → Execution
2. ActionExecutionSystem runs
   - Checks phase → sees Execution (updated this frame!)
   - Executes actions → removes them

Frame N+1 (Execution phase):
1. TurnManagerSystem runs
   - No committed actions remain
   - Transitions Execution → Resolution
2. ActionExecutionSystem runs
   - Checks phase → sees Resolution
   - Returns early (correct behavior)

Frame N+2 (Resolution phase):
1. TurnManagerSystem runs
   - Clears actions
   - Transitions Resolution → Planning
2. ActionExecutionSystem runs
   - Checks phase → sees Planning
   - Returns early (correct behavior)
```

**Solution:**
Ensure `TurnManagerSystem` runs **before** `ActionExecutionSystem` in the dispatcher:

```rust
let mut dispatcher = DispatcherBuilder::new()
    .with(PathExecutionSystem, "path_execution", &[])
    .with(AIActionPlannerSystem, "ai_planner", &["path_execution"])
    .with(TurnManagerSystem, "turn_manager", &["ai_planner"])          // MUST be before action_execution
    .with(ActionExecutionSystem, "action_execution", &["turn_manager"]) // Depends on turn_manager
    .with(ObjectiveCaptureSystem, "objective_capture", &["action_execution"])
    .build();
```

**Prevention:**
1. **Comments in dispatcher builder** (`src/main.rs` ~line 1705): Explains critical ordering
2. **Comments in systems**: Both `TurnManagerSystem` and `ActionExecutionSystem` have header comments explaining the dependency
3. **Runtime validation**: `PositionValidationSystem` (enabled in debug builds) warns if turns complete without position changes
4. **This document**: Reference for future debugging

**How to Diagnose:**
1. Check if event log shows movement but screen doesn't update
2. Look at dispatcher system order in `src/main.rs`
3. Verify `TurnManagerSystem` comes before `ActionExecutionSystem`
4. In debug builds, check stderr for `PositionValidationSystem` warnings

**Related Files:**
- `/Users/why/repos/argue-the-toss/src/main.rs` (lines ~1705-1730): Dispatcher configuration
- `/Users/why/repos/argue-the-toss/src/systems/turn_manager.rs`: Phase transition logic
- `/Users/why/repos/argue-the-toss/src/systems/action_execution.rs`: Action execution logic
- `/Users/why/repos/argue-the-toss/src/systems/position_validation.rs`: Debug validation

---

## Other Common Issues

### Issue Template

**Symptoms:**
[What the user sees]

**Root Cause:**
[Technical explanation]

**Solution:**
[How to fix]

**Prevention:**
[How to avoid in the future]

---

*Add new troubleshooting entries above this line as issues are discovered and resolved.*
