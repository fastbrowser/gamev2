use crate::game::World;


pub struct Input {}

pub enum Event {
    Jump,
    Left,
    Right,
}

impl Input {
    pub fn handle(world: &mut World, evt: Event) {
        match evt {
            Event::Jump => {
                if world.player.onground {
                    world.player.vel = world.player.vel.add(0.0, -0.075);
                }
            }
            Event::Left => {
                world.player.vel.x = -0.1;
            }
            Event::Right => {
                world.player.vel.x = 0.1;
            }
        }
    }
}