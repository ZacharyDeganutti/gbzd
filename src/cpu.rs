use std::fmt::Write;
use std::io::Read;
use std::rc::Rc;
use std::cell::RefCell;

use crate::memory_gb::Address;
use crate::memory_gb::Byte;
use crate::memory_gb::EndianTranslate;
use crate::memory_gb::Signed;
use crate::memory_gb::Word;
use crate::memory_gb::MemoryRegion;
use crate::memory_gb::MemoryMap;

/* Semantics notes
*   Callers of operations are responsible for tracking timing since calls are case by case anyway
*   Attempting to generalize the what and why of how costs accumulate leads to unreasonable code complexity
*   All we need is the number at the end of the day. 
*   Opcodes with costs dependent on branching should report whether they branched or not, and the caller decides the cost

*   Incrementation of the stack pointer is external to operation calls
*   It is to be handled by the caller prior to execution of each operation

*   Externalizing the above side effects will allow maximum reuse of procedures
*/

#[derive(Clone, Copy)]
pub enum ByteRegisterName {
    RegA = 1,
    RegF = 0,
    RegB = 3,
    RegC = 2,
    RegD = 5,
    RegE = 4,
    RegH = 7,
    RegL = 6,
}

#[derive(Clone, Copy)]
pub enum WordRegisterName {
    RegAF = 0,
    RegBC = 1,
    RegDE = 2,
    RegHL = 3,
    RegSP = 4,
    RegPC = 5,
}

type Cost = u8;
const TICK: Cost = 4;

type MemoryMapRef = Rc<RefCell<MemoryMap>>;

// Trait for reading bytes from various Cpu sources
pub trait ReadByte {
    fn read_byte(&self, cpu: &mut Cpu) -> Byte;
}
// Trait for writing bytes to various Cpu sources
pub trait WriteByte {
    fn write_byte(&self, cpu: &mut Cpu, value: Byte);
}

// Trait for reading words from various Cpu sources
pub trait ReadWord {
    fn read_word(&self, cpu: &mut Cpu) -> Word;
}
// Trait for writing words to various Cpu sources
pub trait WriteWord {
    fn write_word(&self, cpu: &mut Cpu, value: Word);
}

pub struct ByteRegister {
    register: ByteRegisterName,
}
impl ReadByte for ByteRegister {
    fn read_byte(&self, cpu: &mut Cpu) -> Byte {
        cpu.registers.read_byte(self.register)
    }
}
impl WriteByte for ByteRegister {
    fn write_byte(&self, cpu: &mut Cpu, value: Byte) {
        cpu.registers.write_byte(self.register, value);
    }
}
impl ByteRegister {
    pub fn new(register: ByteRegisterName) -> ByteRegister {
        ByteRegister { register }
    }
}

pub struct WordRegister {
    register: WordRegisterName
}
impl ReadWord for WordRegister {
    fn read_word(&self, cpu: &mut Cpu) -> Word {
        cpu.registers.read_word(self.register)
    }
}
impl WriteWord for WordRegister {
    fn write_word(&self, cpu: &mut Cpu, value: Word) {
        cpu.registers.write_word(self.register, value);
    }
}
impl WordRegister {
    pub fn new(register: WordRegisterName) -> WordRegister {
        WordRegister { register }
    }
}

pub struct ByteRegisterIndirect {
    register: WordRegisterName,
}
impl ReadByte for ByteRegisterIndirect {
    fn read_byte(&self, cpu: &mut Cpu) -> Byte {
        let address = cpu.registers.read_word(self.register);
        let mut map = cpu.memory.borrow_mut();
        unsafe { map.read::<Byte>(address) }
    }
}
impl WriteByte for ByteRegisterIndirect {
    fn write_byte(&self, cpu: &mut Cpu, value: Byte) {
        let address = cpu.registers.read_word(self.register);
        let mut map = cpu.memory.borrow_mut();
        unsafe { map.write(value, address) }
    }
}
impl ByteRegisterIndirect {
    pub fn new(register: WordRegisterName) -> ByteRegisterIndirect {
        ByteRegisterIndirect { register }
    }
}

