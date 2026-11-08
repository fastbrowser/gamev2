use std::collections::{HashMap, VecDeque};
use std::f32::consts::PI;
use std::time::Instant;

use macroquad::prelude::*;
use ::rand::{random, random_range};

use crate::game::hitbox::Hitbox;
use crate::game::sim::{SIM_TICK_MILLIS, SIM_TICK_SECONDS, SimHandle, SimInput};
use crate::game::vectors::{Vector2f, Vector2i};

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
pub const SCALE_PIXELS_SIZE: f32 = 30.0;
pub const CAM_DELAY_TICKS: i32 = 0;
pub const DEATH_LINGER_MS: f32 = 1000.0;
pub const DRAW_HITBOXES: bool = true;

#[derive(Clone)]
pub struct Player {
    pub hitbox: Hitbox,
    pub vel: Vector2f,
    pub onground: bool,
    pub input_x: f32,
    pub jump_held: bool,
}
impl Player {
    pub fn new() -> Self {
        Self {
            hitbox: Hitbox {
                pos: Vector2f::new(0.0, -1.0),
                size: Vector2f::new(0.9, 0.9)
            },
            vel: Vector2f::default(),
            onground: false,
            input_x: 0.0,
            jump_held: false,
        }
    }
}

#[derive(Clone)]
pub struct Block {
    pub hitbox: Hitbox,
    pub slip: f32
}
impl Block {
    pub async fn default() -> Self {
        Self { 
            hitbox: Hitbox {
                pos: Vector2f::new(0.0, 0.0),
                size: Vector2f::new(1.0, 1.0)
            },
            slip: 0.6
        }
    }
    pub fn new(pos: Vector2i, size: Vector2i, slip: f32) -> Self{
        Self { 
            hitbox: Hitbox {
                pos: Vector2f::from(pos),
                size: Vector2f::from(size)
            },
            slip: slip
        }
    }
}


#[derive(Clone)]
pub struct Spike {
    pub drawbox: Hitbox,
    pub hitbox: Hitbox,
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
            }
        }
    }
    pub fn new(pos: Vector2i, size: Vector2i) -> Self{
        Self { 
            drawbox: Hitbox {
                pos: Vector2f::from(pos),
                size: Vector2f::from(size)
            },
            hitbox: Hitbox {
                pos: Vector2f::from(pos).add(0.3, 0.3),
                size: Vector2f::from(size).sub(0.6, 0.3)
            }
        }
    }
}

#[derive(Default,Clone  )]
pub struct Cam {
    pub pos: Vector2f,
    pub history: VecDeque<Vector2f>,
}

#[derive(Debug, Clone)]
pub struct RenderState {
    pub deathparticles: Vec<DeathParticle>,
    pub dead: bool,
    pub tps: f32,
    pub fps: f32,
    pub tick_state: Option<TickState>,
    pub rendered_spikes: u32,
    pub rendered_blocks: u32,
}

impl Default for RenderState {
    fn default() -> Self {
        Self { deathparticles: vec![], dead: false, tps: 0.0, fps: 0.0, tick_state: None, rendered_blocks: 0, rendered_spikes: 0 }
    }
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

