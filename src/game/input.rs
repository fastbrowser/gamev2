use crate::game::game::SimPlayer;


pub struct Input {}

pub enum Event {
    Jump(bool),
    Horizontal(f32),
}

impl Input {
    pub fn handle(player: &mut SimPlayer, evt: Event) {
        match evt {
            Event::Jump(held) => {
                player.jump_held = held;
            }
            Event::Horizontal(dir) => {
                player.vel.x = dir.clamp(-1.0, 1.0)*10.0;
            }
        }
    }
}
