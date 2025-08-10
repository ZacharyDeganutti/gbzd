use crate::audio::audio::AudioDirection;
use crate::memory_gb::Byte;
use crate::memory_gb::MemoryUnit;
use crate::memory_gb::Word;

const BIT_7_MASK: u8 = 1 << 7;

// This data structure exists to encapsulate state changes driven by writing to / reading from APU registers
impl ApuRegisters {
    // Read registers
    // CH1 Registers
    pub fn read_nr10(&self) -> Byte {
        self.nr10
    }
    pub fn read_nr11(&self) -> Byte {
        self.nr11
    }
    pub fn read_nr12(&self) -> Byte {
        self.nr12
    }
    pub fn read_nr13(&self) -> Byte {
        self.nr13
    }
    pub fn read_nr14(&self) -> Byte {
        self.nr14
    }

    // CH2 Registers
    pub fn read_nr21(&self) -> Byte {
        self.nr21
    }
    pub fn read_nr22(&self) -> Byte {
        self.nr22
    }
    pub fn read_nr23(&self) -> Byte {
        self.nr23
    }
    pub fn read_nr24(&self) -> Byte {
        self.nr24
    }

    // CH3 Registers
    pub fn read_nr30(&self) -> Byte {
        self.nr30
    }
    pub fn read_nr31(&self) -> Byte {
        self.nr31
    }
    pub fn read_nr32(&self) -> Byte {
        self.nr32
    }
    pub fn read_nr33(&self) -> Byte {
        self.nr33
    }
    pub fn read_nr34(&self) -> Byte {
        self.nr34
    }

    // CH4 Registers
    pub fn read_nr41(&self) -> Byte {
        self.nr41
    }
    pub fn read_nr42(&self) -> Byte {
        self.nr42
    }
    pub fn read_nr43(&self) -> Byte {
        self.nr43
    }
    pub fn read_nr44(&self) -> Byte {
        self.nr44
    }

    // Others
    pub fn read_nr50(&self) -> Byte {
        self.nr50
    }
    pub fn read_nr51(&self) -> Byte {
        self.nr51
    }
    pub fn read_nr52(&self) -> Byte {
        self.nr52
    }

    // Write registers
    // CH1 Registers
    pub fn write_nr10(&mut self, value: Byte) {
        self.nr10 = value;
    }
    pub fn write_nr11(&mut self, value: Byte) {
        self.nr11 = value;
    }
    pub fn write_nr12(&mut self, value: Byte) {
        self.nr12 = value;
    }
    pub fn write_nr13(&mut self, value: Byte) {
        self.nr13 = value;
        self.ch1_period_to_update = true;
    }
    pub fn write_nr14(&mut self, value: Byte) {
        self.nr14 = value;
        self.ch1_to_trigger = (BIT_7_MASK & value) > 0; 
        self.ch1_period_to_update = true;
    }

    // CH2 Registers
    pub fn write_nr21(&mut self, value: Byte) {
        self.nr21 = value;
    }
    pub fn write_nr22(&mut self, value: Byte) {
        self.nr22 = value;
    }
    pub fn write_nr23(&mut self, value: Byte) {
        self.nr23 = value;
        self.ch2_period_to_update = true;
    }
    pub fn write_nr24(&mut self, value: Byte) {
        self.nr24 = value;
        self.ch2_to_trigger = (BIT_7_MASK & value) > 0;
        self.ch2_period_to_update = true;
    }

    // CH3 Registers
    pub fn write_nr30(&mut self, value: Byte) {
        self.nr30 = value;
    }
    pub fn write_nr31(&mut self, value: Byte) {
        self.nr31 = value;
    }
    pub fn write_nr32(&mut self, value: Byte) {
        self.nr32 = value;
    }
    pub fn write_nr33(&mut self, value: Byte) {
        self.nr33 = value;
        self.ch3_period_to_update = true;
    }
    pub fn write_nr34(&mut self, value: Byte) {
        self.nr34 = value;
        self.ch3_to_trigger = (BIT_7_MASK & value) > 0;
        self.ch3_period_to_update = true;
    }

