mod processor {
    pub mod cpu;
    pub mod ops;
    pub mod execute;
}
mod audio {
    pub mod audio;
    pub mod apu;
}
mod memory_gb;
mod cart;
mod special_registers;
mod ppu;
mod display;
mod input;

use std::rc::Rc;
use std::cell::RefCell;
use std::thread::sleep;
use std::time::{Duration, Instant};
use audio::audio::{DutyCycle, GbAudioSdl, SquareWave, SampleWave};
use audio::apu::{Apu};
use display::DisplayMiniFB;

use crate::processor::cpu::*;
use crate::ppu::*;
use crate::input::*;

const FRAME_TIME_TOTAL: Duration = Duration::from_micros(16_740);

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        panic!("Incorrect number of arguments supplied. Please provide a path to a ROM file");
    }
    let rom = &args[1];
    let cart = cart::Cart::load_from_file(rom).expect("Problem with ROM file");
    let joypad = input::Joypad::new();
    let mut system_memory_data = memory_gb::MemoryMap::allocate(cart, joypad);
    let system_memory = Rc::new(RefCell::new(memory_gb::MemoryMap::new(&mut system_memory_data)));
    let mut cpu = Cpu::new(system_memory.clone());
    let mut ppu = Ppu::new(system_memory.clone());
    let mut apu = Apu::new(system_memory.clone());
    
    let controllers: Vec<Box<dyn InputDevice>> = {
        let pads = GilControllers::enumerate_gilrs_controllers();
        let mut intermediate = vec![];
        intermediate.push(Box::new(pads) as Box<dyn InputDevice>);
        intermediate
    };
    
    let mut input_handler = InputHandler::new(controllers, system_memory.clone());

    let sdl_context = sdl3::init().unwrap();
    let mut audio_player = GbAudioSdl::new(&sdl_context);

    let default_square_wave = SquareWave {
        duty_cycle: DutyCycle::Half,
        volume: 0.00,
        frequency: 440.0,
    };
    let default_sample_wave = SampleWave {
        volume_samples: [0.0; 32],
        frequency: 440.0,
    };
    audio_player.start_channel_1(default_square_wave);
    audio_player.start_channel_2(default_square_wave);
    audio_player.start_channel_3(default_sample_wave);

    let mut display = DisplayMiniFB::new();

    // Debt represents the timing balance between cpu and ppu.
    // The cpu runs up the debt (positive)
    // The ppu pays down the debt (negative)
    // The ppu only has to do work if its debt is greater than 0
    let mut debt: i16 = 0;
    let mut cpu_locked: bool = false;
    let mut color_buffer = vec![0u32; 160*144];
    let mut frame_time_start = Instant::now();
    let mut frame_time_end = Instant::now();

    loop {
        if !display.is_open() {
            std::process::exit(0);
        }

        let mut dots_elapsed: u16 = 0;
        // Run CPU or PPU
        if debt <= 0 && !cpu_locked {
            dots_elapsed = (cpu.run() * 4) as u16;
            debt += dots_elapsed as i16;
            if dots_elapsed == 0 {
                cpu_locked = true;
            }

            // Update APU after CPU because it operates at a finer dot granularity.
            // CPU/PPU/APU should provide the illusion of operating in parallel
            let (ch_1_wave, ch_2_wave, ch_3_wave) = apu.update_waves(4);
            audio_player.update_channel_1(ch_1_wave);
            audio_player.update_channel_2(ch_2_wave);
            audio_player.update_channel_3(ch_3_wave);
        }
        else {
            if cpu_locked {
                ppu.run();
                cpu_locked = false
            }
            else {
                dots_elapsed = ppu.run() as u16;
                debt -= dots_elapsed as i16;
            }
        }
        
        // Things that happen once per frame go here
        if ppu.frame_is_ready() {
            color_buffer = ppu.display_handle()
                .into_iter()
                .map(|color: Color| {
                    match color {
                        Color::A => 0xe0f8d0u32,
                        Color::B => 0x88c070u32,
                        Color::C => 0x346856u32,
                        Color::D => 0x081820u32
                    }
                })
                .collect::<Vec<u32>>();

            display.update(&color_buffer);
            // Poll input for the next frame (first frame will always have default values, but that's fine)
            input_handler.poll();

            // Clock in the time taken as late as possible for a decent sleep timing
            frame_time_end = Instant::now();
            let frame_time_elapsed = frame_time_end - frame_time_start;
            // println!("frame start {:?}, frame end {:?}, duration {:?}", frame_time_start, frame_time_end, frame_time_elapsed);
            if frame_time_elapsed < FRAME_TIME_TOTAL {
                sleep(FRAME_TIME_TOTAL - frame_time_elapsed);
            }
            frame_time_start = Instant::now();
            // misc debug logging
            // println!("{:?}", ch_1_wave);
            //audio_player.update_channel_1(ch_1_wave);
            //print!("debt: {} \n", debt);
        }
    }
}
