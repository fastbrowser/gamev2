use macroquad::{color::{Color, GRAY, WHITE}, input::{MouseButton, get_keys_down, is_key_down, is_key_pressed, is_mouse_button_down, mouse_position}, shapes::draw_rectangle, text::{draw_text, measure_text}, window::{screen_height, screen_width}};

#[derive(PartialEq, Eq, Clone)]
pub enum GameState {
    Paused,
    Running,
    Stopped
}

pub struct Menu {
    pub enteringlevel: bool,
    pub enteringtimeelapsed: f32,
    pub gamestate: GameState
}

impl Menu {
    pub fn new() -> Self {
        Self { 
            enteringlevel: false,
            enteringtimeelapsed: 0f32,
            gamestate: GameState::Stopped
        }
    }
    pub async fn draw(&mut self) -> GameState {
        let sw = screen_width();
        let sh = screen_height();
        if is_key_pressed(macroquad::input::KeyCode::Escape) {
            self.gamestate = match self.gamestate {
                GameState::Running => GameState::Paused,
                GameState::Paused => GameState::Running,
                GameState::Stopped => GameState::Stopped,
            }
        }
        let mut btnw: f32 = 200.0;
        let mut btnx = sw/2.0-btnw/2.0;
        let mut btnh: f32 = 50.0;
        let mut btny = sh/2.0-btnh/2.0;
        let mut fontsize = 36.0f32;
        const BTNEXP: f32 = 1.5;
        let mp = mouse_position();
        if self.gamestate == GameState::Stopped {
            if mp.0>btnx
                && mp.0<btnx+btnw
                && mp.1>btny
                && mp.1<btny+btnh
            {
                btnw = btnw*BTNEXP;
                btnh = btnh*BTNEXP;
                btnx = sw/2.0-btnw/2.0;
                btny = sh/2.0-btnh/2.0;
                fontsize = fontsize*BTNEXP;
                if is_mouse_button_down(MouseButton::Left) {
                    self.gamestate = GameState::Running;
                }
            }
            draw_rectangle(btnx, btny, btnw, btnh, GRAY);
            let dims = measure_text("Play", None, fontsize as u16, 1.0);
            draw_text("Play", btnx+btnw/2.0-dims.width/2.0, dims.offset_y+btny+btnh/2.0-dims.height/2.0, fontsize, WHITE);
        }
        if self.gamestate == GameState::Paused {
            draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.6 });
            if mp.0>btnx
                && mp.0<btnx+btnw
                && mp.1>btny
                && mp.1<btny+btnh
            {
                btnw = btnw*BTNEXP;
                btnh = btnh*BTNEXP;
                btnx = sw/2.0-btnw/2.0;
                btny = sh/2.0-btnh/2.0;
                fontsize = fontsize*BTNEXP;
                if is_mouse_button_down(MouseButton::Left) {
                    self.gamestate = GameState::Running;
                }
            }
            draw_rectangle(btnx, btny, btnw, btnh, GRAY);
            let dims = measure_text("Resume", None, fontsize as u16, 1.0);
            draw_text("Resume", btnx+btnw/2.0-dims.width/2.0, dims.offset_y+btny+btnh/2.0-dims.height/2.0, fontsize, WHITE);
        }
        self.gamestate.clone()
    }
}