use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::memory_gb::{Address, Byte, MemoryMap, MemoryRegion};

pub enum JoypadMode {
    DPad,
    Buttons,
    Unselected
}

pub struct Joypad {
    button_values: Byte,
    mode: JoypadMode
}

// Things can get funky when tracing Joypad code
// InputHandler is responsible for changing the value of the Joypad buttons, and triggering joypad interrupts when applicable
// The CPU is responsible for setting the Joypad mode indirectly by writing to the select bits of the joypad register
// InputHandler has shared ownership of the memory map (along with CPU), and the memory map owns the Joypad
impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            // bitmask of
            // 7: | down | up | left | right | start | select | b | a | :0
            button_values: 0xFF,
            mode: JoypadMode::Buttons
        }
    }

    pub fn set_mode(&mut self, mode: JoypadMode) {
        self.mode = mode;
    }

    pub fn read(&self) -> Byte {
        match self.mode {
            JoypadMode::Buttons => (1 << 5) | (self.button_values & 0x0F),
            JoypadMode::DPad => (1 << 4) | ((self.button_values >> 4) & 0x0F),
            JoypadMode::Unselected => 0x0F
        }
    }
}

pub enum ButtonState {
    Pressed = 0,
    Unpressed = 1
}

pub trait InputDevice {
    fn a_pressed(&self) -> ButtonState;
    fn b_pressed(&self) -> ButtonState;
    fn up_pressed(&self) -> ButtonState;
    fn down_pressed(&self) -> ButtonState;
    fn left_pressed(&self) -> ButtonState;
    fn right_pressed(&self) -> ButtonState;
    fn start_pressed(&self) -> ButtonState;
    fn select_pressed(&self) -> ButtonState;
}

pub struct DummyDevice {

}

impl InputDevice for DummyDevice {
    fn a_pressed(&self) -> ButtonState {
        ButtonState::Unpressed
    }
    fn b_pressed(&self) -> ButtonState {
        ButtonState::Unpressed
    }
    fn up_pressed(&self) -> ButtonState {
        ButtonState::Unpressed
    }
    fn down_pressed(&self) -> ButtonState {
        ButtonState::Unpressed
    }
    fn left_pressed(&self) -> ButtonState {
        ButtonState::Unpressed
    }
    fn right_pressed(&self) -> ButtonState {
        ButtonState::Unpressed
    }
    fn start_pressed(&self) -> ButtonState {
        ButtonState::Unpressed
    }
    fn select_pressed(&self) -> ButtonState {
        ButtonState::Unpressed
    }
}

pub struct InputHandler<'a> {
    devices: Vec<Box<dyn InputDevice>>,
    memory: Rc<RefCell<MemoryMap<'a>>>,
    last_button_state: Byte
}

impl<'a> InputHandler<'a> {
    pub fn new(devices: Vec<Box<dyn InputDevice>>, memory: Rc<RefCell<MemoryMap<'a>>>) -> Self {
        InputHandler {
            devices,
            memory,
            last_button_state: 0xFF
        }
    }

    pub fn poll(&mut self) {
        let mut sum_of_button_states: u8 = 0;
        for device in self.devices.iter() {
            sum_of_button_states |= !(InputHandler::get_button_state(&**device));
        }
        sum_of_button_states = !sum_of_button_states;

        let mut mem = self.memory.borrow_mut();
        mem.joypad.button_values = sum_of_button_states;

        // Fire off joypad interrupt if one of the button values has gone from high to low
        const IF_REG_ADDR: Address = 0xFF0F;
        let mut interrupt_flag: Byte = mem.read(IF_REG_ADDR);
        if ((self.last_button_state ^ sum_of_button_states) & self.last_button_state) > 0
        {
            // println!("Raising joypad interrupt!");
            interrupt_flag |= (1 << 4);
        }
        mem.write(interrupt_flag, IF_REG_ADDR);

        self.last_button_state = sum_of_button_states;
    }

    fn get_button_state(device: &dyn InputDevice) -> Byte {
        let state: Byte =
            (device.a_pressed() as u8) |
            (device.b_pressed() as u8) << 1 |
            (device.select_pressed() as u8) << 2 |
            (device.start_pressed() as u8) << 3 |
            (device.right_pressed() as u8) << 4 |
            (device.left_pressed() as u8) << 5 |
            (device.up_pressed() as u8) << 6 |
            (device.down_pressed() as u8) << 7;
        state
    }
}