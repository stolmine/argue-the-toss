# Documentation Index - Argue the Toss

This file indexes all major features, bug fixes, and system implementations in the codebase.

## Core Systems

### Turn-Based Game Loop
**Files:** `/Users/why/repos/argue-the-toss/src/main.rs`, `/Users/why/repos/argue-the-toss/CLAUDE.md`

**Critical Requirements:**
1. **Input Handling First** - Process player commands and mark ready in TurnState
2. **Systems Dispatch Second (ONLY IF INPUT OCCURRED!)** - Dispatch only when `input_occurred = true`
3. **Rendering Third** - Draw terminal, show visual effects
4. **Cleanup Last** - Remove temporary effects

**Bug Prevention:**
- GameLoopGuard enforces execution order (type-state pattern)
- `input_occurred` flag prevents excessive dispatch calls in turn-based game
- Without both safeguards, units freeze/don't execute actions

**See:** CLAUDE.md for detailed explanation of "The Movement Bug"

### Visual Feedback Systems
**Files:** `/Users/why/repos/argue-the-toss/src/systems/muzzle_flash.rs`, `/Users/why/repos/argue-the-toss/src/widgets.rs`

**Muzzle Flash Persistence (2025-12-27):**
- Flashes persist through Planning phase into player's turn
- Cleaned when transitioning Planning → Execution phase
- Ensures player sees enemy fire locations before taking action

**FOV/FOW Rendering (2025-12-27):**
- Soldiers only render if visible to player
- Allies always visible
- Enemies only visible in FOV or when recently fired
- Reveal-on-fire: Enemy positions revealed when they shoot

**Corpse Z-Level (2025-12-27):**
- Two-pass rendering: dead soldiers first, living soldiers second
- Living soldiers always render on top of corpses
- Prevents visual clutter in high-casualty areas

### UI/UX Systems

**Cursor Positioning (2025-12-27):**
- Look/Targeting mode cursor starts at viewport center
- Previous behavior: cursor started at player position (often off-screen)
- Enhanced cursor visibility: character display with black-on-yellow styling

**Files:** `/Users/why/repos/argue-the-toss/src/input.rs`, `/Users/why/repos/argue-the-toss/src/widgets.rs`

### AI Systems

**Utility-Based AI (2025-12-21):**
- 7 response curve types for flexible action scoring
- 14 consideration evaluators (core + tactical)
- 6 AI personalities: Aggressive, Defensive, Balanced, Objective-Focused, Scout, RearGuard
- Rank-based personality assignment
- Officer following behavior

**AI Combat Engagement (2025-12-21):**
- Shoot actions use Average scoring (not Multiplicative)
- Base scores: 0.9-1.0 for aggressive personalities
- AI engages competitively in combat

**AI Tactical Movement (2025-12-21):**
- 6 tactical considerations: ExposedDanger, TacticalAdvantage, ForceBalance, SupportProximity, ObjectivePressure, RetreatNecessity
- Context-aware repositioning (cover, force balance, retreat logic)
- Performance optimized with fast heuristics

**SeekObjective Tuning (2025-12-27):**
- Base scores increased: 0.75-0.9 (up from lower values)
- Polynomial exponent 3.0 for aggressive personalities
- SeekCover reduced: 0.5-0.8 to encourage objective pursuit
- AI actively moves toward objectives

**Files:** `/Users/why/repos/argue-the-toss/src/ai/`, `/Users/why/repos/argue-the-toss/src/systems/ai_action_planner.rs`

### Rank & Progression System (2025-12-21)

**Rank System:**
- 5 WWI ranks: Captain, Lieutenant, Sergeant, Corporal, Private
- Rank-based icons (★☆●○■) with faction colors
- Realistic distribution: 70% privates, 2% captains

**Stat System:**
- Individual stat variation (accuracy, movement speed, HP)
- Rank-based stat scaling (officers superior to privates)
- Stats integrated with combat, movement, vision

**Files:** `/Users/why/repos/argue-the-toss/src/components.rs`, `/Users/why/repos/argue-the-toss/src/systems/`

### Objective System (2025-12-21)

**Flag Capture:**
- Two flags per map (one per faction)
- Capture mechanics: occupy for 5 turns
- Victory condition: capture all enemy flags
- Strategic placement near trenches/fortifications
- Opposite territory placement (75%+ apart)
- AI actively seeks and defends objectives

**Files:** `/Users/why/repos/argue-the-toss/src/objectives.rs`, `/Users/why/repos/argue-the-toss/src/systems/objective_capture.rs`

### Menu System (2025-12-21)

**Menus:**
- Main menu (New Game, Load Game placeholder, Settings, Quit)
- New game configuration menu with presets
- Settings menu (turn order, time budget)
- Pause menu (in-game with ESC)
- Full map configuration interface
- Soldier count configuration (5-500 per team)
- Look mode with proper ESC handling

