// Viewport and camera system for battlefield rendering

use crate::game_logic::battlefield::Position;

/// Camera that controls what portion of the battlefield is visible
#[derive(Debug, Clone)]
pub struct Camera {
    /// Center position of the camera on the battlefield
    pub center: Position,
    /// Width of the viewport in tiles
    pub viewport_width: usize,
    /// Height of the viewport in tiles
    pub viewport_height: usize,
}

impl Camera {
    /// Creates a new camera centered at the given position
    pub fn new(center: Position, viewport_width: usize, viewport_height: usize) -> Self {
        Self {
            center,
            viewport_width,
            viewport_height,
        }
    }

    /// Moves the camera by the given offset
    pub fn pan(&mut self, dx: i32, dy: i32) {
        self.center.x += dx;
        self.center.y += dy;
    }

    /// Centers the camera on a specific position
    pub fn center_on(&mut self, pos: Position) {
        self.center = pos;
    }

    /// Gets the top-left position of the viewport
    pub fn top_left(&self) -> Position {
        Position::new(
            self.center.x - (self.viewport_width as i32 / 2),
            self.center.y - (self.viewport_height as i32 / 2),
        )
    }

    /// Gets the bottom-right position of the viewport
    pub fn bottom_right(&self) -> Position {
        Position::new(
            self.center.x + (self.viewport_width as i32 / 2),
            self.center.y + (self.viewport_height as i32 / 2),
        )
    }

    /// Checks if a position is within the current viewport
    pub fn is_visible(&self, pos: &Position) -> bool {
        let top_left = self.top_left();
        let bottom_right = self.bottom_right();

        pos.x >= top_left.x
            && pos.x <= bottom_right.x
            && pos.y >= top_left.y
            && pos.y <= bottom_right.y
    }

    /// Constrains the camera to stay within battlefield bounds
    pub fn constrain(&mut self, battlefield_width: usize, battlefield_height: usize) {
        let half_vp_width = (self.viewport_width as i32 / 2).max(0);
        let half_vp_height = (self.viewport_height as i32 / 2).max(0);

        // Ensure camera doesn't go out of bounds
        self.center.x = self
            .center
            .x
            .max(half_vp_width)
            .min(battlefield_width as i32 - half_vp_width - 1);

        self.center.y = self
            .center
            .y
            .max(half_vp_height)
            .min(battlefield_height as i32 - half_vp_height - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_viewport_bounds() {
        let camera = Camera::new(Position::new(10, 10), 20, 20);
        let top_left = camera.top_left();
        let bottom_right = camera.bottom_right();

        assert_eq!(top_left.x, 0);
        assert_eq!(top_left.y, 0);
        assert_eq!(bottom_right.x, 20);
        assert_eq!(bottom_right.y, 20);
    }

    #[test]
    fn test_camera_pan() {
        let mut camera = Camera::new(Position::new(10, 10), 20, 20);
        camera.pan(5, -3);

        assert_eq!(camera.center.x, 15);
        assert_eq!(camera.center.y, 7);
    }

    #[test]
    fn test_is_visible() {
        let camera = Camera::new(Position::new(10, 10), 20, 20);

        assert!(camera.is_visible(&Position::new(10, 10)));
        assert!(camera.is_visible(&Position::new(0, 0)));
        assert!(!camera.is_visible(&Position::new(25, 25)));
    }
}
