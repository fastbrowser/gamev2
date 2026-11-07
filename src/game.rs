use std::collections::VecDeque;

use macroquad::prelude::*;

use crate::hitbox::Hitbox;
use crate::vectors::{Vector2f, Vector2i};

pub const GRAVITY: f32 = 0.08;
pub const TICK_LEN_MILLIS_NORMAL: f32 = 50.0;
pub const SCALE_PIXELS_SIZE: f32 = 30.0;
pub const CAM_DELAY_TICKS: i32 = 0;

#[derive(Clone)]
pub struct Player {
    pub hitbox: Hitbox,
    pub vel: Vector2f,
    pub texture: Texture2D,
    pub onground: bool,
}
impl Player {
    pub fn new(tex: Texture2D) -> Self {
        Self {
            hitbox: Hitbox {
                pos: Vector2f::new(0.0, -1.0),
                size: Vector2f::new(1.0, 1.0)
            },
            vel: Vector2f::default(),
            texture: tex,
            onground: false,
        }
    }
}

#[derive(Clone)]
pub struct Block {
    pub hitbox: Hitbox,
    pub slip: f32,
    pub texture: Texture2D
}
impl Block {
    pub async fn default() -> Self {
        Self { 
            hitbox: Hitbox {
                pos: Vector2f::new(0.0, 0.0),
                size: Vector2f::new(1.0, 1.0)
            },
            slip: 0.6,
            texture: {
                let tex = load_texture("dirt.png").await.unwrap();
                tex.set_filter(FilterMode::Nearest);
                tex
            }
        }
    }
    pub fn new(pos: Vector2i, size: Vector2i, slip: f32, texture: Texture2D) -> Self{
        Self { 
            hitbox: Hitbox {
                pos: Vector2f::from(pos),
                size: Vector2f::from(size)
            },
            slip: slip,
            texture: texture
        }
    }
}


#[derive(Clone)]
pub struct Spike {
    pub drawbox: Hitbox,
    pub hitbox: Hitbox,
    pub texture: Texture2D
}
impl Spike {
    pub async fn default() -> Self {
        Self {
            drawbox: Hitbox {
                pos: Vector2f::new(0.0, 0.0),
                size: Vector2f::new(1.0, 1.0)
            },
            hitbox: Hitbox {
                pos: Vector2f::new(0.0, 0.0),
                size: Vector2f::new(1.0, 1.0)
            },
            texture: {
                let tex = load_texture("dirt.png").await.unwrap();
                tex.set_filter(FilterMode::Nearest);
                tex
            }
        }
    }
    pub fn new(pos: Vector2i, size: Vector2i, texture: Texture2D) -> Self{
        Self { 
            drawbox: Hitbox {
                pos: Vector2f::from(pos),
                size: Vector2f::from(size)
            },
            hitbox: Hitbox {
                pos: Vector2f::from(pos),
                size: Vector2f::from(size)
            },
            texture: texture
        }
    }
}

#[derive(Default,Clone  )]
pub struct Cam {
    pub pos: Vector2f,
    pub history: VecDeque<Vector2f>,
}

#[derive(Clone)]
pub struct World {
    pub player: Player,
    pub blocks: Vec<Block>,
    pub spikes: Vec<Spike>,
    pub windowsize: Vector2f,
    pub cam: Cam,
}

impl World {
    fn glob2rel(&self, pos: Vector2f) -> Vector2f {
        pos.subvec(self.cam.pos).add((self.windowsize.x/2.0) as f32, (self.windowsize.y/2.0) as f32)
    }
    pub fn draw(&self) {
        clear_background(WHITE);
        for block in &self.blocks {
            let drawpos = self.glob2rel(block.hitbox.pos.mul(SCALE_PIXELS_SIZE, SCALE_PIXELS_SIZE));
            if drawpos.x < -SCALE_PIXELS_SIZE || drawpos.y < -SCALE_PIXELS_SIZE || drawpos.x > self.windowsize.x+SCALE_PIXELS_SIZE || drawpos.y > self.windowsize.y+SCALE_PIXELS_SIZE {continue;}
            draw_texture_ex(&block.texture, drawpos.x, drawpos.y, WHITE, DrawTextureParams { dest_size: Some(Vec2::new( block.hitbox.size.x as f32 * SCALE_PIXELS_SIZE, block.hitbox.size.y as f32 * SCALE_PIXELS_SIZE, )), ..Default::default() });
        }
        for spike in &self.spikes {
            let drawpos = self.glob2rel(spike.drawbox.pos.mul(SCALE_PIXELS_SIZE, SCALE_PIXELS_SIZE));
            if drawpos.x < -SCALE_PIXELS_SIZE || drawpos.y < -SCALE_PIXELS_SIZE || drawpos.x > self.windowsize.x+SCALE_PIXELS_SIZE || drawpos.y > self.windowsize.y+SCALE_PIXELS_SIZE {continue;}
            draw_texture_ex(&spike.texture, drawpos.x, drawpos.y, WHITE, DrawTextureParams { dest_size: Some(Vec2::new( spike.drawbox.size.x as f32 * SCALE_PIXELS_SIZE, spike.drawbox.size.y as f32 * SCALE_PIXELS_SIZE, )), ..Default::default() });
        }
        let drawpos = self.glob2rel(self.player.hitbox.pos.mul(SCALE_PIXELS_SIZE, SCALE_PIXELS_SIZE));
        draw_texture_ex(&self.player.texture, drawpos.x, drawpos.y, WHITE, DrawTextureParams { dest_size: Some(Vec2::new( self.player.hitbox.size.x as f32 * SCALE_PIXELS_SIZE, self.player.hitbox.size.y as f32 * SCALE_PIXELS_SIZE, )), ..Default::default() });
    }
    pub fn getblockat(&self, pos: Vector2f) -> Option<Block> {
        for block in &self.blocks {
            if block.hitbox.contains_point(pos) {
                return Some(block.clone());
            }
        }
        None
    }
    pub fn poscontainsspike(&self, pos: Vector2f) -> bool {
        for spike in &self.spikes {
            if spike.hitbox.contains_point(pos) {
                return true;
            }
        }
        false
    }
    pub fn update_cam(&mut self) {
        let center = self.player.hitbox.pos
            .addvec(self.player.hitbox.size.mul(0.5, 0.5))
            .mul(SCALE_PIXELS_SIZE, SCALE_PIXELS_SIZE);

        if CAM_DELAY_TICKS <= 0 {
            self.cam.pos = center;
            return;
        }

        self.cam.history.push_back(center);
        while self.cam.history.len() > CAM_DELAY_TICKS as usize {
            self.cam.history.pop_front();
        }
        if let Some(&delayed) = self.cam.history.front() {
            self.cam.pos = delayed;
        }
    }
}


pub fn window_conf() -> Conf {
    Conf {
        window_title: "gv2".to_owned(),
        window_width: 800,
        window_height: 600,
        window_resizable: true,
        ..Default::default()
    }
}

pub struct TickState {
    
}

pub fn tickloop(world: &mut World, delta_time: u128, origworld: &World) {
    if world.poscontainsspike(world.player.hitbox.pos)
        || world.poscontainsspike(world.player.hitbox.pos.add(world.player.hitbox.size.x, 0.0))
        || world.poscontainsspike(world.player.hitbox.pos.add(0.0, world.player.hitbox.size.y))
        || world.poscontainsspike(world.player.hitbox.pos.addvec(world.player.hitbox.size))
    {
        world.player.hitbox.pos = origworld.player.hitbox.pos;
    }
}