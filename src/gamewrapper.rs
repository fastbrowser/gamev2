use macroquad::{input::{KeyCode, is_key_pressed}, texture::Texture2D, time::get_frame_time};

use crate::{f3, game::{game::{RenderState, World, drawloop}, sim::SimHandle}, tex::Textures};

pub struct GameWrapper<'a> {
    world: World,
    sim: SimHandle,
    render_state: RenderState,
    i: f32,
    textures: &'a Textures,
    showf3: bool,
    paused: bool,
}

impl<'a> GameWrapper<'a> {
    pub fn new(world: World, textures: &'a Textures) -> Self {
        let player_img = textures.player.clone().expect("shouldn't happen").get_texture_data();
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
            textures: textures,
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
        drawloop(&mut self.world, &self.sim, &mut self.render_state, self.textures).await;
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