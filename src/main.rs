mod processor {
    pub mod cpu;
    pub mod ops;
    pub mod execute;
}
mod memory_gb;
mod cart;

use std::rc::Rc;
use std::cell::RefCell;
use crate::processor::cpu::*;

fn main() {
    let cart = cart::Cart::load_from_file("roms/cpu_instrs.gb").expect("Problem with ROM file");
    let system_memory = Rc::new(RefCell::new(memory_gb::MemoryMap::new(cart)));
    let mut cpu = Cpu::new(system_memory.clone());
    println!("{}", cpu.registers.read_word(WordRegisterName::RegPC));

    cpu.run();
    // let mut i = 0;
    // loop {
    //     print!("{}\n", i);
    //     cpu.step();
    //     i += 1;
    // }
}
