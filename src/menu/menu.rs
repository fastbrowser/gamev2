use macroquad::prelude::*;

use crate::menu::iface::{self, Ui};

#[derive(PartialEq, Eq, Clone)]
pub enum GameState {
    Paused,
    Running,
    Stopped
}

pub struct Menu {
    pub enteringlevel: bool,
    pub enteringtimeelapsed: f32,
    pub gamestate: GameState,
    pub showinghelp: bool,
}

impl Menu {
    pub fn new() -> Self {
        Self { 
            enteringlevel: false,
            enteringtimeelapsed: 0f32,
            gamestate: GameState::Stopped,
            showinghelp: false,
        }
    }
    pub async fn draw(&mut self) -> GameState {
        if is_key_pressed(KeyCode::Escape) {
            self.gamestate = match self.gamestate {
                GameState::Running => GameState::Paused,
                GameState::Paused => GameState::Running,
                GameState::Stopped => GameState::Stopped,
            }
        }
        let ui = Ui::new();
        if self.showinghelp {
            ui.overlay(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.6 });
            if ui.centered_column(iface::Column::new(vec![
                iface::Element::from(iface::Text::new("Help").font_size(56.0)),
                iface::Element::from(iface::Text::new("How to play:\n   A/D: left/right\n   Space: Jump\n   Esc: pause\nYou can make your own levels using the level editor.").font_size(30.0)),
                iface::Element::from(ui.button("Ok", Rect::new(0.0, 0.0, 200.0, 50.0))
                    .font_size(36.0)
                    .hover_scale(1.5))
            ]).gap(24.0)).is_some() {
                self.showinghelp = false;
            }
        }
        match self.gamestate {
            GameState::Stopped => {
                if !self.showinghelp {
                    let play = ui.button("Play", Rect::new(0.0, 0.0, 200.0, 50.0))
                        .font_size(36.0)
                        .hover_scale(1.5)
                        .id("play");
                    let help = ui.button("Help", Rect::new(0.0, 0.0, 200.0, 50.0))
                        .font_size(36.0)
                        .hover_scale(1.5)
                        .id("help");
                    if let Some(id) = ui.centered_column(iface::Column::new(vec![
                        iface::Element::from(iface::Text::new("gv2").font_size(56.0)),
                        iface::Element::from(play),
                        iface::Element::from(help),
                    ]).gap(24.0)) {
                        if id=="play" {
                            self.gamestate = GameState::Running;
                        }
                        if id=="help" {
                            self.showinghelp = true;
                        }
                    }
                }
            }
            GameState::Paused => {
                if !self.showinghelp {
                    ui.overlay(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.6 });
                    let resume = ui.button("Resume", Rect::new(0.0, 0.0, 200.0, 50.0))
                        .font_size(36.0)
                        .hover_scale(1.5)
                        .id("resume");
                    let help = ui.button("Help", Rect::new(0.0, 0.0, 200.0, 50.0))
                        .font_size(36.0)
                        .hover_scale(1.5)
                        .id("help");
                    if let Some(id) = ui.centered_column(iface::Column::new(vec![
                        iface::Element::from(iface::Text::new("Paused").font_size(40.0)),
                        iface::Element::from(resume),
                        iface::Element::from(help),
                    ]).gap(24.0)) {
                        if id=="help" {
                            self.showinghelp = true;
                        }
                        if id=="resume" {
                            self.gamestate = GameState::Running;
                        }
                    }
                }
            }
            _ => {}
        }
        self.gamestate.clone()
    }
}
