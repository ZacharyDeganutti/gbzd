mod cpu;
mod memory_gb;

use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    println!("Hello, world!!");
    let system_memory = Rc::new(RefCell::new(memory_gb::MemoryMap::new()));
    let mut cpu = cpu::Cpu::new(system_memory.clone());
    cpu.ld_byte_op(
        cpu::ByteRegister::new(cpu::ByteRegisterName::RegA),
        cpu::ByteRegister::new(cpu::ByteRegisterName::RegE)
    );
    println!("{}", cpu.registers.a);
    cpu.ld_byte_op(
        cpu::ByteRegister::new(cpu::ByteRegisterName::RegH),
        cpu::ByteImmediate::new(128),
    );
    println!("{}", cpu.registers.h);
}