pub struct ByteRegisterOffsetIndirect {
    register: ByteRegisterName,
}
impl ReadByte for ByteRegisterOffsetIndirect {
    fn read_byte(&self, cpu: &mut Cpu) -> Byte {
        let offset = cpu.registers.read_byte(self.register);
        let address = 0xFF00 + offset as Address;
        let mut map = cpu.memory.borrow_mut();
        unsafe { map.read::<Byte>(address) }
    }
}
impl WriteByte for ByteRegisterOffsetIndirect {
    fn write_byte(&self, cpu: &mut Cpu, value: Byte) {
        let offset = cpu.registers.read_byte(self.register) as Address;
        let address = 0xFF00 + offset;
        let mut map = cpu.memory.borrow_mut();
        unsafe { map.write(value, address) }
    }
}
impl ByteRegisterOffsetIndirect {
    pub fn new(register: ByteRegisterName) -> ByteRegisterOffsetIndirect {
        ByteRegisterOffsetIndirect { register }
    }
}

pub struct ByteImmediate {
    data: Byte,
}
impl ReadByte for ByteImmediate {
    fn read_byte(&self, cpu: &mut Cpu) -> Byte {
        self.data
    }
}
impl ByteImmediate {
    pub fn new(value: Byte) -> ByteImmediate {
        ByteImmediate{ data: value }
    }
}

pub struct ByteImmediateIndirect {
    address: Address,
}
impl ReadByte for ByteImmediateIndirect {
    fn read_byte(&self, cpu: &mut Cpu) -> Byte {
        let mut map = cpu.memory.borrow_mut();
        unsafe { map.read::<Byte>(self.address) } 
    }
}
impl WriteByte for ByteImmediateIndirect {
    fn write_byte(&self, cpu: &mut Cpu, value: Byte) {
        let mut map = cpu.memory.borrow_mut();
        unsafe { map.write(value, self.address) }
    }
}
impl ByteImmediateIndirect {
    pub fn new(address: Address) -> ByteImmediateIndirect {
        ByteImmediateIndirect{ address }
    }
}

pub struct ByteImmediateOffsetIndirect {
    offset: Byte,
}
impl ReadByte for ByteImmediateOffsetIndirect {
    fn read_byte(&self, cpu: &mut Cpu) -> Byte {
        let address = 0xFF00 + self.offset as Address;
        let mut map = cpu.memory.borrow_mut();
        unsafe { map.read::<Byte>(address) }
    }
}
impl WriteByte for ByteImmediateOffsetIndirect {
    fn write_byte(&self, cpu: &mut Cpu, value: Byte) {
        let address = 0xFF00 + self.offset as Address;
        let mut map = cpu.memory.borrow_mut();
        unsafe { map.write(value, address) }
    }
}
impl ByteImmediateOffsetIndirect {
    pub fn fetch(offset: Byte, memory: MemoryMapRef) -> ByteImmediateOffsetIndirect {
        ByteImmediateOffsetIndirect{ offset }
    }
}

pub struct WordImmediate {
    data: Word,
}
impl ReadWord for WordImmediate {
    fn read_word(&self, cpu: &mut Cpu) -> Word {
        self.data
    }
}
impl WordImmediate {
    pub fn new(value: Word) -> WordImmediate {
        WordImmediate{ data: value }
    }
}

pub struct WordImmediateIndirect {
    address: Address,
}
impl WriteWord for WordImmediateIndirect {
    fn write_word(&self, cpu: &mut Cpu, value: Word) {
        let mut map = cpu.memory.borrow_mut();
        unsafe { map.write(value, self.address) }
    }
}
impl WordImmediateIndirect {
    pub fn new(address: Address) -> WordImmediateIndirect {
        WordImmediateIndirect{ address }
    }
}

pub enum Flags {
    Z = 3,
    N = 2,
    H = 1,
    C = 0
}

