use std::rc::Rc;
use std::cell::RefCell;

use crate::memory_gb::Address;
use crate::memory_gb::Byte;
use crate::memory_gb::MemoryBank;
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
    RegBC = 2,
    RegDE = 4,
    RegHL = 6,
    RegSP = 8,
    RegPC = 10,
}

#[derive(Clone, Copy)]
pub enum Flags {
    Z = 3,
    N = 2,
    H = 1,
    C = 0
}

#[derive(Clone, Copy)]
pub enum ConditionCodes {
    C,
    NC,
    NZ,
    Z,
    NA
}

// type MemoryMapRef = Rc<RefCell<MemoryMap>>;

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
        map.read::<Byte>(address)
    }
}
impl WriteByte for ByteRegisterIndirect {
    fn write_byte(&self, cpu: &mut Cpu, value: Byte) {
        let address = cpu.registers.read_word(self.register);
        let mut map = cpu.memory.borrow_mut();
        map.write(value, address)
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
        map.read::<Byte>(address)
    }
}
impl WriteByte for ByteRegisterOffsetIndirect {
    fn write_byte(&self, cpu: &mut Cpu, value: Byte) {
        let offset = cpu.registers.read_byte(self.register) as Address;
        let address = 0xFF00 + offset;
        let mut map = cpu.memory.borrow_mut();
        map.write(value, address)
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
    fn read_byte(&self, _: &mut Cpu) -> Byte {
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
        map.read::<Byte>(self.address)
    }
}
impl WriteByte for ByteImmediateIndirect {
    fn write_byte(&self, cpu: &mut Cpu, value: Byte) {
        let mut map = cpu.memory.borrow_mut();
        map.write(value, self.address)
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
        map.read::<Byte>(address)
    }
}
impl WriteByte for ByteImmediateOffsetIndirect {
    fn write_byte(&self, cpu: &mut Cpu, value: Byte) {
        let address = 0xFF00 + self.offset as Address;
        let mut map = cpu.memory.borrow_mut();
        map.write(value, address)
    }
}
impl ByteImmediateOffsetIndirect {
    pub fn new(offset: Byte) -> ByteImmediateOffsetIndirect {
        ByteImmediateOffsetIndirect{ offset }
    }
}

pub struct WordImmediate {
    data: Word,
}
impl ReadWord for WordImmediate {
    fn read_word(&self, _: &mut Cpu) -> Word {
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
        map.write(value, self.address)
    }
}
impl WordImmediateIndirect {
    pub fn new(address: Address) -> WordImmediateIndirect {
        WordImmediateIndirect{ address }
    }
}

#[repr(C)]
pub struct RegisterBank {
    // Registers are in the following order in memory
    // REGISTER F
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
    // Can be paired with register A

    // REGISTER A
    // Accumulator, typically used as destination for arithmetic ops
    // Can be paired with register F

    // REGISTER BC 
    // paired, general purpose registers

    // REGISTER DE
    // paired, general purpose registers

    // REGISTER HL
    // paired, general purpose registers that can point into memory

    // REGISTER SP
    // Stack pointer,  initialized to 0xFFFE, is exclusively a 16 bit register

    // REGISTER PC
    // Program counter, initialized to 0x100, is exclusively a 16 bit register

    registers: [Byte; 12],
}

impl MemoryRegion for RegisterBank {
    fn get_bank(&mut self, address: Address) -> Option<crate::memory_gb::MemoryBank> {
        if address > WordRegisterName::RegPC as Address {
            None
        } else {
            Some(MemoryBank{ start: 0x0000, data: &mut self.registers[..] })
        }
    }
}
impl RegisterBank {
    pub fn read_byte(&mut self, register: ByteRegisterName) -> Byte {
        self.read::<Byte>(register as Address)
    }
    pub fn read_word(&mut self, register: WordRegisterName) -> Word {
        self.read::<Word>(register as Address)
    }

