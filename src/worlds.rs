use std::{fs::File, io::{self, BufWriter, Read, Write}};

use macroquad::texture::{FilterMode, Texture2D, load_texture};

use crate::{game::{Block, Cam, Player, Spike, World}, vectors::{Vector2f, Vector2i}};

pub fn saveblocks(path: &str, blocks: Vec<Block>, spikes: Vec<Spike>) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    for block in & blocks {
        writer.write_all(&(block.hitbox.pos.x as i32).to_le_bytes())?;
        writer.write_all(&(block.hitbox.pos.y as i32).to_le_bytes())?;
    }
    writer.write(&[b'\n']);
    for spike in &spikes {
        writer.write_all(&(spike.hitbox.pos.x as i32).to_le_bytes())?;
        writer.write_all(&(spike.hitbox.pos.y as i32).to_le_bytes())?;
    }
    writer.flush()?;
    Ok(())
}

pub fn save(path: &str, world: &World) -> io::Result<()> {
    saveblocks(path, world.blocks.clone(), world.spikes.clone())
}

pub fn loadblocks(
    path: &str,
    block_tex: Texture2D,
    spike_tex: Texture2D,
) -> io::Result<(Vec<Block>, Vec<Spike>)> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let mut parts = buffer.splitn(2, |&b| b == b'\n');
    let block_bytes = parts.next().unwrap_or(&[]);
    let spike_bytes = parts.next().unwrap_or(&[]);
    let parse_section = |bytes: &[u8], tex: &Texture2D| -> io::Result<Vec<Block>> {
        if bytes.len() % 8 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid world file section size (must be divisible by 8)",
            ));
        }
        let mut blocks = Vec::new();
        for chunk in bytes.chunks_exact(8) {
            let x = i32::from_le_bytes(chunk[0..4].try_into().unwrap());
            let y = i32::from_le_bytes(chunk[4..8].try_into().unwrap());
            blocks.push(
                Block::new(Vector2i::new(x, y), Vector2i::new(1, 1), 0.6, tex.clone()),
            );
        }
        Ok(blocks)
    };
    let blocks = parse_section(block_bytes, &block_tex)?;
    let spikes: Vec<Spike> = parse_section(spike_bytes, &spike_tex)?.into_iter().map(|b| Spike::new(Vector2i::from(b.hitbox.pos), Vector2i::from(b.hitbox.size), b.texture)).collect();
    Ok((blocks, spikes))
}

pub fn load(path: &str, tex: Texture2D, playertex: Texture2D, spiketex: Texture2D) -> io::Result<World> {
    let bs = loadblocks(path, tex, spiketex)?;
    Ok(World { player: Player::new(playertex), blocks: bs.0, spikes: bs.1, windowsize: Vector2f::new(800.0, 600.0), cam: Cam::default()})
}

pub fn getworld1(tex: Texture2D, playertex: Texture2D, spiketex: Texture2D) -> World {    
    World { player: Player::new(playertex.clone()), blocks: vec![
        Block::new(Vector2i::new(-10, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(-9, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(-8, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(-7, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(-6, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(-5, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(-4, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(-3, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(-2, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(-1, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(0, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(1, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(2, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(3, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(4, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(5, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(6, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
        Block::new(Vector2i::new(7, 0), Vector2i::new(1, 1), 0.6, tex.clone()),
    ], spikes: vec![
        Spike::new(Vector2i::new(-10, -1), Vector2i::new(1, 1), spiketex.clone())
    ], windowsize: Vector2f::new(800.0, 600.0), cam: Cam::default()}
}