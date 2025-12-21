# Project Conventions

## Core Principles

### 1. File Size Limits
**Files should never exceed agent reading limits in length**

- Maximum file size: ~2000 lines (agent reading limit)
- If a file approaches this limit, refactor into smaller modules
- Use imports to compose functionality from smaller files
- Break large systems into logical sub-modules

#### Example Structure
Instead of one large `combat.rs` (3000 lines):
```
combat/
  mod.rs          // Public API and module coordination
  hitscan.rs      // Hitscan weapon mechanics
  projectile.rs   // Physics-based projectiles
  damage.rs       // Damage calculation
  effects.rs      // Combat effects (smoke, fire, etc.)
```

### 2. Modular Code Organization
**Each subsystem needs to have its own file determining its behavior**

- One subsystem = one module (or module directory)
- Clear separation of concerns
- Each module should be independently understandable
- Use Rust's module system effectively

#### Recommended Module Structure
```
src/
  main.rs                    // Entry point only
  lib.rs                     // Library root (if needed)

  components/                // ECS components
    soldier.rs
    weapon.rs
    terrain.rs
    effects.rs

  systems/                   // ECS systems
    movement.rs
    combat.rs
    morale.rs
    ai.rs

  rendering/                 // UI/rendering
    viewport.rs
    layout.rs
    widgets.rs
    animations.rs

  game_logic/               // Game mechanics
    action_points.rs
    line_of_sight.rs
    pathfinding.rs
    fog_of_war.rs

  simulation/               // Simulation layer
    weather.rs
    time.rs
    events.rs
    physics.rs

  utils/                    // Utilities
    config.rs
    math.rs
    constants.rs
```

### 3. Research and Documentation
**When you don't know something, look it up**

- Prefer searching documentation and examples over guessing
- Use agents to research unfamiliar APIs or patterns
- Document findings for future reference
- Add comments explaining non-obvious decisions

#### Research Process
1. Check official documentation first
2. Search for examples in the ecosystem
3. Test assumptions with small experiments
4. Document the discovered approach
5. Share findings in code comments or docs

### 4. Documentation Management
**Maintain document_index.md to track all documentation**

- Update `document_index.md` when creating new docs
- Include brief description of each document's purpose
- Organize by category (planning, technical, guides, etc.)
- Keep the index current - remove references to deleted docs

#### When to Update
- Creating new documentation files
- Removing obsolete documentation
- Reorganizing documentation structure
- Adding significant new sections to existing docs

### 5. Progress Tracking
**Maintain roadmap.md to chart progress**

- Update `roadmap.md` regularly as work progresses
- Mark tasks complete when finished
- Add new tasks as they're discovered
- Track blockers and technical debt
- Include dates for significant milestones

#### Update Frequency
- After completing major features
- When starting new development phases
- When discovering new required work
- During sprint/iteration planning

### 6. Agent Usage
**Employ agents wherever possible for efficient context use**

- Use agents for code exploration and research
- Leverage agents for multi-file refactoring
- Use code-finder agents to locate code (never for changes)
- Use specialized agents for their specific domains
- Parallel agent execution when tasks are independent

#### Agent Best Practices
- Code-finder agents: ONLY for finding code, never for modifications
- Explore agents: For understanding codebase structure
- Plan agents: For implementation strategy
- Task agents: For isolated implementation work
- Always provide clear, specific prompts to agents

---

## Coding Standards

### Rust-Specific Conventions

#### Naming
- Types: `PascalCase`
- Functions/variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

#### Error Handling
- Use `Result<T, E>` for fallible operations
- Use custom error types for subsystems
- Avoid `.unwrap()` except in tests or when truly infallible
- Prefer `?` operator for error propagation

#### Code Organization
- Public API at top of file
- Private helpers below public API
- Tests in `#[cfg(test)]` module at bottom
- Keep related functionality together

#### Performance Considerations
- Profile before optimizing
- Document performance-critical sections
- Use appropriate data structures (Vec vs HashMap)
- Be mindful of allocations in hot loops
- Leverage parallel processing where beneficial (Specs ECS systems)

---

## ECS Patterns

### Component Design
- Components are data only (no methods)
- Keep components small and focused
- Use marker components for flags/states
- Group related data in single component when accessed together

### System Design
- Systems contain logic only
- Each system does one thing well
- Systems read/write appropriate components
- Use parallel systems when no data conflicts
- Document system execution order dependencies

### Resource Management
- Use ECS resources for global state
- Avoid excessive resource locking
- Keep resources focused and minimal

---

## Commit Practices

### Commit Messages
- Use conventional commit format:
  - `feat: Add morale system`
  - `fix: Correct line-of-sight calculation`
  - `refactor: Split combat module into sub-modules`
  - `docs: Update roadmap with Phase 1 progress`
  - `perf: Optimize pathfinding for large maps`
  - `test: Add unit tests for damage calculation`

### Commit Scope
- Keep commits focused and atomic
- One logical change per commit
- Include related tests with feature commits
- Update documentation in same commit as changes

---

## Testing Approach

### Unit Tests
- Test individual functions and systems
- Mock ECS components for system tests
- Test edge cases and error conditions
- Keep tests fast and focused

### Integration Tests
- Test subsystem interactions
- Verify ECS system coordination
- Test complete gameplay scenarios
- Use integration tests for regression prevention

---

## Performance Guidelines

### Scale Targets
- Support 200+ active entities smoothly
- Maintain 60 FPS rendering (turn-based, but animations)
- Pathfinding should complete within 16ms
- FOV calculations should complete within 8ms

### Optimization Strategy
1. Measure first (use profiling tools)
2. Optimize hot paths only
3. Document optimizations and trade-offs
4. Benchmark before and after changes

---

## Design Philosophy Reminders

- **Start simple, layer complexity** - Get core working first
- **Avoid over-engineering** - Build what's needed now
- **Prioritize readability** - Code is read more than written
- **Performance matters** - But only after correctness
- **Realism serves fun** - Not the other way around

---

*These conventions are living guidelines. Update as the project evolves.*

*Last updated: 2025-12-21*
