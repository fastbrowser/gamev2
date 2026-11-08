use std::{collections::HashSet, fs::File, io::{self, BufWriter, Error, Read, Write}};

use macroquad::texture::{FilterMode, Texture2D, load_texture};

use crate::game::{game::{Block, Cam, Player, Spike, World}, vectors::{Vector2f, Vector2i}};

pub fn saveblocks(path: &str, blocks: &Vec<Block>, spikes: &Vec<Spike>) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    for block in blocks {
        writer.write(&[0]);
        writer.write_all(&(block.hitbox.pos.x as i32).to_le_bytes())?;
        writer.write_all(&(block.hitbox.pos.y as i32).to_le_bytes())?;
    }
    for spike in spikes {
        writer.write(&[1]);
        writer.write_all(&(spike.drawbox.pos.x as i32).to_le_bytes())?;
        writer.write_all(&(spike.drawbox.pos.y as i32).to_le_bytes())?;
    }
    writer.flush()?;
    Ok(())
}

pub fn save(path: &str, world: &World) -> io::Result<()> {
    saveblocks(path, &world.blocks, &world.spikes)
}

pub fn loadblocks(
    path: &str,
) -> io::Result<(Vec<Block>, Vec<Spike>)> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let mut blocks: Vec<Block> = vec![];
    let mut spikes: Vec<Spike> = vec![];
    let mut seen: HashSet<(u8, i32, i32)> = HashSet::new();
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
        if !seen.insert((btype, x, y)) {
            continue;
        }
        match btype {
            0 => {
                blocks.push(
                    Block::new(Vector2i::new(x, y), Vector2i::new(1, 1), 0.6),
                );
            }
            1 => {
                spikes.push(
                    Spike::new(Vector2i::new(x, y), Vector2i::new(1, 1)),
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

pub fn load(path: &str) -> io::Result<World> {
    let bs = loadblocks(path)?;
    Ok(World::new(
        Player::new(),
        bs.0,
        bs.1,
        Vector2f::new(800.0, 600.0),
        Cam::default(),
    ))
}

pub fn getworld1() -> World {    
    let mut bytes = include_bytes!("../world.wl");
    let mut blocks: Vec<Block> = vec![];
    let mut spikes: Vec<Spike> = vec![];
    let mut seen: HashSet<(u8, i32, i32)> = HashSet::new();
    for chunk in bytes.chunks_exact(9) {
        let btype = chunk[0];
        let x = i32::from_le_bytes(chunk[1..5].try_into().unwrap());
        let y = i32::from_le_bytes(chunk[5..9].try_into().unwrap());
        if !seen.insert((btype, x, y)) {
            continue;
        }
        match btype {
            0 => {
                blocks.push(
                    Block::new(Vector2i::new(x, y), Vector2i::new(1, 1), 0.6),
                );
            }
            1 => {
                spikes.push(
                    Spike::new(Vector2i::new(x, y), Vector2i::new(1, 1)),
                );
            }
            _=> {}
        }
    }
    World::new(
        Player::new(),
        blocks,
        spikes,
        Vector2f::new(800.0, 600.0),
        Cam::default(),
    )
}