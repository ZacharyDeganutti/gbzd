mod processor {
    pub mod cpu;
    pub mod ops;
    pub mod execute;
}
mod memory_gb;
mod cart;
mod special_registers;

use std::rc::Rc;
use std::cell::RefCell;
use crate::processor::cpu::*;

fn main() {
    let cart = cart::Cart::load_from_file("roms/02-interrupts.gb").expect("Problem with ROM file");
    let mut system_memory_data = memory_gb::MemoryMap::allocate(cart);
    let system_memory = Rc::new(RefCell::new(memory_gb::MemoryMap::new(&mut system_memory_data)));
    let mut cpu = Cpu::new(system_memory.clone());
    //println!("{}", cpu.registers.read_word(WordRegisterName::RegPC));

    cpu.run();
    // let mut i = 0;
    // loop {
    //     print!("{}\n", i);
    //     cpu.step();
    //     i += 1;
    // }
}