**Files:** `/Users/why/repos/argue-the-toss/src/menu/`

### Procedural Generation (Phase 3)

**Terrain Generation:**
- 30+ terrain types
- 7-phase generation algorithm
- 7 historical presets (Verdun, Somme, Ypres, etc.)
- Intelligent trench network placement
- Building clusters, vegetation, no-man's-land
- Faction-specific spawn zones (75+ tiles apart)
- Dynamic spawn radius scaling

**Files:** `/Users/why/repos/argue-the-toss/src/battlefield_generation.rs`

### Combat System (Phase 2)

**Hitscan Combat:**
- 70% base accuracy
- Accuracy falloff with range (full to effective_range, degrades to 30% at max_range)
- Cover-based damage reduction
- Ammo/reload mechanics
- LOS-based targeting

**Files:** `/Users/why/repos/argue-the-toss/src/systems/combat.rs`, `/Users/why/repos/argue-the-toss/src/weapons.rs`

### Vision System (Phase 2)

**Vision Cone:**
- 120° main cone
- 60° peripheral (dimmed)
- 180° blind spot
- Terrain dimming outside vision cone
- Last-seen entity markers (10 turn expiry)
- Shared ally vision with spotter attribution

**Files:** `/Users/why/repos/argue-the-toss/src/systems/vision.rs`, `/Users/why/repos/argue-the-toss/src/systems/fov.rs`

### Movement System (Phase 2)

**8-Direction Movement:**
- qweasdzxc keyboard layout
- Auto-facing on movement
- Time costs: 1.5s × terrain multiplier
- Rotation time: 0.3s
- Pathfinding with A* algorithm
- Visual path preview

**Files:** `/Users/why/repos/argue-the-toss/src/systems/movement.rs`, `/Users/why/repos/argue-the-toss/src/pathfinding.rs`

### Turn System (Phase 2)

**Time Budget:**
- 12 second budget per turn (configurable)
- Time-based actions (move, shoot, reload)
- Multi-turn action tracking
- Action commitment system
- Time debt handling

**Files:** `/Users/why/repos/argue-the-toss/src/systems/turn_manager.rs`, `/Users/why/repos/argue-the-toss/src/components.rs`

## Critical Bug Fixes

### Movement Bug - Final Fix (2025-12-27)
**Problem:** Units would freeze/not execute actions, turn advances before input processed

**Root Cause:** Systems dispatch running every frame in turn-based game

**Solution:**
1. Added `input_occurred` flag in main game loop
2. Dispatch only runs when input processed
3. Complements existing GameLoopGuard type-state enforcement
4. Both safeguards required: GameLoopGuard enforces ORDER, input_occurred prevents EXCESSIVE CALLS

**Files Modified:** `/Users/why/repos/argue-the-toss/src/main.rs`, `/Users/why/repos/argue-the-toss/CLAUDE.md`

### Debug Output Performance (2025-12-27)
**Problem:** eprintln! debug logging breaking TUI rendering

**Solution:** Removed performance logging that was writing to stderr

**Files Modified:** `/Users/why/repos/argue-the-toss/src/systems/ai_action_planner.rs`

## Known Issues

### Barbed Wire Density
**Status:** Needs tuning - density too high in procedural generation

**Reference:** Roadmap Phase 3, line 154

## Testing & Quality

**Integration Tests:**
- Movement system tests
- Runtime position validation (debug builds)

**Documentation:**
- TROUBLESHOOTING.md for common issues
- Comprehensive code comments
- CLAUDE.md for critical game loop order

**Files:** `/Users/why/repos/argue-the-toss/tests/`, `/Users/why/repos/argue-the-toss/TROUBLESHOOTING.md`

## Latest Session (2025-12-27)

**Focus:** Visual feedback, FOV rendering, cursor behavior, movement bug fix

**Major Changes:**
1. Muzzle flash persistence fix
2. FOV/FOW rendering improvements
3. Cursor positioning fix (viewport center)
4. Movement bug final fix (input_occurred flag)
5. AI SeekObjective tuning
6. Debug output cleanup
7. CLAUDE.md documentation update

**Files Modified:**
- `/Users/why/repos/argue-the-toss/src/main.rs`
- `/Users/why/repos/argue-the-toss/src/systems/muzzle_flash.rs`
- `/Users/why/repos/argue-the-toss/src/widgets.rs`
- `/Users/why/repos/argue-the-toss/src/input.rs`
- `/Users/why/repos/argue-the-toss/src/ai/action_generation.rs`
- `/Users/why/repos/argue-the-toss/src/ai/actions.rs`
- `/Users/why/repos/argue-the-toss/src/ai/considerations.rs`
- `/Users/why/repos/argue-the-toss/src/ai/personality.rs`
- `/Users/why/repos/argue-the-toss/src/systems/ai_action_planner.rs`
- `/Users/why/repos/argue-the-toss/CLAUDE.md`

---

*Last updated: 2025-12-27*
