use crate::game::World;


pub struct Input {}

pub enum Event {
    Jump(bool),
    Horizontal(f32),
}

impl Input {
    pub fn handle(world: &mut World, evt: Event) {
        match evt {
            Event::Jump(held) => {
                world.player.jump_held = held;
            }
            Event::Horizontal(dir) => {
                world.player.vel.x = dir.clamp(-1.0, 1.0)*10.0;
            }
        }
    }
}