    pub fn glob2rel(&self, pos: Vector2f) -> Vector2f {
        pos.subvec(self.cam.pos).add((self.windowsize.x/2.0) as f32, (self.windowsize.y/2.0) as f32)
    }
    pub fn draw(&self, state: &mut RenderState, blocktex: &Texture2D, playertex: &Texture2D, spiketex: &Texture2D) {
        clear_background(WHITE);
        for block in &self.blocks {
            let drawpos = self.glob2rel(block.hitbox.pos.mul(SCALE_PIXELS_SIZE, SCALE_PIXELS_SIZE));
            if drawpos.x < -SCALE_PIXELS_SIZE || drawpos.y < -SCALE_PIXELS_SIZE || drawpos.x > self.windowsize.x+SCALE_PIXELS_SIZE || drawpos.y > self.windowsize.y+SCALE_PIXELS_SIZE {continue;}
            draw_texture_ex(blocktex, drawpos.x, drawpos.y, WHITE, DrawTextureParams { dest_size: Some(Vec2::new( block.hitbox.size.x as f32 * SCALE_PIXELS_SIZE, block.hitbox.size.y as f32 * SCALE_PIXELS_SIZE, )), ..Default::default() });
            if DRAW_HITBOXES {
                let drawhitboxpos = self.glob2rel(block.hitbox.pos.mul(SCALE_PIXELS_SIZE, SCALE_PIXELS_SIZE));
                draw_rectangle_lines(drawhitboxpos.x, drawhitboxpos.y, block.hitbox.size.x as f32 * SCALE_PIXELS_SIZE, block.hitbox.size.y as f32 * SCALE_PIXELS_SIZE, 2.0, RED);
            }
            state.rendered_blocks+=1;
        }
        for spike in &self.spikes {
            let drawpos = self.glob2rel(spike.drawbox.pos.mul(SCALE_PIXELS_SIZE, SCALE_PIXELS_SIZE));
            if drawpos.x < -SCALE_PIXELS_SIZE || drawpos.y < -SCALE_PIXELS_SIZE || drawpos.x > self.windowsize.x+SCALE_PIXELS_SIZE || drawpos.y > self.windowsize.y+SCALE_PIXELS_SIZE {continue;}
            draw_texture_ex(spiketex, drawpos.x, drawpos.y, WHITE, DrawTextureParams { dest_size: Some(Vec2::new( spike.drawbox.size.x as f32 * SCALE_PIXELS_SIZE, spike.drawbox.size.y as f32 * SCALE_PIXELS_SIZE, )), ..Default::default() });
            if DRAW_HITBOXES {
                let drawhitboxpos = self.glob2rel(spike.hitbox.pos.mul(SCALE_PIXELS_SIZE, SCALE_PIXELS_SIZE));
                draw_rectangle_lines(drawhitboxpos.x, drawhitboxpos.y, spike.hitbox.size.x as f32 * SCALE_PIXELS_SIZE, spike.hitbox.size.y as f32 * SCALE_PIXELS_SIZE, 2.0, RED);
            }
            state.rendered_spikes+=1;
        }
        
        let mut drawpos: Vector2f;
        if !state.dead {
            drawpos = self.glob2rel(self.player.hitbox.pos.mul(SCALE_PIXELS_SIZE, SCALE_PIXELS_SIZE));
            draw_texture_ex(playertex, drawpos.x, drawpos.y, WHITE, DrawTextureParams { dest_size: Some(Vec2::new( self.player.hitbox.size.x as f32 * SCALE_PIXELS_SIZE, self.player.hitbox.size.y as f32 * SCALE_PIXELS_SIZE, )), ..Default::default() });
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

#[derive(Clone, Debug)]
pub struct SimPlayer {
    pub hitbox: Hitbox,
    pub vel: Vector2f,
    pub onground: bool,
    pub input_x: f32,
    pub jump_held: bool,
}

impl SimPlayer {
    pub fn from_player(player: &Player) -> Self {
        Self {
            hitbox: player.hitbox,
            vel: player.vel,
            onground: player.onground,
            input_x: player.input_x,
            jump_held: player.jump_held,
        }
    }
}

#[derive(Clone)]
pub struct SimWorld {
    pub player: SimPlayer,
    pub blocks: Vec<Hitbox>,
    pub spikes: Vec<Hitbox>,
    pub block_grid: SpatialGrid,
    pub spike_grid: SpatialGrid,
    pub cam: Cam,
    pub spawn_pos: Vector2f,
    pub player_palette: Vec<Color>,
}

impl SimWorld {
    pub fn from_world(world: World, player_palette: Vec<Color>) -> Self {
        let blocks: Vec<Hitbox> = world.blocks.iter().map(|block| block.hitbox).collect();
        let spikes: Vec<Hitbox> = world.spikes.iter().map(|spike| spike.hitbox).collect();
        Self {
            player: SimPlayer::from_player(&world.player),
            block_grid: SpatialGrid::from_hitboxes(blocks.iter().copied()),
            spike_grid: SpatialGrid::from_hitboxes(spikes.iter().copied()),
            blocks,
            spikes,
            cam: world.cam.clone(),
            spawn_pos: world.player.hitbox.pos,
            player_palette,
        }
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

#[derive(Clone, Debug)]
pub struct DeathParticle {
    pub color: Color,
    pub pos: Vector2f,
    pub prev_pos: Vector2f,
    pub vel: Vector2f,
}

#[derive(Debug, Clone)]
pub struct TickState {
    pub deathparticles: Vec<DeathParticle>,
    pub deathtimeelapsed: f32,
    pub dead: bool,
    pub tps: f32,
    pub fps: f32
}

impl Default for TickState {
    fn default() -> Self {
        Self { deathparticles: vec![], deathtimeelapsed: 0.0, dead: false, tps: 0.0, fps: 0.0}
    }
}

pub fn tickloop(world: &mut SimWorld, delta_time: f32, state: &mut TickState) {
    let time_scale = delta_time / SIM_TICK_MILLIS;
    let constant_multiplier = time_scale * time_scale;
    if state.dead {
        state.deathtimeelapsed+=delta_time;
        if state.deathtimeelapsed > DEATH_LINGER_MS {
            state.deathtimeelapsed = 0.0;
        }
    }
    if state.deathtimeelapsed==0.0 && state.dead {
        state.dead = false;
        world.player.hitbox.pos = world.spawn_pos;
        world.player.vel = Vector2f::new(0.0, 0.0);
    }
    let hit_spike = !state.dead && {
        let mut candidates = Vec::new();
        world
            .spike_grid
            .query(&world.player.hitbox, &mut candidates);
        candidates.iter().any(|&index| {
            world.spikes[index as usize].intersects(&world.player.hitbox)
        })
    };
    if hit_spike {
        for _ in 0..64 {
            let a:u8 = random();
            let x = a&0xf;
            let y = a>>4;
            let theta = random::<f32>() * 2.0 * PI;
            let pos = world.player.hitbox.pos.addvec(world.player.hitbox.size.mul(0.5, 0.5));
            state.deathparticles.push(DeathParticle { 
                color: world.player_palette.get((y as usize) * 16 + x as usize).copied().unwrap_or(WHITE),
                pos,
                prev_pos: pos,
                vel: Vector2f::new(random_range(0.0..0.1)*theta.cos(), random_range(0.0..0.1)*theta.sin())
            });
        }
        state.dead = true;
    }
    let mut particles: Vec<DeathParticle> = vec![];
    for particle in &state.deathparticles {
        let nca = particle.color.a-(delta_time/1000.0);
        if nca>0.0 {
            particles.push(DeathParticle {
                color: Color { r: particle.color.r, g: particle.color.g, b: particle.color.b, a: nca },
                pos: particle.pos.addvec(particle.vel.mul(constant_multiplier, constant_multiplier)),
                prev_pos: particle.pos,
                vel: particle.vel
            });
        }
    }

    state.deathparticles = particles;
}

pub async fn drawloop(world: &mut World, sim: &SimHandle, render_state: &mut RenderState, blocktex: &Texture2D, playertex: &Texture2D, spiketex: &Texture2D) {
    world.windowsize = Vector2f::new(screen_width(), screen_height());
    let left = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
    let right = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);
    let input = SimInput {
        jump_held: is_key_down(KeyCode::Space),
        horizontal: (right as i32 - left as i32) as f32,
    };
    if let Some(snapshot) = sim.frame(input) {
        let alpha = ((Instant::now() - snapshot.at).as_secs_f32() / SIM_TICK_SECONDS)
            .clamp(0.0, 1.0);
        world.player.hitbox.pos =
            snapshot.prev_player_pos.lerp(snapshot.curr_player_pos, alpha);
        world.cam.pos = snapshot.prev_cam_pos.lerp(snapshot.curr_cam_pos, alpha);
        render_state.dead = snapshot.dead;
        render_state.tps = snapshot.tps;
        render_state.tick_state = snapshot.state;
        render_state.deathparticles = snapshot
            .particles
            .iter()
            .map(|particle| DeathParticle {
                color: particle.color,
                pos: particle.prev_pos.lerp(particle.pos, alpha),
                prev_pos: particle.pos,
                vel: Vector2f::ZERO,
            })
            .collect();
    }
    world.draw(render_state, blocktex, playertex, spiketex);
    render_state.rendered_blocks = 0;
    render_state.rendered_spikes = 0;
}