use crate::memory_gb::Address;
use crate::memory_gb::Byte;
use crate::memory_gb::Signed;
use crate::memory_gb::Word;
use crate::memory_gb::MemoryRegion;
use crate::processor::cpu::*;


impl Cpu {
    pub fn ld_byte<T: WriteByte, U: ReadByte>(&mut self, dest: T, src: U) {
        let source_value = src.read_byte(self);
        dest.write_byte(self, source_value);
    }

    // Indirect load from register A into address in register HL. Increment HL afterwards
    pub fn ld_byte_0x22(&mut self) {
        self.ld_byte(ByteRegisterIndirect::new(WordRegisterName::RegHL), ByteRegister::new(ByteRegisterName::RegA));
        let pre_increment = self.registers.read_word(WordRegisterName::RegHL);
        self.registers.write_word(WordRegisterName::RegHL, pre_increment + 1);
    }

    // Indirect load from address in register HL into register A. Increment HL afterwards
    pub fn ld_byte_0x2a(&mut self) {
        self.ld_byte(ByteRegister::new(ByteRegisterName::RegA), ByteRegisterIndirect::new(WordRegisterName::RegHL));
        let pre_increment = self.registers.read_word(WordRegisterName::RegHL);
        self.registers.write_word(WordRegisterName::RegHL, pre_increment + 1);
    }

    // Indirect load from register A into address in register HL. Decrement HL afterwards
    pub fn ld_byte_0x32(&mut self) {
        self.ld_byte(ByteRegisterIndirect::new(WordRegisterName::RegHL), ByteRegister::new(ByteRegisterName::RegA));
        let pre_decrement = self.registers.read_word(WordRegisterName::RegHL);
        self.registers.write_word(WordRegisterName::RegHL, pre_decrement - 1);
    }

    // Indirect load from address in register HL into register A. Decrement HL afterwards
    pub fn ld_byte_0x3a(&mut self) {
        self.ld_byte(ByteRegister::new(ByteRegisterName::RegA), ByteRegisterIndirect::new(WordRegisterName::RegHL));
        let pre_decrement = self.registers.read_word(WordRegisterName::RegHL);
        self.registers.write_word(WordRegisterName::RegHL, pre_decrement - 1);
    }

    pub fn ld_word<T: WriteWord, U: ReadWord>(&mut self, dest: T, src: U) {
        let source_value = src.read_word(self);
        dest.write_word(self, source_value)
    }

    pub fn push(&mut self, register: WordRegisterName) {
        // self.registers.sp = (self.registers.sp.from_gb_endian() - 2).to_gb_endian();
        let new_stack_pointer = self.registers.read_word(WordRegisterName::RegSP) - 2;
        self.registers.write_word(WordRegisterName::RegSP, new_stack_pointer);
        let address = new_stack_pointer;
        let contents = self.registers.read_word(register);
        let mut map = self.memory.borrow_mut();
        map.write::<Word>(contents, address);
        self.registers.write_word(WordRegisterName::RegSP, new_stack_pointer);
    }

    pub fn pop(&mut self, register: WordRegisterName) {
        let address = self.registers.read_word(WordRegisterName::RegSP);
        let mut map = self.memory.borrow_mut();
        let contents = map.read::<Word>(address);
        match register {
            WordRegisterName::RegAF => {
                // The flag register must overwrite its 4 lowest bits with 0 to be compliant
                self.registers.write_word(register, contents & 0xFFF0);
            }
            _ => {
                self.registers.write_word(register, contents);
            }
        }
        self.registers.write_word(WordRegisterName::RegSP, address + 2);
    }

    // Byte addition, can specify whether the existing carry flag will be incorporated
    // Returns a tuple of the result and the flags that would be set as a result of adding
    // Some operations may not actually set all of these flags
    fn byte_addition(&mut self, lhs: Byte, rhs: Byte, with_carry: bool) -> (Byte, bool, bool, bool, bool) {
        let lhs: u16 = lhs as u16;
        let rhs = rhs as u16;

        let prior_carry = if with_carry { self.registers.check_flag(Flags::C) as u16 } else { 0 };
        let result = lhs + rhs + prior_carry;
        let zero = result == 0;
        let negate: bool = false;
        let half_carry = ((lhs & 0x0F) + (rhs & 0x0F) + prior_carry) > 0x0F;
        let carry = ((lhs & 0xFF) + (rhs & 0xFF) + prior_carry) > 0xFF ;
        
        (result as Byte, zero, negate, half_carry, carry)
    }

