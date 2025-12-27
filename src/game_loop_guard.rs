/// Game Loop Execution Guard
///
/// Enforces the critical execution order for turn-based gameplay:
/// 1. Input handling (marks player ready)
/// 2. Systems dispatch (turn manager sees ready state)
/// 3. Rendering (shows results)
///
/// This prevents the "movement bug" where dispatch runs before input,
/// causing the turn manager to wait for player input that hasn't been processed yet.

use std::marker::PhantomData;

/// Type-state for tracking game loop execution order
pub struct NeedInput;
pub struct NeedDispatch;
pub struct NeedRender;
pub struct Complete;

/// Game loop guard that enforces execution order at compile time
pub struct GameLoopGuard<State> {
    _state: PhantomData<State>,
}

impl GameLoopGuard<NeedInput> {
    /// Create a new game loop guard
    /// Must call methods in order: input() -> dispatch() -> render()
    pub fn new() -> Self {
        GameLoopGuard {
            _state: PhantomData,
        }
    }

    /// Mark that input has been processed
    /// Transitions to NeedDispatch state
    pub fn input_processed(self) -> GameLoopGuard<NeedDispatch> {
        GameLoopGuard {
            _state: PhantomData,
        }
    }
}

impl GameLoopGuard<NeedDispatch> {
    /// Mark that systems have been dispatched
    /// Transitions to NeedRender state
    pub fn systems_dispatched(self) -> GameLoopGuard<NeedRender> {
        GameLoopGuard {
            _state: PhantomData,
        }
    }
}

impl GameLoopGuard<NeedRender> {
    /// Mark that rendering is complete
    /// Transitions to Complete state
    pub fn rendering_complete(self) -> GameLoopGuard<Complete> {
        GameLoopGuard {
            _state: PhantomData,
        }
    }
}

impl GameLoopGuard<Complete> {
    /// Frame is complete, ready for next iteration
    pub fn frame_complete(self) {
        // Guard consumed, frame complete
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_order_compiles() {
        let guard = GameLoopGuard::new();
        let guard = guard.input_processed();
        let guard = guard.systems_dispatched();
        let guard = guard.rendering_complete();
        guard.frame_complete();
    }

    #[test]
    fn multiple_frames_correct_order() {
        // Simulate multiple game loop iterations
        for _ in 0..3 {
            let guard = GameLoopGuard::new();
            let guard = guard.input_processed();
            let guard = guard.systems_dispatched();
            let guard = guard.rendering_complete();
            guard.frame_complete();
        }
    }

    // COMPILE-TIME ENFORCEMENT EXAMPLES
    // These tests demonstrate that wrong ordering is prevented at compile time.
    // Uncomment any of these to see the compiler error that prevents the movement bug.

    // #[test]
    // fn wrong_order_skip_input_fails() {
    //     let guard = GameLoopGuard::new();
    //     // ERROR: no method named `systems_dispatched` found for struct `GameLoopGuard<NeedInput>`
    //     let guard = guard.systems_dispatched();
    // }

    // #[test]
    // fn wrong_order_skip_dispatch_fails() {
    //     let guard = GameLoopGuard::new();
    //     let guard = guard.input_processed();
    //     // ERROR: no method named `rendering_complete` found for struct `GameLoopGuard<NeedDispatch>`
    //     let guard = guard.rendering_complete();
    // }

    // #[test]
    // fn wrong_order_dispatch_before_input_fails() {
    //     let guard = GameLoopGuard::new();
    //     // This is the exact bug we're preventing!
    //     // ERROR: no method named `systems_dispatched` found for struct `GameLoopGuard<NeedInput>`
    //     let guard = guard.systems_dispatched();
    //     let guard = guard.input_processed();
    // }

    // #[test]
    // fn wrong_order_double_input_fails() {
    //     let guard = GameLoopGuard::new();
    //     let guard = guard.input_processed();
    //     // ERROR: no method named `input_processed` found for struct `GameLoopGuard<NeedDispatch>`
    //     let guard = guard.input_processed();
    // }
}
