use crate::memory_gb::Address;
use crate::memory_gb::Byte;
use crate::memory_gb::ByteExt;
use crate::memory_gb::MemoryRegion;
use crate::memory_gb::Signed;
use crate::memory_gb::Word;
use crate::processor::cpu::*;
use crate::processor::cpu::ByteRegisterName::*;
use crate::processor::cpu::WordRegisterName::*;
use crate::processor::ops;

impl Cpu {
    fn byte_operand(&mut self, position: Address) -> Byte {
        let address = self.registers.read_word(WordRegisterName::RegPC);
        let mut memory = self.memory.borrow_mut();
        unsafe { memory.read::<Byte>(address + (1 + position)) }
    } 
    fn word_operand(&mut self, position: Address) -> Word {
        let address = self.registers.read_word(WordRegisterName::RegPC);
        let mut memory = self.memory.borrow_mut();
        unsafe { memory.read::<Word>(address + (1 + position)) }
    } 
    fn fetch (&mut self) -> Byte {
        let address = self.registers.read_word(WordRegisterName::RegPC);
        let mut memory = self.memory.borrow_mut();
        unsafe { memory.read(address) }
    }

    pub fn step(&mut self) -> u8{
        let instruction = self.fetch();

        let cost = match instruction {
            0x00 => {
                self.registers.step_pc(1);
                self.nop();
                1
            }
            0x01 => {
                let value = self.word_operand(0);
                self.registers.step_pc(3);
                self.ld_word(WordRegister::new(RegBC), WordImmediate::new(value));
                3
            }
            0x02 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegisterIndirect::new(RegBC), ByteRegister::new(RegA));
                2
            }
            0x03 => {
                self.registers.step_pc(1);
                self.inc_word(WordRegister::new(RegBC));
                2
            }
            0x04 => {
                self.registers.step_pc(1);
                self.inc_byte(ByteRegister::new(RegB));
                1
            }
            0x05 => {
                self.registers.step_pc(1);
                self.dec_byte(ByteRegister::new(RegB));
                1
            }
            0x06 => {
                let value = self.byte_operand(0);
                self.registers.step_pc(2);
                self.ld_byte(ByteRegister::new(RegB), ByteImmediate::new(value));
                2
            }
            0x07 => {
                self.registers.step_pc(1);
                self.rlc(ByteRegister::new(RegA));
                1
            }
            0x08 => {
                let address = self.word_operand(0);
                self.registers.step_pc(3);
                self.ld_word(WordImmediateIndirect::new(address), WordRegister::new(RegSP));
                5
            }
            0x09 => {
                self.registers.step_pc(1);
                self.add_hl_word(WordRegister::new(RegBC));
                2
            }
            0x0A => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegA), ByteRegisterIndirect::new(RegBC));
                2
            }
            0x0B => {
                self.registers.step_pc(1);
                self.dec_word(WordRegister::new(RegBC));
                2
            }
            0x0C => {
                self.registers.step_pc(1);
                self.inc_byte(ByteRegister::new(RegC));
                1
            }
            0x0D => {
                self.registers.step_pc(1);
                self.dec_byte(ByteRegister::new(RegC));
                1
            }
            0x0E => {
                let value = self.byte_operand(0);
                self.registers.step_pc(2);
                self.ld_byte(ByteRegister::new(RegC), ByteImmediate::new(value));
                2
            }
            0x0F => {
                self.registers.step_pc(1);
                self.rrc(ByteRegister::new(RegA));
                1
            }
            0x10 => {
                self.registers.step_pc(1);
                // TODO: STOP instruction
                1
            }
            0x11 => {
                let value = self.word_operand(0);
                self.registers.step_pc(3);
                self.ld_word(WordRegister::new(RegDE), WordImmediate::new(value));
                3
            }
            0x12 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegisterIndirect::new(RegDE), ByteRegister::new(RegA));
                2
            }
            0x13 => {
                self.registers.step_pc(1);
                self.inc_word(WordRegister::new(RegDE));
                2
            }
            0x14 => {
                self.registers.step_pc(1);
                self.inc_byte(ByteRegister::new(RegD));
                1
            }
            0x15 => {
                self.registers.step_pc(1);
                self.dec_byte(ByteRegister::new(RegD));
                1
            }
            0x16 => {
                let value = self.byte_operand(0);
                self.registers.step_pc(2);
                self.ld_byte(ByteRegister::new(RegD), ByteImmediate::new(value));
                2
            }
            0x17 => {
                self.registers.step_pc(1);
                self.rl(ByteRegister::new(RegA));
                1
            }
            0x18 => {
                let offset = self.byte_operand(0).interpret_as_signed();
                self.registers.step_pc(2);
                self.jr(offset, ConditionCodes::NA);
                3
            }
            0x19 => {
                self.registers.step_pc(1);
                self.add_hl_word(WordRegister::new(RegDE));
                2
            }
            0x1A => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegA), ByteRegisterIndirect::new(RegDE));
                2
            }
            0x1B => {
                self.registers.step_pc(1);
                self.dec_word(WordRegister::new(RegDE));
                2
            }
            0x1C => {
                self.registers.step_pc(1);
                self.inc_byte(ByteRegister::new(RegE));
                1
            }
            0x1D => {
                self.registers.step_pc(1);
                self.dec_byte(ByteRegister::new(RegE));
                1
            }
            0x1E => {
                let value = self.byte_operand(0);
                self.registers.step_pc(2);
                self.ld_byte(ByteRegister::new(RegE), ByteImmediate::new(value));
                2
            }
            0x1F => {
                self.registers.step_pc(1);
                self.rr(ByteRegister::new(RegA));
                1
            }
            0x20 => {
                let offset = self.byte_operand(0).interpret_as_signed();
                self.registers.step_pc(2);
                let branched = self.jr(offset, ConditionCodes::NZ);
                if branched { 3 } else { 2 }
            }
            0x21 => {
                let value = self.word_operand(0);
                self.registers.step_pc(3);
                self.ld_word(WordRegister::new(RegHL), WordImmediate::new(value));
                3
            }
            0x22 => {
                self.registers.step_pc(1);
                self.ld_byte_0x22();
                2
            }
            0x23 => {
                self.registers.step_pc(1);
                self.inc_word(WordRegister::new(RegHL));
                2
            }
            0x24 => {
                self.registers.step_pc(1);
                self.inc_byte(ByteRegister::new(RegH));
                1
            }
            0x25 => {
                self.registers.step_pc(1);
                self.dec_byte(ByteRegister::new(RegH));
                1
            }
            0x26 => {
                let value = self.byte_operand(0);
                self.registers.step_pc(2);
                self.ld_byte(ByteRegister::new(RegH), ByteImmediate::new(value));
                2
            }
            0x27 => {
                self.registers.step_pc(1);
                self.daa();
                1
            }
            0x28 => {
                let offset = self.byte_operand(0).interpret_as_signed();
                self.registers.step_pc(2);
                let branched = self.jr(offset, ConditionCodes::Z);
                if branched { 3 } else { 2 }
            }
            0x29 => {
                self.registers.step_pc(1);
                self.add_hl_word(WordRegister::new(RegHL));
                2
            }
            0x2A => {
                self.registers.step_pc(1);
                self.ld_byte_0x2a();
                2
            }
            0x2B => {
                self.registers.step_pc(1);
                self.dec_word(WordRegister::new(RegHL));
                2
            }
            0x2C => {
                self.registers.step_pc(1);
                self.inc_byte(ByteRegister::new(RegL));
                1
            }
            0x2D => {
                self.registers.step_pc(1);
                self.dec_byte(ByteRegister::new(RegL));
                1
            }
            0x2E => {
                let value = self.byte_operand(0);
                self.registers.step_pc(2);
                self.ld_byte(ByteRegister::new(RegL), ByteImmediate::new(value));
                2
            }
            0x2F => {
                self.registers.step_pc(1);
                self.cpl();
                1
            }
            0x30 => {
                let offset = self.byte_operand(0).interpret_as_signed();
                self.registers.step_pc(2);
                let branched = self.jr(offset, ConditionCodes::NC);
                if branched { 3 } else { 2 }
            }
            0x31 => {
                let value = self.word_operand(0);
                self.registers.step_pc(3);
                self.ld_word(WordRegister::new(RegSP), WordImmediate::new(value));
                3
            }
            0x32 => {
                self.registers.step_pc(1);
                self.ld_byte_0x32();
                2
            }
            0x33 => {
                self.registers.step_pc(1);
                self.inc_word(WordRegister::new(RegSP));
                2
            }
            0x34 => {
                self.registers.step_pc(1);
                self.inc_byte(ByteRegisterIndirect::new(RegHL));
                3
            }
            0x35 => {
                self.registers.step_pc(1);
                self.dec_byte(ByteRegisterIndirect::new(RegHL));
                3
            }
            0x36 => {
                let value = self.byte_operand(0);
                self.registers.step_pc(2);
                self.ld_byte(ByteRegisterIndirect::new(RegHL), ByteImmediate::new(value));
                3
            }
            0x37 => {
                self.registers.step_pc(1);
                self.scf();
                1
            }
            0x38 => {
                let offset = self.byte_operand(0).interpret_as_signed();
                self.registers.step_pc(2);
                let branched = self.jr(offset, ConditionCodes::C);
                if branched { 3 } else { 2 }
            }
            0x39 => {
                self.registers.step_pc(1);
                self.add_hl_word(WordRegister::new(RegSP));
                2
            }
            0x3A => {
                self.registers.step_pc(1);
                self.ld_byte_0x3a();
                2
            }
            0x3B => {
                self.registers.step_pc(1);
                self.dec_word(WordRegister::new(RegSP));
                2
            }
            0x3C => {
                self.registers.step_pc(1);
                self.inc_byte(ByteRegister::new(RegA));
                1
            }
            0x3D => {
                self.registers.step_pc(1);
                self.dec_byte(ByteRegister::new(RegA));
                1
            }
            0x3E => {
                let value = self.byte_operand(0);
                self.registers.step_pc(2);
                self.ld_byte(ByteRegister::new(RegA), ByteImmediate::new(value));
                2
            }
            0x3F => {
                self.registers.step_pc(1);
                self.ccf();
                1
            }
            0x40 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegB), ByteRegister::new(RegB));
                1
            }
            _ => 0
        };
        cost
    }
}