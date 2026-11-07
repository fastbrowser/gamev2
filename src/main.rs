use std::time::Instant;

use macroquad::prelude::*;

use gamev2::game::{Block, TICK_LEN_MILLIS_NORMAL, World, window_conf};
use gamev2::input::{self, Input};
use gamev2::phys;
use gamev2::vectors::Vector2f;
use gamev2::worlds;

#[macroquad::main(window_conf)]
async fn main() {
    let mut last_tick = Instant::now();
    let blocktex = Texture2D::from_file_with_format(include_bytes!("../grass.png"), None);
    blocktex.set_filter(FilterMode::Nearest);
    let spiketex = Texture2D::from_file_with_format(include_bytes!("../spike.png"), None);
    spiketex.set_filter(FilterMode::Nearest);
    let playertex = Texture2D::from_file_with_format(include_bytes!("../player.png"), None);
    playertex.set_filter(FilterMode::Nearest);
    //let mut world: World;
    //if let Ok(w) = worlds::load("world.wl", blocktex.clone(), playertex.clone(), spiketex.clone()) {
    //    world = w;
    //} else {
    //    world = worlds::getworld1(blocktex.clone(), playertex.clone(), spiketex.clone());
    //    worlds::save("world.wl", &world);
    //}
    let mut world = worlds::getworld1(blocktex.clone(), playertex.clone(), spiketex.clone());
    let origworld = world.clone();
    loop {
        let delta_time = Instant::now().duration_since(last_tick).as_millis();
        last_tick = Instant::now();
        world.windowsize = Vector2f::new(screen_width(), screen_height());
        phys::tick(&mut world, delta_time as f32);
        world.update_cam();
        world.draw();
        if is_key_down(KeyCode::Space) {Input::handle(&mut world, input::Event::Jump);}
        if is_key_down(KeyCode::D)||is_key_down(KeyCode::Right) {Input::handle(&mut world, input::Event::Right);}
        if is_key_down(KeyCode::A)||is_key_down(KeyCode::Left) {Input::handle(&mut world, input::Event::Left);}
        next_frame().await;
    }
}
