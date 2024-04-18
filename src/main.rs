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
    let rom = "roms/dmg-acid2.gb";
    // let rom = "roms/11-op a,(hl).gb";
    let cart = cart::Cart::load_from_file(rom).expect("Problem with ROM file");
    let mut system_memory_data = memory_gb::MemoryMap::allocate(cart);
    let system_memory = Rc::new(RefCell::new(memory_gb::MemoryMap::new(&mut system_memory_data)));
    let mut cpu = Cpu::new(system_memory.clone());
    let mut ppu = Ppu::new(system_memory.clone());

    // Debt represents the timing balance between cpu and ppu.
    // The cpu runs up the debt (positive)
    // The ppu pays down the debt (negative)
    // The ppu only has to do work if its debt is greater than 0
    let mut debt: i16 = 0;
    let mut cpu_locked: bool = false;
    loop {
        if debt <= 0 && !cpu_locked {
            let payment = (cpu.run() * 4) as i16;
            debt += payment;
            if payment == 0 {
                cpu_locked = true;
            }
        }
        else {
            if cpu_locked {
                ppu.run();
                cpu_locked = false
            }
            else {
                debt -= ppu.run();
            }
        }
        //println!("debt: {}", debt);
    }
    
    // let mut i = 0;
    // loop {
    //     print!("{}\n", i);
    //     cpu.step();
    //     i += 1;
    // }
}
