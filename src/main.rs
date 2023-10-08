mod processor {
    pub mod cpu;
    pub mod ops;
    pub mod execute;
}
mod memory_gb;

use std::rc::Rc;
use std::cell::RefCell;
use crate::processor::cpu::*;

fn main() {
    let system_memory = Rc::new(RefCell::new(memory_gb::MemoryMap::new()));
    let mut cpu = Cpu::new(system_memory.clone());
    println!("{}", cpu.registers.read_word(WordRegisterName::RegPC));

    let mut i = 0;
    loop {
        print!("{}\n", i);
        cpu.step();
        i += 1;
    }
}
