use gamev2::game::{game::{Block, Spike}, vectors::{Vector2f, Vector2i}};
use gamev2::tex::Textures;
use gamev2::worlds;
use macroquad::prelude::*;
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageLevel};

pub struct Camera {
    pub pos: Vector2f,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool {
    SelectDrag,
    DrawBlock,
    DrawSpike,
    Erase,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectedKind {
    Block,
    Spike,
}

#[derive(Clone, Copy)]
struct ObjectRef {
    kind: SelectedKind,
    index: usize,
}

#[derive(Clone, Default)]
struct Selection {
    blocks: Vec<usize>,
    spikes: Vec<usize>,
}

impl Selection {
    fn is_empty(&self) -> bool {
        self.blocks.is_empty() && self.spikes.is_empty()
    }
    fn clear(&mut self) {
        self.blocks.clear();
        self.spikes.clear();
    }
    fn contains(&self, object: ObjectRef) -> bool {
        match object.kind {
            SelectedKind::Block => self.blocks.contains(&object.index),
            SelectedKind::Spike => self.spikes.contains(&object.index),
        }
    }
    fn set_single(&mut self, object: ObjectRef) {
        self.clear();
        match object.kind {
            SelectedKind::Block => self.blocks.push(object.index),
            SelectedKind::Spike => self.spikes.push(object.index),
        }
    }
}

#[derive(Clone, Copy)]
struct DragItem {
    object: ObjectRef,
    start_draw_pos: Vector2f,
    start_hit_pos: Vector2f,
}

struct DragState {
    items: Vec<DragItem>,
    start_cell: Vector2i,
}

#[derive(Clone, Copy)]
struct Marquee {
    start_world: Vector2f,
    current_world: Vector2f,
}

pub struct Canvas {
    pub blocks: Vec<Block>,
    pub spikes: Vec<Spike>,
    pub cam: Camera,
    pub zoom: f32,
    pub windowsize: Vector2f,
    pub textures: Textures,
}

impl Canvas {
    async fn new(textures: Textures, blocks: Vec<Block>, spikes: Vec<Spike>) -> Self {
        Self { blocks: blocks, spikes: spikes, cam: Camera { pos: Vector2f::new(0.0, 0.0) }, zoom: 30.0, windowsize: Vector2f::new(800.0, 600.0), textures: textures }
    }
    fn block_texture(&self) -> &Texture2D {
        self.textures.block.as_ref().expect("editor needs a block texture")
    }
    fn spike_texture(&self) -> &Texture2D {
        self.textures.spike.as_ref().expect("editor needs a spike texture")
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
            draw_texture_ex(self.block_texture(), drawpos.x, drawpos.y, WHITE, DrawTextureParams { dest_size: Some(Vec2::new( block.hitbox.size.x as f32 * self.zoom, block.hitbox.size.y as f32 * self.zoom, )), ..Default::default() });
        }
        for spike in &self.spikes {
            let drawpos = self.glob2rel(spike.drawbox.pos.mul(self.zoom, self.zoom));
            if drawpos.x < -self.zoom || drawpos.y < -self.zoom || drawpos.x > self.windowsize.x+self.zoom || drawpos.y > self.windowsize.y+self.zoom {continue;}
            draw_texture_ex(self.spike_texture(), drawpos.x, drawpos.y, WHITE, DrawTextureParams { dest_size: Some(Vec2::new( spike.drawbox.size.x as f32 * self.zoom, spike.drawbox.size.y as f32 * self.zoom, )), ..Default::default() });
        }
        
        let half = self.windowsize.mul(0.5, 0.5);
        let gxi = (mx-half.x+self.cam.pos.x)/self.zoom;
        let gyi = (my-half.y+self.cam.pos.y)/self.zoom;
        let gx = gxi.floor() as f32 * self.zoom - self.cam.pos.x + half.x;
        let gy = gyi.floor() as f32 * self.zoom - self.cam.pos.y + half.y;
        draw_rectangle(gx, gy, self.zoom, self.zoom, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.3 });
        draw_rectangle( - self.cam.pos.x + half.x, -self.zoom - self.cam.pos.y + half.y, self.zoom, self.zoom, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.3 });
    }
    fn screen_to_world(&self, mx: f32, my: f32) -> Vector2f {
        let half = self.windowsize.mul(0.5, 0.5);
        Vector2f::new(
            (mx - half.x + self.cam.pos.x) / self.zoom,
            (my - half.y + self.cam.pos.y) / self.zoom,
        )
    }

    fn screen_to_cell(&self, mx: f32, my: f32) -> Vector2i {
        let world = self.screen_to_world(mx, my);
        Vector2i::new(world.x.floor() as i32, world.y.floor() as i32)
    }

    fn block_at_cell(&self, cell: Vector2i) -> Option<usize> {
        let pos = Vector2f::new(cell.x as f32, cell.y as f32);
        self.blocks
            .iter()
            .position(|b| b.hitbox.pos == pos && b.hitbox.size == Vector2f::ONE)
    }

    fn spike_at_cell(&self, cell: Vector2i) -> Option<usize> {
        let pos = Vector2f::new(cell.x as f32, cell.y as f32);
        self.spikes
            .iter()
            .position(|s| s.drawbox.pos == pos && s.drawbox.size == Vector2f::ONE)
    }

    fn object_at(&self, cell: Vector2i) -> Option<ObjectRef> {
        if let Some(index) = self.spike_at_cell(cell) {
            return Some(ObjectRef { kind: SelectedKind::Spike, index });
        }
        self.block_at_cell(cell)
            .map(|index| ObjectRef { kind: SelectedKind::Block, index })
    }

    fn set_selection_pos(&mut self, object: ObjectRef, draw_pos: Vector2f, hit_pos: Vector2f) {
        match object.kind {
            SelectedKind::Block => {
                if let Some(block) = self.blocks.get_mut(object.index) {
                    block.hitbox.pos = draw_pos;
                }
            }
            SelectedKind::Spike => {
                if let Some(spike) = self.spikes.get_mut(object.index) {
                    spike.drawbox.pos = draw_pos;
                    spike.hitbox.pos = hit_pos;
                }
            }
        }
    }

    fn selection_items(&self, selection: &Selection) -> Vec<DragItem> {
        let mut items = Vec::new();
        for &index in &selection.blocks {
            if let Some(block) = self.blocks.get(index) {
                items.push(DragItem {
                    object: ObjectRef { kind: SelectedKind::Block, index },
                    start_draw_pos: block.hitbox.pos,
                    start_hit_pos: block.hitbox.pos,
                });
            }
        }
        for &index in &selection.spikes {
            if let Some(spike) = self.spikes.get(index) {
                items.push(DragItem {
                    object: ObjectRef { kind: SelectedKind::Spike, index },
                    start_draw_pos: spike.drawbox.pos,
                    start_hit_pos: spike.hitbox.pos,
                });
            }
        }
        items
    }

    fn selection_bounds(&self, selection: &Selection) -> Option<(Vector2f, Vector2f)> {
        let mut min = Vector2f::new(f32::INFINITY, f32::INFINITY);
        let mut max = Vector2f::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut found = false;
        let mut include = |pos: Vector2f, size: Vector2f| {
            min.x = min.x.min(pos.x);
            min.y = min.y.min(pos.y);
            max.x = max.x.max(pos.x + size.x);
            max.y = max.y.max(pos.y + size.y);
            found = true;
        };
        for &index in &selection.blocks {
            if let Some(block) = self.blocks.get(index) {
                include(block.hitbox.pos, block.hitbox.size);
            }
        }
        for &index in &selection.spikes {
            if let Some(spike) = self.spikes.get(index) {
                include(spike.drawbox.pos, spike.drawbox.size);
            }
        }
        found.then_some((min, max))
    }

    fn select_region(&self, min: Vector2i, max: Vector2i) -> Selection {
        let in_region = |pos: Vector2f| {
            let x = pos.x.floor() as i32;
            let y = pos.y.floor() as i32;
            x >= min.x && x <= max.x && y >= min.y && y <= max.y
        };
        Selection {
            blocks: self.blocks.iter().enumerate()
                .filter(|(_, block)| in_region(block.hitbox.pos))
                .map(|(index, _)| index)
                .collect(),
            spikes: self.spikes.iter().enumerate()
                .filter(|(_, spike)| in_region(spike.drawbox.pos))
                .map(|(index, _)| index)
                .collect(),
        }
    }
}

