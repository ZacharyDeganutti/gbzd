use std::{cell::RefCell, rc::Rc};
use crate::memory_gb::{Address, Byte, MemoryMap, MemoryRegion, Word};

use super::audio::{DutyCycle, SquareWave};

// Dots per second = 2^22
const DOTS_PER_LENGTH_TICK: u32 = 2_u32.pow(14); // 256 hz tick
const DOTS_PER_SWEEP_TICK: u32 = 2_u32.pow(15); // 128 hz tick
const DOTS_PER_VOLUME_ENVELOPE_TICK: u32 = 2_u32.pow(16); // 64 hz tick
const DOTS_MODULO: u32 = DOTS_PER_VOLUME_ENVELOPE_TICK * 64;
const DOT_DURATION: f32 = 1.0 / 2_u32.pow(22) as f32;

pub struct Apu<'a> {
    memory: Rc<RefCell<MemoryMap<'a>>>,
    current_timing_dot: u32,
    channel_1_volume_current: u8,
    channel_1_length_timer_current: u8,
    channel_1_sweep_pace_current: u8,
    channel_1_period_current: u32,
    channel_2_volume_current: u8,
    channel_2_length_timer_current: u8,
    channel_2_sweep_pace_current: u8,
    channel_2_period_current: u32,
}

