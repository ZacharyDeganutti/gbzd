use std::{cell::RefCell, rc::Rc};
use crate::memory_gb::{Address, Byte, MemoryMap, MemoryRegion, Word};

use super::audio::{DutyCycle, SquareWave};

// Dots per second = 2^22
const DOTS_PER_LENGTH_TICK: u32 = 2_u32.pow(14); // 256 hz tick
const DOTS_PER_SWEEP_TICK: u32 = 2_u32.pow(15); // 128 hz tick
const DOTS_PER_VOLUME_ENVELOPE_TICK: u32 = 2_u32.pow(16); // 64 hz tick
const DOTS_MODULO: u32 = DOTS_PER_VOLUME_ENVELOPE_TICK;
const DOT_DURATION: f32 = 1.0 / 2_u32.pow(22) as f32;

pub struct Apu<'a> {
    memory: Rc<RefCell<MemoryMap<'a>>>,
    current_timing_dot: u32,
    channel_1_volume_current: u8,
    channel_1_length_timer_current: u8,
    channel_1_envelope_timer_current: u8,
    channel_1_frequency_current: f32,
    channel_1_phase_current: f32,
}

impl<'a> Apu<'a> {
    pub fn new(memory_map: Rc<RefCell<MemoryMap<'a>>>) -> Apu<'a> {
        Apu {
            memory: memory_map,
            current_timing_dot: 0,
            channel_1_volume_current: 0,
            channel_1_length_timer_current: 0,
            channel_1_envelope_timer_current: 0,
            channel_1_frequency_current: 0.0,
            channel_1_phase_current: 0.0
        }
    }

    fn catchup_registers(&mut self, dots_elapsed: u16) {
        const LENGTH_TIMER_EXPIRY: u8 = 64;
        let mut map = self.memory.borrow_mut();

        const LENGTH_AND_DUTY_CYCLE_ADDRESS: Address = 0xFF11;

        // handle trigger events
        const BIT_7_MASK: u8 = (1 << 7);

        // channel 1 trigger
        const NR14_ADDR: Address = 0xFF14;
        let nr14_contents = map.read::<Byte>(NR14_ADDR);
        let ch1_triggered = (nr14_contents & BIT_7_MASK) > 0;
        // clear nr14 trigger bit after read
        map.write::<Byte>(nr14_contents & !(BIT_7_MASK), NR14_ADDR);
        // refresh internal values with triggered values
        if ch1_triggered {
            const NR12_ADDR: Address = 0xFF12;
            let nr12_contents = map.read::<Byte>(NR12_ADDR);
            // Reset the volume
            let ch1_init_volume = nr12_contents >> 4;
            self.channel_1_volume_current = ch1_init_volume;
            // Reload the period/frequency value
            const NR13_ADDR: Address = 0xFF13;
            let nr13_contents = map.read::<Byte>(NR13_ADDR);
            let period = (nr13_contents as Word) | ((nr14_contents as Word & 0b111) << 8);
            self.channel_1_frequency_current = 131072.0 / (2048.0 - period as f32);
            // Reset the length counter
            let length_and_duty_cycle = map.read::<Byte>(LENGTH_AND_DUTY_CYCLE_ADDRESS);
            if self.channel_1_length_timer_current >= LENGTH_TIMER_EXPIRY {
                self.channel_1_length_timer_current = length_and_duty_cycle & 0x3F;
            }
        }
        // Calculate change in phase
        let elapsed_time = DOT_DURATION * dots_elapsed as f32;
        let phase_shift = elapsed_time * self.channel_1_frequency_current;
        self.channel_1_phase_current = (phase_shift + self.channel_1_frequency_current) % 1.0;

        // do timed events
        for _ in 0..dots_elapsed {
            self.current_timing_dot = (self.current_timing_dot + 1) % DOTS_MODULO;

            if (self.current_timing_dot % DOTS_PER_LENGTH_TICK) == 0 {
                // adjust length
                let length_timer_enabled = (nr14_contents & (1 << 6)) > 0;
                if length_timer_enabled && self.channel_1_length_timer_current < LENGTH_TIMER_EXPIRY {
                    self.channel_1_length_timer_current += 1;
                }
            }

            if (self.current_timing_dot % DOTS_PER_SWEEP_TICK) == 0 {
                // do sweep
            }

            if (self.current_timing_dot % DOTS_PER_VOLUME_ENVELOPE_TICK) == 0 {
                // do volume envelope
            }
        }
    }

    fn parse_channel_1(&self) -> SquareWave {
        const LENGTH_AND_DUTY_CYCLE_ADDRESS: Address = 0xFF11;
        const VOLUME_AND_ENVELOPE_ADDRESS: Address = 0xFF12;

        let mut mem = self.memory.borrow_mut();

        let length_and_duty_cycle = mem.read::<Byte>(LENGTH_AND_DUTY_CYCLE_ADDRESS);
        let volume_and_envelope = mem.read::<Byte>(VOLUME_AND_ENVELOPE_ADDRESS);

        let duty_cycle = match length_and_duty_cycle >> 6 {
            0b00 => DutyCycle::Eighth,
            0b01 => DutyCycle::Quarter,
            0b10 => DutyCycle::Half,
            _    => DutyCycle::ThreeQuarter
        };

        // TODO: Have length set volume to 0 if it's at 64

        let frequency = self.channel_1_frequency_current;

        let phase = self.channel_1_phase_current;

        const VOLUME_CAP: f32 = 0.05;
        // let volume: f32 = VOLUME_CAP * if (length_and_duty_cycle & 0x3F) == 0x3F {
        let volume: f32 = VOLUME_CAP * if self.channel_1_length_timer_current == 64 {
            0.0
        } 
        else {
            let vol = (volume_and_envelope >> 4) as f32 / 15.0;
            vol
        };

        SquareWave {
            duty_cycle,
            volume,
            phase,
            frequency,
            sample_rate: 44100.0
        }
    }

    pub fn update_waves(&mut self, dots_elapsed: u16) -> SquareWave {
        self.catchup_registers(dots_elapsed);
        self.parse_channel_1()
    }
}