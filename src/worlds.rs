use std::{fs::File, io::{self, BufWriter, Error, Read, Write}};

use macroquad::texture::{FilterMode, Texture2D, load_texture};

use crate::{game::{Block, Cam, Player, Spike, World}, vectors::{Vector2f, Vector2i}};

pub fn saveblocks(path: &str, blocks: Vec<Block>, spikes: Vec<Spike>) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    for block in & blocks {
        writer.write(&[0]);
        writer.write_all(&(block.hitbox.pos.x as i32).to_le_bytes())?;
        writer.write_all(&(block.hitbox.pos.y as i32).to_le_bytes())?;
    }
    for spike in &spikes {
        writer.write(&[1]);
        writer.write_all(&(spike.drawbox.pos.x as i32).to_le_bytes())?;
        writer.write_all(&(spike.drawbox.pos.y as i32).to_le_bytes())?;
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
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let mut blocks: Vec<Block> = vec![];
    let mut spikes: Vec<Spike> = vec![];
    if bytes.len() % 9 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid world file size (size must be divisible by 9)",
        ));
    }
    for chunk in bytes.chunks_exact(9) {
        let btype = chunk[0];
        let x = i32::from_le_bytes(chunk[1..5].try_into().unwrap());
        let y = i32::from_le_bytes(chunk[5..9].try_into().unwrap());
        match btype {
            0 => {
                blocks.push(
                    Block::new(Vector2i::new(x, y), Vector2i::new(1, 1), 0.6, block_tex.clone()),
                );
            }
            1 => {
                spikes.push(
                    Spike::new(Vector2i::new(x, y), Vector2i::new(1, 1), spike_tex.clone()),
                );
            }
            _=> {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid block type found (likely generated from newer engine)",
                ));
            }
        }
    }
    Ok((blocks, spikes))
}

pub fn load(path: &str, tex: Texture2D, playertex: Texture2D, spiketex: Texture2D) -> io::Result<World> {
    let bs = loadblocks(path, tex, spiketex)?;
    Ok(World::new(
        Player::new(playertex),
        bs.0,
        bs.1,
        Vector2f::new(800.0, 600.0),
        Cam::default(),
    ))
}

pub fn getworld1(tex: Texture2D, playertex: Texture2D, spiketex: Texture2D) -> World {    
    let mut bytes = include_bytes!("../world.wl");
    let mut blocks: Vec<Block> = vec![];
    let mut spikes: Vec<Spike> = vec![];
    for chunk in bytes.chunks_exact(9) {
        let btype = chunk[0];
        let x = i32::from_le_bytes(chunk[1..5].try_into().unwrap());
        let y = i32::from_le_bytes(chunk[5..9].try_into().unwrap());
        match btype {
            0 => {
                blocks.push(
                    Block::new(Vector2i::new(x, y), Vector2i::new(1, 1), 0.6, tex.clone()),
                );
            }
            1 => {
                spikes.push(
                    Spike::new(Vector2i::new(x, y), Vector2i::new(1, 1), spiketex.clone()),
                );
            }
            _=> {}
        }
    }
    World::new(
        Player::new(playertex),
        blocks,
        spikes,
        Vector2f::new(800.0, 600.0),
        Cam::default(),
    )
}