    pub fn add_byte<T: ReadByte>(&mut self, src: T, with_carry: bool) {
        let lhs = self.registers.read_byte(ByteRegisterName::RegA);
        let rhs = src.read_byte(self);

        let (result, zero, negate, half_carry, carry) = self.byte_addition(lhs, rhs, with_carry);

        self.registers.set_flag(Flags::Z, zero);
        self.registers.set_flag(Flags::N, negate);
        self.registers.set_flag(Flags::H, half_carry);
        self.registers.set_flag(Flags::C, carry);

        self.registers.write_byte(ByteRegisterName::RegA, result);
    }

    // 0xE8 and 0xF8 are really the same operation with different destinations (HL/SP), so no duplication
    pub fn add_sp_i8(&mut self, destination: WordRegisterName, offset: Signed) {
        let sp_value = self.registers.read_word(WordRegisterName::RegSP);
        let sp_upper = sp_value & 0xFF00;
        let sp_lower = sp_value & 0x00FF;
        let abs_offset = offset.abs() as Byte;
        let (sum, half_carry, carry) = if offset < 0 {
            let (result, _, _, half_carry, carry) = self.byte_subtraction(sp_lower as Byte, abs_offset, false);
            let sum = (sp_upper - (carry as u16)) | (result as u16);
            (sum, half_carry, carry)
        } else {
            let (result, _, _, half_carry, carry) = self.byte_addition(sp_lower as Byte, abs_offset, false);
            let sum = (sp_upper + (carry as u16)) | (result as u16);
            (sum, half_carry, carry)
        };

        self.registers.set_flag_off(Flags::Z);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag(Flags::H, half_carry);
        self.registers.set_flag(Flags::C, carry);

        self.registers.write_word(destination, sum);
    }

    pub fn add_hl_word<T: ReadWord>(&mut self, operand: T) {
        let lhs = self.registers.read_word(WordRegisterName::RegHL);
        let rhs = operand.read_word(self);
        let lhs_lower = (lhs & 0x00FF) as Byte;
        let lhs_upper = ((lhs & 0xFF00) >> 4) as Byte;
        let rhs_lower = (rhs & 0x00FF) as Byte;
        let rhs_upper = ((rhs & 0xFF00) >> 4) as Byte;

        let (lower_sum, _, _, lower_half_carry, lower_carry) = self.byte_addition(lhs_lower, rhs_lower, false);

        self.registers.set_flag(Flags::H, lower_half_carry);
        self.registers.set_flag(Flags::C, lower_carry);

        let (upper_sum, _, _, upper_half_carry, upper_carry) = self.byte_addition(lhs_upper, rhs_upper, true);

        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag(Flags::H, upper_half_carry);
        self.registers.set_flag(Flags::C, upper_carry);

        let sum = ((upper_sum as Word) << 4) | (lower_sum as Word);

        self.registers.write_word(WordRegisterName::RegHL, sum);
    }

    // Byte subtraction, can specify whether the existing carry flag will be incorporated
    // Returns a tuple of the result and the flags that would be set as a result of subtracting
    // Some operations may not actually set all of these flags
    fn byte_subtraction(&mut self, lhs: Byte, rhs: Byte, with_carry: bool) -> (Byte, bool, bool, bool, bool) {
        let lhs: u16 = ( lhs as u16 ) << 8;
        let rhs = ( rhs as u16) << 8;

        let prior_carry = if with_carry { (self.registers.check_flag(Flags::C) as u16) << 8 } else { 0 };
        let result = lhs - rhs - prior_carry;
        let zero = result == 0;
        let negate = true;
        let half_carry = ((lhs & 0xF000) - (rhs & 0xF000) - prior_carry) < 0xF000;
        let carry = ((lhs & 0xFF00) - (rhs & 0xFF00) + prior_carry) < 0xFF00 ;
        ((result >> 8) as Byte, zero, negate, half_carry, carry)
    }

    pub fn sub_byte<T: ReadByte>(&mut self, src: T, with_carry: bool) {
        let lhs = self.registers.read_byte(ByteRegisterName::RegA);
        let rhs = src.read_byte(self);

        let (result, zero, negate, half_carry, carry) = self.byte_subtraction(lhs, rhs, with_carry);
        
        self.registers.set_flag(Flags::Z, zero);
        self.registers.set_flag(Flags::N, negate);
        self.registers.set_flag(Flags::H, half_carry);
        self.registers.set_flag(Flags::C, carry);

        self.registers.write_byte(ByteRegisterName::RegA, result);
    }

    pub fn cp<T: ReadByte>(&mut self, src: T) {
        let lhs = self.registers.read_byte(ByteRegisterName::RegA);
        let rhs = src.read_byte(self);
        
        let (_, zero, negate, half_carry, carry) = self.byte_subtraction(lhs, rhs, false);
        
        self.registers.set_flag(Flags::Z, zero);
        self.registers.set_flag(Flags::N, negate);
        self.registers.set_flag(Flags::H, half_carry);
        self.registers.set_flag(Flags::C, carry);
    }

