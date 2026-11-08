use std::collections::{HashMap, VecDeque};
use std::f32::consts::PI;

use macroquad::prelude::*;
use ::rand::{random, random_range};

use crate::hitbox::Hitbox;
use crate::vectors::{Vector2f, Vector2i};

#[derive(Clone, Default)]
pub struct SpatialGrid {
    cells: HashMap<(i32, i32), Vec<u32>>,
}

impl SpatialGrid {
    pub fn from_hitboxes(hitboxes: impl IntoIterator<Item = Hitbox>) -> Self {
        let mut grid = Self::default();
        for (index, hitbox) in hitboxes.into_iter().enumerate() {
            let (min_x, min_y, max_x, max_y) = cell_bounds(&hitbox);
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    grid.cells.entry((x, y)).or_default().push(index as u32);
                }
            }
        }
        grid
    }

    pub fn query(&self, hitbox: &Hitbox, out: &mut Vec<u32>) {
        out.clear();
        let (min_x, min_y, max_x, max_y) = cell_bounds(hitbox);
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if let Some(indices) = self.cells.get(&(x, y)) {
                    out.extend_from_slice(indices);
                }
            }
        }
        out.sort_unstable();
        out.dedup();
    }

    pub fn point_cells(&self, x: f32, y: f32) -> impl Iterator<Item = u32> + '_ {
        let cx = x.floor() as i32;
        let cy = y.floor() as i32;
        [(cx, cy), (cx - 1, cy), (cx, cy - 1), (cx - 1, cy - 1)]
            .into_iter()
            .filter_map(move |key| self.cells.get(&key))
            .flat_map(|indices| indices.iter().copied())
    }
}

fn cell_bounds(hitbox: &Hitbox) -> (i32, i32, i32, i32) {
    (
        hitbox.pos.x.floor() as i32,
        hitbox.pos.y.floor() as i32,
        ((hitbox.pos.x + hitbox.size.x).ceil() as i32 - 1).max(hitbox.pos.x.floor() as i32),
        ((hitbox.pos.y + hitbox.size.y).ceil() as i32 - 1).max(hitbox.pos.y.floor() as i32),
    )
}

pub const GRAVITY: f32 = 32.0;
pub const JUMP_VELOCITY: f32 = 9.2;
pub const MAX_FALL_SPEED: f32 = 78.4;
pub const GROUND_ACCEL: f32 = 52.0;
pub const AIR_ACCEL: f32 = 9.0;
pub const GROUND_FRICTION: f32 = 12.09;
pub const AIR_FRICTION: f32 = 1.887;
pub const VERTICAL_DRAG: f32 = 0.404;
pub const TICK_LEN_MILLIS_NORMAL: f32 = 50.0;
pub const SCALE_PIXELS_SIZE: f32 = 30.0;
pub const CAM_DELAY_TICKS: i32 = 0;
pub const DEATH_LINGER_MS: f32 = 1000.0;
pub const DRAW_HITBOXES: bool = true;

