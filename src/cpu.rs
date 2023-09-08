use std::mem;
use std::slice;

enum AddressMode {
    Mode8,
    Mode16,
}

pub enum Register8 {
    RegA = 1,
    RegF = 0,
    RegB = 3,
    RegC = 2,
    RegD = 5,
    RegE = 4,
    RegH = 7,
    RegL = 6,
}

pub enum Register16 {
    RegAF = 0,
    RegBC = 1,
    RegDE = 2,
    RegHL = 3,
    RegSP = 4,
    RegPC = 5,
}

type Cost = u8;
const TICK: Cost = 4;

// RHS operand possibilities for 8 bit loads
pub enum LD8Source {
    RegisterValue(Register8),
    FromRegisterAddress(Register16),
    ImmediateValue(u8),
    FromImmediateAddress(u16),
}

// LHS operand possibilities for 8 bit loads
pub enum LD8Destination {
    RegisterValue(Register8),
    ToRegisterAddress(Register16),
    ToImmediateAddress(u16),
}

#[repr(C)]
pub struct RegisterBank {
    f: u8,
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
    pub a: u8,
    // Accumulator, typically used as destination for arithmetic ops
    pub c: u8,
    pub b: u8,
    // BC paired, general purpose registers
    pub e: u8,
    pub d: u8,
    // DE paired, general purpose registers
    pub l: u8,
    pub h: u8,
    // HL paired, general purpose registers that can point into memory
    pub sp: u16, // Stack pointer,  initialized to 0xFFFE
    pub pc: u16, // Program counter, initialized to 0x100
}

trait RegisterSize {}
impl RegisterSize for u8 {}
impl RegisterSize for u16 {}

impl RegisterBank {
    unsafe fn slice<T: RegisterSize>(&mut self) -> &mut [T] {
        let ptr = mem::transmute::<&mut RegisterBank, *mut T>(self);
        slice::from_raw_parts_mut(ptr, mem::size_of::<RegisterBank>() / mem::size_of::<T>())
    }

    pub fn read8(&mut self, register: Register8) -> u8 {
        let slice = unsafe { self.slice::<u8>() };
        slice[register as usize]
    }
    pub fn read16(&mut self, register: Register16) -> u16 {
        let slice = unsafe { self.slice::<u16>() };
        slice[register as usize]
    }

    pub fn write8(&mut self, register: Register8, value: u8) -> () {
        let slice = unsafe { self.slice::<u8>() };
        slice[register as usize] = value
    }
    pub fn write16(&mut self, register: Register16, value: u16) -> () {
        let slice = unsafe { self.slice::<u16>() };
        slice[register as usize] = value
    }
}

pub struct Cpu {
    pub registers: RegisterBank,
}

impl Cpu {
    pub fn new() -> Cpu {
        let regs = RegisterBank {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 32,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        };
        Cpu { registers: regs }
    }

    pub fn ld8(&mut self, dest: LD8Destination, src: LD8Source) -> Cost {
        let mut cost = TICK;
        let source_value = match src {
            LD8Source::RegisterValue(register) => self.registers.read8(register),
            LD8Source::FromRegisterAddress(register) => {
                cost += TICK;
                0
            } // TODO: RAM fetch
            LD8Source::ImmediateValue(value) => {
                cost += TICK;
                value
            }
            LD8Source::FromImmediateAddress(address) => {
                cost += TICK * 3;
                0
            } // TODO: RAM fetch
        };
        match dest {
            LD8Destination::RegisterValue(register) => {
                self.registers.write8(register, source_value)
            }
            LD8Destination::ToRegisterAddress(register) => cost += TICK,
            LD8Destination::ToImmediateAddress(address) => cost += TICK * 3,
        }
        cost
    }
}
