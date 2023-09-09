use std::mem;
use std::slice;

const MAP_SIZE: usize = 0x10000; 

pub type Single = u8;
pub type Double = u16;

pub type Address = u16;

#[repr(C)]
pub struct MemoryMap {
    memory: [Single; MAP_SIZE]
}

pub trait MemoryUnit: Copy {}
impl MemoryUnit for Single {}
impl MemoryUnit for Double {}

// MemoryRegion structs should generally use #[repr(C)] and contain POD types to ensure understandable behavior 
// MemoryRegion read/write are unsafe and need additional assurances when the MemoryRegion is smaller than the span of numbers that Address can represent
//      These can be presumed safe only if the MemoryRegion is equal in size or larger than the Address space
//      Where are my dependent types?
pub trait MemoryRegion: Sized {
    unsafe fn interpret_as<T: MemoryUnit>(&mut self) -> &mut [T] {
        let ptr = mem::transmute::<&mut Self, *mut T>(self);
        slice::from_raw_parts_mut(ptr, mem::size_of::<Self>() / mem::size_of::<T>())
    }

    unsafe fn read<T: MemoryUnit>(&mut self, from: Address) -> T {
        let slice =  self.interpret_as::<T>();
        slice[from as usize]
    }

    unsafe fn write<T: MemoryUnit>(&mut self, value: T, to: Address) -> () {
        let slice = self.interpret_as::<T>();
        slice[to as usize] = value
    }
}

impl MemoryRegion for MemoryMap {}
impl MemoryMap {
    pub fn new() -> MemoryMap {
        MemoryMap { memory: [0; MAP_SIZE] }
    }
}