pub enum ConditionCodes {
    C,
    NC,
    NZ,
    Z,
    NA
}

#[repr(C)]
pub struct RegisterBank {
    f: Byte,
    // Flag register
    // +-+-+-+-+-+-+-+-+
    // |7|6|5|4|3|2|1|0|
    // +-+-+-+-+-+-+-+-+
    // |Z|N|H|C|0|0|0|0|
    // +-+-+-+-+-+-+-+-+
    // Z: Zero flag, set when math op result is zero or 2 values match on a compare op
    // N: Subtract flag, set if a subtraction was performed in the last math op
    // H: Half carry flag, set if a carry occurred from the lower nibble in the last math op
    // C: Carry flag, set if a carry occurred from the last math op or if register A is the smaller value when executing compare op
    // AF paired
    pub a: Byte,
    // Accumulator, typically used as destination for arithmetic ops
    pub c: Byte,
    pub b: Byte,
    // BC paired, general purpose registers
    pub e: Byte,
    pub d: Byte,
    // DE paired, general purpose registers
    pub l: Byte,
    pub h: Byte,
    // HL paired, general purpose registers that can point into memory
    pub sp: Address, // Stack pointer,  initialized to 0xFFFE
    pub pc: Address, // Program counter, initialized to 0x100
}

impl MemoryRegion for RegisterBank {}
impl RegisterBank {
    pub fn read_byte(&mut self, register: ByteRegisterName) -> Byte {
        unsafe { self.read::<Byte>(register as Address) }
    }
    pub fn read_word(&mut self, register: WordRegisterName) -> Word {
        unsafe { self.read::<Word>(register as Address) }
    }

    pub fn write_byte(&mut self, register: ByteRegisterName, value: Byte) -> () {
        unsafe { self.write::<Byte>(value, register as Address) }
    }
    pub fn write_word(&mut self, register: WordRegisterName, value: Word) -> () {
        unsafe { self.write::<Word>(value, register as Address) }
    }

    pub fn set_flag(&mut self, flag: Flags, value: bool) -> () {
        if value {
            self.set_flag_on(flag)
        }
        else {
            self.set_flag_off(flag)
        }
    }
    pub fn set_flag_on(&mut self, flag: Flags) -> () {
        let prev = self.read_byte(ByteRegisterName::RegF);
        let mask: u8 = 1 << (4 + flag as u8);
        self.write_byte(ByteRegisterName::RegF, prev | mask);
    }
    pub fn set_flag_off(&mut self, flag: Flags) -> () {
        let prev = self.read_byte(ByteRegisterName::RegF);
        let mask: u8 = !(1 << (4 + flag as u8));
        self.write_byte(ByteRegisterName::RegF, prev & mask);
    }

    pub fn check_flag(&mut self, flag: Flags) -> bool {
        let flags = self.read_byte(ByteRegisterName::RegF);
        let mask: u8 = 1 << (4 + flag as u8);
        (flags & mask) > 0
    }

    pub fn check_condition(&mut self, condition: ConditionCodes) -> bool {
        match condition {
            ConditionCodes::NA => {
                true
            }
            ConditionCodes::NC => {
                !self.check_flag(Flags::C)
            }
            ConditionCodes::NZ => {
                !self.check_flag(Flags::Z)
            }
            ConditionCodes::C => {
                self.check_flag(Flags::C)
            }
            ConditionCodes::Z => {
                self.check_flag(Flags::Z)
            }
        }
    }
}

pub struct Cpu {
    pub registers: RegisterBank,
    pub memory: Rc<RefCell<MemoryMap>>
}

impl Cpu {
    pub fn new(system_memory: Rc<RefCell<MemoryMap>>) -> Cpu {
        let regs = RegisterBank {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 32,
            h: 0,
            l: 0,
            sp: 0xFFFE.to_gb_endian(),
            pc: 0.to_gb_endian(),
        };
        Cpu { 
            registers: regs,
            memory: system_memory
        }
    }

