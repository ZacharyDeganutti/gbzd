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

    pub fn step_pc(&mut self, increment: u16) {
        let pc = self.read_word(WordRegisterName::RegPC);
        self.write_word(WordRegisterName::RegPC, pc + increment);
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
}