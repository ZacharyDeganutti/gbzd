

use crate::memory_gb::Address;
use crate::memory_gb::Byte;
use crate::memory_gb::MemoryBank;
use crate::memory_gb::Word;
use crate::memory_gb::MemoryRegion;

// Cart types go here

// 32kb
const DEFAULT_CART_SIZE: usize = 0x8000;
const CART_BASE_ADDRESS: Address = 0x0000;
struct NoMBC {
    data: Vec<Byte>
}

impl MemoryRegion for NoMBC {
    fn get_bank(&mut self, address: Address) -> Option<MemoryBank> {
        if address < DEFAULT_CART_SIZE as Address {
            Some(MemoryBank { start: CART_BASE_ADDRESS, data: &mut self.data[..] })
        }
        else {
            None
        }
    }
}

// End cart types

enum Mapper {
    NoMBC(NoMBC)
}
pub struct Cart {
    data: Mapper
}

impl Cart {
    pub fn load_from_file(path: &str) -> Result<Cart, std::io::Error> {
        const MAPPER_TYPE_LOCATION: usize = 0x0147;
        let contents = std::fs::read(path)?;
        let mapper = match contents[MAPPER_TYPE_LOCATION] {
            0x00 => {
                Ok(Mapper::NoMBC(NoMBC { data: contents }))
            }
            _ => {
                println!("{}", contents[MAPPER_TYPE_LOCATION]);
                Err(std::io::ErrorKind::InvalidData)
            }
        }?;
        Ok( Cart { data: mapper } )
    }
}

impl MemoryRegion for Cart {
    fn get_bank(&mut self, address: Address) -> Option<MemoryBank> {
        match self.data {
            Mapper::NoMBC(ref mut no_mbc) => {
                no_mbc.get_bank(address)
            }
        }
    }
}
