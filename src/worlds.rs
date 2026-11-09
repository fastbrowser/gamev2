use std::{collections::HashSet, fs::File, io::{self, BufWriter, Error, Read, Write}};

use macroquad::texture::{FilterMode, Image, Texture2D, load_texture};

use crate::{game::{game::{Block, Cam, Player, Spike, World}, vectors::{Vector2f, Vector2i}}, tex::Textures};

pub fn saveblocks(path: &str, blocks: &Vec<Block>, spikes: &Vec<Spike>, textures: &Textures) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    
    for block in blocks {
        writer.write_all(&[0])?;
        writer.write_all(&(block.hitbox.pos.x as i32).to_le_bytes())?;
        writer.write_all(&(block.hitbox.pos.y as i32).to_le_bytes())?;
    }
    
    for spike in spikes {
        writer.write_all(&[1])?;
        writer.write_all(&(spike.drawbox.pos.x as i32).to_le_bytes())?;
        writer.write_all(&(spike.drawbox.pos.y as i32).to_le_bytes())?;
    }
    
    writer.write_all(&[2u8])?; // 2 = textures section marker
    writer.write_all(&(3u64).to_le_bytes())?; // n textures = 3
    
    // 0 = block texture
    let b_tex = textures.block.get_texture_data();
    writer.write_all(&[0u8])?;
    writer.write_all(&b_tex.width.to_le_bytes())?;
    writer.write_all(&b_tex.height.to_le_bytes())?;
    writer.write_all(&(b_tex.bytes.len() as u64).to_le_bytes())?;
    writer.write_all(&b_tex.bytes)?;
    
    // 1 = spike texture
    let s_tex = textures.spike.get_texture_data();
    writer.write_all(&[1u8])?;
    writer.write_all(&s_tex.width.to_le_bytes())?;
    writer.write_all(&s_tex.height.to_le_bytes())?;
    writer.write_all(&(s_tex.bytes.len() as u64).to_le_bytes())?;
    writer.write_all(&s_tex.bytes)?;
    
    // 2 = player texture
    let p_tex = textures.player.get_texture_data();
    writer.write_all(&[2u8])?;
    writer.write_all(&p_tex.width.to_le_bytes())?;
    writer.write_all(&p_tex.height.to_le_bytes())?;
    writer.write_all(&(p_tex.bytes.len() as u64).to_le_bytes())?;
    writer.write_all(&p_tex.bytes)?;
    
    writer.flush()?;
    Ok(())
}

pub fn save(path: &str, world: &World, textures: &Textures) -> io::Result<()> {
    saveblocks(path, &world.blocks, &world.spikes, textures)
}

fn fread(file: &mut File, b: u64) -> io::Result<Vec<u8>> {
    let mut cl = vec![0; b as usize];
    file.read_exact(&mut cl)?;
    Ok(cl)
}

fn fread_exact<const N: usize>(file: &mut File) -> io::Result<[u8; N]> {
    let mut buf = [0; N];
    file.read_exact(&mut buf)?;
    Ok(buf)
}

pub fn loadblocks(path: &str) -> io::Result<(Vec<Block>, Vec<Spike>, Textures)> {
    let mut file = File::open(path)?;
    let mut blocks: Vec<Block> = vec![];
    let mut spikes: Vec<Spike> = vec![];
    let mut seen: HashSet<(u8, i32, i32)> = HashSet::new();
    let mut textures = Textures {
        block: Texture2D::empty(),
        spike: Texture2D::empty(),
        player: Texture2D::empty(),
    };
    
    let total = file.metadata()?.len();
    let mut read = 0u64;
    let mut hs2 = false;

    while read < total {
        let tag_buf = fread(&mut file, 1)?;
        if tag_buf.is_empty() {
            break;
        }
        read += 1;

        match tag_buf[0] {
            0 => {
                let x = i32::from_le_bytes(fread_exact(&mut file)?);
                let y = i32::from_le_bytes(fread_exact(&mut file)?);
                read += 8; // 4 bytes for x + 4 bytes for y
                
                if seen.insert((0, x, y)) {
                    blocks.push(Block::new(Vector2i::new(x, y), Vector2i::new(1, 1), 0.6));
                }
            }
            1 => {
                let x = i32::from_le_bytes(fread_exact(&mut file)?);
                let y = i32::from_le_bytes(fread_exact(&mut file)?);
                read += 8; // 4 bytes for x + 4 bytes for y
                
                if seen.insert((1, x, y)) {
                    spikes.push(Spike::new(Vector2i::new(x, y), Vector2i::new(1, 1)));
                }
            }
            2 => {
                if hs2 { 
                    // If we encounter section 2 again or skip remaining, handle gracefully
                    break; 
                }
                hs2 = true;
                read += 8; // ntex u64
                let ntex = u64::from_le_bytes(fread_exact(&mut file)?);

                for _ in 0..ntex {
                    let t = fread(&mut file, 1)?[0];
                    read += 1;
                    
                    let w = u16::from_le_bytes(fread_exact(&mut file)?);
                    read += 2;
                    
                    let h = u16::from_le_bytes(fread_exact(&mut file)?);
                    read += 2;
                    
                    let tlen = u64::from_le_bytes(fread_exact(&mut file)?);
                    read += 8;
                    
                    let tdata = fread(&mut file, tlen)?;
                    read += tlen;

                    match t {
                        0 => { textures.block = Texture2D::from_rgba8(w, h, &tdata); }
                        1 => { textures.spike = Texture2D::from_rgba8(w, h, &tdata); }
                        2 => { textures.player = Texture2D::from_rgba8(w, h, &tdata); }
                        _ => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Invalid texture type found",
                            ));
                        }
                    }
                }
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid block type found",
                ));
            }
        }
    }
    Ok((blocks, spikes, textures))
}

pub fn load(path: &str) -> io::Result<(World, Textures)> {
    let bs = loadblocks(path)?;
    Ok((World::new(
        Player::new(),
        bs.0,
        bs.1,
        Vector2f::new(800.0, 600.0),
        Cam::default(),
    ),bs.2))
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