    pub fn ld_byte_op<T: WriteByte, U: ReadByte>(&mut self, dest: T, src: U) {
        let source_value = src.read_byte(self);
        dest.write_byte(self, source_value);
    }

    // Indirect load from register A into address in register HL. Increment HL afterwards
    pub fn ld_byte_0x22_op(&mut self) {
        self.ld_byte_op(ByteRegisterIndirect::new(WordRegisterName::RegHL), ByteRegister::new(ByteRegisterName::RegA));
        let pre_increment = self.registers.read_word(WordRegisterName::RegHL);
        self.registers.write_word(WordRegisterName::RegHL, pre_increment + 1);
    }

    // Indirect load from address in register HL into register A. Increment HL afterwards
    pub fn ld_byte_0x2a_op(&mut self) {
        self.ld_byte_op(ByteRegister::new(ByteRegisterName::RegA), ByteRegisterIndirect::new(WordRegisterName::RegHL));
        let pre_increment = self.registers.read_word(WordRegisterName::RegHL);
        self.registers.write_word(WordRegisterName::RegHL, pre_increment + 1);
    }

    // Indirect load from register A into address in register HL. Decrement HL afterwards
    pub fn ld_byte_0x32_op(&mut self) {
        self.ld_byte_op(ByteRegisterIndirect::new(WordRegisterName::RegHL), ByteRegister::new(ByteRegisterName::RegA));
        let pre_decrement = self.registers.read_word(WordRegisterName::RegHL);
        self.registers.write_word(WordRegisterName::RegHL, pre_decrement - 1);
    }

    // Indirect load from address in register HL into register A. Decrement HL afterwards
    pub fn ld_byte_0x3a_op(&mut self) {
        self.ld_byte_op(ByteRegister::new(ByteRegisterName::RegA), ByteRegisterIndirect::new(WordRegisterName::RegHL));
        let pre_decrement = self.registers.read_word(WordRegisterName::RegHL);
        self.registers.write_word(WordRegisterName::RegHL, pre_decrement - 1);
    }

    pub fn ld_word_op<T: WriteWord, U: ReadWord>(&mut self, dest: T, src: U) {
        let source_value = src.read_word(self);
        dest.write_word(self, source_value)
    }

    pub fn push_op(&mut self, register: WordRegisterName) {
        // self.registers.sp = (self.registers.sp.from_gb_endian() - 2).to_gb_endian();
        let new_stack_pointer = self.registers.read_word(WordRegisterName::RegSP) - 2;
        self.registers.write_word(WordRegisterName::RegSP, new_stack_pointer);
        let address = new_stack_pointer;
        let contents = self.registers.read_word(register);
        let mut map = self.memory.borrow_mut();
        unsafe { map.write(contents, address) };
        self.registers.write_word(WordRegisterName::RegSP, new_stack_pointer);
    }

