use macroquad::{input::{KeyCode, is_key_pressed}, texture::Texture2D, time::get_frame_time};

use crate::{f3, game::{game::{RenderState, World, drawloop}, sim::SimHandle}};

pub struct GameWrapper {
    world: World,
    sim: SimHandle,
    render_state: RenderState,
    i: f32,
    blocktex: Texture2D,
    playertex: Texture2D,
    spiketex: Texture2D,
    showf3: bool,
    paused: bool,
}

impl GameWrapper {
    pub fn new(world: World, blocktex: Texture2D, playertex: Texture2D, spiketex: Texture2D) -> Self {
        let player_img = playertex.get_texture_data();
        let mut player_palette = Vec::with_capacity(16 * 16);
        for y in 0..16 {
            for x in 0..16 {
                player_palette.push(player_img.get_pixel(x, y));
            }
        }
        Self {
            world: world.clone(),
            sim: SimHandle::spawn(world.clone(), player_palette),
            render_state: RenderState::default(),
            i: 0f32,
            blocktex,
            playertex,
            spiketex,
            showf3: false,
            paused: false,
        }
    }
    pub async fn frame(&mut self) {
        if is_key_pressed(KeyCode::F3) {self.showf3=!self.showf3;}
        if self.i>1.0 {
            self.i=0.0;
            self.render_state.fps = 1.0/get_frame_time();
        }
        self.i+=get_frame_time();
        drawloop(&mut self.world, &self.sim, &mut self.render_state, &self.blocktex, &self.playertex, &self.spiketex).await;
        if self.showf3 {
            f3::draw(&self.render_state, &self.world);
        }
    }
    pub fn pause(&mut self) {
        if !self.paused {
            self.sim.setpaused(true);
            self.paused = self.sim.paused();
        };
    }
    pub fn unpause(&mut self) {
        if self.paused {
            self.sim.setpaused(false);
            self.paused = self.sim.paused();
        };
    }
    pub fn kill(&mut self) {
        self.sim.stop();
    }
}