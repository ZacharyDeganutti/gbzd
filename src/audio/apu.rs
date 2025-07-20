use std::{cell::RefCell, ops::Add, rc::Rc};
use crate::{audio::audio::SampleWave, memory_gb::{Address, Byte, MemoryMap, MemoryRegion, Word}};

use super::audio::{DutyCycle, SquareWave};

// Dots per second = 2^22
const DOTS_PER_LENGTH_TICK: u32 = 2_u32.pow(14); // 256 hz tick
const DOTS_PER_SWEEP_TICK: u32 = 2_u32.pow(15); // 128 hz tick
const DOTS_PER_VOLUME_ENVELOPE_TICK: u32 = 2_u32.pow(16); // 64 hz tick
const DOTS_MODULO: u32 = DOTS_PER_VOLUME_ENVELOPE_TICK * 64;
const DOT_DURATION: f32 = 1.0 / 2_u32.pow(22) as f32;

pub struct Apu<'a> {
    memory: Rc<RefCell<MemoryMap<'a>>>,
    channel_1_volume_current: u8,
    channel_1_length_timer_current: u8,
    channel_1_sweep_pace_current: u8,
    channel_1_period_current: u32,
    channel_1_active: bool,

    channel_2_volume_current: u8,
    channel_2_length_timer_current: u8,
    channel_2_period_current: u32,
    channel_2_active: bool,

    channel_3_volume_shift: u8,
    channel_3_length_timer_current: u8,
    channel_3_period_current: u32,
    channel_3_wave_ram: [u8; 16],
    channel_3_active: bool,

    divider_previous: u8,
    divider_counter: u32,
}

