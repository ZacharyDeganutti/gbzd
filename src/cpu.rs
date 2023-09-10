use std::rc::Rc;
use std::cell::RefCell;

use crate::memory_gb::Address;
use crate::memory_gb::Byte;
use crate::memory_gb::Word;
use crate::memory_gb::MemoryRegion;
use crate::memory_gb::MemoryMap;

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

// RHS operand possibilities for 8 bit loads
pub enum LDByteSource {
    RegisterValue(ByteRegister),
    FromRegisterAddress(WordRegister),
    FromRegisterOffsetAddress(ByteRegister),
    ImmediateValue(Byte),
    FromImmediateAddress(Address),
    FromImmediateOffsetAddress(Byte),
}
// LHS operand possibilities for 8 bit loads
pub enum LDByteDestination {
    RegisterValue(ByteRegister),
    ToRegisterAddress(WordRegister),
    ToRegisterOffsetAddress(ByteRegister),
    ToImmediateAddress(Address),
    ToImmediateOffsetAddress(Byte),
}

// RHS operand possibilities for 16 bit loads
pub enum LDWordSource {
    RegisterValue(WordRegister),
    ImmediateValue(Word),
}
// LHS operand possibilities for 16 bit loads
pub enum LDWordDestination {
    RegisterValue(WordRegister),
    ToImmediateAddress(Address),
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
    pub sp: Word, // Stack pointer,  initialized to 0xFFFE
    pub pc: Word, // Program counter, initialized to 0x100
}

impl MemoryRegion for RegisterBank {}
impl RegisterBank {
    pub fn read_byte_register(&mut self, register: ByteRegister) -> Byte {
        unsafe { self.read::<Byte>(register as Address) }
    }
    pub fn read_word_register(&mut self, register: WordRegister) -> Word {
        unsafe { self.read::<Word>(register as Address) }
    }

    pub fn write_byte_register(&mut self, register: ByteRegister, value: Byte) -> () {
        unsafe { self.write::<Byte>(value, register as Address) }
    }
    pub fn write_word_register(&mut self, register: WordRegister, value: Word) -> () {
        unsafe { self.write::<Word>(value, register as Address) }
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
            sp: 0xFFFE,
            pc: 0,
        };
        Cpu { 
            registers: regs,
            memory: system_memory
        }
    }

    pub fn ld_byte(&mut self, dest: LDByteDestination, src: LDByteSource) -> Cost {
        let mut cost = TICK; // fetch
        let source_value = match src {
            LDByteSource::RegisterValue(register) => {
                self.registers.read_byte_register(register)
            }
            LDByteSource::FromRegisterAddress(register) => {
                cost += TICK; // read register address
                let address = self.registers.read_word_register(register);
                let mut map = self.memory.borrow_mut();
                unsafe { map.read::<Byte>(address) }
            } 
            LDByteSource::FromRegisterOffsetAddress(register) => {
                cost += TICK; // read register offset address
                let offset = self.registers.read_byte_register(register);
                let address = 0xFF00 + offset as Address;
                let mut map = self.memory.borrow_mut();
                unsafe { map.read::<Byte>(address) }
            }
            LDByteSource::ImmediateValue(value) => {
                cost += TICK; // read immediate value
                value
            }
            LDByteSource::FromImmediateAddress(address) => {
                cost += TICK * 3; // read immediate upper, read immediate lower, read from address
                let mut map = self.memory.borrow_mut();
                unsafe { map.read::<Byte>(address) }
            }
            LDByteSource::FromImmediateOffsetAddress(offset) => {
                cost += TICK * 2; // read immediate, read offset address
                let address = 0xFF00 + offset as Address;
                let mut map = self.memory.borrow_mut();
                unsafe { map.read::<Byte>(address) }
            }
        };
        match dest {
            LDByteDestination::RegisterValue(register) => {
                self.registers.write_byte_register(register, source_value)
            }
            LDByteDestination::ToRegisterAddress(register) => { 
                cost += TICK; // write to register address
                let address = self.registers.read_word_register(register);
                let mut map = self.memory.borrow_mut();
                unsafe { map.write(source_value, address) }
            }
            LDByteDestination::ToRegisterOffsetAddress(register) => {
                cost += TICK; // write offset address
                let offset = self.registers.read_byte_register(register) as Address;
                let address = 0xFF00 + offset;
                let mut map = self.memory.borrow_mut();
                unsafe { map.write(source_value, address) }
            }
            LDByteDestination::ToImmediateAddress(address) => {
                cost += TICK * 3; // read immediate lower, read immediate upper, write to immediate address
                let mut map = self.memory.borrow_mut();
                unsafe { map.write(source_value, address) }
            }
            LDByteDestination::ToImmediateOffsetAddress(offset) => {
                cost += TICK * 2; // read immediate, write offset address
                let address = 0xFF00 + offset as Address;
                let mut map = self.memory.borrow_mut();
                unsafe { map.write(source_value, address) }
            }
        }
        cost
    }

    pub fn ld_word(&mut self, dest: LDWordDestination, src: LDWordSource) -> Cost {
        let mut cost = TICK; // fetch
        let mut register_sourced = false;
        let source_value = match src {
            LDWordSource::RegisterValue(register) => {
                register_sourced = true;
                self.registers.read_word_register(register)
            }
            LDWordSource::ImmediateValue(value) => {
                cost += TICK * 2; // read low byte, read high byte
                value
            }
        };
        match dest {
            LDWordDestination::RegisterValue(register) => {
                // Word register to Word register copying has an intrinsic 1 tick cost
                // Needs special case because loading immediate Word to registers is equivalent to loading 2 Bytes
                cost += if register_sourced { TICK } else { 0 };
                self.registers.write_word_register(register, source_value)
            }
            LDWordDestination::ToImmediateAddress(address) => {
                cost += TICK * 4; // read immediate lower, read immediate upper, write to immediate address, write to immediate address + 1
                let mut map = self.memory.borrow_mut();
                unsafe { map.write(source_value, address) }
            }
        }
        cost
    }
}