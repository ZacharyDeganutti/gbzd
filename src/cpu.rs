use std::rc::Rc;
use std::cell::RefCell;

use crate::memory_gb::Address;
use crate::memory_gb::Single;
use crate::memory_gb::Double;
use crate::memory_gb::MemoryRegion;
use crate::memory_gb::MemoryMap;

pub enum SingleRegister {
    RegA = 1,
    RegF = 0,
    RegB = 3,
    RegC = 2,
    RegD = 5,
    RegE = 4,
    RegH = 7,
    RegL = 6,
}

pub enum DoubleRegister {
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
pub enum LDSingleSource {
    RegisterValue(SingleRegister),
    FromRegisterAddress(DoubleRegister),
    ImmediateValue(Single),
    FromImmediateAddress(Address),
}

// LHS operand possibilities for 8 bit loads
// TODO: Genericize this against MemoryUnit so that LD can be made generic against MemoryUnit
pub enum LDSingleDestination {
    RegisterValue(SingleRegister),
    ToRegisterAddress(DoubleRegister),
    ToImmediateAddress(Address),
}

#[repr(C)]
pub struct RegisterBank {
    f: Single,
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
    pub a: Single,
    // Accumulator, typically used as destination for arithmetic ops
    pub c: Single,
    pub b: Single,
    // BC paired, general purpose registers
    pub e: Single,
    pub d: Single,
    // DE paired, general purpose registers
    pub l: Single,
    pub h: Single,
    // HL paired, general purpose registers that can point into memory
    pub sp: Double, // Stack pointer,  initialized to 0xFFFE
    pub pc: Double, // Program counter, initialized to 0x100
}

impl MemoryRegion for RegisterBank {}
impl RegisterBank {
    pub fn read_single_register(&mut self, register: SingleRegister) -> Single {
        unsafe { self.read::<Single>(register as Address) }
    }
    pub fn read_double_register(&mut self, register: DoubleRegister) -> Double {
        unsafe { self.read::<Double>(register as Address) }
    }

    pub fn write_single_register(&mut self, register: SingleRegister, value: Single) -> () {
        unsafe { self.write::<Single>(value, register as Address) }
    }
    pub fn write_double_register(&mut self, register: DoubleRegister, value: Double) -> () {
        unsafe { self.write::<Double>(value, register as Address) }
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
            sp: 0,
            pc: 0,
        };
        Cpu { 
            registers: regs,
            memory: system_memory
        }
    }

    pub fn ld_single(&mut self, dest: LDSingleDestination, src: LDSingleSource) -> Cost {
        let mut cost = TICK; // fetch
        let source_value = match src {
            LDSingleSource::RegisterValue(register) => {
                self.registers.read_single_register(register)
            }
            LDSingleSource::FromRegisterAddress(register) => {
                cost += TICK; // read register address
                let address = self.registers.read_double_register(register);
                let mut map = self.memory.borrow_mut();
                unsafe { map.read::<Single>(address) }
            } 
            LDSingleSource::ImmediateValue(value) => {
                cost += TICK; // read immediate value
                value
            }
            LDSingleSource::FromImmediateAddress(address) => {
                cost += TICK * 3; // read immediate upper, read immediate lower, read from address
                let mut map = self.memory.borrow_mut();
                unsafe { map.read::<Single>(address) }
            }
        };
        match dest {
            LDSingleDestination::RegisterValue(register) => {
                self.registers.write_single_register(register, source_value)
            }
            LDSingleDestination::ToRegisterAddress(register) => { 
                cost += TICK; // write to register address
                let address = self.registers.read_double_register(register);
                let mut map = self.memory.borrow_mut();
                unsafe { map.write(source_value, address) }
            }
            LDSingleDestination::ToImmediateAddress(address) => {
                cost += TICK * 3; // read immediate lower, read immediate upper, write to immediate address
                let mut map = self.memory.borrow_mut();
                unsafe { map.write(source_value, address) }
            }
        }
        cost
    }
}
