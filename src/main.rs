mod cpu;
mod memory_gb;

use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    println!("Hello, world!!");
    let system_memory = Rc::new(RefCell::new(memory_gb::MemoryMap::new()));
    let mut cpu = cpu::Cpu::new(system_memory.clone());
    cpu.ld_byte(
        cpu::LDByteDestination::RegisterValue(cpu::ByteRegister::RegA),
        cpu::LDByteSource::RegisterValue(cpu::ByteRegister::RegE),
    );
    println!("{}", cpu.registers.a);
    cpu.ld_byte(
        cpu::LDByteDestination::RegisterValue(cpu::ByteRegister::RegH),
        cpu::LDByteSource::ImmediateValue(128),
    );
    println!("{}", cpu.registers.h);
}
