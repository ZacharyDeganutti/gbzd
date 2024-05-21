use std::mem;

use crate::{cart::Cart, input::{self, Joypad}, special_registers::Timer};

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

pub trait MemoryUnit: EndianTranslate + Sized + TryInto<u8> {
    type A;

    fn copy_into_le_bytes(self, destination: &mut [Byte]) -> ();
    fn from_le_bytes(bytes: &[Byte]) -> Self;
    fn invalid_read_value() -> Self;
    fn as_ascii(self) -> String;
    fn as_hex(self) -> String;
    fn demote(self) -> Byte;
    fn promote(byte: Byte) -> Self;
}

// These impls are probably good candidates for a macro
impl MemoryUnit for Byte {
    type A = [Byte; mem::size_of::<Self>()];
    fn copy_into_le_bytes(self, destination: &mut [Byte]) -> () { 
        let bytes = self.to_le_bytes();
        destination.copy_from_slice(&bytes)
    }

    fn from_le_bytes(bytes: &[Byte]) -> Self {
        bytes[0]
    }

    fn invalid_read_value() -> Self {
        0xFF
    }

    fn as_ascii(self) -> String {
        let borrowed_bytes = &self.to_le_bytes();
        let ascii = std::str::from_utf8(borrowed_bytes).unwrap_or("?");
        String::from(ascii)
    }

    fn as_hex(self) -> String {
        let bytes = self.to_le_bytes().to_vec();
        bytes.iter()
            .map(|b| format!("{:02x}", b).to_string().to_ascii_uppercase())
            .rev()
            .collect::<String>()
    }

    fn demote(self) -> Byte {
        self
    }

    fn promote(byte: Byte) -> Self {
        byte
    }
}

impl MemoryUnit for Word {
    type A = [Byte; mem::size_of::<Self>()];
    fn copy_into_le_bytes(self, destination: &mut [Byte]) -> () { 
        let bytes = self.to_le_bytes();
        destination.copy_from_slice(&bytes)
    }

    #[cfg(target_endian = "big")]
    fn from_le_bytes(bytes: &[Byte]) -> Self {
        (bytes[0] as Word) << 8 | (bytes[1] as Word)
    }

    #[cfg(target_endian = "little")]
    fn from_le_bytes(bytes: &[Byte]) -> Self {
        (bytes[1] as Word) << 8 | (bytes[0] as Word)
    }

    fn invalid_read_value() -> Self {
        0xFFFF
    }

    fn as_ascii(self) -> String {
        let borrowed_bytes = &self.to_le_bytes();
        let ascii = std::str::from_utf8(borrowed_bytes).unwrap_or("?");
        String::from(ascii)
    }

    fn as_hex(self) -> String {
        let bytes = self.to_le_bytes().to_vec();
        bytes.iter()
            .map(|b| format!("{:02x}", b).to_string().to_ascii_uppercase())
            .rev()
            .collect::<String>()
    }

    fn demote(self) -> Byte {
        *self.to_le_bytes().to_vec().last().unwrap()
    }

    fn promote(byte: Byte) -> Self {
        byte as Word
    }
}

pub trait MemoryRegion {
    fn read<T: MemoryUnit>(&mut self, address: Address) -> T;
    fn write<T: MemoryUnit>(&mut self, value: T, address: Address) -> ();
}

pub struct SimpleRegion<'a> {
    pub start: Address,
    pub data: &'a mut [Byte],
}

// Read from a given buffer at a specific address within a 16 bit address space
pub fn read_from_buffer<T: MemoryUnit>(buffer: &[u8], address: Address) -> T {
    read_from_buffer_extended(buffer, address as usize)
}

// Read from some arbitrary buffer without the 16 bit address space limitation. Useful for cartridge mapping
pub fn read_from_buffer_extended<T: MemoryUnit>(buffer: &[u8], address: usize) -> T {
    let read_slice = &buffer[address ..(address + mem::size_of::<T>())];
    T::from_le_bytes(read_slice)
}

// Write a value to a mutable buffer at a specific address within a 16 bit address space
pub fn write_to_buffer<T: MemoryUnit>(buffer: &mut [u8], value: T, address: Address) -> () {
    write_to_buffer_extended(buffer, value, address as usize)
}

