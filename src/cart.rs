

use crate::memory_gb::Address;
use crate::memory_gb::Byte;
use crate::memory_gb::MemoryBank;
use crate::memory_gb::MemoryRegion;
use crate::memory_gb::MemoryUnit;

// Cart types go here

const CART_BASE_ADDRESS: usize = 0x0000;
const BANK_WIDTH: usize = 0x4000;
const ROM_WIDTH: usize = 0x8000;

struct NoMBC {
    data: Vec<Byte>
}

impl MemoryRegion for NoMBC {
    fn get_bank(&mut self, address: Address) -> Option<MemoryBank> {
        if address < ROM_WIDTH as Address {
            Some(MemoryBank { start: CART_BASE_ADDRESS as Address, data: &mut self.data[..] })
        }
        else {
            None
        }
    }
}

// MBC1, has multiple mappable banks
// TODO: Support MBC1 with RAM, for now just assumes everything is a ROM blob
struct MBC1 {
    data: Vec<Byte>,
    active_bank: u8
}

impl MemoryRegion for MBC1 {
    fn get_bank(&mut self, address: Address) -> Option<MemoryBank> {
        const SWAPPABLE_BASE_ADDRESS: usize = 0x4000;
        // ROM bank 1
        if address < 0x4000 as Address {
            Some(MemoryBank { start: CART_BASE_ADDRESS as Address, data: &mut self.data[CART_BASE_ADDRESS..BANK_WIDTH] })
        }
        // Swappable ROM bank
        else if (address >= SWAPPABLE_BASE_ADDRESS as Address) && (address < 0x8000 ) {
            let bank_lower = self.active_bank as usize * BANK_WIDTH;
            let bank_upper = bank_lower + BANK_WIDTH;
            Some(MemoryBank { 
                start: bank_lower as Address,
                data: &mut self.data[bank_lower..bank_upper]
            })
        } 
        // TODO: Add more cases for finding and mapping RAM banks
        else {
            None
        }
    }

    fn write<T: MemoryUnit>(&mut self, value: T, address: Address) -> () {
        // RAM address space
        if (address >= 0xA000) && (address < 0xC000) {
            // TODO: Unstub
            // self._write(value, address)
        }
        // RAM enable register
        else if address < 0x2000 {
            // TODO: Unstub
        }
        // ROM bank select register
        else if (address >= 0x2000) && (address < 0x4000) {
            let byte_value: Byte = value.try_into().unwrap_or_else(|_| panic!("Bad MemoryUnit conversion, should never happen!"));
            self.active_bank = byte_value & 0x1F;
        }
        //  RAM bank select register
        else if (address >= 0x4000) && (address < 0x6000) {
            // TODO: Unstub
        }
        // Banking mode select register
        else if (address >= 0x6000) && (address < 0x8000) {
            // TODO: Unstub
        }
        else {}
    }
}

// End cart types

enum Mapper {
    NoMBC(NoMBC),
    MBC1(MBC1)
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
            0x01 => {
                Ok(Mapper::MBC1(MBC1 { data: contents, active_bank: 1 }))
            }
            _ => {
                println!("Bad mapper: {}", contents[MAPPER_TYPE_LOCATION]);
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
            Mapper::MBC1(ref mut mbc1) => {
                mbc1.get_bank(address)
            }
        }
    }
}
