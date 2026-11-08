use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use macroquad::prelude::Color;

use crate::game::{
    game::{SimWorld, TickState, World, tickloop}, input::{Event, Input}, phys, vectors::Vector2f,
};

pub const SIM_TICK_MILLIS: f32 = 1000.0 / 60.0;
pub const SIM_TICK_SECONDS: f32 = SIM_TICK_MILLIS / 1000.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct SimInput {
    pub jump_held: bool,
    pub horizontal: f32,
}

#[derive(Clone, Debug)]
pub struct ParticleSnapshot {
    pub color: Color,
    pub prev_pos: Vector2f,
    pub pos: Vector2f,
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub at: Instant,
    pub prev_player_pos: Vector2f,
    pub curr_player_pos: Vector2f,
    pub prev_cam_pos: Vector2f,
    pub curr_cam_pos: Vector2f,
    pub particles: Vec<ParticleSnapshot>,
    pub dead: bool,
    pub tps: f32,
    pub state: Option<TickState>,
}

struct Shared {
    input: SimInput,
    snapshot: Option<Snapshot>,
}

pub struct SimHandle {
    shared: Arc<Mutex<Shared>>,
    running: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    paused: Arc<AtomicBool>,
}

impl SimHandle {
    pub fn spawn(world: World, player_palette: Vec<Color>) -> Self {
        let sim = SimWorld::from_world(world, player_palette);
        let state = TickState::default();

        let initial = Snapshot {
            at: Instant::now(),
            prev_player_pos: sim.player.hitbox.pos,
            curr_player_pos: sim.player.hitbox.pos,
            prev_cam_pos: sim.cam.pos,
            curr_cam_pos: sim.cam.pos,
            particles: Vec::new(),
            dead: false,
            tps: 0.0,
            state: None,
        };

        let shared = Arc::new(Mutex::new(Shared {
            input: SimInput::default(),
            snapshot: Some(initial),
        }));
        let running = Arc::new(AtomicBool::new(true));
        let paused = Arc::new(AtomicBool::new(false));

        let thread_shared = Arc::clone(&shared);
        let thread_running = Arc::clone(&running);
        let thread_paused = Arc::clone(&paused);
        let thread = thread::Builder::new()
            .name("physics".to_owned())
            .spawn(move || sim_thread(sim, state, thread_shared, thread_running, thread_paused))
            .expect("failed to spawn physics thread");

        Self {
            shared,
            running,
            paused,
            thread: Some(thread),
        }
    }

    pub fn frame(&self, input: SimInput) -> Option<Snapshot> {
        let mut shared = self.shared.lock().unwrap();
        shared.input = input;
        shared.snapshot.clone()
    }

    pub fn paused(&mut self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    pub fn setpaused(&mut self, paused: bool) {
        self.paused.store(paused, Ordering::Relaxed);
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for SimHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

fn sim_thread(
    mut sim: SimWorld,
    mut state: TickState,
    shared: Arc<Mutex<Shared>>,
    running: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
) {
    let tick_duration = Duration::from_secs_f32(SIM_TICK_SECONDS);
    let mut next_tick = Instant::now() + tick_duration;

    let mut tps_window_start = Instant::now();
    let mut ticks_in_window: u32 = 0;

    while running.load(Ordering::Relaxed) {
        let now = Instant::now();
        if now < next_tick {
            thread::sleep(next_tick - now);
            continue;
        }

        let input = shared.lock().unwrap().input;

        let prev_player_pos = sim.player.hitbox.pos;
        let prev_cam_pos = sim.cam.pos;
        let was_dead = state.dead;

        let is_paused = paused.load(Ordering::Relaxed);
        let dt = if is_paused { 0.0 } else { SIM_TICK_MILLIS };

        if !state.dead {
            Input::handle(&mut sim.player, Event::Jump(input.jump_held));
            Input::handle(&mut sim.player, Event::Horizontal(input.horizontal));
            phys::tick(&mut sim, dt);
        }
        tickloop(&mut sim, dt, &mut state);
        sim.update_cam();

        ticks_in_window += 1;
        let window_elapsed = tps_window_start.elapsed();
        if window_elapsed >= Duration::from_secs(1) {
            state.tps = ticks_in_window as f32 / window_elapsed.as_secs_f32();
            ticks_in_window = 0;
            tps_window_start = Instant::now();
        }

        let curr_player_pos = sim.player.hitbox.pos;
        let curr_cam_pos = sim.cam.pos;
        let respawned = was_dead && !state.dead;

        let snapshot = Snapshot {
            at: Instant::now(),
            prev_player_pos: if respawned { curr_player_pos } else { prev_player_pos },
            curr_player_pos,
            prev_cam_pos: if respawned { curr_cam_pos } else { prev_cam_pos },
            curr_cam_pos,
            particles: state
                .deathparticles
                .iter()
                .map(|particle| ParticleSnapshot {
                    color: particle.color,
                    prev_pos: particle.prev_pos,
                    pos: particle.pos,
                })
                .collect(),
            dead: state.dead,
            tps: state.tps,
            state: Some(state.clone()),
        };

        shared.lock().unwrap().snapshot = Some(snapshot);

        next_tick += tick_duration;
        let now = Instant::now();
        if next_tick <= now {
            next_tick = now + tick_duration;
        }
    }
}
