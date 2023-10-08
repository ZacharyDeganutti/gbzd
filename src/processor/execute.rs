use crate::memory_gb::Byte;
use crate::memory_gb::ByteExt;
use crate::memory_gb::MemoryRegion;
use crate::memory_gb::Word;
use crate::processor::cpu::*;
use crate::processor::cpu::ByteRegisterName::*;
use crate::processor::cpu::WordRegisterName::*;

impl Cpu {
    fn byte_operand(&mut self) -> Byte {
        let address = self.registers.read_word(WordRegisterName::RegPC);
        let mut memory = self.memory.borrow_mut();
        memory.read::<Byte>(address + 1)
    } 
    fn word_operand(&mut self) -> Word {
        let address = self.registers.read_word(WordRegisterName::RegPC);
        let mut memory = self.memory.borrow_mut();
        memory.read::<Word>(address + 1)
    } 
    fn fetch (&mut self) -> Byte {
        let address = self.registers.read_word(WordRegisterName::RegPC);
        let mut memory = self.memory.borrow_mut();
        memory.read::<Byte>(address)
    }

    pub fn step(&mut self) -> u8 {
        let instruction = self.fetch();

        let cost = match instruction {
            0x00 => {
                self.registers.step_pc(1);
                self.nop();
                1
            }
            0x01 => {
                let value = self.word_operand();
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
                let value = self.byte_operand();
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
                let address = self.word_operand();
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
                let value = self.byte_operand();
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
                let value = self.word_operand();
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
                let value = self.byte_operand();
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
                let offset = self.byte_operand().interpret_as_signed();
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
                let value = self.byte_operand();
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
                let offset = self.byte_operand().interpret_as_signed();
                self.registers.step_pc(2);
                let branched = self.jr(offset, ConditionCodes::NZ);
                if branched { 3 } else { 2 }
            }
            0x21 => {
                let value = self.word_operand();
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
                let value = self.byte_operand();
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
                let offset = self.byte_operand().interpret_as_signed();
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
                let value = self.byte_operand();
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
                let offset = self.byte_operand().interpret_as_signed();
                self.registers.step_pc(2);
                let branched = self.jr(offset, ConditionCodes::NC);
                if branched { 3 } else { 2 }
            }
            0x31 => {
                let value = self.word_operand();
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
                let value = self.byte_operand();
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
                let offset = self.byte_operand().interpret_as_signed();
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
                let value = self.byte_operand();
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
            0xC0 => {
                self.registers.step_pc(1);
                let branched = self.ret(ConditionCodes::NZ);
                if branched { 5 } else { 2 }
            }
            0xC1 => {
                self.registers.step_pc(1);
                self.pop(RegBC);
                3
            }
            0xC2 => {
                let address = self.word_operand();
                self.registers.step_pc(3);
                let branched = self.jp(WordImmediate::new(address), ConditionCodes::NZ);
                if branched { 4 } else { 3 }
            }
            0xC3 => {
                let address = self.word_operand();
                self.registers.step_pc(3);
                self.jp(WordImmediate::new(address), ConditionCodes::NA);
                4
            }
            0xC4 => {
                let address = self.word_operand();
                self.registers.step_pc(3);
                let branched = self.call(address, ConditionCodes::NZ);
                if branched { 6 } else { 3 }
            }
            0xC5 => {
                self.registers.step_pc(1);
                self.push(RegBC);
                4
            }
            0xC6 => {
                let value = self.byte_operand();
                self.registers.step_pc(2);
                self.add_byte(ByteImmediate::new(value), false);
                2
            }
            0xC7 => {
                self.registers.step_pc(1);
                self.rst(0x00);
                4
            }
            0xC8 => {
                self.registers.step_pc(1);
                let branched = self.ret(ConditionCodes::Z);
                if branched { 5 } else { 2 }
            }
            0xC9 => {
                self.registers.step_pc(1);
                self.ret(ConditionCodes::NA);
                4
            }
            0xCA => {
                let address = self.word_operand();
                self.registers.step_pc(3);
                let branched = self.jp(WordImmediate::new(address), ConditionCodes::Z);
                if branched { 4 } else { 3 }
            }
            0xCB => {
                let op = self.byte_operand();
                self.registers.step_pc(1);
                self.step_cb(op)
            }
            0xCC => {
                let address = self.word_operand();
                self.registers.step_pc(3);
                let branched = self.call(address, ConditionCodes::Z);
                if branched { 6 } else { 3 }
            }
            0xCD => {
                let address = self.word_operand();
                self.registers.step_pc(3);
                self.call(address, ConditionCodes::NA);
                6
            }
            0xCE => {
                let value = self.byte_operand();
                self.registers.step_pc(2);
                self.add_byte(ByteImmediate::new(value), true);
                2
            }
            0xCF => {
                self.registers.step_pc(1);
                self.rst(0x08);
                4
            }
            0xD0 => {
                self.registers.step_pc(1);
                let branched = self.ret(ConditionCodes::NC);
                if branched { 5 } else { 2 }
            }
            0xD1 => {
                self.registers.step_pc(1);
                self.pop(RegDE);
                3
            }
            0xD2 => {
                let address = self.word_operand();
                self.registers.step_pc(3);
                let branched = self.jp(WordImmediate::new(address), ConditionCodes::NC);
                if branched { 4 } else { 3 }
            }
            0xD4 => {
                let address = self.word_operand();
                self.registers.step_pc(3);
                let branched = self.call(address, ConditionCodes::NC);
                if branched { 6 } else { 3 }
            }
            0xD5 => {
                self.registers.step_pc(1);
                self.push(RegDE);
                4
            }
            0xD6 => {
                let value = self.byte_operand();
                self.registers.step_pc(2);
                self.sub_byte(ByteImmediate::new(value), false);
                2
            }
            0xD7 => {
                self.registers.step_pc(1);
                self.rst(0x10);
                4
            }
            0xD8 => {
                self.registers.step_pc(1);
                let branched = self.ret(ConditionCodes::C);
                if branched { 5 } else { 2 }
            }
            0xD9 => {
                // TODO verify if reti = ret in practice
                self.registers.step_pc(1);
                self.ret(ConditionCodes::NA);
                4
            }
            0xDA => {
                let address = self.word_operand();
                self.registers.step_pc(3);
                let branched = self.jp(WordImmediate::new(address), ConditionCodes::Z);
                if branched { 4 } else { 3 }
            }
            0xDC => {
                let address = self.word_operand();
                self.registers.step_pc(3);
                let branched = self.call(address, ConditionCodes::C);
                if branched { 6 } else { 3 }
            }
            0xDE => {
                let value = self.byte_operand();
                self.registers.step_pc(2);
                self.sub_byte(ByteImmediate::new(value), true);
                2
            }
            0xDF => {
                self.registers.step_pc(1);
                self.rst(0x18);
                4
            }
            0xE0 => {
                let offset = self.byte_operand();
                self.registers.step_pc(2);
                self.ld_byte(ByteImmediateOffsetIndirect::new(offset), ByteRegister::new(RegA));
                3
            }
            0xE1 => {
                self.registers.step_pc(1);
                self.pop(RegHL);
                3
            }
            0xE2 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegisterOffsetIndirect::new(RegC), ByteRegister::new(RegA));
                2
            }
            0xE5 => {
                self.registers.step_pc(1);
                self.push(RegHL);
                4
            }
            0xE6 => {
                let value = self.byte_operand();
                self.registers.step_pc(2);
                self.and(ByteImmediate::new(value));
                2
            }
            0xE7 => {
                self.registers.step_pc(1);
                self.rst(0x20);
                4
            }
            0xE8 => {
                let offset = self.byte_operand().interpret_as_signed();
                self.registers.step_pc(2);
                self.add_sp_i8(RegSP, offset);
                4
            }
            0xE9 => {
                self.registers.step_pc(1);
                self.jp(WordRegister::new(RegHL), ConditionCodes::NA);
                1
            }
            0xEA => {
                let address = self.word_operand();
                self.registers.step_pc(3);
                self.ld_byte(ByteImmediateIndirect::new(address), ByteRegister::new(RegA));
                4
            }
            0xEE => {
                let value = self.byte_operand();
                self.registers.step_pc(2);
                self.xor(ByteImmediate::new(value));
                2
            }
            0xEF => {
                self.registers.step_pc(1);
                self.rst(0x28);
                4
            }
            0xF0 => {
                let offset = self.byte_operand();
                self.registers.step_pc(2);
                self.ld_byte(ByteRegister::new(RegA), ByteImmediateOffsetIndirect::new(offset));
                3
            }
            0xF1 => {
                self.registers.step_pc(1);
                self.pop(RegAF);
                3
            }
            0xF2 => {
                self.registers.step_pc(1);
                self.ld_byte(ByteRegister::new(RegA), ByteRegisterOffsetIndirect::new(RegC));
                2
            }
            0xF3 => {
                self.registers.step_pc(1);
                // TODO: DI
                1
            }
            0xF5 => {
                self.registers.step_pc(1);
                self.push(RegAF);
                4
            }
            0xF6 => {
                let value = self.byte_operand();
                self.registers.step_pc(2);
                self.or(ByteImmediate::new(value));
                2
            }
            0xF7 => {
                self.registers.step_pc(1);
                self.rst(0x30);
                4
            }
            0xF8 => {
                let offset = self.byte_operand().interpret_as_signed();
                self.registers.step_pc(2);
                self.add_sp_i8(RegHL, offset);
                3
            }
            0xF9 => {
                // TODO verify if reti = ret in practice
                self.registers.step_pc(1);
                self.ld_word(WordRegister::new(RegSP), WordRegister::new(RegHL));
                2
            }
            0xFA => {
                let address = self.word_operand();
                self.registers.step_pc(3);
                self.ld_byte(ByteRegister::new(RegA), ByteImmediateIndirect::new(address));
                4
            }
            0xFB => {
                self.registers.step_pc(1);
                // TODO: EI
                1
            }
            0xFE => {
                let value = self.byte_operand();
                self.registers.step_pc(2);
                self.cp(ByteImmediate::new(value));
                2
            }
            0xFF => {
                self.registers.step_pc(1);
                self.rst(0x38);
                4
            }
            _ => 0
        };
        cost
    }

    fn step_cb(&mut self, operand: Byte) -> u8 {
        let instruction = operand;
        self.registers.step_pc(1);
        match instruction {
            0x00 => {
                self.rlc(ByteRegister::new(RegB));
                2
            }
            0x01 => {
                self.rlc(ByteRegister::new(RegC));
                2
            }
            0x02 => {
                self.rlc(ByteRegister::new(RegD));
                2
            }
            0x03 => {
                self.rlc(ByteRegister::new(RegE));
                2
            }
            0x04 => {
                self.rlc(ByteRegister::new(RegH));
                2
            }
            0x05 => {
                self.rlc(ByteRegister::new(RegL));
                2
            }
            0x06 => {
                self.rlc(ByteRegisterIndirect::new(RegHL));
                4
            }
            0x07 => {
                self.rlc(ByteRegister::new(RegA));
                2
            }
            0x08 => {
                self.rrc(ByteRegister::new(RegB));
                2
            }
            0x09 => {
                self.rrc(ByteRegister::new(RegC));
                2
            }
            0x0A => {
                self.rrc(ByteRegister::new(RegD));
                2
            }
            0x0B => {
                self.rrc(ByteRegister::new(RegE));
                2
            }
            0x0C => {
                self.rrc(ByteRegister::new(RegH));
                2
            }
            0x0D => {
                self.rrc(ByteRegister::new(RegL));
                2
            }
            0x0E => {
                self.rrc(ByteRegisterIndirect::new(RegHL));
                4
            }
            0x0F => {
                self.rrc(ByteRegister::new(RegA));
                2
            }
            0x10 => {
                self.rl(ByteRegister::new(RegB));
                2
            }
            0x11 => {
                self.rl(ByteRegister::new(RegC));
                2
            }
            0x12 => {
                self.rl(ByteRegister::new(RegD));
                2
            }
            0x13 => {
                self.rl(ByteRegister::new(RegE));
                2
            }
            0x14 => {
                self.rl(ByteRegister::new(RegH));
                2
            }
            0x15 => {
                self.rl(ByteRegister::new(RegL));
                2
            }
            0x16 => {
                self.rl(ByteRegisterIndirect::new(RegHL));
                4
            }
            0x17 => {
                self.rl(ByteRegister::new(RegA));
                2
            }
            0x18 => {
                self.rr(ByteRegister::new(RegB));
                2
            }
            0x19 => {
                self.rr(ByteRegister::new(RegC));
                2
            }
            0x1A => {
                self.rr(ByteRegister::new(RegD));
                2
            }
            0x1B => {
                self.rr(ByteRegister::new(RegE));
                2
            }
            0x1C => {
                self.rr(ByteRegister::new(RegH));
                2
            }
            0x1D => {
                self.rr(ByteRegister::new(RegL));
                2
            }
            0x1E => {
                self.rr(ByteRegisterIndirect::new(RegHL));
                4
            }
            0x1F => {
                self.rr(ByteRegister::new(RegA));
                2
            }
            0x20 => {
                self.sla(ByteRegister::new(RegB));
                2
            }
            0x21 => {
                self.sla(ByteRegister::new(RegC));
                2
            }
            0x22 => {
                self.sla(ByteRegister::new(RegD));
                2
            }
            0x23 => {
                self.sla(ByteRegister::new(RegE));
                2
            }
            0x24 => {
                self.sla(ByteRegister::new(RegH));
                2
            }
            0x25 => {
                self.sla(ByteRegister::new(RegL));
                2
            }
            0x26 => {
                self.sla(ByteRegisterIndirect::new(RegHL));
                4
            }
            0x27 => {
                self.sla(ByteRegister::new(RegA));
                2
            }
            0x28 => {
                self.sra(ByteRegister::new(RegB));
                2
            }
            0x29 => {
                self.sra(ByteRegister::new(RegC));
                2
            }
            0x2A => {
                self.sra(ByteRegister::new(RegD));
                2
            }
            0x2B => {
                self.sra(ByteRegister::new(RegE));
                2
            }
            0x2C => {
                self.sra(ByteRegister::new(RegH));
                2
            }
            0x2D => {
                self.sra(ByteRegister::new(RegL));
                2
            }
            0x2E => {
                self.sra(ByteRegisterIndirect::new(RegHL));
                4
            }
            0x2F => {
                self.sra(ByteRegister::new(RegA));
                2
            }
            0x30 => {
                self.swap(ByteRegister::new(RegB));
                2
            }
            0x31 => {
                self.swap(ByteRegister::new(RegC));
                2
            }
            0x32 => {
                self.swap(ByteRegister::new(RegD));
                2
            }
            0x33 => {
                self.swap(ByteRegister::new(RegE));
                2
            }
            0x34 => {
                self.swap(ByteRegister::new(RegH));
                2
            }
            0x35 => {
                self.swap(ByteRegister::new(RegL));
                2
            }
            0x36 => {
                self.swap(ByteRegisterIndirect::new(RegHL));
                4
            }
            0x37 => {
                self.swap(ByteRegister::new(RegA));
                2
            }
            0x38 => {
                self.srl(ByteRegister::new(RegB));
                2
            }
            0x39 => {
                self.srl(ByteRegister::new(RegC));
                2
            }
            0x3A => {
                self.srl(ByteRegister::new(RegD));
                2
            }
            0x3B => {
                self.srl(ByteRegister::new(RegE));
                2
            }
            0x3C => {
                self.srl(ByteRegister::new(RegH));
                2
            }
            0x3D => {
                self.srl(ByteRegister::new(RegL));
                2
            }
            0x3E => {
                self.srl(ByteRegisterIndirect::new(RegHL));
                4
            }
            0x3F => {
                self.srl(ByteRegister::new(RegA));
                2
            }
            0x40 => {
                self.bit(0, ByteRegister::new(RegB));
                2
            }
            0x41 => {
                self.bit(0, ByteRegister::new(RegC));
                2
            }
            0x42 => {
                self.bit(0, ByteRegister::new(RegD));
                2
            }
            0x43 => {
                self.bit(0, ByteRegister::new(RegE));
                2
            }
            0x44 => {
                self.bit(0, ByteRegister::new(RegH));
                2
            }
            0x45 => {
                self.bit(0, ByteRegister::new(RegL));
                2
            }
            0x46 => {
                self.bit(0, ByteRegisterIndirect::new(RegHL));
                3
            }
            0x47 => {
                self.bit(0, ByteRegister::new(RegA));
                2
            }
            0x48 => {
                self.bit(1, ByteRegister::new(RegB));
                2
            }
            0x49 => {
                self.bit(1, ByteRegister::new(RegC));
                2
            }
            0x4A => {
                self.bit(1, ByteRegister::new(RegD));
                2
            }
            0x4B => {
                self.bit(1, ByteRegister::new(RegE));
                2
            }
            0x4C => {
                self.bit(1, ByteRegister::new(RegH));
                2
            }
            0x4D => {
                self.bit(1, ByteRegister::new(RegL));
                2
            }
            0x4E => {
                self.bit(1, ByteRegisterIndirect::new(RegHL));
                3
            }
            0x4F => {
                self.bit(1, ByteRegister::new(RegA));
                2
            }
            0x50 => {
                self.bit(2, ByteRegister::new(RegB));
                2
            }
            0x51 => {
                self.bit(2, ByteRegister::new(RegC));
                2
            }
            0x52 => {
                self.bit(2, ByteRegister::new(RegD));
                2
            }
            0x53 => {
                self.bit(2, ByteRegister::new(RegE));
                2
            }
            0x54 => {
                self.bit(2, ByteRegister::new(RegH));
                2
            }
            0x55 => {
                self.bit(2, ByteRegister::new(RegL));
                2
            }
            0x56 => {
                self.bit(2, ByteRegisterIndirect::new(RegHL));
                3
            }
            0x57 => {
                self.bit(2, ByteRegister::new(RegA));
                2
            }
            0x58 => {
                self.bit(3, ByteRegister::new(RegB));
                2
            }
            0x59 => {
                self.bit(3, ByteRegister::new(RegC));
                2
            }
            0x5A => {
                self.bit(3, ByteRegister::new(RegD));
                2
            }
            0x5B => {
                self.bit(3, ByteRegister::new(RegE));
                2
            }
            0x5C => {
                self.bit(3, ByteRegister::new(RegH));
                2
            }
            0x5D => {
                self.bit(3, ByteRegister::new(RegL));
                2
            }
            0x5E => {
                self.bit(3, ByteRegisterIndirect::new(RegHL));
                3
            }
            0x5F => {
                self.bit(3, ByteRegister::new(RegA));
                2
            }
            0x60 => {
                self.bit(4, ByteRegister::new(RegB));
                2
            }
            0x61 => {
                self.bit(4, ByteRegister::new(RegC));
                2
            }
            0x62 => {
                self.bit(4, ByteRegister::new(RegD));
                2
            }
            0x63 => {
                self.bit(4, ByteRegister::new(RegE));
                2
            }
            0x64 => {
                self.bit(4, ByteRegister::new(RegH));
                2
            }
            0x65 => {
                self.bit(4, ByteRegister::new(RegL));
                2
            }
            0x66 => {
                self.bit(4, ByteRegisterIndirect::new(RegHL));
                3
            }
            0x67 => {
                self.bit(4, ByteRegister::new(RegA));
                2
            }
            0x68 => {
                self.bit(5, ByteRegister::new(RegB));
                2
            }
            0x69 => {
                self.bit(5, ByteRegister::new(RegC));
                2
            }
            0x6A => {
                self.bit(5, ByteRegister::new(RegD));
                2
            }
            0x6B => {
                self.bit(5, ByteRegister::new(RegE));
                2
            }
            0x6C => {
                self.bit(5, ByteRegister::new(RegH));
                2
            }
            0x6D => {
                self.bit(5, ByteRegister::new(RegL));
                2
            }
            0x6E => {
                self.bit(5, ByteRegisterIndirect::new(RegHL));
                3
            }
            0x6F => {
                self.bit(5, ByteRegister::new(RegA));
                2
            }
            0x70 => {
                self.bit(6, ByteRegister::new(RegB));
                2
            }
            0x71 => {
                self.bit(6, ByteRegister::new(RegC));
                2
            }
            0x72 => {
                self.bit(6, ByteRegister::new(RegD));
                2
            }
            0x73 => {
                self.bit(6, ByteRegister::new(RegE));
                2
            }
            0x74 => {
                self.bit(6, ByteRegister::new(RegH));
                2
            }
            0x75 => {
                self.bit(6, ByteRegister::new(RegL));
                2
            }
            0x76 => {
                self.bit(6, ByteRegisterIndirect::new(RegHL));
                3
            }
            0x77 => {
                self.bit(6, ByteRegister::new(RegA));
                2
            }
            0x78 => {
                self.bit(7, ByteRegister::new(RegB));
                2
            }
            0x79 => {
                self.bit(7, ByteRegister::new(RegC));
                2
            }
            0x7A => {
                self.bit(7, ByteRegister::new(RegD));
                2
            }
            0x7B => {
                self.bit(7, ByteRegister::new(RegE));
                2
            }
            0x7C => {
                self.bit(7, ByteRegister::new(RegH));
                2
            }
            0x7D => {
                self.bit(7, ByteRegister::new(RegL));
                2
            }
            0x7E => {
                self.bit(7, ByteRegisterIndirect::new(RegHL));
                3
            }
            0x7F => {
                self.bit(7, ByteRegister::new(RegA));
                2
            }
            0x80 => {
                self.res(0, ByteRegister::new(RegB));
                2
            }
            0x81 => {
                self.res(0, ByteRegister::new(RegC));
                2
            }
            0x82 => {
                self.res(0, ByteRegister::new(RegD));
                2
            }
            0x83 => {
                self.res(0, ByteRegister::new(RegE));
                2
            }
            0x84 => {
                self.res(0, ByteRegister::new(RegH));
                2
            }
            0x85 => {
                self.res(0, ByteRegister::new(RegL));
                2
            }
            0x86 => {
                self.res(0, ByteRegisterIndirect::new(RegHL));
                4
            }
            0x87 => {
                self.res(0, ByteRegister::new(RegA));
                2
            }
            0x88 => {
                self.res(1, ByteRegister::new(RegB));
                2
            }
            0x89 => {
                self.res(1, ByteRegister::new(RegC));
                2
            }
            0x8A => {
                self.res(1, ByteRegister::new(RegD));
                2
            }
            0x8B => {
                self.res(1, ByteRegister::new(RegE));
                2
            }
            0x8C => {
                self.res(1, ByteRegister::new(RegH));
                2
            }
            0x8D => {
                self.res(1, ByteRegister::new(RegL));
                2
            }
            0x8E => {
                self.res(1, ByteRegisterIndirect::new(RegHL));
                4
            }
            0x8F => {
                self.res(1, ByteRegister::new(RegA));
                2
            }
            0x90 => {
                self.res(2, ByteRegister::new(RegB));
                2
            }
            0x91 => {
                self.res(2, ByteRegister::new(RegC));
                2
            }
            0x92 => {
                self.res(2, ByteRegister::new(RegD));
                2
            }
            0x93 => {
                self.res(2, ByteRegister::new(RegE));
                2
            }
            0x94 => {
                self.res(2, ByteRegister::new(RegH));
                2
            }
            0x95 => {
                self.res(2, ByteRegister::new(RegL));
                2
            }
            0x96 => {
                self.res(2, ByteRegisterIndirect::new(RegHL));
                4
            }
            0x97 => {
                self.res(2, ByteRegister::new(RegA));
                2
            }
            0x98 => {
                self.res(3, ByteRegister::new(RegB));
                2
            }
            0x99 => {
                self.res(3, ByteRegister::new(RegC));
                2
            }
            0x9A => {
                self.res(3, ByteRegister::new(RegD));
                2
            }
            0x9B => {
                self.res(3, ByteRegister::new(RegE));
                2
            }
            0x9C => {
                self.res(3, ByteRegister::new(RegH));
                2
            }
            0x9D => {
                self.res(3, ByteRegister::new(RegL));
                2
            }
            0x9E => {
                self.res(3, ByteRegisterIndirect::new(RegHL));
                4
            }
            0x9F => {
                self.res(3, ByteRegister::new(RegA));
                2
            }
            0xA0 => {
                self.res(4, ByteRegister::new(RegB));
                2
            }
            0xA1 => {
                self.res(4, ByteRegister::new(RegC));
                2
            }
            0xA2 => {
                self.res(4, ByteRegister::new(RegD));
                2
            }
            0xA3 => {
                self.res(4, ByteRegister::new(RegE));
                2
            }
            0xA4 => {
                self.res(4, ByteRegister::new(RegH));
                2
            }
            0xA5 => {
                self.res(4, ByteRegister::new(RegL));
                2
            }
            0xA6 => {
                self.res(4, ByteRegisterIndirect::new(RegHL));
                4
            }
            0xA7 => {
                self.res(4, ByteRegister::new(RegA));
                2
            }
            0xA8 => {
                self.res(5, ByteRegister::new(RegB));
                2
            }
            0xA9 => {
                self.res(5, ByteRegister::new(RegC));
                2
            }
            0xAA => {
                self.res(5, ByteRegister::new(RegD));
                2
            }
            0xAB => {
                self.res(5, ByteRegister::new(RegE));
                2
            }
            0xAC => {
                self.res(5, ByteRegister::new(RegH));
                2
            }
            0xAD => {
                self.res(5, ByteRegister::new(RegL));
                2
            }
            0xAE => {
                self.res(5, ByteRegisterIndirect::new(RegHL));
                4
            }
            0xAF => {
                self.res(5, ByteRegister::new(RegA));
                2
            }
            0xB0 => {
                self.res(6, ByteRegister::new(RegB));
                2
            }
            0xB1 => {
                self.res(6, ByteRegister::new(RegC));
                2
            }
            0xB2 => {
                self.res(6, ByteRegister::new(RegD));
                2
            }
            0xB3 => {
                self.res(6, ByteRegister::new(RegE));
                2
            }
            0xB4 => {
                self.res(6, ByteRegister::new(RegH));
                2
            }
            0xB5 => {
                self.res(6, ByteRegister::new(RegL));
                2
            }
            0xB6 => {
                self.res(6, ByteRegisterIndirect::new(RegHL));
                4
            }
            0xB7 => {
                self.res(6, ByteRegister::new(RegA));
                2
            }
            0xB8 => {
                self.res(7, ByteRegister::new(RegB));
                2
            }
            0xB9 => {
                self.res(7, ByteRegister::new(RegC));
                2
            }
            0xBA => {
                self.res(7, ByteRegister::new(RegD));
                2
            }
            0xBB => {
                self.res(7, ByteRegister::new(RegE));
                2
            }
            0xBC => {
                self.res(7, ByteRegister::new(RegH));
                2
            }
            0xBD => {
                self.res(7, ByteRegister::new(RegL));
                2
            }
            0xBE => {
                self.res(7, ByteRegisterIndirect::new(RegHL));
                4
            }
            0xBF => {
                self.res(7, ByteRegister::new(RegA));
                2
            }
            0xC0 => {
                self.set(0, ByteRegister::new(RegB));
                2
            }
            0xC1 => {
                self.set(0, ByteRegister::new(RegC));
                2
            }
            0xC2 => {
                self.set(0, ByteRegister::new(RegD));
                2
            }
            0xC3 => {
                self.set(0, ByteRegister::new(RegE));
                2
            }
            0xC4 => {
                self.set(0, ByteRegister::new(RegH));
                2
            }
            0xC5 => {
                self.set(0, ByteRegister::new(RegL));
                2
            }
            0xC6 => {
                self.set(0, ByteRegisterIndirect::new(RegHL));
                4
            }
            0xC7 => {
                self.set(0, ByteRegister::new(RegA));
                2
            }
            0xC8 => {
                self.set(1, ByteRegister::new(RegB));
                2
            }
            0xC9 => {
                self.set(1, ByteRegister::new(RegC));
                2
            }
            0xCA => {
                self.set(1, ByteRegister::new(RegD));
                2
            }
            0xCB => {
                self.set(1, ByteRegister::new(RegE));
                2
            }
            0xCC => {
                self.set(1, ByteRegister::new(RegH));
                2
            }
            0xCD => {
                self.set(1, ByteRegister::new(RegL));
                2
            }
            0xCE => {
                self.set(1, ByteRegisterIndirect::new(RegHL));
                4
            }
            0xCF => {
                self.set(1, ByteRegister::new(RegA));
                2
            }
            0xD0 => {
                self.set(2, ByteRegister::new(RegB));
                2
            }
            0xD1 => {
                self.set(2, ByteRegister::new(RegC));
                2
            }
            0xD2 => {
                self.set(2, ByteRegister::new(RegD));
                2
            }
            0xD3 => {
                self.set(2, ByteRegister::new(RegE));
                2
            }
            0xD4 => {
                self.set(2, ByteRegister::new(RegH));
                2
            }
            0xD5 => {
                self.set(2, ByteRegister::new(RegL));
                2
            }
            0xD6 => {
                self.set(2, ByteRegisterIndirect::new(RegHL));
                4
            }
            0xD7 => {
                self.set(2, ByteRegister::new(RegA));
                2
            }
            0xD8 => {
                self.set(3, ByteRegister::new(RegB));
                2
            }
            0xD9 => {
                self.set(3, ByteRegister::new(RegC));
                2
            }
            0xDA => {
                self.set(3, ByteRegister::new(RegD));
                2
            }
            0xDB => {
                self.set(3, ByteRegister::new(RegE));
                2
            }
            0xDC => {
                self.set(3, ByteRegister::new(RegH));
                2
            }
            0xDD => {
                self.set(3, ByteRegister::new(RegL));
                2
            }
            0xDE => {
                self.set(3, ByteRegisterIndirect::new(RegHL));
                4
            }
            0xDF => {
                self.set(3, ByteRegister::new(RegA));
                2
            }
            0xE0 => {
                self.set(4, ByteRegister::new(RegB));
                2
            }
            0xE1 => {
                self.set(4, ByteRegister::new(RegC));
                2
            }
            0xE2 => {
                self.set(4, ByteRegister::new(RegD));
                2
            }
            0xE3 => {
                self.set(4, ByteRegister::new(RegE));
                2
            }
            0xE4 => {
                self.set(4, ByteRegister::new(RegH));
                2
            }
            0xE5 => {
                self.set(4, ByteRegister::new(RegL));
                2
            }
            0xE6 => {
                self.set(4, ByteRegisterIndirect::new(RegHL));
                4
            }
            0xE7 => {
                self.set(4, ByteRegister::new(RegA));
                2
            }
            0xE8 => {
                self.set(5, ByteRegister::new(RegB));
                2
            }
            0xE9 => {
                self.set(5, ByteRegister::new(RegC));
                2
            }
            0xEA => {
                self.set(5, ByteRegister::new(RegD));
                2
            }
            0xEB => {
                self.set(5, ByteRegister::new(RegE));
                2
            }
            0xEC => {
                self.set(5, ByteRegister::new(RegH));
                2
            }
            0xED => {
                self.set(5, ByteRegister::new(RegL));
                2
            }
            0xEE => {
                self.set(5, ByteRegisterIndirect::new(RegHL));
                4
            }
            0xEF => {
                self.set(5, ByteRegister::new(RegA));
                2
            }
            0xF0 => {
                self.set(6, ByteRegister::new(RegB));
                2
            }
            0xF1 => {
                self.set(6, ByteRegister::new(RegC));
                2
            }
            0xF2 => {
                self.set(6, ByteRegister::new(RegD));
                2
            }
            0xF3 => {
                self.set(6, ByteRegister::new(RegE));
                2
            }
            0xF4 => {
                self.set(6, ByteRegister::new(RegH));
                2
            }
            0xF5 => {
                self.set(6, ByteRegister::new(RegL));
                2
            }
            0xF6 => {
                self.set(6, ByteRegisterIndirect::new(RegHL));
                4
            }
            0xF7 => {
                self.set(6, ByteRegister::new(RegA));
                2
            }
            0xF8 => {
                self.set(7, ByteRegister::new(RegB));
                2
            }
            0xF9 => {
                self.set(7, ByteRegister::new(RegC));
                2
            }
            0xFA => {
                self.set(7, ByteRegister::new(RegD));
                2
            }
            0xFB => {
                self.set(7, ByteRegister::new(RegE));
                2
            }
            0xFC => {
                self.set(7, ByteRegister::new(RegH));
                2
            }
            0xFD => {
                self.set(7, ByteRegister::new(RegL));
                2
            }
            0xFE => {
                self.set(7, ByteRegisterIndirect::new(RegHL));
                4
            }
            0xFF => {
                self.set(7, ByteRegister::new(RegA));
                2
            }
        }
    }
}