    pub fn write_byte(&mut self, register: ByteRegisterName, value: Byte) -> () {
        self.write::<Byte>(value, register as Address)
    }
    pub fn write_word(&mut self, register: WordRegisterName, value: Word) -> () {
        self.write::<Word>(value, register as Address)
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
        println!("pc: {:#02x}", pc);
        self.write_word(WordRegisterName::RegPC, pc + increment);
    }
}

pub enum SideEffect {
    Halt,
    Stop,
    EnableInterrupt,
    EnableInterruptDelayed,
    DisableInterrupt,
}

pub enum StepResult {
    Step(u8),
    StepSideEffect(u8, SideEffect),
}

pub struct Cpu {
    pub registers: RegisterBank,
    pub memory: Rc<RefCell<MemoryMap>>,
    pub ime: bool,
    pub halted: bool,
    pub stopped: bool
}

impl Cpu {
    pub fn new(system_memory: Rc<RefCell<MemoryMap>>) -> Cpu {
        let regs = RegisterBank {

            registers: [
                0x00, // A
                0x00, // F
                0x00, // B
                0x00, // C
                0x00, // D
                0x00, // E
                0x00, // H
                0x00, // L
                0xFE, // SP LOW
                0xFF, // SP HIGH
                0x00, // PC LOW
                0x01, // PC HIGH
            ]
        };
        let mut new_cpu = Cpu { 
            registers: regs,
            memory: system_memory,
            ime: false,
            halted: false,
            stopped: false,
        };
        // TODO: Clean out after PPU is implemented. Cheat V-blank on 
        new_cpu.ld_byte(ByteImmediateIndirect::new(0xFF44), ByteImmediate::new(0x90));
        new_cpu
    }

    pub fn service_interrupt(&mut self) -> bool {
        // Check if there are serviceable interrupts and if there are, toggle off the highest priority IF bit
        // and hand back the ISR address of the associated interrupt to jump to
        let isr_location = {
            let mut memory = self.memory.borrow_mut();
            const IF_REG_ADDR: Address = 0xFF0F;
            const IE_REG_ADDR: Address = 0xFFFF;
            let reg_if = memory.read::<Byte>(IF_REG_ADDR);
            let reg_ie = memory.read::<Byte>(IE_REG_ADDR);
            let has_serviceable_interrupts = self.ime && ((reg_ie & reg_if) > 0);
            if has_serviceable_interrupts {
                let place = 0x01;
                let (new_if, isr_location) = if (reg_if & (place << 0)) > 0 {
                    const VBLANK_ISR_LOCATION: Address = 0x0040;
                    (!(place << 0) & reg_if, VBLANK_ISR_LOCATION)
                }
                else if (reg_if & (place << 1)) > 0 {
                    const STAT_ISR_LOCATION: Address = 0x0048;
                    (!(place << 1) & reg_if, STAT_ISR_LOCATION)
                }
                else if (reg_if & (place << 2)) > 0 {
                    const TIMER_ISR_LOCATION: Address = 0x0050;
                    (!(place << 2) & reg_if, TIMER_ISR_LOCATION)
                }
                else if (reg_if & (place << 3)) > 0 {
                    const SERIAL_ISR_LOCATION: Address = 0x0058;
                    (!(place << 3) & reg_if, SERIAL_ISR_LOCATION)
                }
                else {
                    const JOYPAD_ISR_LOCATION: Address = 0x0060;
                    (!(place << 4) & reg_if, JOYPAD_ISR_LOCATION)
                };
                memory.write::<Byte>(new_if, IF_REG_ADDR);
                Some(isr_location)
            } else {
                None
            }
        };
        // If an interrupt is to be serviced, toggle off master interrupt enable, push PC, and jump PC to ISR address
        match isr_location {
            Some(address) => {
                self.ime = false;
                self.call(address, ConditionCodes::NA);
                true
            }
            None => {
                false
            }
        }
    }

    pub fn run(&mut self) -> () {
        let mut enable_ime_next_frame = false;
        let mut enable_ime_this_frame = false;
        loop {
            if self.service_interrupt() {
                self.halted = false;
                self.stopped = false;
                continue
            }
            if !self.halted && !self.stopped  {
                // This song and dance needs to be done so that the IME is turned on only after the instruction following EI executes
                if enable_ime_next_frame {
                    enable_ime_next_frame = false;
                    enable_ime_this_frame = true;
                }
                let step_info = self.step();
                let cost = match step_info {
                    StepResult::StepSideEffect(cost, effect) => {
                        match effect {
                            SideEffect::Halt => {
                                self.halted = true;
                            }
                            SideEffect::Stop => {
                                self.halted = true;
                            }
                            SideEffect::EnableInterrupt => {
                                self.ime = true
                            }
                            SideEffect::EnableInterruptDelayed => {
                                enable_ime_next_frame = true;
                            }
                            SideEffect::DisableInterrupt => {
                                self.ime = false
                            }
                        }
                        cost
                    }
                    StepResult::Step(cost) => cost
                };
                if enable_ime_this_frame {
                    self.ime = true;
                    enable_ime_this_frame = false;
                }
            } 
        }
    }
}