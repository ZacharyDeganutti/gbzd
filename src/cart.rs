

use crate::memory_gb;
use crate::memory_gb::Address;
use crate::memory_gb::Byte;
use crate::memory_gb::MemoryBank;
use crate::memory_gb::BankType;
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
    fn read<T: MemoryUnit>(&mut self, address: Address) -> T {
        memory_gb::read_from_buffer(&self.data, address + CART_BASE_ADDRESS as Address)
    }

    fn write<T: MemoryUnit>(&mut self, value: T, address: Address) -> () {
        // Writing has no function without MBC
    }
}

// MBC1, has multiple mappable banks
// TODO: Support MBC1 with RAM, for now just assumes everything is a ROM blob
struct MBC1 {
    data: Vec<Byte>,
    active_bank: u8
}

impl MemoryRegion for MBC1 {
    fn read<T: MemoryUnit>(&mut self, address: Address) -> T {
        const SWAPPABLE_BASE_ADDRESS: usize = 0x4000;
        // ROM bank 0
        if address < SWAPPABLE_BASE_ADDRESS as Address {
            memory_gb::read_from_buffer(&self.data, address)
        }
        // Swappable ROM bank
        else {
            // active_bank 0 and 1 are both treated as a 0 offset, active_bank 2 as a 1 offset, continued...
            let bank_offset = (std::cmp::max(self.active_bank, 1) - 1) as usize;
            let bank_adjusted_address = address + (bank_offset * BANK_WIDTH) as Address;
            memory_gb::read_from_buffer(&self.data, bank_adjusted_address)
        } 
        // Let it panic if out of bounds somehow, probably indicates a mistake or exram access which is unsupported
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
                println!("Loaded No MBC");
                Ok(Mapper::NoMBC(NoMBC { data: contents }))
            }
            0x01 => {
                println!("Loaded MBC1");
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
    fn read<T: MemoryUnit>(&mut self, address: Address) -> T {
        match self.data {
            Mapper::NoMBC(ref mut no_mbc_cart) => {
                no_mbc_cart.read(address)
            }
            Mapper::MBC1(ref mut mbc1_cart) => {
                mbc1_cart.read(address)
            }
        }
    }

    fn write<T: MemoryUnit>(&mut self, value: T, address: Address) -> () {
        match self.data {
            Mapper::NoMBC(ref mut no_mbc_cart) => {
                no_mbc_cart.write(value, address)
            }
            Mapper::MBC1(ref mut mbc1_cart) => {
                mbc1_cart.write(value, address)
            }
        }
    }
}
