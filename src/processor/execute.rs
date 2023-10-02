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
            0x41 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegB), ByteRegister::new(RegC));
                1
            }
            0x42 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegB), ByteRegister::new(RegD));
                1
            }
            0x43 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegB), ByteRegister::new(RegE));
                1
            }
            0x44 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegB), ByteRegister::new(RegH));
                1
            }
            0x45 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegB), ByteRegister::new(RegL));
                1
            }
            0x46 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegB), ByteRegisterIndirect::new(RegHL));
                2
            }
            0x47 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegB), ByteRegister::new(RegA));
                1
            }
            0x48 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegC), ByteRegister::new(RegB));
                1
            }
            0x49 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegC), ByteRegister::new(RegC));
                1
            }
            0x4A => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegC), ByteRegister::new(RegD));
                1
            }
            0x4B => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegC), ByteRegister::new(RegE));
                1
            }
            0x4C => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegC), ByteRegister::new(RegH));
                1
            }
            0x4D => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegC), ByteRegister::new(RegL));
                1
            }
            0x4E => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegC), ByteRegisterIndirect::new(RegHL));
                2
            }
            0x4F => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegC), ByteRegister::new(RegA));
                1
            }
            0x50 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegD), ByteRegister::new(RegB));
                1
            }
            0x51 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegD), ByteRegister::new(RegC));
                1
            }
            0x52 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegD), ByteRegister::new(RegD));
                1
            }
            0x53 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegD), ByteRegister::new(RegE));
                1
            }
            0x54 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegD), ByteRegister::new(RegH));
                1
            }
            0x55 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegD), ByteRegister::new(RegL));
                1
            }
            0x56 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegD), ByteRegisterIndirect::new(RegHL));
                2
            }
            0x57 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegD), ByteRegister::new(RegA));
                1
            }
            0x58 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegE), ByteRegister::new(RegB));
                1
            }
            0x59 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegE), ByteRegister::new(RegC));
                1
            }
            0x5A => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegE), ByteRegister::new(RegD));
                1
            }
            0x5B => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegE), ByteRegister::new(RegE));
                1
            }
            0x5C => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegE), ByteRegister::new(RegH));
                1
            }
            0x5D => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegE), ByteRegister::new(RegL));
                1
            }
            0x5E => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegE), ByteRegisterIndirect::new(RegHL));
                2
            }
            0x5F => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegE), ByteRegister::new(RegA));
                1
            }
            0x60 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegH), ByteRegister::new(RegB));
                1
            }
            0x61 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegH), ByteRegister::new(RegC));
                1
            }
            0x62 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegH), ByteRegister::new(RegD));
                1
            }
            0x63 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegH), ByteRegister::new(RegE));
                1
            }
            0x64 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegH), ByteRegister::new(RegH));
                1
            }
            0x65 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegH), ByteRegister::new(RegL));
                1
            }
            0x66 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegH), ByteRegisterIndirect::new(RegHL));
                2
            }
            0x67 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegH), ByteRegister::new(RegA));
                1
            }
            0x68 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegL), ByteRegister::new(RegB));
                1
            }
            0x69 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegL), ByteRegister::new(RegC));
                1
            }
            0x6A => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegL), ByteRegister::new(RegD));
                1
            }
            0x6B => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegL), ByteRegister::new(RegE));
                1
            }
            0x6C => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegL), ByteRegister::new(RegH));
                1
            }
            0x6D => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegL), ByteRegister::new(RegL));
                1
            }
            0x6E => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegL), ByteRegisterIndirect::new(RegHL));
                2
            }
            0x6F => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegL), ByteRegister::new(RegA));
                1
            }
            0x70 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegisterIndirect::new(RegHL), ByteRegister::new(RegB));
                2
            }
            0x71 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegisterIndirect::new(RegHL), ByteRegister::new(RegC));
                2
            }
            0x72 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegisterIndirect::new(RegHL), ByteRegister::new(RegD));
                2
            }
            0x73 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegisterIndirect::new(RegHL), ByteRegister::new(RegE));
                2
            }
            0x74 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegisterIndirect::new(RegHL), ByteRegister::new(RegH));
                2
            }
            0x75 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegisterIndirect::new(RegHL), ByteRegister::new(RegL));
                2
            }
            0x76 => {
                self.registers.step_pc(1);
                // TODO: IMPLEMENT HALT
                1
            }
            0x77 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegisterIndirect::new(RegHL), ByteRegister::new(RegA));
                2
            }
            0x78 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegA), ByteRegister::new(RegB));
                1
            }
            0x79 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegA), ByteRegister::new(RegC));
                1
            }
            0x7A => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegA), ByteRegister::new(RegD));
                1
            }
            0x7B => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegA), ByteRegister::new(RegE));
                1
            }
            0x7C => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegA), ByteRegister::new(RegH));
                1
            }
            0x7D => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegA), ByteRegister::new(RegL));
                1
            }
            0x7E => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegA), ByteRegisterIndirect::new(RegHL));
                2
            }
            0x7F => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegA), ByteRegister::new(RegA));
                1
            }
            0x80 => {
                self.registers.step_pc(1);
                self.add_byte(ByteRegister::new(RegB), false);
                1
            }
            0x81 => {
                self.registers.step_pc(1);
                self.add_byte(ByteRegister::new(RegC), false);
                1
            }
            0x82 => {
                self.registers.step_pc(1);
                self.add_byte(ByteRegister::new(RegD), false);
                1
            }
            0x83 => {
                self.registers.step_pc(1);
                self.add_byte(ByteRegister::new(RegE), false);
                1
            }
            0x84 => {
                self.registers.step_pc(1);
                self.add_byte(ByteRegister::new(RegH), false);
                1
            }
            0x85 => {
                self.registers.step_pc(1);
                self.add_byte(ByteRegister::new(RegL), false);
                1
            }
            0x86 => {
                self.registers.step_pc(1);
                self.add_byte(ByteRegisterIndirect::new(RegHL), false);
                2
            }
            0x87 => {
                self.registers.step_pc(1);
                self.add_byte(ByteRegister::new(RegA), false);
                1
            }
            0x88 => {
                self.registers.step_pc(1);
                self.add_byte(ByteRegister::new(RegB), true);
                1
            }
            0x89 => {
                self.registers.step_pc(1);
                self.add_byte(ByteRegister::new(RegC), true);
                1
            }
            0x8A => {
                self.registers.step_pc(1);
                self.add_byte(ByteRegister::new(RegD), true);
                1
            }
            0x8B => {
                self.registers.step_pc(1);
                self.add_byte(ByteRegister::new(RegE), true);
                1
            }
            0x8C => {
                self.registers.step_pc(1);
                self.add_byte(ByteRegister::new(RegH), true);
                1
            }
            0x8D => {
                self.registers.step_pc(1);
                self.add_byte(ByteRegister::new(RegL), true);
                1
            }
            0x8E => {
                self.registers.step_pc(1);
                self.add_byte(ByteRegisterIndirect::new(RegHL), true);
                2
            }
            0x8F => {
                self.registers.step_pc(1);
                self.add_byte(ByteRegister::new(RegA), true);
                1
            }
            0x90 => {
                self.registers.step_pc(1);
                self.sub_byte(ByteRegister::new(RegB), false);
                1
            }
            0x91 => {
                self.registers.step_pc(1);
                self.sub_byte(ByteRegister::new(RegC), false);
                1
            }
            0x92 => {
                self.registers.step_pc(1);
                self.sub_byte(ByteRegister::new(RegD), false);
                1
            }
            0x93 => {
                self.registers.step_pc(1);
                self.sub_byte(ByteRegister::new(RegE), false);
                1
            }
            0x94 => {
                self.registers.step_pc(1);
                self.sub_byte(ByteRegister::new(RegH), false);
                1
            }
            0x95 => {
                self.registers.step_pc(1);
                self.sub_byte(ByteRegister::new(RegL), false);
                1
            }
            0x96 => {
                self.registers.step_pc(1);
                self.sub_byte(ByteRegisterIndirect::new(RegHL), false);
                2
            }
            0x97 => {
                self.registers.step_pc(1);
                self.sub_byte(ByteRegister::new(RegA), false);
                1
            }
            0x98 => {
                self.registers.step_pc(1);
                self.sub_byte(ByteRegister::new(RegB), true);
                1
            }
            0x99 => {
                self.registers.step_pc(1);
                self.sub_byte(ByteRegister::new(RegC), true);
                1
            }
            0x9A => {
                self.registers.step_pc(1);
                self.sub_byte(ByteRegister::new(RegD), true);
                1
            }
            0x9B => {
                self.registers.step_pc(1);
                self.sub_byte(ByteRegister::new(RegE), true);
                1
            }
            0x9C => {
                self.registers.step_pc(1);
                self.sub_byte(ByteRegister::new(RegH), true);
                1
            }
            0x9D => {
                self.registers.step_pc(1);
                self.sub_byte(ByteRegister::new(RegL), true);
                1
            }
            0x9E => {
                self.registers.step_pc(1);
                self.sub_byte(ByteRegisterIndirect::new(RegHL), true);
                2
            }
            0x9F => {
                self.registers.step_pc(1);
                self.sub_byte(ByteRegister::new(RegA), true);
                1
            }
            0xA0 => {
                self.registers.step_pc(1);
                self.and(ByteRegister::new(RegB));
                1
            }
            0xA1 => {
                self.registers.step_pc(1);
                self.and(ByteRegister::new(RegC));
                1
            }
            0xA2 => {
                self.registers.step_pc(1);
                self.and(ByteRegister::new(RegD));
                1
            }
            0xA3 => {
                self.registers.step_pc(1);
                self.and(ByteRegister::new(RegE));
                1
            }
            0xA4 => {
                self.registers.step_pc(1);
                self.and(ByteRegister::new(RegH));
                1
            }
            0xA5 => {
                self.registers.step_pc(1);
                self.and(ByteRegister::new(RegL));
                1
            }
            0xA6 => {
                self.registers.step_pc(1);
                self.and(ByteRegisterIndirect::new(RegHL));
                2
            }
            0xA7 => {
                self.registers.step_pc(1);
                self.and(ByteRegister::new(RegA));
                1
            }
            0xA8 => {
                self.registers.step_pc(1);
                self.xor(ByteRegister::new(RegB));
                1
            }
            0xA9 => {
                self.registers.step_pc(1);
                self.xor(ByteRegister::new(RegC));
                1
            }
            0xAA => {
                self.registers.step_pc(1);
                self.xor(ByteRegister::new(RegD));
                1
            }
            0xAB => {
                self.registers.step_pc(1);
                self.xor(ByteRegister::new(RegE));
                1
            }
            0xAC => {
                self.registers.step_pc(1);
                self.xor(ByteRegister::new(RegH));
                1
            }
            0xAD => {
                self.registers.step_pc(1);
                self.xor(ByteRegister::new(RegL));
                1
            }
            0xAE => {
                self.registers.step_pc(1);
                self.xor(ByteRegisterIndirect::new(RegHL));
                2
            }
            0xAF => {
                self.registers.step_pc(1);
                self.xor(ByteRegister::new(RegA));
                1
            }
            0xB0 => {
                self.registers.step_pc(1);
                self.or(ByteRegister::new(RegB));
                1
            }
            0xB1 => {
                self.registers.step_pc(1);
                self.or(ByteRegister::new(RegC));
                1
            }
            0xB2 => {
                self.registers.step_pc(1);
                self.or(ByteRegister::new(RegD));
                1
            }
            0xB3 => {
                self.registers.step_pc(1);
                self.or(ByteRegister::new(RegE));
                1
            }
            0xB4 => {
                self.registers.step_pc(1);
                self.or(ByteRegister::new(RegH));
                1
            }
            0xB5 => {
                self.registers.step_pc(1);
                self.or(ByteRegister::new(RegL));
                1
            }
            0xB6 => {
                self.registers.step_pc(1);
                self.or(ByteRegisterIndirect::new(RegHL));
                2
            }
            0xB7 => {
                self.registers.step_pc(1);
                self.or(ByteRegister::new(RegA));
                1
            }
            0xB8 => {
                self.registers.step_pc(1);
                self.cp(ByteRegister::new(RegB));
                1
            }
            0xB9 => {
                self.registers.step_pc(1);
                self.cp(ByteRegister::new(RegC));
                1
            }
            0xBA => {
                self.registers.step_pc(1);
                self.cp(ByteRegister::new(RegD));
                1
            }
            0xBB => {
                self.registers.step_pc(1);
                self.cp(ByteRegister::new(RegE));
                1
            }
            0xBC => {
                self.registers.step_pc(1);
                self.cp(ByteRegister::new(RegH));
                1
            }
            0xBD => {
                self.registers.step_pc(1);
                self.cp(ByteRegister::new(RegL));
                1
            }
            0xBE => {
                self.registers.step_pc(1);
                self.cp(ByteRegisterIndirect::new(RegHL));
                2
            }
            0xBF => {
                self.registers.step_pc(1);
                self.cp(ByteRegister::new(RegA));
                1
            }
            _ => 0
        };
        cost
    }
}