    pub fn pop_op(&mut self, register: WordRegisterName) {
        let address = self.registers.read_word(WordRegisterName::RegSP);
        let mut map = self.memory.borrow_mut();
        let contents = unsafe { map.read(address) };
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

    pub fn add_byte_op<T: ReadByte>(&mut self, src: T, with_carry: bool) {
        let mut cost = TICK; // fetch
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
    pub fn add_sp_i8_op(&mut self, destination: WordRegisterName, offset: Signed) {
        let sp_value = self.registers.read_word(WordRegisterName::RegSP);
        let sp_upper = (sp_value & 0xFF00);
        let sp_lower = (sp_value & 0x00FF);
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

    pub fn add_hl_word_op<T: ReadWord>(&mut self, operand: T) {
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

    pub fn sub_byte_op<T: ReadByte>(&mut self, src: T, with_carry: bool) {
        let lhs = self.registers.read_byte(ByteRegisterName::RegA);
        let rhs = src.read_byte(self);

        let (result, zero, negate, half_carry, carry) = self.byte_subtraction(lhs, rhs, with_carry);
        
        self.registers.set_flag(Flags::Z, zero);
        self.registers.set_flag(Flags::N, negate);
        self.registers.set_flag(Flags::H, half_carry);
        self.registers.set_flag(Flags::C, carry);

        self.registers.write_byte(ByteRegisterName::RegA, result);
    }

    pub fn cp_op<T: ReadByte>(&mut self, src: T) {
        let lhs = self.registers.read_byte(ByteRegisterName::RegA);
        let rhs = src.read_byte(self);
        
        let (result, zero, negate, half_carry, carry) = self.byte_subtraction(lhs, rhs, false);
        
        self.registers.set_flag(Flags::Z, zero);
        self.registers.set_flag(Flags::N, negate);
        self.registers.set_flag(Flags::H, half_carry);
        self.registers.set_flag(Flags::C, carry);
    }

    pub fn and_op<T: ReadByte>(&mut self, src: T) {
        let lhs = self.registers.read_byte(ByteRegisterName::RegA);
        let rhs = src.read_byte(self);
        
        let result = lhs & rhs;

        self.registers.set_flag(Flags::Z, result == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_on(Flags::H);
        self.registers.set_flag_off(Flags::C);

        self.registers.write_byte(ByteRegisterName::RegA, result)
    }

    pub fn or_op<T: ReadByte>(&mut self, src: T) {
        let lhs = self.registers.read_byte(ByteRegisterName::RegA);
        let rhs = src.read_byte(self);
        
        let result = lhs | rhs;

        self.registers.set_flag(Flags::Z, result == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag_off(Flags::C);

        self.registers.write_byte(ByteRegisterName::RegA, result)
    }

    pub fn xor_op<T: ReadByte>(&mut self, src: T) {
        let lhs = self.registers.read_byte(ByteRegisterName::RegA);
        let rhs = src.read_byte(self);
        
        let result = lhs ^ rhs;

        self.registers.set_flag(Flags::Z, result == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag_off(Flags::C);

        self.registers.write_byte(ByteRegisterName::RegA, result)
    }

    pub fn inc_byte_op<T: ReadByte + WriteByte>(&mut self, operand: T) {
        let pre_increment = operand.read_byte(self);
        let (post_increment, zero, negate, half_carry, _) = self.byte_addition(pre_increment, 1, false);

        self.registers.set_flag(Flags::Z, zero);
        self.registers.set_flag(Flags::N, negate);
        self.registers.set_flag(Flags::H, half_carry);
        
        operand.write_byte(self, post_increment);
    }

    pub fn inc_word_op<T: ReadWord + WriteWord>(&mut self, operand: T) {
        let pre_increment = operand.read_word(self);
        let post_increment = pre_increment + 1;

        operand.write_word(self, post_increment);
    }

    pub fn dec_byte_op<T: ReadByte + WriteByte>(&mut self, operand: T) {
        let pre_decrement = operand.read_byte(self);
        let (post_decrement, zero, negate, half_carry, _) = self.byte_subtraction(pre_decrement, 1, false);

        self.registers.set_flag(Flags::Z, zero);
        self.registers.set_flag(Flags::N, negate);
        self.registers.set_flag(Flags::H, half_carry);
        
        operand.write_byte(self, post_decrement);
    }

    pub fn dec_word_op<T: ReadWord + WriteWord>(&mut self, operand: T) {
        let pre_decrement = operand.read_word(self);
        let post_decrement = pre_decrement + 1;

        operand.write_word(self, post_decrement);
    }

    pub fn jp_op<T: ReadWord>(&mut self, to: T, condition: ConditionCodes) -> bool {
        if self.registers.check_condition(condition) {
            let address = to.read_word(self);

            self.registers.write_word(WordRegisterName::RegPC, address);
            true
        } else {
            false
        }
    }

    pub fn jr_op(&mut self, offset: i8, condition: ConditionCodes) -> bool {
        if self.registers.check_condition(condition) {
            let current_address = self.registers.read_word(WordRegisterName::RegPC);
            let address = current_address.checked_add_signed(offset as i16).unwrap();

            self.registers.write_word(WordRegisterName::RegSP, address);
            true
        } else {
            false
        }
    }

    pub fn call_op(&mut self, address: Address, condition: ConditionCodes) -> bool {
        if self.registers.check_condition(condition) {
            self.push_op(WordRegisterName::RegPC);
            self.registers.write_word(WordRegisterName::RegPC, address);
            true
        } else {
            false
        }
    }

    pub fn ret_op(&mut self, condition: ConditionCodes) -> bool {
        if self.registers.check_condition(condition) {
            self.pop_op(WordRegisterName::RegPC);
            true
        } else {
            false
        }
    }

    pub fn nop_op(&mut self) {

    }

    pub fn rst_op(&mut self, address: Address) {
        self.push_op(WordRegisterName::RegPC);
        self.registers.write_word(WordRegisterName::RegPC, address)
    }

    pub fn cpl_op(&mut self) {
        let a_original = self.registers.read_byte(ByteRegisterName::RegA);
        self.registers.set_flag_on(Flags::N);
        self.registers.set_flag_on(Flags::H);

        self.registers.write_byte(ByteRegisterName::RegA, !a_original);
    }

    pub fn ccf_op(&mut self) {
        let carry = self.registers.check_flag(Flags::C);

        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        
        self.registers.set_flag(Flags::C, !carry);
        
    }

    pub fn rl_op<T: ReadByte + WriteByte>(&mut self, item: T) {
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

    pub fn rr_op<T: ReadByte + WriteByte>(&mut self, item: T) {
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

    pub fn rlc_op<T: ReadByte + WriteByte>(&mut self, item: T) {
        let original_value = item.read_byte(self);
        let end = original_value >> 7;

        let new_value = end | (original_value << 1);

        self.registers.set_flag(Flags::Z, new_value == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag(Flags::C, end > 0);

        item.write_byte(self, new_value);
    }

    pub fn rrc_op<T: ReadByte + WriteByte>(&mut self, item: T) {
        let original_value = item.read_byte(self);
        let end = original_value & 1;

        let new_value = (end << 7) | (original_value >> 1);

        self.registers.set_flag(Flags::Z, new_value == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag(Flags::C, end > 0);

        item.write_byte(self, new_value);
    }

    pub fn sla_op<T: ReadByte + WriteByte>(&mut self, item: T) {
        let original_value = item.read_byte(self);
        let end = original_value >> 7;

        let new_value = original_value << 1;

        self.registers.set_flag(Flags::Z, new_value == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag(Flags::C, end > 0);

        item.write_byte(self, new_value);
    }

    pub fn srl_op<T: ReadByte + WriteByte>(&mut self, item: T) {
        let original_value = item.read_byte(self);
        let end = original_value & 1;

        let new_value = original_value >> 1;

        self.registers.set_flag(Flags::Z, new_value == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag(Flags::C, end > 0);

        item.write_byte(self, new_value);
    }

    pub fn sra_op<T: ReadByte + WriteByte>(&mut self, item: T) {
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

    pub fn swap_op<T: ReadByte + WriteByte>(&mut self, item: T) {
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

    pub fn bit_op<T: ReadByte>(&mut self, bit_position: u8, item: T) {
        let value = item.read_byte(self) | (1 << bit_position);

        self.registers.set_flag(Flags::Z, value > 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_on(Flags::H);
    }

    pub fn res_op<T: ReadByte + WriteByte>(&mut self, bit_position: u8, item: T) {
        let value = item.read_byte(self);
        let mask = !(1 << bit_position);

        item.write_byte(self, value & mask);
    }

    pub fn set_op<T: ReadByte + WriteByte>(&mut self, bit_position: u8, item: T) {
        let value = item.read_byte(self);
        let mask = 1 << bit_position;

        item.write_byte(self, value | mask);
    }

    pub fn scf_op(&mut self) {
        self.registers.set_flag_on(Flags::C);
    }

    pub fn daa_op(&mut self) {
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