mod cpu;
mod memory_gb;

use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    println!("Hello, world!!");
    let system_memory = Rc::new(RefCell::new(memory_gb::MemoryMap::new()));
    let mut cpu = cpu::Cpu::new(system_memory.clone());
    cpu.ld_byte(
        cpu::ByteDestination::RegisterValue(cpu::ByteRegister::RegA),
        cpu::RegisterValue::new(cpu::ByteRegister::RegE)
    );
    println!("{}", cpu.registers.a);
    cpu.ld_byte(
        cpu::ByteDestination::RegisterValue(cpu::ByteRegister::RegH),
        cpu::ImmediateValue::new(128),
    );
    println!("{}", cpu.registers.h);
}