// Write a value to a mutable buffer at a specific address within a 16 bit address space
pub fn write_to_buffer_extended<T: MemoryUnit>(buffer: &mut [u8], value: T, address: usize) -> () {
    let destination_slice = &mut buffer[address..(address + mem::size_of::<T>())];
    value.copy_into_le_bytes(destination_slice)
}

impl<'a> MemoryRegion for SimpleRegion<'a> {
    fn read<T: MemoryUnit>(&mut self, address: Address) -> T {
        read_from_buffer(&self.data, address - self.start)
    }
    fn write<T: MemoryUnit>(&mut self, value: T, address: Address) -> () {
        write_to_buffer(self.data, value, address - self.start)
    }
}

// const MAP_SIZE: usize = 0x10000; 
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
// TODO: hide rom, rom_swappable, external_ram behind cart abstraction
#[repr(C)]
pub struct MemoryMapData { 
    cart: Cart,
    timer: Timer,
    joypad: Joypad,
    vram: [Byte; EXRAM_START - VRAM_START],
    work_ram: [Byte; WRAM_S_START - WRAM_START],
    work_ram_swappable: [Byte; ECHORAM_START - WRAM_S_START],
    echo_ram: [Byte; OAM_START - ECHORAM_START],
    oam: [Byte; UNUSABLE_START - OAM_START],
    unusable: [Byte; IOREGS_START - UNUSABLE_START],
    io_registers: [Byte; HRAM_START - IOREGS_START],
    hram: [Byte; IE_START - HRAM_START],
    ie: [Byte; 1],
}

pub struct MemoryMap<'a> { 
    cart: &'a mut Cart,
    pub timer: &'a mut Timer,
    pub joypad: &'a mut Joypad,
    vram: SimpleRegion<'a>,
    work_ram: SimpleRegion<'a>,
    work_ram_swappable: SimpleRegion<'a>,
    echo_ram: SimpleRegion<'a>,
    oam: SimpleRegion<'a>,
    unusable: SimpleRegion<'a>,
    pub io_registers: SimpleRegion<'a>,
    hram: SimpleRegion<'a>,
    ie: SimpleRegion<'a>,
}

// TODO: Override get_bank to implement mapped addressing against a structure full of MemoryRegions
impl<'a> MemoryRegion for MemoryMap<'a> {

    fn read<T: MemoryUnit>(&mut self, address: Address) -> T {
        let _address = address as usize;
        if _address == IE_START {
            self.ie.read(address)
        }
        else if _address >= HRAM_START {
            self.hram.read(address)
        }
        else if _address >= IOREGS_START {
            // Some registers have special behaviors
            // TODO: Implement joypad
            if address == 0xFF00 {
                T::promote(self.joypad.read())
            }
            else if address == 0xFF04 {
                T::promote(self.timer.read_divider())
            }
            else if address == 0xFF05 {
                T::promote(self.timer.read_counter())
            }
            else if address == 0xFF06 {
                T::promote(self.timer.read_modulo())
            }
            else if address == 0xFF07 {
                T::promote(self.timer.read_control())
            }
            else {
                self.io_registers.read(address)
            }
        }
        else if _address >= UNUSABLE_START {
            self.unusable.read(address)
        }
        else if _address >= OAM_START {
            self.oam.read(address)
        }
        else if _address >= ECHORAM_START {
            self.echo_ram.read(address)
        }
        else if _address >= WRAM_S_START {
            self.work_ram_swappable.read(address)
        }
        else if _address >= WRAM_START {
            self.work_ram.read(address)
        }
        else if _address >= EXRAM_START {
            // External RAM is on the cartridge
            self.cart.read(address)
        }
        else if _address >= VRAM_START {
            // Likely to have a different BankType later
            self.vram.read(address)
        }
        else {
            // The rest of the address space is mapped from the cartridge ROM
            self.cart.read(address)
        } 
    }

