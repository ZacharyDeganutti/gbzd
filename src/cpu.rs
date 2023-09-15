use std::io::Read;
use std::rc::Rc;
use std::cell::RefCell;

use crate::memory_gb::Address;
use crate::memory_gb::Byte;
use crate::memory_gb::EndianTranslate;
use crate::memory_gb::Word;
use crate::memory_gb::MemoryRegion;
use crate::memory_gb::MemoryMap;

/* Semantics notes
*   Callers of operations are responsible for tracking timing since calls are case by case anyway
*   Attempting to generalize the what and why of how costs accumulate leads to unreasonable code complexity
*   All we need is the number at the end of the day. 
*   Opcodes with costs dependent on branching should report whether they branched or not, and the caller decides the cost
*/

#[derive(Clone, Copy)]
pub enum ByteRegister {
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
pub enum WordRegister {
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

pub trait ReadByte {
    fn read_byte(&self, cpu: &mut Cpu) -> Byte;
}

pub struct RegisterValue {
    register: ByteRegister,
}
impl ReadByte for RegisterValue {
    fn read_byte(&self, cpu: &mut Cpu) -> Byte {
        cpu.registers.read_byte(self.register)
    }
}
impl RegisterValue {
    pub fn new(register: ByteRegister) -> RegisterValue {
        RegisterValue { register: register }
    }
}

pub struct RegisterIndirect {
    register: WordRegister,
}
impl ReadByte for RegisterIndirect {
    fn read_byte(&self, cpu: &mut Cpu) -> Byte {
        let address = cpu.registers.read_word(self.register);
        let mut map = cpu.memory.borrow_mut();
        unsafe { map.read::<Byte>(address) }
    }
}
impl RegisterIndirect {
    pub fn new(register: WordRegister) -> RegisterIndirect {
        RegisterIndirect { register: register }
    }
}

pub struct RegisterOffsetIndirect {
    register: ByteRegister,
}
impl ReadByte for RegisterOffsetIndirect {
    fn read_byte(&self, cpu: &mut Cpu) -> Byte {
        let offset = cpu.registers.read_byte(self.register);
        let address = 0xFF00 + offset as Address;
        let mut map = cpu.memory.borrow_mut();
        unsafe { map.read::<Byte>(address) }
    }
}
impl RegisterOffsetIndirect {
    pub fn new(register: ByteRegister) -> RegisterOffsetIndirect {
        RegisterOffsetIndirect { register: register }
    }
}

pub struct ImmediateValue {
    data: Byte,
}
impl ReadByte for ImmediateValue {
    fn read_byte(&self, cpu: &mut Cpu) -> Byte {
        self.data
    }
}
impl ImmediateValue {
    pub fn new(value: Byte) -> ImmediateValue {
        ImmediateValue{ data: value }
    }
}

pub struct ImmediateIndirect {
    address: Address,
}
impl ReadByte for ImmediateIndirect {
    fn read_byte(&self, cpu: &mut Cpu) -> Byte {
        let mut map = cpu.memory.borrow_mut();
        unsafe { map.read::<Byte>(self.address) } 
    }
}
impl ImmediateIndirect {
    pub fn fetch(address: Address) -> ImmediateIndirect {
        ImmediateIndirect{ address: address }
    }
}

pub struct ImmediateOffsetIndirect {
    offset: Byte,
}
impl ReadByte for ImmediateOffsetIndirect {
    fn read_byte(&self, cpu: &mut Cpu) -> Byte {
        let address = 0xFF00 + self.offset as Address;
        let mut map = cpu.memory.borrow_mut();
        unsafe { map.read::<Byte>(address) }
    }
}
impl ImmediateOffsetIndirect {
    pub fn fetch(offset: Byte, memory: MemoryMapRef) -> ImmediateOffsetIndirect {
        ImmediateOffsetIndirect{ offset: offset }
    }
}

// Destination operand possibilities for 8 bit operations
pub enum ByteDestination {
    RegisterValue(ByteRegister),
    ToRegisterAddress(WordRegister),
    ToRegisterOffsetAddress(ByteRegister),
    ToImmediateAddress(Address),
    ToImmediateOffsetAddress(Byte),
}

// Source operand possibilities for 16 bit loads
pub enum WordSource {
    RegisterValue(WordRegister),
    ImmediateValue(Word),
}
// Destination operand possibilities for for 16 bit loads
pub enum WordDestination {
    RegisterValue(WordRegister),
    ToImmediateAddress(Address),
}

pub enum Flags {
    Z = 3,
    N = 2,
    H = 1,
    C = 0
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
    pub fn read_byte(&mut self, register: ByteRegister) -> Byte {
        unsafe { self.read::<Byte>(register as Address) }
    }
    pub fn read_word(&mut self, register: WordRegister) -> Word {
        unsafe { self.read::<Word>(register as Address) }
    }