fn merge_textures(base: &Textures, loaded: Textures) -> Textures {
    Textures {
        block: loaded.block.or_else(|| base.block.as_ref().map(|t| t.weak_clone())),
        spike: loaded.spike.or_else(|| base.spike.as_ref().map(|t| t.weak_clone())),
        player: loaded.player.or_else(|| base.player.as_ref().map(|t| t.weak_clone())),
    }
}

async fn choose_texture() -> Option<Texture2D> {
    let path = FileDialog::new()
        .set_title("Choose texture")
        .add_filter("Images", &["png"])
        .set_directory(std::env::current_dir().ok()?)
        .pick_file()?;
    match load_texture(path.to_str()?).await {
        Ok(texture) => {
            texture.set_filter(FilterMode::Nearest);
            Some(texture)
        }
        Err(e) => {
            MessageDialog::new()
                .set_title("Texture load error")
                .set_description(&format!("Could not load texture:\n{}\n\nError: {}", path.display(), e))
                .set_level(MessageLevel::Error)
                .set_buttons(MessageButtons::Ok)
                .show();
            None
        }
    }
}

fn cell_rect(a: Vector2f, b: Vector2f) -> (Vector2i, Vector2i) {
    let min_x = a.x.min(b.x).floor() as i32;
    let min_y = a.y.min(b.y).floor() as i32;
    let max_x = a.x.max(b.x).floor() as i32;
    let max_y = a.y.max(b.y).floor() as i32;
    (Vector2i::new(min_x, min_y), Vector2i::new(max_x, max_y))
}

