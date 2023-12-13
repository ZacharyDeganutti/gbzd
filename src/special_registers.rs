use std::mem;

use crate::memory_gb::Address;
use crate::memory_gb::Byte;
use crate::memory_gb::EndianTranslate;
use crate::memory_gb::Word;
use crate::memory_gb::MemoryBank;
use crate::memory_gb::BankType;
use crate::memory_gb::MemoryRegion;
use crate::memory_gb::MemoryUnit;

impl MemoryRegion for Divider {
    fn read<T: MemoryUnit>(&mut self, _: Address) -> T {
        let read_slice = &self.data[0..mem::size_of::<T>()];
        T::from_le_bytes(read_slice)
    }

    // Front-end write for the Divider clears it to 0
    fn write<T: MemoryUnit>(&mut self, _: T, _: Address) -> () {
        self.data = [0x00, 0x00]
    }
}

impl Divider {
    fn increment<T: MemoryUnit>(&mut self, _: T, _: Address) -> () {
        let value = Word::from_le_bytes(self.data);
        self.data = value.wrapping_add(1).to_le_bytes();
    }
}

// End cart types
pub struct Divider {
    pub data: [Byte; 2]
}