    fn write<T: MemoryUnit>(&mut self, value: T, address: Address) -> () {
        let _address = address as usize;
        if _address == IE_START {
            self.ie.write(value, address)
        }
        else if _address >= HRAM_START {
            self.hram.write(value, address)
        }
        else if _address >= IOREGS_START {
            // Some registers have special behaviors
            if address == 0xFF00 {
                // Check upper nibble of the written byte
                match value.demote() >> 4 {
                    0 => self.joypad.set_mode(input::JoypadMode::Unselected),
                    1 => self.joypad.set_mode(input::JoypadMode::DPad),
                    2 => self.joypad.set_mode(input::JoypadMode::Buttons),
                    _ => () // Ignore the write if an invalid combination is supplied
                }
            }
            // Redirect serial to printed ascii
            else if address == 0xFF01 {
                let a = value.as_ascii();
                print!("{}", a);
            }
            else if address == 0xFF04 {
                self.timer.write_divider(value.demote())
            }
            else if address == 0xFF05 {
                self.timer.write_counter(value.demote())
            }
            else if address == 0xFF06 {
                self.timer.write_modulo(value.demote())
            }
            else if address == 0xFF07 {
                self.timer.write_control(value.demote())
            }
            else if address == 0xFF46 {
                self.dma(value.demote())
            }
            else {
                self.io_registers.write(value, address)
            }
        }
        else if _address >= UNUSABLE_START {
            self.unusable.write(value, address)
        }
        else if _address >= OAM_START {
            self.oam.write(value, address)
        }
        else if _address >= ECHORAM_START {
            self.echo_ram.write(value, address)
        }
        else if _address >= WRAM_S_START {
            self.work_ram_swappable.write(value, address)
        }
        else if _address >= WRAM_START {
            self.work_ram.write(value, address)
        }
        else if _address >= EXRAM_START {
            // External RAM is on the cartridge
            self.cart.write(value, address)
        }
        else if _address >= VRAM_START {
            // Likely to have a different BankType later
            self.vram.write(value, address)
        }
        else {
            // The rest of the address space is mapped from the cartridge ROM
            self.cart.write(value, address)
        }
    }
}

impl<'a> MemoryMap<'a> {
    pub fn allocate(cart: Cart, joypad: Joypad) -> MemoryMapData {
        let timer: Timer = Timer::new() ;
        MemoryMapData { 
            cart,
            timer,
            joypad,
            vram: [0; EXRAM_START - VRAM_START],
            work_ram: [0; WRAM_S_START - WRAM_START],
            work_ram_swappable: [0; ECHORAM_START - WRAM_S_START],
            echo_ram: [0; OAM_START - ECHORAM_START],
            oam: [0; UNUSABLE_START - OAM_START],
            unusable: [0; IOREGS_START - UNUSABLE_START],
            io_registers: [0xFF; HRAM_START - IOREGS_START],
            hram: [0; IE_START - HRAM_START],
            ie: [0; 1],
        }
    }

    pub fn new(data: &mut MemoryMapData) -> MemoryMap {
        MemoryMap { 
            cart: &mut data.cart,
            timer: &mut data.timer,
            joypad: &mut data.joypad,
            vram: SimpleRegion { start: VRAM_START as Address, data: &mut data.vram },
            work_ram: SimpleRegion { start: WRAM_START as Address, data: &mut data.work_ram },
            work_ram_swappable: SimpleRegion { start: WRAM_S_START as Address, data: &mut data.work_ram_swappable },
            echo_ram: SimpleRegion { start: ECHORAM_START as Address, data: &mut data.echo_ram },
            oam: SimpleRegion { start: OAM_START as Address, data: &mut data.oam },
            unusable: SimpleRegion { start: UNUSABLE_START as Address, data: &mut data.unusable },
            io_registers: SimpleRegion { start: IOREGS_START as Address, data: &mut data.io_registers },
            hram: SimpleRegion { start: HRAM_START as Address, data: &mut data.hram },
            ie: SimpleRegion { start: IE_START as Address, data: &mut data.ie },
        }
    }

    // Cheating DMA function that completes instantly instead of in 160 dots
    fn dma(&mut self, source_upper_byte: Byte) {
        const DMA_BYTES: Address = 0xA0;
        let dma_base = (source_upper_byte as Address) << 8;
        for i in 0..DMA_BYTES {
            let source = dma_base + i;
            // Copy to OAM
            let destination = 0xFE00 + i;
            let copy_byte: Byte = self.read(source);
            self.write(copy_byte, destination);
        }
    }
}