#[derive(Clone)]
pub struct Player {
    pub hitbox: Hitbox,
    pub vel: Vector2f,
    pub texture: Texture2D,
    pub onground: bool,
    pub input_x: f32,
    pub jump_held: bool,
}
impl Player {
    pub fn new(tex: Texture2D) -> Self {
        Self {
            hitbox: Hitbox {
                pos: Vector2f::new(0.0, -1.0),
                size: Vector2f::new(0.9, 0.9)
            },
            vel: Vector2f::default(),
            texture: tex,
            onground: false,
            input_x: 0.0,
            jump_held: false,
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
                pos: Vector2f::from(pos).add(0.3, 0.3),
                size: Vector2f::from(size).sub(0.6, 0.3)
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
    pub block_grid: SpatialGrid,
    pub spike_grid: SpatialGrid,
}

impl World {
    pub fn new(
        player: Player,
        blocks: Vec<Block>,
        spikes: Vec<Spike>,
        windowsize: Vector2f,
        cam: Cam,
    ) -> Self {
        let mut world = Self {
            player,
            blocks,
            spikes,
            windowsize,
            cam,
            block_grid: SpatialGrid::default(),
            spike_grid: SpatialGrid::default(),
        };
        world.rebuild_grids();
        world
    }

    pub fn rebuild_grids(&mut self) {
        self.block_grid = SpatialGrid::from_hitboxes(self.blocks.iter().map(|block| block.hitbox));
        self.spike_grid = SpatialGrid::from_hitboxes(self.spikes.iter().map(|spike| spike.hitbox));
    }

    fn glob2rel(&self, pos: Vector2f) -> Vector2f {
        pos.subvec(self.cam.pos).add((self.windowsize.x/2.0) as f32, (self.windowsize.y/2.0) as f32)
    }
    pub fn draw(&self, state: &TickState) {
        clear_background(WHITE);
        for block in &self.blocks {
            let drawpos = self.glob2rel(block.hitbox.pos.mul(SCALE_PIXELS_SIZE, SCALE_PIXELS_SIZE));
            if drawpos.x < -SCALE_PIXELS_SIZE || drawpos.y < -SCALE_PIXELS_SIZE || drawpos.x > self.windowsize.x+SCALE_PIXELS_SIZE || drawpos.y > self.windowsize.y+SCALE_PIXELS_SIZE {continue;}
            draw_texture_ex(&block.texture, drawpos.x, drawpos.y, WHITE, DrawTextureParams { dest_size: Some(Vec2::new( block.hitbox.size.x as f32 * SCALE_PIXELS_SIZE, block.hitbox.size.y as f32 * SCALE_PIXELS_SIZE, )), ..Default::default() });
            if DRAW_HITBOXES {
                let drawhitboxpos = self.glob2rel(block.hitbox.pos.mul(SCALE_PIXELS_SIZE, SCALE_PIXELS_SIZE));
                draw_rectangle_lines(drawhitboxpos.x, drawhitboxpos.y, block.hitbox.size.x as f32 * SCALE_PIXELS_SIZE, block.hitbox.size.y as f32 * SCALE_PIXELS_SIZE, 1.0, RED);
            }
        }
        for spike in &self.spikes {
            let drawpos = self.glob2rel(spike.drawbox.pos.mul(SCALE_PIXELS_SIZE, SCALE_PIXELS_SIZE));
            if drawpos.x < -SCALE_PIXELS_SIZE || drawpos.y < -SCALE_PIXELS_SIZE || drawpos.x > self.windowsize.x+SCALE_PIXELS_SIZE || drawpos.y > self.windowsize.y+SCALE_PIXELS_SIZE {continue;}
            draw_texture_ex(&spike.texture, drawpos.x, drawpos.y, WHITE, DrawTextureParams { dest_size: Some(Vec2::new( spike.drawbox.size.x as f32 * SCALE_PIXELS_SIZE, spike.drawbox.size.y as f32 * SCALE_PIXELS_SIZE, )), ..Default::default() });
            if DRAW_HITBOXES {
                let drawhitboxpos = self.glob2rel(spike.hitbox.pos.mul(SCALE_PIXELS_SIZE, SCALE_PIXELS_SIZE));
                draw_rectangle_lines(drawhitboxpos.x, drawhitboxpos.y, spike.hitbox.size.x as f32 * SCALE_PIXELS_SIZE, spike.hitbox.size.y as f32 * SCALE_PIXELS_SIZE, 1.0, RED);
            }
        }
        
        let mut drawpos: Vector2f;
        if !state.dead {
            drawpos = self.glob2rel(self.player.hitbox.pos.mul(SCALE_PIXELS_SIZE, SCALE_PIXELS_SIZE));
            draw_texture_ex(&self.player.texture, drawpos.x, drawpos.y, WHITE, DrawTextureParams { dest_size: Some(Vec2::new( self.player.hitbox.size.x as f32 * SCALE_PIXELS_SIZE, self.player.hitbox.size.y as f32 * SCALE_PIXELS_SIZE, )), ..Default::default() });
        }
        
        for particle in &state.deathparticles {
            drawpos = self.glob2rel(particle.pos.mul(SCALE_PIXELS_SIZE, SCALE_PIXELS_SIZE));
            draw_rectangle(drawpos.x, drawpos.y, 2.0/16.0*SCALE_PIXELS_SIZE, 2.0/16.0*SCALE_PIXELS_SIZE, particle.color);
        }
    }
    pub fn getblockat(&self, pos: Vector2f) -> Option<&Block> {
        for index in self.block_grid.point_cells(pos.x, pos.y) {
            let block = &self.blocks[index as usize];
            if block.hitbox.contains_point(pos) {
                return Some(block);
            }
        }
        None
    }
    pub fn poscontainsspike(&self, pos: Vector2f) -> bool {
        self.spike_grid
            .point_cells(pos.x, pos.y)
            .any(|index| self.spikes[index as usize].hitbox.contains_point(pos))
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

#[derive(Clone, Debug)]
pub struct DeathParticle {
    pub color: Color,
    pub pos: Vector2f,
    pub vel: Vector2f,
}

#[derive(Debug, Clone)]
pub struct TickState {
    pub deathparticles: Vec<DeathParticle>,
    pub deathtimeelapsed: f32,
    pub dead: bool
}

impl Default for TickState {
    fn default() -> Self {
        Self { deathparticles: vec![], deathtimeelapsed: 0.0, dead: false }
    }
}

pub fn tickloop(world: &mut World, delta_time: u128, origworld: &World, state: &mut TickState) {
    let time_scale = delta_time as f32 / TICK_LEN_MILLIS_NORMAL;
    let constant_multiplier = time_scale * time_scale;
    if state.dead {
        state.deathtimeelapsed+=delta_time as f32;
        if state.deathtimeelapsed > DEATH_LINGER_MS {
            state.deathtimeelapsed = 0.0;
        }
    }
    if state.deathtimeelapsed==0.0 && state.dead {
        state.dead = false;
        world.player.hitbox.pos = origworld.player.hitbox.pos;
        world.player.vel = Vector2f::new(0.0, 0.0);
    }
    let hit_spike = !state.dead && {
        let mut candidates = Vec::new();
        world
            .spike_grid
            .query(&world.player.hitbox, &mut candidates);
        candidates.iter().any(|&index| {
            world.spikes[index as usize]
                .hitbox
                .intersects(&world.player.hitbox)
        })
    };
    if hit_spike {
        let img = world.player.texture.get_texture_data();
        for _ in 0..64 {
            let a:u8 = random();
            let x = a&0xf;
            let y = a>>4;
            let theta = random::<f32>() * 2.0 * PI;
            state.deathparticles.push(DeathParticle { 
                color: img.get_pixel(x as u32, y as u32),
                pos: world.player.hitbox.pos.addvec(world.player.hitbox.size.mul(0.5, 0.5)),
                vel: Vector2f::new(random_range(0.0..1.0)*theta.cos(), random_range(0.0..1.0)*theta.sin())
            });
        }
        state.dead = true;
    }
    let mut particles: Vec<DeathParticle> = vec![];
    for particle in &state.deathparticles {
        let nca = particle.color.a-(delta_time as f32/1000.0);
        if nca>0.0 {
            particles.push(DeathParticle { color: Color { r: particle.color.r, g: particle.color.g, b: particle.color.b, a: nca }, pos: particle.pos.addvec(particle.vel.mul(constant_multiplier, constant_multiplier)), vel: particle.vel });
        }
    }

    state.deathparticles = particles;
}