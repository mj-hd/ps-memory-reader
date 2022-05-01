use crate::io::Io;
use anyhow::Result;
use log::debug;
use num_derive::FromPrimitive;

pub struct MemoryCard {
    pub blocks: u8,
    pub size: u32,

    io: Io,
    //header: Header,
    //directory: Directory,
}

impl MemoryCard {
    pub fn new(io: Io) -> Result<Self> {
        //let header = MemoryCard::read_header(io)?;
        //let directory = MemoryCard::read_directory(io)?;

        Ok(Self {
            io,
            //header,
            //directory,
            blocks: 0,
            size: 0,
        })
    }

    //fn read_header(io: Io) -> Result<Header> {
    //    let data = io.read_sector(0)?;

    //    Ok(Header {
    //        id: std::str::from_utf8(&data[0..1]).unwrap_or("ERR"),
    //    })
    //}

    //fn read_directory(io: Io) -> Result<Directory> {
    //    for i in 1..16 {
    //        let data = io.read_sector(i)?;

    //        let allocation = Allocation::from_u8(&data[0]);
    //        // TODO
    //    }

    //    Ok(Directory {})
    //}

    //fn read_broken_sector_list(io: Io) -> Result<Vec<u16>> {
    //    Ok(vec![])
    //}

    //fn read_shift_jis_char_set(io: Io) -> Result<ShiftJisCharSet> {
    //    Ok(())
    //}

    pub fn read_block(&mut self, block: u8) -> Result<[u8; 64 * 128]> {
        let mut result = [0; 64 * 128];

        for i in 0..64usize {
            let data = self.io.read_sector((block as u16) * 64 + i as u16)?;

            result[i * 128..(i + 1) * 128].copy_from_slice(&data);
        }

        Ok(result)
    }
}

#[derive(Debug)]
struct Header {
    id: String,
}

#[derive(Debug)]
struct Directory {
    allocation: Allocation,
    file_size: u64,
    next_block: u8,
    file_name: String,
}

#[derive(Debug, Clone, Copy, FromPrimitive)]
enum Allocation {
    InUseFirst = 0x51,
    InUseMiddle = 0x52,
    InUseLast = 0x53,
    FreeFresh = 0xA0,
    FreeFirst = 0xA1,
    FreeMiddle = 0xA2,
    FreeLast = 0xA3,
}

#[derive(Debug)]
struct Title {
    id: String,
    icon_flag: IconFlag,
    block_number: u8,
    title: String,
    palette: [u8; 32],
}

#[derive(Debug, Clone, Copy, FromPrimitive)]
enum IconFlag {
    Static = 0x11,
    AnimatedSlow = 0x12,
    AnimatedFast = 0x13,
}

struct ShiftJisCharSet([u8; 32]);