    pub fn and<T: ReadByte>(&mut self, src: T) {
        let lhs = self.registers.read_byte(ByteRegisterName::RegA);
        let rhs = src.read_byte(self);
        
        let result = lhs & rhs;

        self.registers.set_flag(Flags::Z, result == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_on(Flags::H);
        self.registers.set_flag_off(Flags::C);

        self.registers.write_byte(ByteRegisterName::RegA, result)
    }

    pub fn or<T: ReadByte>(&mut self, src: T) {
        let lhs = self.registers.read_byte(ByteRegisterName::RegA);
        let rhs = src.read_byte(self);
        
        let result = lhs | rhs;

        self.registers.set_flag(Flags::Z, result == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag_off(Flags::C);

        self.registers.write_byte(ByteRegisterName::RegA, result)
    }

    pub fn xor<T: ReadByte>(&mut self, src: T) {
        let lhs = self.registers.read_byte(ByteRegisterName::RegA);
        let rhs = src.read_byte(self);
        
        let result = lhs ^ rhs;

        self.registers.set_flag(Flags::Z, result == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag_off(Flags::C);

        self.registers.write_byte(ByteRegisterName::RegA, result)
    }

    pub fn inc_byte<T: ReadByte + WriteByte>(&mut self, operand: T) {
        let pre_increment = operand.read_byte(self);
        let (post_increment, zero, negate, half_carry, _) = self.byte_addition(pre_increment, 1, false);

        self.registers.set_flag(Flags::Z, zero);
        self.registers.set_flag(Flags::N, negate);
        self.registers.set_flag(Flags::H, half_carry);
        
        operand.write_byte(self, post_increment);
    }

    pub fn inc_word<T: ReadWord + WriteWord>(&mut self, operand: T) {
        let pre_increment = operand.read_word(self);
        let post_increment = pre_increment + 1;

        operand.write_word(self, post_increment);
    }

    pub fn dec_byte<T: ReadByte + WriteByte>(&mut self, operand: T) {
        let pre_decrement = operand.read_byte(self);
        let (post_decrement, zero, negate, half_carry, _) = self.byte_subtraction(pre_decrement, 1, false);

        self.registers.set_flag(Flags::Z, zero);
        self.registers.set_flag(Flags::N, negate);
        self.registers.set_flag(Flags::H, half_carry);
        
        operand.write_byte(self, post_decrement);
    }

    pub fn dec_word<T: ReadWord + WriteWord>(&mut self, operand: T) {
        let pre_decrement = operand.read_word(self);
        let post_decrement = pre_decrement + 1;

        operand.write_word(self, post_decrement);
    }

    pub fn jp<T: ReadWord>(&mut self, to: T, condition: ConditionCodes) -> bool {
        if self.registers.check_condition(condition) {
            let address = to.read_word(self);

            self.registers.write_word(WordRegisterName::RegPC, address);
            true
        } else {
            false
        }
    }

    pub fn jr(&mut self, offset: i8, condition: ConditionCodes) -> bool {
        if self.registers.check_condition(condition) {
            let current_address = self.registers.read_word(WordRegisterName::RegPC);
            let address = current_address.checked_add_signed(offset as i16).unwrap();

            self.registers.write_word(WordRegisterName::RegSP, address);
            true
        } else {
            false
        }
    }

    pub fn call(&mut self, address: Address, condition: ConditionCodes) -> bool {
        if self.registers.check_condition(condition) {
            self.push(WordRegisterName::RegPC);
            self.registers.write_word(WordRegisterName::RegPC, address);
            true
        } else {
            false
        }
    }

    pub fn ret(&mut self, condition: ConditionCodes) -> bool {
        if self.registers.check_condition(condition) {
            self.pop(WordRegisterName::RegPC);
            true
        } else {
            false
        }
    }

    pub fn nop(&mut self) {

    }

    pub fn rst(&mut self, address: Address) {
        self.push(WordRegisterName::RegPC);
        self.registers.write_word(WordRegisterName::RegPC, address)
    }

    pub fn cpl(&mut self) {
        let a_original = self.registers.read_byte(ByteRegisterName::RegA);
        self.registers.set_flag_on(Flags::N);
        self.registers.set_flag_on(Flags::H);

        self.registers.write_byte(ByteRegisterName::RegA, !a_original);
    }

    pub fn ccf(&mut self) {
        let carry = self.registers.check_flag(Flags::C);

        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        
        self.registers.set_flag(Flags::C, !carry);
        
    }

    pub fn rl<T: ReadByte + WriteByte>(&mut self, item: T) {
        let original_value = item.read_byte(self);
        let end = original_value >> 7;
        let original_carry = self.registers.check_flag(Flags::C) as u8;

        let new_value = original_carry | (original_value << 1);

        self.registers.set_flag(Flags::Z, new_value == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag(Flags::C, end > 0);

        item.write_byte(self, new_value);
    }

    pub fn rr<T: ReadByte + WriteByte>(&mut self, item: T) {
        let original_value = item.read_byte(self);
        let end = original_value & 1;
        let original_carry = self.registers.check_flag(Flags::C) as u8;

        let new_value = (original_carry << 7) | (original_value >> 1);

        self.registers.set_flag(Flags::Z, new_value == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag(Flags::C, end > 0);

        item.write_byte(self, new_value);
    }

    pub fn rlc<T: ReadByte + WriteByte>(&mut self, item: T) {
        let original_value = item.read_byte(self);
        let end = original_value >> 7;

        let new_value = end | (original_value << 1);

        self.registers.set_flag(Flags::Z, new_value == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag(Flags::C, end > 0);

        item.write_byte(self, new_value);
    }

    pub fn rrc<T: ReadByte + WriteByte>(&mut self, item: T) {
        let original_value = item.read_byte(self);
        let end = original_value & 1;

        let new_value = (end << 7) | (original_value >> 1);

        self.registers.set_flag(Flags::Z, new_value == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag(Flags::C, end > 0);

        item.write_byte(self, new_value);
    }

    pub fn sla<T: ReadByte + WriteByte>(&mut self, item: T) {
        let original_value = item.read_byte(self);
        let end = original_value >> 7;

        let new_value = original_value << 1;

        self.registers.set_flag(Flags::Z, new_value == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag(Flags::C, end > 0);

        item.write_byte(self, new_value);
    }

    pub fn srl<T: ReadByte + WriteByte>(&mut self, item: T) {
        let original_value = item.read_byte(self);
        let end = original_value & 1;

        let new_value = original_value >> 1;

        self.registers.set_flag(Flags::Z, new_value == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag(Flags::C, end > 0);

        item.write_byte(self, new_value);
    }

    pub fn sra<T: ReadByte + WriteByte>(&mut self, item: T) {
        let original_value = item.read_byte(self);
        let msb = original_value & 0x80;
        let end = original_value & 1;

        let new_value = msb | (original_value >> 1);

        self.registers.set_flag(Flags::Z, new_value == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag(Flags::C, end > 0);

        item.write_byte(self, new_value);
    }

    pub fn swap<T: ReadByte + WriteByte>(&mut self, item: T) {
        let original_value = item.read_byte(self);
        let high = original_value & 0xF0;
        let low = original_value & 0xF;

        let new_value = (low << 4) | high;

        self.registers.set_flag(Flags::Z, new_value == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag_off(Flags::C);

        item.write_byte(self, new_value);
    }

    pub fn bit<T: ReadByte>(&mut self, bit_position: u8, item: T) {
        let value = item.read_byte(self) | (1 << bit_position);

        self.registers.set_flag(Flags::Z, value > 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_on(Flags::H);
    }

    pub fn res<T: ReadByte + WriteByte>(&mut self, bit_position: u8, item: T) {
        let value = item.read_byte(self);
        let mask = !(1 << bit_position);

        item.write_byte(self, value & mask);
    }

    pub fn set<T: ReadByte + WriteByte>(&mut self, bit_position: u8, item: T) {
        let value = item.read_byte(self);
        let mask = 1 << bit_position;

        item.write_byte(self, value | mask);
    }

    pub fn scf(&mut self) {
        self.registers.set_flag_on(Flags::C);
    }

    pub fn daa(&mut self) {
        let original_value = self.registers.read_byte(ByteRegisterName::RegA);

        let previous_n_flag = self.registers.check_flag(Flags::N);
        let previous_carry = self.registers.check_flag(Flags::C);
        let previous_half_carry = self.registers.check_flag(Flags::H);

        let mut new_carry: bool = false;
        let mut result = original_value;
        if !previous_n_flag {
            if previous_carry || (result > 0x99) {
                new_carry = true;
                result += 0x60;
            }
            if previous_half_carry || ((result & 0x0f) > 0x09) {
                result += 0x6;
            }
        } else {
            if previous_carry {
                result -= 0x60;
            }
            if previous_half_carry {
                result -= 0x6;
            }
        }
            
        self.registers.set_flag(Flags::Z, result == 0);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag(Flags::C, new_carry);

        self.registers.write_byte(ByteRegisterName::RegA, result);
    }
}