impl<'a> Apu<'a> {
    pub fn new(memory_map: Rc<RefCell<MemoryMap<'a>>>) -> Apu<'a> {
        Apu {
            memory: memory_map,
            current_timing_dot: 0,
            channel_1_volume_current: 0,
            channel_1_length_timer_current: 0,
            channel_1_sweep_pace_current: 0,
            channel_1_period_current: 0,
            channel_2_volume_current: 0,
            channel_2_length_timer_current: 0,
            channel_2_sweep_pace_current: 0,
            channel_2_period_current: 0,
        }
    }

    fn catchup_registers(&mut self, dots_elapsed: u16) {
        const LENGTH_TIMER_EXPIRY: u8 = 64;
        let mut map = self.memory.borrow_mut();

        // handle trigger events
        const BIT_7_MASK: u8 = 1 << 7;

        // channel 1 trigger
        const NR10_ADDR: Address = 0xFF10;
        let nr10_contents = map.read::<Byte>(NR10_ADDR);
        const NR11_ADDRESS: Address = 0xFF11;
        let nr11_contents = map.read::<Byte>(NR11_ADDRESS);
        const NR12_ADDR: Address = 0xFF12;
        let nr12_contents = map.read::<Byte>(NR12_ADDR);
        const NR13_ADDR: Address = 0xFF13;
        let nr13_contents = map.read::<Byte>(NR13_ADDR);
        const NR14_ADDR: Address = 0xFF14;
        let nr14_contents = map.read::<Byte>(NR14_ADDR);
        
        let ch1_triggered = (nr14_contents & BIT_7_MASK) > 0;
        // clear nr14 trigger bit after read
        map.write::<Byte>(nr14_contents & !(BIT_7_MASK), NR14_ADDR);
        // refresh internal values with triggered values
        if ch1_triggered {
            // Reset the volume
            let ch1_init_volume = nr12_contents >> 4;
            self.channel_1_volume_current = ch1_init_volume;
            // Reload the period/frequency value
            
            let period = (nr13_contents as Word) | ((nr14_contents as Word & 0b111) << 8);
            self.channel_1_period_current = period as u32;
            // Reset the length counter
            if self.channel_1_length_timer_current >= LENGTH_TIMER_EXPIRY {
                self.channel_1_length_timer_current = nr11_contents & 0x3F;
            }
            // Reset sweep values
            self.channel_1_sweep_pace_current = (nr10_contents >> 4) & 0b111;
        }

        // channel 2 trigger
        const NR21_ADDRESS: Address = 0xFF16;
        let nr21_contents = map.read::<Byte>(NR21_ADDRESS);
        const NR22_ADDR: Address = 0xFF17;
        let nr22_contents = map.read::<Byte>(NR22_ADDR);
        const NR23_ADDR: Address = 0xFF18;
        let nr23_contents = map.read::<Byte>(NR23_ADDR);
        const NR24_ADDR: Address = 0xFF19;
        let nr24_contents = map.read::<Byte>(NR24_ADDR);
        
        let ch2_triggered = (nr24_contents & BIT_7_MASK) > 0;
        // clear nr24 trigger bit after read
        map.write::<Byte>(nr24_contents & !(BIT_7_MASK), NR24_ADDR);
        // refresh internal values with triggered values
        if ch2_triggered {
            // Reset the volume
            let ch2_init_volume = nr22_contents >> 4;
            self.channel_2_volume_current = ch2_init_volume;
            // Reload the period/frequency value
            
            let period = (nr23_contents as Word) | ((nr24_contents as Word & 0b111) << 8);
            self.channel_2_period_current = period as u32;
            // Reset the length counter
            if self.channel_2_length_timer_current >= LENGTH_TIMER_EXPIRY {
                self.channel_2_length_timer_current = nr21_contents & 0x3F;
            }
        }

        // do timed events
        for _ in 0..dots_elapsed {
            self.current_timing_dot = (self.current_timing_dot + 1) % DOTS_MODULO;

            // Adjust length timers
            if (self.current_timing_dot % DOTS_PER_LENGTH_TICK) == 0 {
                // adjust length
                let channel_1_length_timer_enabled = (nr14_contents & (1 << 6)) > 0;
                if channel_1_length_timer_enabled && self.channel_1_length_timer_current < LENGTH_TIMER_EXPIRY {
                    self.channel_1_length_timer_current += 1;
                }
                let channel_2_length_timer_enabled = (nr24_contents & (1 << 6)) > 0;
                if channel_2_length_timer_enabled && self.channel_2_length_timer_current < LENGTH_TIMER_EXPIRY {
                    self.channel_2_length_timer_current += 1;
                }
            }

            // Channel 1 period sweep
            let channel_1_sweep_pace = self.channel_1_sweep_pace_current;
            let channel_1_sweep_increasing = (nr10_contents & 0b1000) > 0;
            let channel_1_sweep_step = nr10_contents & 0b111;
            if (channel_1_sweep_pace > 0) && ((self.current_timing_dot % (DOTS_PER_SWEEP_TICK * channel_1_sweep_pace as u32)) == 0) {
                let dbg_pre = self.channel_1_period_current;
                if channel_1_sweep_increasing && ((self.channel_1_period_current >> 8) < 0x7FF) {
                    self.channel_1_period_current -= self.channel_1_period_current / 2_u32.pow(channel_1_sweep_step as u32);
                } else if !channel_1_sweep_increasing && (self.channel_1_period_current > 0) {
                    self.channel_1_period_current += self.channel_1_period_current / 2_u32.pow(channel_1_sweep_step as u32);
                }
                // Write back modified period value
                let new_nr13 = (self.channel_1_period_current & 0xFF) as u8;
                let new_nr14 = (nr14_contents & 0xC0) | ((self.channel_1_period_current >> 8) as u8 & 0b111);
                map.write(new_nr13, NR13_ADDR);
                map.write(new_nr14, NR14_ADDR);
                println!("DOT: {}, PACE: {}, STEP: {}, PRE: {}, POST: {}", self.current_timing_dot, channel_1_sweep_pace, channel_1_sweep_step, dbg_pre, self.channel_1_period_current);
            }

            // Volume sweeps
            // CH1
            let channel_1_volume_sweep_pace = nr12_contents & 0b111;
            let channel_1_volume_sweep_increasing = (nr12_contents & 0b1000) > 0;
            if (channel_1_volume_sweep_pace > 0) && (self.current_timing_dot % (channel_1_volume_sweep_pace as u32 * DOTS_PER_VOLUME_ENVELOPE_TICK)) == 0 {
                if channel_1_volume_sweep_increasing && (self.channel_1_volume_current < 0b1111) {
                    self.channel_1_volume_current += 1;
                } else if !channel_1_volume_sweep_increasing && (self.channel_1_volume_current > 0) {
                    self.channel_1_volume_current -= 1;
                }
            }
            // CH2
            let channel_2_volume_sweep_pace = nr22_contents & 0b111;
            let channel_2_volume_sweep_increasing = (nr22_contents & 0b1000) > 0;
            if (channel_2_volume_sweep_pace > 0) && (self.current_timing_dot % (channel_2_volume_sweep_pace as u32 * DOTS_PER_VOLUME_ENVELOPE_TICK)) == 0 {
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
        let volume: f32 = VOLUME_CAP * if self.channel_1_length_timer_current == 64 {
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
        let volume: f32 = VOLUME_CAP * if self.channel_2_length_timer_current == 64 {
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

    pub fn update_waves(&mut self, dots_elapsed: u16) -> (SquareWave, SquareWave) {
        self.catchup_registers(dots_elapsed);
        return (
            self.parse_channel_1(),
            self.parse_channel_2()
        )
    }
}