    // CH4 Registers
    pub fn write_nr41(&mut self, value: Byte) {
        self.nr41 = value;
    }
    pub fn write_nr42(&mut self, value: Byte) {
        self.nr42 = value;
    }
    pub fn write_nr43(&mut self, value: Byte) {
        self.nr43 = value;
    }
    pub fn write_nr44(&mut self, value: Byte) {
        self.nr44 = value;
        self.ch4_to_trigger = (BIT_7_MASK & value) > 0;
    }

    // Others
    pub fn write_nr50(&mut self, value: Byte) {
        self.nr50 = value.demote();
    }

    pub fn write_nr51(&mut self, value: Byte) {
        self.nr51 = value.demote();
    }

    pub fn write_nr52(&mut self, value: Byte) {
        // Channel status bits are read only
        self.nr52 = value & 0x80;
    }

    pub fn update_nr52_channel_status(&mut self, ch1_on: bool, ch2_on: bool, ch3_on: bool, ch4_on: bool) {
        let bits = (ch1_on as u8) 
            | ((ch2_on as u8) << 1)
            | ((ch3_on as u8) << 2)
            | ((ch4_on as u8) << 3);
        
        self.nr52 = (self.nr52 & 0xF0) | bits;
    }

    // Convenience methods for readability of APU implementation
    // General
    pub fn master_audio_is_enabled(&self) -> bool {
        (self.nr52 & BIT_7_MASK) > 0
    }

    // Left output, ~0-1 range. Value is always greater than 0
    pub fn left_volume_multiplier(&self) -> f32 {
        let raw_volume = ((self.nr52 & 0x70) >> 4) + 1;
        raw_volume as f32 / 8.0
    } 

    // Right output, ~0-1 range. Value is always greater than 0
    pub fn right_volume_multiplier(&self) -> f32 {
        let raw_volume = (self.nr52 & 0x7) + 1;
        raw_volume as f32 / 8.0
    } 

    // Generic helpers
    fn audio_direction(register: u8, half_mask: u8) -> AudioDirection {
        let upper_mask = half_mask << 4;
        let lower_mask = half_mask;
        let mask = upper_mask | lower_mask;
        match register & mask {
            x if x == upper_mask => AudioDirection::Left,
            x if x == lower_mask => AudioDirection::Right,
            x if x == mask => AudioDirection::Center,
            _ => AudioDirection::None
        }
    }

    // Channel 1
    pub fn channel_1_sweep_pace(&self) -> u8 {
        (self.nr10 >> 4) & 0b111
    }

    pub fn channel_1_sweep_increasing(&self) -> bool {
        (self.nr10 & 0b1000) > 0
    }

    pub fn channel_1_sweep_step(&self) -> u8 {
        self.nr10 & 0b111
    }

    pub fn channel_1_length_timer(&self) -> u8 {
        self.nr11 & 0x3F
    }

    pub fn channel_1_initial_volume(&self) -> u8 {
        self.nr12 >> 4
    }

    pub fn channel_1_volume_sweep_pace(&self) -> u8 {
        self.nr12 & 0b111
    }

    pub fn channel_1_volume_sweep_increasing(&self) -> bool {
        (self.nr12 & 0b1000) > 0
    }

    pub fn channel_1_period(&self) -> u16 {
        (self.nr13 as u16) | ((self.nr14 as u16 & 0b111) << 8)
    }

    pub fn channel_1_length_timer_enabled(&self) -> bool {
        (self.nr14 & (1 << 6)) > 0
    }

    pub fn channel_1_audio_direction(&self) -> AudioDirection {
        Self::audio_direction(self.nr51, 0x1)
    }

    pub fn channel_1_dac_enabled(&self) -> bool {
        (self.nr12 & 0xF8) != 0
    }

    // Channel 2
    pub fn channel_2_length_timer(&self) -> u8 {
        self.nr21 & 0x3F
    }

    pub fn channel_2_initial_volume(&self) -> u8 {
        self.nr22 >> 4
    }

    pub fn channel_2_volume_sweep_pace(&self) -> u8 {
        self.nr22 & 0b111
    }

    pub fn channel_2_volume_sweep_increasing(&self) -> bool {
        (self.nr22 & 0b1000) > 0
    }

    pub fn channel_2_period(&self) -> u16 {
        (self.nr23 as u16) | ((self.nr24 as u16 & 0b111) << 8)
    }

