use gamev2::{game::game::World, gamewrapper::GameWrapper, menu::menu::{GameState, Menu}, tex::Textures};
use macroquad::prelude::*;

use gamev2::{f3, worlds};

pub fn window_conf() -> Conf {
    let mut conf = Conf {
        window_title: "gv2".to_owned(),
        window_width: 800,
        window_height: 600,
        window_resizable: true,
        ..Default::default()
    };
    conf.platform.swap_interval = Some(0);
    conf
}

#[macroquad::main(window_conf)]
async fn main() {
    let defaulttextures = Textures {
        player: {let t = Texture2D::from_file_with_format(include_bytes!("../player.png"), None);t.set_filter(FilterMode::Nearest);Some(t)},
        block: {let t = Texture2D::from_file_with_format(include_bytes!("../block.png"), None);t.set_filter(FilterMode::Nearest);Some(t)},
        spike: {let t = Texture2D::from_file_with_format(include_bytes!("../spike.png"), None);t.set_filter(FilterMode::Nearest);Some(t)},
    };
    let mut textures = defaulttextures.clone();
    let mut menu = Menu::new();
    let mut gw = None;
    let mut gs = GameState::Running;
    loop {
        if gs==GameState::Running && gw.is_none() {
            let world: World;
            if let Ok(w) = worlds::load("world.wl") {
                world = w.0;
                textures = Textures {
                    block: w.1.block.or_else(|| defaulttextures.block.clone()),
                    spike: w.1.spike.or_else(|| defaulttextures.spike.clone()),
                    player: w.1.player.or_else(|| defaulttextures.player.clone()),
                };
            } else {
                world = worlds::getworld1();
                let _ = worlds::save("world.wl", &world, &textures);
            }
            gw = Some(GameWrapper::new(world, textures.clone()));
        }
        if gs == GameState::Stopped {
            if let Some(mut g) = gw.take() {
                g.kill();
            }
        }
        if gs == GameState::Paused && let Some(g) = gw.as_mut() {
            g.frame().await;
            g.pause();
        }
        if gs == GameState::Running {
            if gw.is_none() {
                gw = match worlds::load("world.wl") {
                    Ok(d) => Some(GameWrapper::new(d.0, d.1)),
                    Err(_) => {
                        let w = worlds::getworld1();
                        let _ = worlds::save("world.wl", &w, &defaulttextures);
                        Some(GameWrapper::new(w, defaulttextures.clone()))
                    }
                };
            }
            if let Some(g) = gw.as_mut() {
                g.frame().await;
                g.unpause();
            }
        }
        gs = menu.draw().await;
        next_frame().await;
    }
}
