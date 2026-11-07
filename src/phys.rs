use crate::{game::{GRAVITY, TICK_LEN_MILLIS_NORMAL, World}, hitbox::{Hitbox, YSide}, vectors::Vector2f};

pub fn tick(world: &mut World, delta_time: f32) {
    let time_scale = delta_time / TICK_LEN_MILLIS_NORMAL;
    let constant_multiplier = time_scale * time_scale;
    world.player.vel.y += GRAVITY*constant_multiplier;

    let mut x_pos = world.player.hitbox.pos.x + world.player.vel.x;
    let x_hitbox = Hitbox {
        pos: Vector2f { x: x_pos, y: world.player.hitbox.pos.y },
        size: world.player.hitbox.size,
    };

    let mut xcol: Option<f32> = None;
    for block in &world.blocks {
        let (xc1, _) = block.hitbox.collision_clamp(&x_hitbox);
        if let Some((xc, _side)) = xc1 {
            if let Some(xc2) = xcol {
                if (xc - x_hitbox.pos.x).abs() < (xc2 - x_hitbox.pos.x).abs() {
                    xcol = Some(xc);
                }
            } else {
                xcol = Some(xc);
            }
        }
    }

    if let Some(xc) = xcol {
        x_pos = xc;
        world.player.vel.x = 0.0;
    }
    world.player.hitbox.pos.x = x_pos;

    let mut y_pos = world.player.hitbox.pos.y + world.player.vel.y;
    let y_hitbox = Hitbox {
        pos: Vector2f { x: world.player.hitbox.pos.x, y: y_pos },
        size: world.player.hitbox.size,
    };

    world.player.onground = false;

    let mut ycol: Option<(f32, YSide)> = None;
    for block in &world.blocks {
        let (_, yc1) = block.hitbox.collision_clamp(&y_hitbox);
        if let Some((yc, side)) = yc1 {
            if let Some((yc2, _)) = ycol {
                if (yc - y_hitbox.pos.y).abs() < (yc2 - y_hitbox.pos.y).abs() {
                    ycol = Some((yc, side));
                }
            } else {
                ycol = Some((yc, side));
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
    if world.player.onground {
        if let Some(block) = world.getblockat(world.player.hitbox.pos.add(0.0, 1.1)) {
            world.player.vel.x*=0.91*constant_multiplier*block.slip;
        }
    }
    world.player.vel = world.player.vel.mul(0.98, 0.98);
}