impl<'a> Apu<'a> {
    pub fn new(memory_map: Rc<RefCell<MemoryMap<'a>>>) -> Apu<'a> {
        Apu {
            memory: memory_map,
            channel_1_volume_current: 0,
            channel_1_length_timer_current: 0,
            channel_1_sweep_pace_current: 0,
            channel_1_period_current: 0,
            channel_1_active: false,

            channel_2_volume_current: 0,
            channel_2_length_timer_current: 0,
            channel_2_period_current: 0,
            channel_2_active: false,

            channel_3_volume_shift: 0,
            channel_3_length_timer_current: 0,
            channel_3_period_current: 0,
            channel_3_wave_ram: [0; 16],
            channel_3_active: false,

            divider_previous: 0,
            divider_counter: 0,
        }
    }

    fn catchup_registers(&mut self) {
        const LENGTH_TIMER_EXPIRY: u8 = 64;
        let mut map = self.memory.borrow_mut();

        const BIT_4_MASK: u8 = 1 << 4;
        const BIT_7_MASK: u8 = 1 << 7;

        // handle global stuff
        const NR52_ADDR: Address = 0xFF26;
        let nr52_contents = map.read::<Byte>(NR52_ADDR);
        // clumsy audio disable handling. todo: make it clear the registers, also probably handle all of it in the memory map with a special handler
        if (nr52_contents & BIT_7_MASK) == 0 {
            self.channel_1_active = false;
            self.channel_2_active = false;
            self.channel_3_active = false;
            return
        }

        // handle trigger events
        // channel 1 trigger
        let ch1_triggered = map.apu_state.ch1_to_trigger;
        if ch1_triggered {
            map.apu_state.ch1_to_trigger = false;
        }

        // refresh internal values with triggered values
        if ch1_triggered {
            // Reset the volume
            let ch1_init_volume = map.apu_state.channel_1_initial_volume();
            self.channel_1_volume_current = ch1_init_volume;

            // Reload the period/frequency value
            let period = map.apu_state.channel_1_period();
            self.channel_1_period_current = period as u32;
            // Reset the length counter
            if (self.channel_1_length_timer_current >= LENGTH_TIMER_EXPIRY) || (self.channel_1_length_timer_current == 0) {
                self.channel_1_length_timer_current = map.apu_state.channel_1_length_timer();
            }
            // Reset sweep values
            self.channel_1_sweep_pace_current = map.apu_state.channel_1_sweep_pace();
            // Activate channel
            self.channel_1_active = true;
            // println!("TRIGGER CH1");
        }

        // update ch1 period if it was overwritten
        if map.apu_state.ch1_period_to_update {
            self.channel_1_period_current = map.apu_state.channel_1_period() as u32;
            map.apu_state.ch1_period_to_update = false;
        }

        // channel 2 trigger
        let ch2_triggered = map.apu_state.ch2_to_trigger;
        if ch2_triggered {
            map.apu_state.ch2_to_trigger = false;
        }

        // refresh internal values with triggered values
        if ch2_triggered {
            // Reset the volume
            let ch2_init_volume = map.apu_state.channel_2_initial_volume();
            self.channel_2_volume_current = ch2_init_volume;
            // Reload the period/frequency value
            let period = map.apu_state.channel_2_period();
            self.channel_2_period_current = period as u32;
            // Reset the length counter
            if (self.channel_2_length_timer_current >= LENGTH_TIMER_EXPIRY) || (self.channel_2_length_timer_current == 0) {
                self.channel_2_length_timer_current = map.apu_state.channel_2_length_timer();
            }
            // Activate channel
            self.channel_2_active = true;
        }

        // update ch2 period if it was overwritten
        if map.apu_state.ch2_period_to_update {
            self.channel_2_period_current = map.apu_state.channel_2_period() as u32;
            map.apu_state.ch2_period_to_update = false;
        }

        // channel 3 trigger
        let ch3_triggered = map.apu_state.ch3_to_trigger;
        if ch3_triggered {
            map.apu_state.ch3_to_trigger = false;
        }

        // Constantly reload wave RAM regardless of triggering because we're going offroading from the real behavior anyway
        const WAVE_RAM_BASE_ADDRESS: Address = 0xFF30;
        for wave_ram_byte_offset in 0..self.channel_3_wave_ram.len() {
            self.channel_3_wave_ram[wave_ram_byte_offset] = map.read(WAVE_RAM_BASE_ADDRESS + wave_ram_byte_offset as Address);
        }

        // refresh internal values with triggered values
        if ch3_triggered {
            // Check the DAC status
            let dac_enabled = map.apu_state.channel_3_dac_enabled();

            // Reset the volume
            let ch3_output_level = map.apu_state.channel_3_output_level();
            self.channel_3_volume_shift = if ch3_output_level == 0 { 4 } else { ch3_output_level - 1 };
            
            // Reload the period/frequency value
            let period = map.apu_state.channel_3_period();
            self.channel_3_period_current = period as u32;

            // Reset the length counter
            if (self.channel_3_length_timer_current >= LENGTH_TIMER_EXPIRY) || (self.channel_3_length_timer_current == 0) {
                self.channel_3_length_timer_current = map.apu_state.channel_3_length_timer();
            }

            // Don't bother with resetting the phase on trigger, it's probably not that noticeable

            // Activate channel if the dac is enabled
            self.channel_3_active = dac_enabled;
        }
        
        // update ch3 period if it was overwritten
        if map.apu_state.ch3_period_to_update {
            self.channel_3_period_current = map.apu_state.channel_3_period() as u32;
            map.apu_state.ch3_period_to_update = false;
        }

        // Reset the volume. The one in the trigger block can probably be removed for redundancy,
        // but that's explicitly defined as something that happens on trigger. There's probably something
        // incorrect about doing this every time, but it sounds correct in more cases to do so.
        let ch3_output_level = map.apu_state.channel_3_output_level();
        self.channel_3_volume_shift = if ch3_output_level == 0 { 4 } else { ch3_output_level - 1 };
        
        // do timed events when the apu divider counter triggers
        const DIV_ADDR: Address = 0xFF04;
        let current_divider = map.read::<Byte>(DIV_ADDR);

        let pre_check_divider_counter = self.divider_counter;

        let mut update_div_apu_counter = |prev_divider| {
            let bit_4_falling_edge = ((prev_divider & BIT_4_MASK) & ((prev_divider ^ current_divider) & BIT_4_MASK)) > 0;
            if bit_4_falling_edge {
                self.divider_counter = self.divider_counter.wrapping_add(1);
            }
        };
        // If the divider was reset, just check against the last value once
        if map.div_reset {
            update_div_apu_counter(self.divider_previous);
            map.div_reset = false;
            self.divider_previous = current_divider;
        }
        // Otherwise it's necessary to 'catch up' the cycles.
        else {
            while self.divider_previous != current_divider {
                update_div_apu_counter(self.divider_previous);
                self.divider_previous = self.divider_previous.wrapping_add(1);
            }
        }
        let post_check_divider_counter = self.divider_counter;

        let divider_counter_changed = (pre_check_divider_counter ^ post_check_divider_counter) > 0;

        if divider_counter_changed {
            // Channel 1 period sweep (every 4 * pace div-apu ticks)
            let channel_1_sweep_pace = self.channel_1_sweep_pace_current;
            let channel_1_sweep_increasing = map.apu_state.channel_1_sweep_increasing();
            let channel_1_sweep_step = map.apu_state.channel_1_sweep_step();
            if (self.channel_1_active) && (channel_1_sweep_pace > 0) && ((self.divider_counter % (channel_1_sweep_pace as u32 * 4)) == 0) {
                if ((self.channel_1_period_current >> 8) >= 0x7FF) || (self.channel_1_period_current == 0) {
                    // on overflow, cut the channel
                    self.channel_1_active = false;
                } else if channel_1_sweep_increasing {
                    self.channel_1_period_current -= self.channel_1_period_current / 2_u32.pow(channel_1_sweep_step as u32);
                } else if !channel_1_sweep_increasing {
                    self.channel_1_period_current += self.channel_1_period_current / 2_u32.pow(channel_1_sweep_step as u32);
                }
                // Write back modified period value
                let new_nr13 = (self.channel_1_period_current & 0xFF) as u8;
                let new_nr14 = (map.apu_state.read_nr14() & 0xC0) | ((self.channel_1_period_current >> 8) as u8 & 0b111);
                map.apu_state.write_nr13(new_nr13);
                map.apu_state.write_nr14(new_nr14);
            }
            
            // Adjust length timers (every 2 div-apu ticks)
            if (self.divider_counter % 2) == 0 {
                let channel_1_length_timer_enabled = map.apu_state.channel_1_length_timer_enabled();
                if channel_1_length_timer_enabled && (self.channel_1_length_timer_current < LENGTH_TIMER_EXPIRY) {
                    self.channel_1_length_timer_current += 1;
                    if self.channel_1_length_timer_current == LENGTH_TIMER_EXPIRY {
                        self.channel_1_active = false;
                    }
                }
                let channel_2_length_timer_enabled = map.apu_state.channel_2_length_timer_enabled();
                if channel_2_length_timer_enabled && (self.channel_2_length_timer_current < LENGTH_TIMER_EXPIRY) {
                    self.channel_2_length_timer_current += 1;
                    if self.channel_2_length_timer_current == LENGTH_TIMER_EXPIRY {
                        self.channel_2_active = false;
                    }
                }
                let channel_3_length_timer_enabled = map.apu_state.channel_3_length_timer_enabled();
                if channel_3_length_timer_enabled && (self.channel_3_length_timer_current < LENGTH_TIMER_EXPIRY) {
                    self.channel_3_length_timer_current += 1;
                    if self.channel_3_length_timer_current == LENGTH_TIMER_EXPIRY {
                        self.channel_3_active = false;
                    }
                }
            }

            // Volume sweeps (every 8 * pace div-apu ticks)
            // CH1
            let channel_1_volume_sweep_pace = map.apu_state.channel_1_volume_sweep_pace();
            let channel_1_volume_sweep_increasing = map.apu_state.channel_1_volume_sweep_increasing();
            if (channel_1_volume_sweep_pace > 0) && (self.divider_counter % (channel_1_volume_sweep_pace as u32 * 8)) == 0 {
                if channel_1_volume_sweep_increasing && (self.channel_1_volume_current < 0b1111) {
                    self.channel_1_volume_current += 1;
                } else if !channel_1_volume_sweep_increasing && (self.channel_1_volume_current > 0) {
                    self.channel_1_volume_current -= 1;
                }
            }
            // CH2
            let channel_2_volume_sweep_pace = map.apu_state.channel_2_volume_sweep_pace();
            let channel_2_volume_sweep_increasing = map.apu_state.channel_2_volume_sweep_increasing();
            if (channel_2_volume_sweep_pace > 0) && (self.divider_counter % (channel_2_volume_sweep_pace as u32 * 8)) == 0 {
                if channel_2_volume_sweep_increasing && (self.channel_2_volume_current < 0b1111) {
                    self.channel_2_volume_current += 1;
                } else if !channel_2_volume_sweep_increasing && (self.channel_2_volume_current > 0) {
                    self.channel_2_volume_current -= 1;
                }
            }
        }
    }

    fn parse_channel_1(&self) -> SquareWave {
        const LENGTH_AND_DUTY_CYCLE_ADDRESS: Address = 0xFF11;

        let mut mem = self.memory.borrow_mut();

        let length_and_duty_cycle = mem.read::<Byte>(LENGTH_AND_DUTY_CYCLE_ADDRESS);

        let duty_cycle = match length_and_duty_cycle >> 6 {
            0b00 => DutyCycle::Eighth,
            0b01 => DutyCycle::Quarter,
            0b10 => DutyCycle::Half,
            _    => DutyCycle::ThreeQuarter
        };

        // TODO: Have length set volume to 0 if it's at 64

        let frequency = 131072.0 / (2048.0 - self.channel_1_period_current as f32);

        const VOLUME_CAP: f32 = 0.05;
        // Mute if the length timer is maxed or the channel is off
        let volume: f32 = VOLUME_CAP * if !self.channel_1_active {
            0.0
        } 
        else {
            let vol = self.channel_1_volume_current as f32 / 15.0;
            vol
        };

        SquareWave {
            duty_cycle,
            volume,
            frequency,
        }
    }

    fn parse_channel_2(&self) -> SquareWave {
        const LENGTH_AND_DUTY_CYCLE_ADDRESS: Address = 0xFF16;

        let mut mem = self.memory.borrow_mut();

        let length_and_duty_cycle = mem.read::<Byte>(LENGTH_AND_DUTY_CYCLE_ADDRESS);

        let duty_cycle = match length_and_duty_cycle >> 6 {
            0b00 => DutyCycle::Eighth,
            0b01 => DutyCycle::Quarter,
            0b10 => DutyCycle::Half,
            _    => DutyCycle::ThreeQuarter
        };

        // TODO: Have length set volume to 0 if it's at 64

        let frequency = 131072.0 / (2048.0 - self.channel_2_period_current as f32);

        const VOLUME_CAP: f32 = 0.05;
        let volume: f32 = VOLUME_CAP * if !self.channel_2_active {
            0.0
        } 
        else {
            let vol = self.channel_2_volume_current as f32 / 15.0;
            vol
        };

        SquareWave {
            duty_cycle,
            volume,
            frequency,
        }
    }

    fn parse_channel_3(&self) -> SampleWave<32> {
        let frequency = 65536.0 / (2048.0 - self.channel_3_period_current as f32);
        let volume_shift = if self.channel_3_active { self.channel_3_volume_shift } else { 4 };
        
        const SAMPLE_COUNT: usize = 32;

        let mut volume_samples: [f32; SAMPLE_COUNT] = [0.0; SAMPLE_COUNT];

        const VOLUME_CAP: f32 = 0.05;

        for byte_offset in 0..self.channel_3_wave_ram.len() {
            let index = byte_offset * 2;

            let upper_sample = ((self.channel_3_wave_ram[byte_offset] & 0xF0) >> 4) >> volume_shift;
            let lower_sample = (self.channel_3_wave_ram[byte_offset] & 0x0F) >> volume_shift;

            let upper_f32_sample = (upper_sample as f32 - ((0xF >> volume_shift) as f32 / 2.0)) / 7.5;
            let lower_f32_sample = (lower_sample as f32 - ((0xF >> volume_shift) as f32 / 2.0)) / 7.5;

            volume_samples[index] = VOLUME_CAP * upper_f32_sample;
            volume_samples[index + 1] = VOLUME_CAP * lower_f32_sample;
        }

        SampleWave { 
            volume_samples, 
            frequency
        }
    }

    pub fn update_waves(&mut self) -> (SquareWave, SquareWave, SampleWave<32>) {
        self.catchup_registers();
        return (
            self.parse_channel_1(),
            self.parse_channel_2(),
            self.parse_channel_3()
        )
    }
}