    pub fn write_byte(&mut self, register: ByteRegister, value: Byte) -> () {
        unsafe { self.write::<Byte>(value, register as Address) }
    }
    pub fn write_word(&mut self, register: WordRegister, value: Word) -> () {
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
        let prev = self.read_byte(ByteRegister::RegF);
        let mask: u8 = 1 << (4 + flag as u8);
        self.write_byte(ByteRegister::RegF, prev | mask);
    }
    pub fn set_flag_off(&mut self, flag: Flags) -> () {
        let prev = self.read_byte(ByteRegister::RegF);
        let mask: u8 = !(1 << (4 + flag as u8));
        self.write_byte(ByteRegister::RegF, prev & mask);
    }

    pub fn check_flag(&mut self, flag: Flags) -> bool {
        let flags = self.read_byte(ByteRegister::RegF);
        let mask: u8 = 1 << (4 + flag as u8);
        (flags & mask) > 0
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

    pub fn ld_byte<T: ReadByte>(&mut self, dest: ByteDestination, src: T) {
        let source_value = src.read_byte(self);
        match dest {
            ByteDestination::RegisterValue(register) => {
                self.registers.write_byte(register, source_value)
            }
            ByteDestination::ToRegisterAddress(register) => { 
                let address = self.registers.read_word(register);
                let mut map = self.memory.borrow_mut();
                unsafe { map.write(source_value, address) }
            }
            ByteDestination::ToRegisterOffsetAddress(register) => {
                let offset = self.registers.read_byte(register) as Address;
                let address = 0xFF00 + offset;
                let mut map = self.memory.borrow_mut();
                unsafe { map.write(source_value, address) }
            }
            ByteDestination::ToImmediateAddress(address) => {
                let mut map = self.memory.borrow_mut();
                unsafe { map.write(source_value, address) }
            }
            ByteDestination::ToImmediateOffsetAddress(offset) => {
                let address = 0xFF00 + offset as Address;
                let mut map = self.memory.borrow_mut();
                unsafe { map.write(source_value, address) }
            }
        }
    }

    pub fn ld_word(&mut self, dest: WordDestination, src: WordSource) {
        let mut register_sourced = false;
        let source_value = match src {
            WordSource::RegisterValue(register) => {
                register_sourced = true;
                self.registers.read_word(register)
            }
            WordSource::ImmediateValue(value) => {
                value
            }
        };
        match dest {
            WordDestination::RegisterValue(register) => {
                self.registers.write_word(register, source_value)
            }
            WordDestination::ToImmediateAddress(address) => {
                let mut map = self.memory.borrow_mut();
                unsafe { map.write(source_value, address) }
            }
        }
    }

    pub fn push(&mut self, register: WordRegister) {
        // self.registers.sp = (self.registers.sp.from_gb_endian() - 2).to_gb_endian();
        let new_stack_pointer = self.registers.read_word(WordRegister::RegSP) - 2;
        self.registers.write_word(WordRegister::RegSP, new_stack_pointer);
        let address = new_stack_pointer;
        let contents = self.registers.read_word(register);
        let mut map = self.memory.borrow_mut();
        unsafe { map.write(contents, address) };
        self.registers.write_word(WordRegister::RegSP, new_stack_pointer);
    }

    pub fn pop(&mut self, register: WordRegister) {
        let address = self.registers.read_word(WordRegister::RegSP);
        let mut map = self.memory.borrow_mut();
        let contents = unsafe { map.read(address) };
        match register {
            WordRegister::RegAF => {
                // The flag register must overwrite its 4 lowest bits with 0 to be compliant
                self.registers.write_word(register, contents & 0xFFF0);
            }
            _ => {
                self.registers.write_word(register, contents);
            }
        }
        self.registers.write_word(WordRegister::RegSP, address + 2);
    }

    pub fn add_byte<T: ReadByte>(&mut self, src: T, with_carry: bool) {
        let mut cost = TICK; // fetch
        let lhs: u16 = self.registers.read_byte(ByteRegister::RegA) as u16;
        let rhs = src.read_byte(self) as u16;

        let prior_carry = if with_carry { self.registers.check_flag(Flags::C) as u16 } else { 0 };
        let result = lhs + rhs + prior_carry;
        let half_carry = ((lhs & 0x0F) + (rhs & 0x0F) + prior_carry) > 0x0F;
        let carry = ((lhs & 0xFF) + (rhs & 0xFF) + prior_carry) > 0xFF ;

        self.registers.set_flag(Flags::Z, result == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag(Flags::H, half_carry);
        self.registers.set_flag(Flags::C, carry);

        self.registers.write_byte(ByteRegister::RegA, result as Byte);
    }

    fn general_sub_byte<T: ReadByte>(&mut self, src:T, with_carry: bool) -> Byte {
        let mut cost = TICK; // fetch
        let lhs: u16 = ( self.registers.read_byte(ByteRegister::RegA) as u16 ) << 8;
        let rhs = (src.read_byte(self) as u16) << 8;

        let prior_carry = if with_carry { self.registers.check_flag(Flags::C) as u16 } else { 0 };
        let result = lhs - rhs - prior_carry;
        let half_carry = ((lhs & 0xF000) - (rhs & 0xF000) - prior_carry) < 0xF000;
        let carry = ((lhs & 0xFF00) - (rhs & 0xFF00) + prior_carry) < 0xFF00 ;

        self.registers.set_flag(Flags::Z, result == 0);
        self.registers.set_flag_on(Flags::N);
        self.registers.set_flag(Flags::H, half_carry);
        self.registers.set_flag(Flags::C, carry);

        
        (result >> 8) as Byte
    }

    pub fn sub_byte<T: ReadByte>(&mut self, src: T, with_carry: bool) {
        let result = self.general_sub_byte(src, with_carry);
        self.registers.write_byte(ByteRegister::RegA, result);
    }

    pub fn cp_op<T: ReadByte>(&mut self, src: T) {
        let result = self.general_sub_byte(src, false);
    }

    pub fn and_op<T: ReadByte>(&mut self, src: T) {
        let lhs = self.registers.read_byte(ByteRegister::RegA);
        let rhs = src.read_byte(self);
        
        let result = lhs & rhs;

        self.registers.set_flag(Flags::Z, result == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_on(Flags::H);
        self.registers.set_flag_off(Flags::C);
    }

    pub fn or_op<T: ReadByte>(&mut self, src: T) {
        let lhs = self.registers.read_byte(ByteRegister::RegA);
        let rhs = src.read_byte(self);
        
        let result = lhs | rhs;

        self.registers.set_flag(Flags::Z, result == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag_off(Flags::C);
    }

    pub fn xor_op<T: ReadByte>(&mut self, src: T) {
        let lhs = self.registers.read_byte(ByteRegister::RegA);
        let rhs = src.read_byte(self);
        
        let result = lhs ^ rhs;

        self.registers.set_flag(Flags::Z, result == 0);
        self.registers.set_flag_off(Flags::N);
        self.registers.set_flag_off(Flags::H);
        self.registers.set_flag_off(Flags::C);
    }
}