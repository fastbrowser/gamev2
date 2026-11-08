use std::time::Instant;

use macroquad::prelude::*;

use gamev2::game::{Block, TICK_LEN_MILLIS_NORMAL, TickState, World, tickloop, window_conf};
use gamev2::input::{self, Input};
use gamev2::phys;
use gamev2::vectors::Vector2f;
use gamev2::worlds;

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = TickState::default();
    let mut last_tick = Instant::now();
    let blocktex = Texture2D::from_file_with_format(include_bytes!("../grass.png"), None);
    blocktex.set_filter(FilterMode::Nearest);
    let spiketex = Texture2D::from_file_with_format(include_bytes!("../spike.png"), None);
    spiketex.set_filter(FilterMode::Nearest);
    let playertex = Texture2D::from_file_with_format(include_bytes!("../player.png"), None);
    playertex.set_filter(FilterMode::Nearest);
    let mut world: World;
    if let Ok(w) = worlds::load("world.wl", blocktex.clone(), playertex.clone(), spiketex.clone()) {
        world = w;
    } else {
        world = worlds::getworld1(blocktex.clone(), playertex.clone(), spiketex.clone());
        worlds::save("world.wl", &world);
    }
    let origworld = world.clone();
    loop {
        let mut delta_time = Instant::now().duration_since(last_tick).as_millis();
        last_tick = Instant::now();
        if delta_time > 100 {
            next_frame().await;
            continue;
        }
        world.windowsize = Vector2f::new(screen_width(), screen_height());
        if !state.dead {
            Input::handle(&mut world, input::Event::Jump(is_key_down(KeyCode::Space)));
            let left = is_key_down(KeyCode::A)||is_key_down(KeyCode::Left);
            let right = is_key_down(KeyCode::D)||is_key_down(KeyCode::Right);
            let dir = (right as i32 - left as i32) as f32;
            Input::handle(&mut world, input::Event::Horizontal(dir));
            phys::tick(&mut world, delta_time as f32);
        }
        tickloop(&mut world, delta_time, &origworld, &mut state);
        world.update_cam();
        world.draw(&state);
        next_frame().await;
    }
}
