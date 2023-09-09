mod cpu;
mod memory_gb;

use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    println!("Hello, world!!");
    let system_memory = Rc::new(RefCell::new(memory_gb::MemoryMap::new()));
    let mut cpu = cpu::Cpu::new(system_memory.clone());
    cpu.ld_single(
        cpu::LDSingleDestination::RegisterValue(cpu::SingleRegister::RegA),
        cpu::LDSingleSource::RegisterValue(cpu::SingleRegister::RegE),
    );
    println!("{}", cpu.registers.a);
    cpu.ld_single(
        cpu::LDSingleDestination::RegisterValue(cpu::SingleRegister::RegH),
        cpu::LDSingleSource::ImmediateValue(128),
    );
    println!("{}", cpu.registers.h);
}
