mod processor {
    pub mod cpu;
    pub mod ops;
    pub mod execute;
}
mod memory_gb;
mod cart;
mod special_registers;
mod ppu;

use std::rc::Rc;
use std::cell::RefCell;
use crate::processor::cpu::*;
use crate::ppu::*;

fn main() {
    let cart = cart::Cart::load_from_file("roms/11-op a,(hl).gb").expect("Problem with ROM file");
    let mut system_memory_data = memory_gb::MemoryMap::allocate(cart);
    let system_memory = Rc::new(RefCell::new(memory_gb::MemoryMap::new(&mut system_memory_data)));
    let mut cpu = Cpu::new(system_memory.clone());
    let mut ppu = Ppu::new(system_memory.clone());

    // Debt represents the timing balance between cpu and ppu.
    // The cpu runs up the debt (positive)
    // The ppu pays down the debt (negative)
    // The ppu only has to do work if its debt is greater than 0
    let mut debt: i16 = 0;
    loop {
        if debt <= 0 {
            debt += (cpu.run() * 4) as i16;
        }
        else {
            debt -= ppu.run();
        }
        // Run the PPU to catch up with the approximate amount of work that the cpu did
        //debt -= ppu.run(cpu_cycles);
    }
    // let mut i = 0;
    // loop {
    //     print!("{}\n", i);
    //     cpu.step();
    //     i += 1;
    // }
}
