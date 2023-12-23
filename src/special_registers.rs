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
        // The divider internally is 2 bytes, but only the top byte is exposed in the address space
        T::from_le_bytes(&self.data[1..]) 
    }

    // Writing directly to the divider clears it out
    fn write<T: MemoryUnit>(&mut self, _: T, _: Address) -> () {
        self.data = [0x00, 0x00]
    }
}

impl Divider {
    pub fn increment(&mut self) -> () {
        let value = Word::from_le_bytes(self.data);
        self.data = value.wrapping_add(1).to_le_bytes();
    }
}

// End cart types
pub struct Divider {
    pub data: [Byte; 2]
}