    pub fn channel_2_length_timer_enabled(&self) -> bool {
        (self.nr24 & (1 << 6)) > 0
    }

    pub fn channel_2_audio_direction(&self) -> AudioDirection {
        Self::audio_direction(self.nr51, 0x2)
    }

    pub fn channel_2_dac_enabled(&self) -> bool {
        (self.nr22 & 0xF8) != 0
    }

    // Channel 3
    pub fn channel_3_dac_enabled(&self) -> bool {
        (self.nr30 & BIT_7_MASK) > 0
    }

    pub fn channel_3_length_timer(&self) -> u8 {
        self.nr31 & 0x3F
    }

    pub fn channel_3_output_level(&self) -> u8 {
        self.nr32 >> 5
    }

    pub fn channel_3_period(&self) -> u16 {
        (self.nr33 as Word) | ((self.nr34 as Word & 0b111) << 8)
    }

    pub fn channel_3_length_timer_enabled(&self) -> bool {
        (self.nr34 & (1 << 6)) > 0
    }

    pub fn channel_3_audio_direction(&self) -> AudioDirection {
        Self::audio_direction(self.nr51, 0x4)
    }

    // Channel 4
    pub fn tick_lfsr(&mut self) -> () {
        self.lfsr_counter = self.lfsr_counter.wrapping_add(1);
    }

    pub fn channel_4_length_timer(&self) -> u8 {
        self.nr41 & 0x3F
    }

    pub fn channel_4_initial_volume(&self) -> u8 {
        self.nr42 >> 4
    }

    pub fn channel_4_volume_sweep_pace(&self) -> u8 {
        self.nr42 & 0b111
    }

    pub fn channel_4_volume_sweep_increasing(&self) -> bool {
        (self.nr42 & 0b1000) > 0
    }

    pub fn channel_4_clock_shift(&self) -> u8 {
        self.nr43 >> 4
    }

    pub fn channel_4_lfsr_width(&self) -> LfsrWidth {
        if (self.nr43 & 0b1000) > 0 {
            LfsrWidth::Seven
        }
        else {
            LfsrWidth::Fifteen
        }
    }

    pub fn channel_4_clock_divider(&self) -> u8 {
        self.nr43 & 0b111
    }

    pub fn channel_4_length_timer_enabled(&self) -> bool {
        (self.nr44 & (1 << 6)) > 0
    }

    pub fn channel_4_audio_direction(&self) -> AudioDirection {
        Self::audio_direction(self.nr51, 0x8)
    }

    pub fn channel_4_dac_enabled(&self) -> bool {
        (self.nr42 & 0xF8) != 0
    }

    // We can load this with zeroes, cpu init handles populating these with post-boot values
    pub fn new() -> ApuRegisters {
        ApuRegisters {
            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
            nr30: 0,
            nr31: 0,
            nr32: 0,
            nr33: 0,
            nr34: 0,
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
            nr50: 0,
            nr51: 0,
            nr52: 0,
            ch1_to_trigger: false,
            ch2_to_trigger: false,
            ch3_to_trigger: false,
            ch4_to_trigger: false,
            ch1_period_to_update: false,
            ch2_period_to_update: false,
            ch3_period_to_update: false,
            lfsr_counter: 0,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum LfsrWidth {
    Fifteen,
    Seven,
}

pub struct ApuRegisters {
    nr10: Byte,
    nr11: Byte,
    nr12: Byte,
    nr13: Byte,
    nr14: Byte,
    nr21: Byte,
    nr22: Byte,
    nr23: Byte,
    nr24: Byte,
    nr30: Byte,
    nr31: Byte,
    nr32: Byte,
    nr33: Byte,
    nr34: Byte,
    nr41: Byte,
    nr42: Byte,
    nr43: Byte,
    nr44: Byte,
    nr50: Byte,
    nr51: Byte,
    nr52: Byte,
    pub ch1_to_trigger: bool,
    pub ch2_to_trigger: bool,
    pub ch3_to_trigger: bool,
    pub ch4_to_trigger: bool,
    pub ch1_period_to_update: bool,
    pub ch2_period_to_update: bool,
    pub ch3_period_to_update: bool,
    pub lfsr_counter: u64,
}
