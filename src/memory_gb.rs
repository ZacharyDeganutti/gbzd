use std::mem;
use std::slice;

const MAP_SIZE: usize = 0x10000; 

pub type Byte = u8;
pub type Word = u16;

pub type Address = Word;

#[repr(C)]
pub struct MemoryMap {
    memory: [Byte; MAP_SIZE]
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

pub trait MemoryUnit: EndianTranslate {}
impl MemoryUnit for Byte {}
impl MemoryUnit for Word {}

// MemoryRegion structs should generally use #[repr(C)] and contain POD types to ensure understandable behavior 
// MemoryRegion read/write are unsafe and need additional assurances when the MemoryRegion is smaller than the span of numbers that Address can represent
//      These can be presumed safe only if the MemoryRegion is equal in size to the Address space of 0x10000
//      Where are my dependent types?
pub trait MemoryRegion: Sized {
    unsafe fn interpret_as<T: MemoryUnit>(&mut self) -> &mut [T] {
        let ptr = mem::transmute::<&mut Self, *mut T>(self);
        slice::from_raw_parts_mut(ptr, mem::size_of::<Self>() / mem::size_of::<T>())
    }

    // Can be presumed safe for memory regions of size 0x10000
    unsafe fn read<T: MemoryUnit>(&mut self, from: Address) -> T {
        let slice =  self.interpret_as::<T>();
        slice[from as usize].from_gb_endian()
    }

    // Can be presumed safe for memory regions of size 0x10000
    unsafe fn write<T: MemoryUnit>(&mut self, value: T, to: Address) -> () {
        let slice = self.interpret_as::<T>();
        slice[to as usize] = value.to_gb_endian()
    }
}

impl MemoryRegion for MemoryMap {}
impl MemoryMap {
    pub fn new() -> MemoryMap {
        MemoryMap { memory: [0; MAP_SIZE] }
    }
}