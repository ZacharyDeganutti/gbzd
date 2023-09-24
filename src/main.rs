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
    cpu.ld_byte(
        ByteRegister::new(ByteRegisterName::RegA),
        ByteRegister::new(ByteRegisterName::RegE)
    );
    println!("{}", cpu.registers.a);
    cpu.ld_byte(
        ByteRegister::new(ByteRegisterName::RegH),
        ByteImmediate::new(128),
    );
    cpu.inc_byte(ByteRegister::new(ByteRegisterName::RegH));
    println!("{}", cpu.registers.h);
}
