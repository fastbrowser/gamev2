use macroquad::{color::Color, shapes::draw_rectangle, text::{draw_text, measure_text}};

use crate::game::game::{RenderState, World};

fn drawentry(name: &str, value: &str, font_size: u16, y: f32) -> f32 {
    let text = name.to_owned()+": "+value;
    let dims = measure_text(&text, None, font_size, 1.0);
    draw_rectangle(0.0, y, dims.width+10.0, dims.height+10.0, Color {r: 0.0, g: 0.0, b: 0.0, a: 0.5});
    draw_text(text, 5.0, y+dims.offset_y+5.0, font_size.into(), Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 });
    dims.height+10.0
}

pub fn draw(state: &RenderState, world: &World) {
    let mut y = 0.0f32;
    y+=drawentry("TPS", &format!("{:.1}", state.tps), 20, y);
    y+=drawentry("FPS", &format!("{:.1}", state.fps), 20, y);
    y+=drawentry("Blocks", &format!("{:.1}", world.blocks.len()), 20, y);
    y+=drawentry("Spikes", &format!("{:.1}", world.spikes.len()), 20, y);
    y+=drawentry("Blocks rendered", &format!("{:.1}", state.rendered_blocks), 20, y);
    y+=drawentry("Spikes rendered", &format!("{:.1}", state.rendered_spikes), 20, y);
}