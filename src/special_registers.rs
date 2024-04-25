use std::mem;

use crate::memory_gb::Address;
use crate::memory_gb::Byte;
use crate::memory_gb::EndianTranslate;
use crate::memory_gb::SimpleRegion;
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

    pub fn full_read(&mut self) -> Word {
        Word::from_le_bytes(self.data)
    }
}

// End cart types
struct Divider {
    data: [Byte; 2]
}

impl Timer {
    pub fn read_divider(&mut self) -> Byte {
        self.divider.read(0xFF04)
    }
    pub fn read_counter(&mut self) -> Byte {
        self.counter
    }
    pub fn read_modulo(&mut self) -> Byte {
        self.modulo
    }
    pub fn read_control(&mut self) -> Byte {
        self.control
    }

    pub fn write_divider(&mut self, value: Byte) {
        self.divider.write(value, 0xFF04)
    }
    pub fn write_counter(&mut self, value: Byte) {
        self.counter = value
    }
    pub fn write_modulo(&mut self, value: Byte) {
        self.modulo = value
    }
    pub fn write_control(&mut self, value: Byte) {
        self.control = value & 0x7
    }

    // TODO: Double check if reference shenanigans are handled correctly
    pub fn tick(&mut self) -> bool {
        let mut fire_interrupt_ready_status: bool = false;
        let pre_tick = self.divider.full_read();
        self.divider.increment();
        let post_tick = self.divider.full_read();
        // Detect which bits have changed in the divider
        let delta = pre_tick ^ post_tick;
        let timer_mask = self.control_mask();

        // If we overflowed on the last tick, take a detour to clean that up before doing counter increment logic
        if self.overflowing {
            self.counter = self.modulo;
            // Hard coded interrupt flag address
            fire_interrupt_ready_status = true;
            self.overflowing = false;
        }

        // Counter should increment when bits of the timer mask have changed, and they were 1 before the tick
        // In short, the timer counter increments on a falling edge of the divider bits
        let timer_counter_to_tick = (timer_mask & delta & pre_tick) > 0;
        if timer_counter_to_tick && ((self.control & 0x4) > 0) {
            if self.counter == 0xFF {
                self.overflowing = true;
                self.counter = 0;
            }
            else {
                self.counter += 1;
            }
            //println!("Counter: {}, Control: {}", self.counter.as_hex(), self.control.as_hex());
        }
        fire_interrupt_ready_status
    }

    fn control_mask(&mut self) -> Word {
        match self.control & 0x3 {
            0 => 1 << 9,
            1 => 1 << 3,
            2 => 1 << 5,
            3 => 1 << 7,
            _ => 0          // This shouldn't happen but something sure is going to spin fast if it does!
        }
    }

    pub fn new() -> Timer {
        let divider: Divider = Divider { data: [0x00; 2] };
        Timer {
            overflowing: false,
            divider,
            counter: 0x00,
            modulo: 0x00,
            control: 0x04
        }
    }
}

pub struct Timer {
    overflowing: bool,
    divider: Divider,
    counter: Byte,
    modulo: Byte,
    control: Byte
}
