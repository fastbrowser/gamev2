use gamev2::{game::game::World, gamewrapper::GameWrapper, menu::menu::{GameState, Menu}};
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
    let blocktex = Texture2D::from_file_with_format(include_bytes!("../grass.png"), None);
    blocktex.set_filter(FilterMode::Nearest);
    let spiketex = Texture2D::from_file_with_format(include_bytes!("../spike.png"), None);
    spiketex.set_filter(FilterMode::Nearest);
    let playertex = Texture2D::from_file_with_format(include_bytes!("../player.png"), None);
    playertex.set_filter(FilterMode::Nearest);
    let mut menu = Menu::new();
    let mut gw = None;
    let mut gs = GameState::Running;
    loop {
        if gs==GameState::Running && gw.is_none() {
            let world: World;
            if let Ok(w) = worlds::load("world.wl") {
                world = w;
            } else {
                world = worlds::getworld1();
                let _ = worlds::save("world.wl", &world);
            }
            gw = Some(GameWrapper::new(world, blocktex.clone(), playertex.clone(), spiketex.clone()));
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
                let world: World = worlds::load("world.wl").unwrap_or_else(|_| {
                    let w = worlds::getworld1();
                    let _ = worlds::save("world.wl", &w);
                    w
                });
                gw = Some(GameWrapper::new(world, blocktex.clone(), playertex.clone(), spiketex.clone()));
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
