# rules to live by:

## project org:

    - File size limits (max ~2000 lines per file)
    - Modular code organization patterns
    - Research and lookup practices
    - Documentation maintenance requirements
    - Progress tracking guidelines
    - Agent usage best practices
    - Rust-specific coding standards
    - ECS patterns and performance guidelines

    - stay DRY as much as possible
    - do not say 'you're absolutely right', avoid flattery

## CRITICAL: Turn-Based Game Loop Order

**THE MOVEMENT BUG** - If units freeze/don't execute actions, check game loop order!

### Required Execution Order (main.rs game loop):
```
1. INPUT HANDLING FIRST
   - event::poll() and event::read()
   - Process player commands (Space key, movement, etc.)
   - Mark player as ready in TurnState
   - Track if input occurred (input_occurred flag)

2. SYSTEMS DISPATCH SECOND (ONLY IF INPUT OCCURRED!)
   - CRITICAL: Only call dispatcher.dispatch() when input_occurred = true
   - In turn-based games, don't dispatch every frame!
   - Systems see updated input state
   - Turn manager advances phases
   - Actions execute

3. RENDERING THIRD
   - terminal.draw()
   - Shows visual effects (muzzle flashes, etc.)
   - Renders EVERY frame (independent of input)

4. CLEANUP LAST
   - Remove temporary visual effects
   - Prepare for next frame
```

### Why This Order Matters:
- Turn manager waits for player to mark ready (via input)
- If dispatch runs BEFORE input, turn manager sees old state → DEADLOCK
- If dispatch runs EVERY frame (even without input), turn advances before input processed → DEADLOCK
- Systems must run BEFORE rendering for visual effects to appear
- Cleanup must run AFTER rendering so effects are visible
- GameLoopGuard enforces ORDER but can't prevent excessive dispatch calls
- input_occurred flag prevents dispatch when unnecessary

### System Dispatcher Order (see turn_manager.rs comments):
```
CRITICAL: TurnManagerSystem MUST run BEFORE ActionExecutionSystem
1. PathExecutionSystem
2. AIActionPlannerSystem
3. TurnManagerSystem (transitions Planning → Execution)
4. ActionExecutionSystem (executes actions in Execution phase)
5. ObjectiveCaptureSystem
```

### Symptoms of Wrong Order:
- Units stuck in place (actions planned but never executed)
- Log shows "Planning phase" but no "selected" actions
- Turn never advances despite player input
- AI generates actions but doesn't choose/execute them
