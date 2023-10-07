use std::mem;
use std::slice;

const MAP_SIZE: usize = 0x10000; 

pub type Byte = u8;
pub type Word = u16;
pub type Signed = i8;

pub type Address = Word;

#[repr(C)]
pub struct MemoryMap {
    memory: [Byte; MAP_SIZE]
}

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

pub trait MemoryUnit: EndianTranslate + Sized {}
impl MemoryUnit for Byte {}
impl MemoryUnit for Word {}

// MemoryRegion structs should generally use #[repr(C)] and contain POD types to ensure understandable behavior 
// MemoryRegion read/write are unsafe and need additional assurances when the MemoryRegion is smaller than the span of numbers that Address can represent
//      These can be presumed safe only if the MemoryRegion is equal in size to the Address space of 0x10000
//      Where are my dependent types?
pub trait MemoryRegion: Sized {
    fn boundary_check<T: MemoryUnit>(&self, address: Address) -> () {
        let space = (std::mem::size_of::<Self>() as isize) - (address as isize);
        if space < (mem::size_of::<T>() as isize) {
            panic!("Bad memory access attempted in MemoryRegion of type {}. Attempt to access {} of size {} with {} bytes left",
                std::any::type_name::<Self>(),
                std::any::type_name::<T>(),
                mem::size_of::<T>(),
                space
            )
        } else if space > (mem::size_of::<Self>() as isize) {
            panic!("Bad memory access attempted in MemoryRegion of type {}. Attempt to access {} of size {} {} bytes before the region",
                std::any::type_name::<Self>(),
                std::any::type_name::<T>(),
                mem::size_of::<T>(),
                space - (mem::size_of::<Self>() as isize)
            )
        }
    }

    unsafe fn get_ptr<T: MemoryUnit>(&mut self, address: Address) -> *mut T {
        mem::transmute::<*mut Byte, *mut T>((self as *mut Self as *mut Byte).offset(address as isize))
    }

    fn read<T: MemoryUnit>(&mut self, from: Address) -> T {
        self.boundary_check::<T>(from);
        unsafe {
            let read_location =  self.get_ptr::<T>(from);
            (*read_location).from_gb_endian()
        }
    }

    fn write<T: MemoryUnit>(&mut self, value: T, to: Address) -> () {
        self.boundary_check::<T>(to);
        unsafe {
            let write_location = self.get_ptr::<T>(to);
            (*write_location) = value.to_gb_endian()
        }
    }
}

// TODO: Override read and write to use virtual addressing against a structure full of MemoryRegions
impl MemoryRegion for MemoryMap {}
impl MemoryMap {
    pub fn new() -> MemoryMap {
        MemoryMap { memory: [0; MAP_SIZE] }
    }
}