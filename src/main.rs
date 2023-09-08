mod cpu;
mod memory_gb;

fn main() {
    println!("Hello, world!!");
    let mut cpu = cpu::Cpu::new();
    cpu.ld8(
        cpu::LD8Destination::RegisterValue(cpu::Register8::RegA),
        cpu::LD8Source::RegisterValue(cpu::Register8::RegE),
    );
    println!("{}", cpu.registers.a);
    cpu.ld8(
        cpu::LD8Destination::RegisterValue(cpu::Register8::RegH),
        cpu::LD8Source::ImmediateValue(128),
    );
    println!("{}", cpu.registers.h);
}
