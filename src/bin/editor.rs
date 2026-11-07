use gamev2::{game::Block, vectors::{Vector2f, Vector2i}, worlds};
use macroquad::prelude::*;
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageLevel};

struct Camera {
    pub pos: Vector2f,
}

pub struct Canvas {
    pub blocks: Vec<Block>,
    pub cam: Camera,
    pub zoom: f32,
    pub windowsize: Vector2f,
    pub blocktexture: Texture2D,
}

impl Canvas {
    async fn new(tex: Texture2D, blocks: Vec<Block>) -> Self {
        Self { blocks: blocks, cam: Camera { pos: Vector2f::new(0.0, 0.0) }, zoom: 30.0, windowsize: Vector2f::new(800.0, 600.0), blocktexture: tex}
    }
    fn glob2rel(&self, pos: Vector2f) -> Vector2f {
        pos.subvec(self.cam.pos).add((self.windowsize.x/2.0) as f32, (self.windowsize.y/2.0) as f32)
    }
    pub fn draw_grid(&self) {
        let half = self.windowsize.mul(0.5, 0.5);

        let start_x = ((self.cam.pos.x - half.x) / self.zoom).floor() as i32;
        let end_x = ((self.cam.pos.x + half.x) / self.zoom).ceil() as i32;
        for x in start_x..=end_x {
            let xpos = x as f32 * self.zoom - self.cam.pos.x + half.x;
            draw_line(xpos, 0.0, xpos, self.windowsize.y, 1.0, BLACK);
        }

        let start_y = ((self.cam.pos.y - half.y) / self.zoom).floor() as i32;
        let end_y = ((self.cam.pos.y + half.y) / self.zoom).ceil() as i32;
        for y in start_y..=end_y {
            let ypos = y as f32 * self.zoom - self.cam.pos.y + half.y;
            draw_line(0.0, ypos, self.windowsize.x, ypos, 1.0, BLACK);
        }
    }
    pub fn draw(&self, mx: f32, my: f32) {
        clear_background(WHITE);
        self.draw_grid();
        for block in &self.blocks {
            let drawpos = self.glob2rel(block.hitbox.pos.mul(self.zoom, self.zoom));
            if drawpos.x < -self.zoom || drawpos.y < -self.zoom || drawpos.x > self.windowsize.x+self.zoom || drawpos.y > self.windowsize.y+self.zoom {continue;}
            draw_texture_ex(&block.texture, drawpos.x, drawpos.y, WHITE, DrawTextureParams { dest_size: Some(Vec2::new( block.hitbox.size.x as f32 * self.zoom, block.hitbox.size.y as f32 * self.zoom, )), ..Default::default() });
        }
        let half = self.windowsize.mul(0.5, 0.5);
        let gxi = (mx-half.x+self.cam.pos.x)/self.zoom;
        let gyi = (my-half.y+self.cam.pos.y)/self.zoom;
        let gx = gxi.floor() as f32 * self.zoom - self.cam.pos.x + half.x;
        let gy = gyi.floor() as f32 * self.zoom - self.cam.pos.y + half.y;
        draw_rectangle(gx, gy, self.zoom, self.zoom, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.3 });
    }
    pub fn getblockat(&self, pos: Vector2f) -> Option<Block> {
        for block in &self.blocks {
            if block.hitbox.contains_point(pos) {
                return Some(block.clone());
            }
        }
        None
    }
    fn screen_to_world(&self, mx: f32, my: f32) -> Vector2f {
        let half = self.windowsize.mul(0.5, 0.5);
        Vector2f::new(
            (mx - half.x + self.cam.pos.x) / self.zoom,
            (my - half.y + self.cam.pos.y) / self.zoom,
        )
    }

    fn block_at_cell(&self, cell: Vector2i) -> Option<usize> {
        let pos = Vector2f::new(cell.x as f32, cell.y as f32);
        self.blocks
            .iter()
            .position(|b| b.hitbox.pos == pos && b.hitbox.size == Vector2f::ONE)
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

#[macroquad::main(window_conf)]
async fn main() {
    let tex = Texture2D::from_file_with_format(include_bytes!("../../grass.png"), None);
    tex.set_filter(FilterMode::Nearest);
    let spiketex = Texture2D::from_file_with_format(include_bytes!("../../spike.png"), None);
    spiketex.set_filter(FilterMode::Nearest);
    let mut canvas = Canvas::new(tex.clone(), vec![Block::new(Vector2i::new(0,0), Vector2i::new(1,1), 0.6, tex.clone())]).await;
    let mut lastframepos = Vector2f::new(0.0, 0.0);
    let mut erasing = false;
    let penciltex = Texture2D::from_file_with_format(include_bytes!("../../pencil.png"), None);
    let backpenciltex = Texture2D::from_file_with_format(include_bytes!("../../backwardspencil.png"), None);
    penciltex.set_filter(FilterMode::Nearest);
    backpenciltex.set_filter(FilterMode::Nearest);
    loop {
        canvas.windowsize = Vector2f::new(screen_width(), screen_height());
        let mp = mouse_position();
        let mpv = Vector2f::new(mp.0, mp.1);
        let mpdif = mpv.subvec(lastframepos);
        lastframepos = mpv;
        let wheel = mouse_wheel().1.clamp(-1.0, 1.0);
        if wheel != 0.0 {
            let old_zoom = canvas.zoom;
            let new_zoom = (old_zoom * (1.0 + wheel * 0.1)).clamp(4.0, 200.0);
            if new_zoom != old_zoom {
                let half = canvas.windowsize.mul(0.5, 0.5);
                let canvas_under_cursor = mpv
                    .subvec(half)
                    .addvec(canvas.cam.pos)
                    .mul(1.0 / old_zoom, 1.0 / old_zoom);
                canvas.zoom = new_zoom;
                canvas.cam.pos = canvas_under_cursor
                    .mul(new_zoom, new_zoom)
                    .addvec(half)
                    .subvec(mpv);
            }
        }
        if is_key_down(KeyCode::A) {
            if let Some(path) = FileDialog::new()
                .set_title("Open World file")
                .add_filter("World files", &["wl"])
                .set_directory(std::env::current_dir().expect("this shouldn't happen"))
                .pick_file() 
            {
                match worlds::loadblocks(path.to_str().expect("this shouldn't happen"), tex.clone(), spiketex.clone()) {
                    Ok(blocks) => {
                        canvas = Canvas::new(tex.clone(), blocks.0).await;
                    }
                    Err(e) => {
                        MessageDialog::new()
                            .set_title("World load error")
                            .set_description(&format!("Could not load world:\n{}\n\nError: {}", path.display(), e))
                            .set_level(MessageLevel::Error)
                            .set_buttons(MessageButtons::Ok)
                            .show();
                    }
                }

            }
        }
        if is_key_down(KeyCode::S) {
            if let Some(path) = FileDialog::new()
                .set_title("Open World file")
                .add_filter("World files", &["wl"])
                .set_directory(std::env::current_dir().expect("this shouldn't happen"))
                .save_file() 
            {
                match worlds::saveblocks(path.to_str().expect("this shouldn't happen"), canvas.blocks.clone(), vec![]) {
                    Ok(_) => {}
                    Err(e) => {
                        MessageDialog::new()
                            .set_title("World save error")
                            .set_description(&format!("Could not save world:\n{}\n\nError: {}", path.display(), e))
                            .set_level(MessageLevel::Error)
                            .set_buttons(MessageButtons::Ok)
                            .show();
                    }
                }

            }
        }
        if is_mouse_button_down(MouseButton::Right) {
            canvas.cam.pos = canvas.cam.pos.sub(mpdif.x, mpdif.y);
        }
        if is_mouse_button_pressed(MouseButton::Middle) {
            erasing = !erasing;
        }
        if is_mouse_button_down(MouseButton::Left) {
            let world = canvas.screen_to_world(mp.0, mp.1);
            let cell = Vector2i::new(
                world.x.floor() as i32,
                world.y.floor() as i32,
            );
            let cell_pos = Vector2f::new(cell.x as f32, cell.y as f32);

            if erasing {
                canvas.blocks.retain(|b| {
                    !(b.hitbox.pos == cell_pos && b.hitbox.size == Vector2f::ONE)
                });
            } else if canvas.block_at_cell(cell).is_none() {
                canvas.blocks.push(
                    Block::new(cell, Vector2i::new(1, 1), 0.6, tex.clone()),
                );
            }
        }
        canvas.draw(mp.0, mp.1);
        draw_texture_ex(if erasing {&backpenciltex} else {&penciltex}, mp.0, mp.1-16.0, WHITE, DrawTextureParams { dest_size: Some(Vec2::new( 30.0, 30.0)), ..Default::default() });
        draw_text("RMB+Drag=move canvas LMB=draw MIDDLE=erase/draw toggle", 10.0, 25.0, 30.0, BLACK);
        draw_text("S=Save A=Load", 10.0, 55.0, 30.0, BLACK);
        next_frame().await;
    }
}