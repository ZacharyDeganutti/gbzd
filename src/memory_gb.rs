use std::mem;

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

// MemoryRegion structs should generally use #[repr(C)] and contain POD types to ensure understandable behavior 
// MemoryRegion read/write are unsafe and need additional assurances when the MemoryRegion is smaller than the span of numbers that Address can represent
//      These can be presumed safe only if the MemoryRegion is equal in size to the Address space of 0x10000
//      Where are my dependent types?
// pub trait MemoryRegion: SliceIndex<[Byte]> + Sized {
pub trait MemoryRegion: Sized {
    fn region_slice(&mut self, address: Address) -> MemoryBank;

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

    fn read<T: MemoryUnit>(&mut self, address: Address) -> T {
        // self.boundary_check::<T>(from);
        let bank = self.region_slice(address);
        let adjusted_index = (address - bank.start) as usize; 
        let read_slice = &bank.data[adjusted_index..(adjusted_index + mem::size_of::<T>())];
        T::from_le_bytes(read_slice)
    }

    fn write<T: MemoryUnit>(&mut self, value: T, address: Address) -> () {
        // self.boundary_check::<T>(to);
        let bank = self.region_slice(address);
        let adjusted_index = (address - bank.start) as usize;
        let destination_slice = &mut bank.data[adjusted_index..(adjusted_index + mem::size_of::<T>())];
        value.copy_into_le_bytes(destination_slice)
    }
}

// TODO: Override read and write to use virtual addressing against a structure full of MemoryRegions
impl MemoryRegion for MemoryMap {
    fn region_slice(&mut self, _: Address) -> MemoryBank {
        MemoryBank { start: 0x0000, data: &mut self.memory[..] }
    }
}
impl MemoryMap {
    pub fn new() -> MemoryMap {
        MemoryMap { memory: [0; MAP_SIZE] }
    }
}