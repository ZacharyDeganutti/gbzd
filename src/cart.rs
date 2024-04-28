

use crate::memory_gb;
use crate::memory_gb::Address;
use crate::memory_gb::Byte;
use crate::memory_gb::MemoryBank;
use crate::memory_gb::BankType;
use crate::memory_gb::MemoryRegion;
use crate::memory_gb::MemoryUnit;

// Cart types go here

const CART_BASE_ADDRESS: usize = 0x0000;
const ROM_BANK_WIDTH: usize = 0x4000;
const ROM_WIDTH: usize = 0x8000;
const RAM_BANK_WIDTH: usize = 0x2000;

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
    active_rom_bank: u8
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
            let bank_offset = (std::cmp::max(self.active_rom_bank, 1) - 1) as usize;
            let bank_adjusted_address = address as usize + (bank_offset * ROM_BANK_WIDTH);
            memory_gb::read_from_buffer_extended(&self.data, bank_adjusted_address)
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
            self.active_rom_bank = byte_value & 0x1F;
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

// MBC3, has multiple mappable banks
struct MBC3 {
    data: Vec<Byte>,
    active_rom_bank: u8,
    ram_enabled: bool,
    active_ram_bank: u8,
    ram_banks: Vec<Byte>
}

// TODO: This is mostly lifted from the incomplete MBC1 implementation. Fix registers
impl MemoryRegion for MBC3 {
    fn read<T: MemoryUnit>(&mut self, address: Address) -> T {
        // ROM bank 0
        if address < 0x4000 as Address {
            memory_gb::read_from_buffer(&self.data, address)
        }
        // Swappable ROM bank
        else if (address >= 0x4000) && (address < 0x8000) {
            // active_bank 0 and 1 are both treated as a 0 offset, active_bank 2 as a 1 offset, continued...
            let bank_offset = (std::cmp::max(self.active_rom_bank, 1) - 1) as usize;
            let bank_adjusted_address = address as usize + (bank_offset * ROM_BANK_WIDTH);
            memory_gb::read_from_buffer_extended(&self.data, bank_adjusted_address)
        } 
        // RTC Registers or RAM
        else if (address >= 0xA000) && (address < 0xC000) {
            // TODO: RTC unsupported, just return 0 and cross fingers
            match self.active_ram_bank {
                0x08 | 0x09 | 0x0A | 0x0B | 0x0C => T::promote(0x00),
                _ => {
                    let bank_adjusted_address = (address as usize - 0xA000) + (self.active_ram_bank as usize * RAM_BANK_WIDTH);
                    if self.ram_enabled {
                        memory_gb::read_from_buffer_extended(&self.ram_banks, bank_adjusted_address)
                    }
                    else {
                        T::promote(Byte::invalid_read_value())
                    }
                }
            }
        }
        // Let it panic if out of bounds somehow, probably indicates a mistake
        else {
            panic!("Invalid cart read address");
        }
    }

    fn write<T: MemoryUnit>(&mut self, value: T, address: Address) -> () {
        // RAM address space
        if (address >= 0xA000) && (address < 0xC000) {
            // TODO: RTC unsupported, just do nothing and cross fingers
            match self.active_ram_bank {
                0x08 | 0x09 | 0x0A | 0x0B | 0x0C => (),
                _ => {
                    let bank_adjusted_address = (address as usize - 0xA000) + (self.active_ram_bank as usize * RAM_BANK_WIDTH);
                    if self.ram_enabled { memory_gb::write_to_buffer_extended(&mut self.ram_banks, value, bank_adjusted_address) }
                }
            }
        }
        // RAM enable register
        else if address < 0x2000 {
            match value.demote() {
                0x0A => self.ram_enabled = true,
                0x00 => self.ram_enabled = false,
                _ => ()
            }
        }
        // ROM bank select register
        else if (address >= 0x2000) && (address < 0x4000) {
            let byte_value: Byte = value.demote();
            self.active_rom_bank = byte_value & 0x7F;
        }
        //  RAM bank select register
        else if (address >= 0x4000) && (address < 0x6000) {
            // Let whatever get written here. What can go wrong?
            self.active_ram_bank = value.demote();
        }
        // Latch Clock Data
        else if (address >= 0x6000) && (address < 0x8000) {
            // TODO: Unstub
        }
        else {}
    }
}

// End cart types

enum Mapper {
    NoMBC(NoMBC),
    MBC1(MBC1),
    MBC3(MBC3)
}
pub struct Cart {
    data: Mapper
}

impl Cart {
    pub fn load_from_file(path: &str) -> Result<Cart, std::io::Error> {
        const MAPPER_TYPE_LOCATION: usize = 0x0147;
        const RAM_SIZE_LOCATION: usize = 0x0149;
        let contents = std::fs::read(path)?;
        let calc_ram = | bank_count: usize | {
            let mut ram_banks = Vec::<Byte>::with_capacity(bank_count*RAM_BANK_WIDTH);
            ram_banks.resize_with(ram_banks.capacity(), || Byte::invalid_read_value());
            (bank_count, ram_banks)
        };
        let (ram_bank_count, ram_banks) = match contents[RAM_SIZE_LOCATION] {
            0x00 => calc_ram(0),
            0x01 => calc_ram(0),
            0x02 => calc_ram(1),
            0x03 => calc_ram(4),
            0x04 => calc_ram(16),
            0x05 => calc_ram(8),
            _ => {
                panic!("Cartridge reports impossible RAM bank count")
            }
        };
        let mapper = match contents[MAPPER_TYPE_LOCATION] {
            0x00 => {
                println!("Loaded No MBC");
                Ok(Mapper::NoMBC(NoMBC { data: contents }))
            }
            0x01 => {
                println!("Loaded MBC1");
                Ok(Mapper::MBC1(MBC1 { data: contents, active_rom_bank: 1 }))
            }
            0x0F | 0x10 | 0x11 | 0x12 | 0x13 => {
                println!("Loaded MBC3");
                Ok(Mapper::MBC3(MBC3 { 
                    data: contents, 
                    active_rom_bank: 1, 
                    ram_enabled: false, 
                    active_ram_bank: 0, 
                    ram_banks }))
            }
            _ => {
                println!("Bad or unsupported MBC mapper: {:x}", contents[MAPPER_TYPE_LOCATION]);
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
            Mapper::MBC3(ref mut mbc3_cart) => {
                mbc3_cart.read(address)
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
            Mapper::MBC3(ref mut mbc3_cart) => {
                mbc3_cart.write(value, address)
            }
        }
    }
}
