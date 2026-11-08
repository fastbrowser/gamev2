use crate::{
    game::{
        AIR_ACCEL, AIR_FRICTION, GRAVITY, GROUND_ACCEL, GROUND_FRICTION, JUMP_VELOCITY,
        MAX_FALL_SPEED, VERTICAL_DRAG, World,
    },
    hitbox::{Hitbox, YSide},
    vectors::Vector2f,
};

const MAX_PHYSICS_STEP: f32 = 1.0 / 240.0;

pub fn tick(world: &mut World, delta_time: f32) {
    let mut remaining = (delta_time / 1000.0).clamp(0.0, 0.1);
    let mut candidates: Vec<u32> = Vec::new();
    while remaining > 0.0 {
        let dt = remaining.min(MAX_PHYSICS_STEP);
        step(world, dt, &mut candidates);
        remaining -= dt;
    }
}

fn step(world: &mut World, dt: f32, candidates: &mut Vec<u32>) {
    let was_ground = world.player.onground;

    if world.player.input_x != 0.0 {
        let accel = if was_ground { GROUND_ACCEL } else { AIR_ACCEL };
        world.player.vel.x += world.player.input_x * accel * dt;
    }

    if world.player.jump_held && was_ground {
        world.player.vel.y = -JUMP_VELOCITY;
    }

    let old = world.player.hitbox;
    let new_x = old.pos.x + world.player.vel.x * dt;
    let new_y = old.pos.y + world.player.vel.y * dt;

    let sweep = Hitbox {
        pos: Vector2f::new(old.pos.x.min(new_x), old.pos.y.min(new_y)),
        size: Vector2f::new(
            old.size.x + (new_x - old.pos.x).abs(),
            old.size.y + (new_y - old.pos.y).abs(),
        ),
    };
    world.block_grid.query(&sweep, candidates);

    let x_hitbox = Hitbox {
        pos: Vector2f { x: new_x, y: old.pos.y },
        size: old.size,
    };

    let mut x_pos = new_x;
    let mut xcol: Option<f32> = None;
    for &index in candidates.iter() {
        let block = &world.blocks[index as usize];
        let (xc1, _) = block.hitbox.collision_clamp(&x_hitbox);
        if let Some((xc, _side)) = xc1 {
            match xcol {
                Some(xc2) if (xc - x_hitbox.pos.x).abs() >= (xc2 - x_hitbox.pos.x).abs() => {}
                _ => xcol = Some(xc),
            }
        }
    }

    if let Some(xc) = xcol {
        x_pos = xc;
        world.player.vel.x = 0.0;
    }
    world.player.hitbox.pos.x = x_pos;

    let y_hitbox = Hitbox {
        pos: Vector2f { x: x_pos, y: new_y },
        size: old.size,
    };

    world.player.onground = false;

    let mut y_pos = new_y;
    let mut ycol: Option<(f32, YSide)> = None;
    for &index in candidates.iter() {
        let block = &world.blocks[index as usize];
        let (_, yc1) = block.hitbox.collision_clamp(&y_hitbox);
        if let Some((yc, side)) = yc1 {
            match ycol {
                Some((yc2, _)) if (yc - y_hitbox.pos.y).abs() >= (yc2 - y_hitbox.pos.y).abs() => {}
                _ => ycol = Some((yc, side)),
            }
        }
    }

    if let Some((yc, side)) = ycol {
        y_pos = yc;
        world.player.vel.y = 0.0;
        if side == YSide::Bottom {
            world.player.onground = true;
        }
    }
    world.player.hitbox.pos.y = y_pos;

    world.player.vel.y =
        (world.player.vel.y + GRAVITY * dt) * (-VERTICAL_DRAG * dt).exp();
    if world.player.vel.y > MAX_FALL_SPEED {
        world.player.vel.y = MAX_FALL_SPEED;
    }

    let friction = if was_ground {
        GROUND_FRICTION
    } else {
        AIR_FRICTION
    };
    world.player.vel.x *= (-friction * dt).exp();
}
