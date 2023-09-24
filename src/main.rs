mod cpu;
mod memory_gb;
mod cpu_ops;
mod cpu_execute;

use std::rc::Rc;
use std::cell::RefCell;
use crate::cpu::*;

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
