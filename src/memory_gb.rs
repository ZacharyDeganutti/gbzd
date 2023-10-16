use std::mem;

pub type Byte = u8;
pub type Word = u16;
pub type Signed = i8;

pub type Address = Word;

pub trait ByteExt : {
    fn interpret_as_signed(self) -> Signed;
}

impl ByteExt for Byte {
    fn interpret_as_signed(self) -> Signed {
        unsafe { i8::from_le_bytes(mem::transmute::<u8, [u8; 1]>(self)) }
    }
}

pub trait EndianTranslate: Copy + Sized {
    fn to_gb_endian(self) -> Self;
    fn from_gb_endian(self) -> Self;
}
impl EndianTranslate for Byte {
    fn to_gb_endian(self) -> Self {
        self.to_le()
    }
    fn from_gb_endian(self) -> Self {
        Self::from_le(self)
    }
}
impl EndianTranslate for Word {
    fn to_gb_endian(self) -> Self {
        self.to_le()
    }
    fn from_gb_endian(self) -> Self {
        Self::from_le(self)
    }
}

pub trait MemoryUnit: EndianTranslate + Sized {
    // type A : TryFrom<&'a [Byte]>;
    type A;

    fn copy_into_le_bytes(self, destination: &mut [Byte]) -> ();
    fn from_le_bytes(bytes: &[Byte]) -> Self;
}
// These impls are probably good candidates for a macro
impl MemoryUnit for Byte {
    type A = [Byte; mem::size_of::<Self>()];
    fn copy_into_le_bytes(self, destination: &mut [Byte]) -> () { 
        let bytes = self.to_le_bytes();
        destination.copy_from_slice(&bytes)
    }

    fn from_le_bytes(bytes: &[Byte]) -> Self {
        Self::from_le_bytes(bytes.try_into().unwrap())
    }
}
impl MemoryUnit for Word {
    type A = [Byte; mem::size_of::<Self>()];
    fn copy_into_le_bytes(self, destination: &mut [Byte]) -> () { 
        let bytes = self.to_le_bytes();
        destination.copy_from_slice(&bytes)
    }

    fn from_le_bytes(bytes: &[Byte]) -> Self {
        Self::from_le_bytes(bytes.try_into().unwrap())
    }
}

pub struct MemoryBank<'a> {
    pub start: Address,
    pub data: &'a mut [Byte]
}

pub trait MemoryRegion: Sized {
    fn get_bank(&mut self, address: Address) -> MemoryBank;

    fn _read<T: MemoryUnit>(&mut self, address: Address) -> T {
        let bank = self.get_bank(address);
        let adjusted_index = (address - bank.start) as usize; 
        let read_slice = &bank.data[adjusted_index..(adjusted_index + mem::size_of::<T>())];
        T::from_le_bytes(read_slice)
    }

    fn _write<T: MemoryUnit>(&mut self, value: T, address: Address) -> () {
        let bank = self.get_bank(address);
        let adjusted_index = (address - bank.start) as usize;
        let destination_slice = &mut bank.data[adjusted_index..(adjusted_index + mem::size_of::<T>())];
        value.copy_into_le_bytes(destination_slice)
    }

    fn read<T: MemoryUnit>(&mut self, address: Address) -> T {
        self._read::<T>(address)
    }

    fn write<T: MemoryUnit>(&mut self, value: T, address: Address) -> () {
        self._write::<T>(value, address)
    }
}

// const MAP_SIZE: usize = 0x10000; 

const ROM_START: usize = 0x0000;
const ROM_S_START: usize = 0x4000;
const VRAM_START: usize = 0x8000;
const EXRAM_START: usize = 0xA000;
const WRAM_START: usize = 0xC000;
const WRAM_S_START: usize = 0xD000;
const ECHORAM_START: usize = 0xE000;
const OAM_START: usize = 0xFE00;
const UNUSABLE_START: usize = 0xFEA0;
const IOREGS_START: usize = 0xFF00;
const HRAM_START: usize = 0xFF80;
const IE_START: usize = 0xFFFF;

// TODO: revisit if repr(C) is necessary
// TODO: double check if MemoryRegions themselves still need to be sized now that they all return MemoryBanks
// TODO: hide rom, rom_swappable, external_ram behind cart abstraction
#[repr(C)]
pub struct MemoryMap { 
    rom: [Byte; ROM_S_START - ROM_START],
    rom_swappable: [Byte; VRAM_START - ROM_S_START],
    vram: [Byte; EXRAM_START - VRAM_START],
    external_ram: [Byte; WRAM_START - EXRAM_START],
    work_ram: [Byte; WRAM_S_START - WRAM_START],
    work_ram_swappable: [Byte; ECHORAM_START - WRAM_S_START],
    echo_ram: [Byte; OAM_START - ECHORAM_START],
    oam: [Byte; UNUSABLE_START - OAM_START],
    unusable: [Byte; IOREGS_START - UNUSABLE_START],
    io_registers: [Byte; HRAM_START - IOREGS_START],
    hram: [Byte; IE_START - HRAM_START],
    ie: [Byte; 1],
}

// TODO: Override get_bank to implement mapped addressing against a structure full of MemoryRegions
impl MemoryRegion for MemoryMap {
    fn get_bank(&mut self, address: Address) -> MemoryBank {
        // MemoryBank { start: 0x0000, data: &mut self.memory[..] }
        let _address = address as usize;
        if _address == IE_START {
            MemoryBank { start: IE_START as Address, data: &mut self.ie[..] }
        }
        else if _address >= HRAM_START {
            MemoryBank { start: HRAM_START as Address, data: &mut self.hram[..] }
        }
        else if _address >= IOREGS_START {
            MemoryBank { start: IOREGS_START as Address, data: &mut self.io_registers[..] }
        }
        else if _address >= UNUSABLE_START {
            MemoryBank { start: UNUSABLE_START as Address, data: &mut self.unusable[..] }
        }
        else if _address >= OAM_START {
            MemoryBank { start: OAM_START as Address, data: &mut self.oam[..] }
        }
        else if _address >= ECHORAM_START {
            MemoryBank { start: OAM_START as Address, data: &mut self.echo_ram[..] }
        }
        else if _address >= WRAM_S_START {
            MemoryBank { start: WRAM_S_START as Address, data: &mut self.work_ram_swappable[..] }
        }
        else if _address >= WRAM_START {
            MemoryBank { start: WRAM_START as Address, data: &mut self.work_ram[..] }
        }
        else if _address >= VRAM_START {
            MemoryBank { start: VRAM_START as Address, data: &mut self.vram[..] }
        }
        else if _address >= ROM_S_START {
            MemoryBank { start: ROM_S_START as Address, data: &mut self.rom_swappable[..] }
        }
        else {
            MemoryBank { start: ROM_START as Address, data: &mut self.rom[..] }
        }
    }
}
impl MemoryMap {
    pub fn new() -> MemoryMap {
        MemoryMap { 
            rom: [0; ROM_S_START - ROM_START],
            rom_swappable: [0; VRAM_START - ROM_S_START],
            vram: [0; EXRAM_START - VRAM_START],
            external_ram: [0; WRAM_START - EXRAM_START],
            work_ram: [0; WRAM_S_START - WRAM_START],
            work_ram_swappable: [0; ECHORAM_START - WRAM_S_START],
            echo_ram: [0; OAM_START - ECHORAM_START],
            oam: [0; UNUSABLE_START - OAM_START],
            unusable: [0; IOREGS_START - UNUSABLE_START],
            io_registers: [0; HRAM_START - IOREGS_START],
            hram: [0; IE_START - HRAM_START],
            ie: [0; 1],
        }
    }
}