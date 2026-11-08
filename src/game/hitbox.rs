use crate::game::vectors::Vector2f;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hitbox {
    pub pos: Vector2f,
    pub size: Vector2f,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YSide {
    Top,
    Bottom,
}

impl Hitbox {
    pub const fn new(pos: Vector2f, size: Vector2f) -> Self {
        Self { pos, size }
    }

    pub const fn new_raw(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            pos: Vector2f { x, y },
            size: Vector2f { x: width, y: height },
        }
    }

    pub fn intersects(&self, other: &Hitbox) -> bool {
        self.pos.x < other.pos.x + other.size.x
            && self.pos.x + self.size.x > other.pos.x
            && self.pos.y < other.pos.y + other.size.y
            && self.pos.y + self.size.y > other.pos.y
    }

    pub fn collision_clamp(&self, other: &Hitbox) -> (Option<(f32, XSide)>, Option<(f32, YSide)>) {
        if !self.intersects(other) {
            return (None, None);
        }

        let x_clamped = {
            let self_center_x = self.pos.x + self.size.x * 0.5;
            let other_center_x = other.pos.x + other.size.x * 0.5;
            if self_center_x < other_center_x {
                Some((self.pos.x + self.size.x, XSide::Left))
            } else {
                Some((self.pos.x - other.size.x, XSide::Right))
            }
        };

        let y_clamped = {
            let self_center_y = self.pos.y + self.size.y * 0.5;
            let other_center_y = other.pos.y + other.size.y * 0.5;
            if self_center_y < other_center_y {
                Some((self.pos.y + self.size.y, YSide::Top))
            } else {
                Some((self.pos.y - other.size.y, YSide::Bottom))
            }
        };

        (x_clamped, y_clamped)
    }

    pub fn contains_point(&self, point: Vector2f) -> bool {
        point.x >= self.pos.x
            && point.x <= self.pos.x + self.size.x
            && point.y >= self.pos.y
            && point.y <= self.pos.y + self.size.y
    }
}