fn sidebar_button(x: f32, y: f32, w: f32, h: f32, label: &str, active: bool, mp: (f32, f32)) -> bool {
    let hovered = mp.0 > x && mp.0 < x+w && mp.1 > y && mp.1 < y+h;
    const GROW: f32 = 3.0;
    let (bx, by, bw, bh) = if hovered {
        (x-GROW, y-GROW, w+GROW*2.0, h+GROW*2.0)
    } else {
        (x, y, w, h)
    };
    let color = if active {
        Color { r: 0.22, g: 0.52, b: 0.28, a: 1.0 }
    } else if hovered {
        Color { r: 0.45, g: 0.45, b: 0.45, a: 1.0 }
    } else {
        Color { r: 0.30, g: 0.30, b: 0.30, a: 1.0 }
    };
    draw_rectangle(bx, by, bw, bh, color);
    let font_size = 20u16;
    let dims = measure_text(label, None, font_size, 1.0);
    draw_text(label, bx+bw/2.0-dims.width/2.0, dims.offset_y+by+bh/2.0-dims.height/2.0, font_size as f32, WHITE);
    hovered && is_mouse_button_pressed(MouseButton::Left)
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
    let grasstex = Texture2D::from_file_with_format(include_bytes!("../../block.png"), None);
    grasstex.set_filter(FilterMode::Nearest);
    let spiketex = Texture2D::from_file_with_format(include_bytes!("../../spike.png"), None);
    spiketex.set_filter(FilterMode::Nearest);
    let playertex = Texture2D::from_file_with_format(include_bytes!("../../player.png"), None);
    playertex.set_filter(FilterMode::Nearest);
    let mut canvas = Canvas::new(
        Textures {
            block: Some(grasstex),
            spike: Some(spiketex),
            player: Some(playertex),
        },
        vec![Block::new(Vector2i::new(0,0), Vector2i::new(1,1), 0.6)],
        vec![Spike::new(Vector2i::new(0,-1), Vector2i::new(1,1))],
    ).await;
    let mut lastframepos = Vector2f::new(0.0, 0.0);
    let mut tool = Tool::DrawBlock;
    let mut selection = Selection::default();
    let mut drag: Option<DragState> = None;
    let mut marquee: Option<Marquee> = None;
    let penciltex = Texture2D::from_file_with_format(include_bytes!("../../pencil.png"), None);
    let backpenciltex = Texture2D::from_file_with_format(include_bytes!("../../backwardspencil.png"), None);
    penciltex.set_filter(FilterMode::Nearest);
    backpenciltex.set_filter(FilterMode::Nearest);
    loop {
        let sw = screen_width();
        let sh = screen_height();
        canvas.windowsize = Vector2f::new(sw, sh);
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
        if is_key_pressed(KeyCode::A) {
            if let Some(path) = FileDialog::new()
                .set_title("Open World file")
                .add_filter("World files", &["wl"])
                .set_directory(std::env::current_dir().expect("this shouldn't happen"))
                .pick_file() 
            {
                match worlds::loadblocks(path.to_str().expect("this shouldn't happen")) {
                    Ok((blocks, spikes, textures)) => {
                        let textures = merge_textures(&canvas.textures, textures);
                        canvas = Canvas::new(textures, blocks, spikes).await;
                        selection.clear();
                        drag = None;
                        marquee = None;
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
        if is_key_pressed(KeyCode::S) {
            if let Some(path) = FileDialog::new()
                .set_title("Open World file")
                .add_filter("World files", &["wl"])
                .set_directory(std::env::current_dir().expect("this shouldn't happen"))
                .save_file() 
            {
                match worlds::saveblocks(path.to_str().expect("this shouldn't happen"), &canvas.blocks, &canvas.spikes, &canvas.textures) {
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
        if is_key_pressed(KeyCode::D) {
            tool = match tool {
                Tool::SelectDrag => Tool::DrawBlock,
                Tool::DrawBlock => Tool::DrawSpike,
                Tool::DrawSpike => Tool::Erase,
                Tool::Erase => Tool::SelectDrag,
            };
            selection.clear();
            drag = None;
            marquee = None;
        }

        const SIDEBAR_W: f32 = 220.0;
        let sidebar_x = sw - SIDEBAR_W;
        let over_ui = mp.0 >= sidebar_x;

        if is_mouse_button_down(MouseButton::Right) && !over_ui {
            canvas.cam.pos = canvas.cam.pos.sub(mpdif.x, mpdif.y);
        }
        if is_mouse_button_down(MouseButton::Left) && !over_ui {
            let cell = canvas.screen_to_cell(mp.0, mp.1);
            match tool {
                Tool::DrawBlock => {
                    if canvas.block_at_cell(cell).is_none() {
                        canvas.blocks.push(
                            Block::new(cell, Vector2i::new(1, 1), 0.6),
                        );
                    }
                }
                Tool::DrawSpike => {
                    if canvas.spike_at_cell(cell).is_none() {
                        canvas.spikes.push(
                            Spike::new(cell, Vector2i::new(1, 1)),
                        );
                    }
                }
                Tool::Erase => {
                    let cell_pos = Vector2f::new(cell.x as f32, cell.y as f32);
                    canvas.blocks.retain(|b| {
                        !(b.hitbox.pos == cell_pos && b.hitbox.size == Vector2f::ONE)
                    });
                    canvas.spikes.retain(|s| {
                        !(s.drawbox.pos == cell_pos && s.drawbox.size == Vector2f::ONE)
                    });
                    selection.clear();
                    drag = None;
                    marquee = None;
                }
                Tool::SelectDrag => {}
            }
        }
        if tool == Tool::SelectDrag && !over_ui && is_mouse_button_pressed(MouseButton::Left) {
            let cell = canvas.screen_to_cell(mp.0, mp.1);
            let world = canvas.screen_to_world(mp.0, mp.1);
            drag = None;
            marquee = None;
            if let Some(object) = canvas.object_at(cell) {
                if !selection.contains(object) {
                    selection.set_single(object);
                }
                drag = Some(DragState {
                    items: canvas.selection_items(&selection),
                    start_cell: cell,
                });
            } else if !selection.is_empty()
                && canvas.selection_bounds(&selection)
                    .is_some_and(|(min, max)| world.x >= min.x && world.x <= max.x && world.y >= min.y && world.y <= max.y)
            {
                drag = Some(DragState {
                    items: canvas.selection_items(&selection),
                    start_cell: cell,
                });
            } else {
                selection.clear();
                marquee = Some(Marquee { start_world: world, current_world: world });
            }
        }

        if is_mouse_button_down(MouseButton::Left) {
            if let Some(m) = marquee.as_mut() {
                m.current_world = canvas.screen_to_world(mp.0, mp.1);
            }
            if let Some(d) = &drag {
                let cell = canvas.screen_to_cell(mp.0, mp.1);
                let offset = Vector2f::from(cell.subvec(d.start_cell));
                for item in &d.items {
                    canvas.set_selection_pos(
                        item.object,
                        item.start_draw_pos.addvec(offset),
                        item.start_hit_pos.addvec(offset),
                    );
                }
            }
        } else {
            if is_mouse_button_released(MouseButton::Left) {
                if let Some(m) = marquee.take() {
                    let (min, max) = cell_rect(m.start_world, m.current_world);
                    selection = canvas.select_region(min, max);
                }
            }
            drag = None;
        }

        canvas.draw(mp.0, mp.1);
        for &index in &selection.blocks {
            if let Some(block) = canvas.blocks.get(index) {
                let drawpos = canvas.glob2rel(block.hitbox.pos.mul(canvas.zoom, canvas.zoom));
                draw_rectangle_lines(drawpos.x, drawpos.y, block.hitbox.size.x * canvas.zoom, block.hitbox.size.y * canvas.zoom, 3.0, Color { r: 1.0, g: 0.55, b: 0.0, a: 1.0 });
            }
        }
        for &index in &selection.spikes {
            if let Some(spike) = canvas.spikes.get(index) {
                let drawpos = canvas.glob2rel(spike.drawbox.pos.mul(canvas.zoom, canvas.zoom));
                draw_rectangle_lines(drawpos.x, drawpos.y, spike.drawbox.size.x * canvas.zoom, spike.drawbox.size.y * canvas.zoom, 3.0, Color { r: 1.0, g: 0.55, b: 0.0, a: 1.0 });
            }
        }
        if let Some(m) = marquee {
            let a = canvas.glob2rel(m.start_world.mul(canvas.zoom, canvas.zoom));
            let b = canvas.glob2rel(m.current_world.mul(canvas.zoom, canvas.zoom));
            let x = a.x.min(b.x);
            let y = a.y.min(b.y);
            let w = (a.x - b.x).abs();
            let h = (a.y - b.y).abs();
            draw_rectangle(x, y, w, h, Color { r: 0.15, g: 0.55, b: 1.0, a: 0.2 });
            draw_rectangle_lines(x, y, w, h, 2.0, Color { r: 0.15, g: 0.55, b: 1.0, a: 1.0 });
        }

        draw_rectangle(sidebar_x, 0.0, SIDEBAR_W, sh, Color { r: 0.12, g: 0.12, b: 0.12, a: 1.0 });
        draw_text("Tools", sidebar_x+16.0, 36.0, 28.0, WHITE);
        let margin = 14.0;
        let bw = SIDEBAR_W - margin*2.0;
        let bh = 44.0;
        let mut by = 56.0;
        let mut new_tool = tool;
        if sidebar_button(sidebar_x+margin, by, bw, bh, "Select + Drag", tool == Tool::SelectDrag, mp) {
            new_tool = Tool::SelectDrag;
        }
        by += bh + 8.0;
        if sidebar_button(sidebar_x+margin, by, bw, bh, "Draw Block", tool == Tool::DrawBlock, mp) {
            new_tool = Tool::DrawBlock;
        }
        by += bh + 8.0;
        if sidebar_button(sidebar_x+margin, by, bw, bh, "Draw Spike", tool == Tool::DrawSpike, mp) {
            new_tool = Tool::DrawSpike;
        }
        by += bh + 8.0;
        if sidebar_button(sidebar_x+margin, by, bw, bh, "Erase", tool == Tool::Erase, mp) {
            new_tool = Tool::Erase;
        }
        by += bh + 24.0;

        draw_text("Textures", sidebar_x+16.0, by, 24.0, WHITE);
        by += 16.0;
        if sidebar_button(sidebar_x+margin, by, bw, bh, "Block Texture", false, mp) {
            if let Some(texture) = choose_texture().await {
                canvas.textures.block = Some(texture);
            }
        }
        by += bh + 8.0;
        if sidebar_button(sidebar_x+margin, by, bw, bh, "Spike Texture", false, mp) {
            if let Some(texture) = choose_texture().await {
                canvas.textures.spike = Some(texture);
            }
        }
        by += bh + 8.0;
        if sidebar_button(sidebar_x+margin, by, bw, bh, "Player Texture", false, mp) {
            if let Some(texture) = choose_texture().await {
                canvas.textures.player = Some(texture);
            }
        }

        if new_tool != tool {
            tool = new_tool;
            selection.clear();
            drag = None;
            marquee = None;
        }

        if tool == Tool::SelectDrag {
            draw_rectangle(mp.0, mp.1-14.0, 12.0, 12.0, Color { r: 0.1, g: 0.1, b: 0.1, a: 0.7 });
            draw_rectangle_lines(mp.0, mp.1-14.0, 12.0, 12.0, 2.0, WHITE);
        } else if tool == Tool::Erase {
            draw_texture_ex(&backpenciltex, mp.0, mp.1-16.0, WHITE, DrawTextureParams { dest_size: Some(Vec2::new( 30.0, 30.0)), ..Default::default() });
        } else {
            draw_texture_ex(&penciltex, mp.0, mp.1-16.0, WHITE, DrawTextureParams { dest_size: Some(Vec2::new( 30.0, 30.0)), ..Default::default() });
            draw_texture_ex(match tool {Tool::DrawBlock=>canvas.block_texture(),Tool::DrawSpike=>canvas.spike_texture(), _=>canvas.block_texture()}, mp.0, mp.1-16.0, WHITE, DrawTextureParams { dest_size: Some(Vec2::new( 10.0, 10.0)), ..Default::default() });
        }
        draw_text("RMB+Drag=move canvas LMB=use tool D=cycle tools", 10.0, 25.0, 22.0, BLACK);
        draw_text("S=Save A=Load | Select: drag empty area to box-select", 10.0, 50.0, 22.0, BLACK);
        next_frame().await;
    }
}
