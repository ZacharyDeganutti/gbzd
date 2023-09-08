use std::mem;
use std::slice;

const MAP_SIZE: usize = 0x10000; 

type Single = u8;
type Double = u16;

type Address = u16;

#[repr(C)]
pub struct MemoryMap {
    memory: [u8; MAP_SIZE]
}

pub trait MemoryUnit: Copy {}
impl MemoryUnit for Single {}
impl MemoryUnit for Double {}

// MemoryRegion structs should generally use #[repr(C)] and contain POD types to ensure understandable behavior 
pub trait MemoryRegion: Sized {
    unsafe fn interpretAs<T: MemoryUnit>(&mut self) -> &mut [T] {
        let ptr = mem::transmute::<&mut Self, *mut T>(self);
        slice::from_raw_parts_mut(ptr, mem::size_of::<Self>() / mem::size_of::<T>())
    }

    unsafe fn read<T: MemoryUnit>(&mut self, from: Address) -> T {
        let slice =  self.interpretAs::<T>();
        slice[from as usize]
    }

    unsafe fn write<T: MemoryUnit>(&mut self, value: T, to: Address) -> () {
        let slice = self.interpretAs::<T>();
        slice[to as usize] = value
    }
}

impl MemoryRegion for MemoryMap {}