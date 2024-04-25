mod processor {
    pub mod cpu;
    pub mod ops;
    pub mod execute;
}
mod memory_gb;
mod cart;
mod special_registers;
mod ppu;
mod display;

use std::rc::Rc;
use std::cell::RefCell;
use std::thread::sleep;
use std::time::{Duration, Instant};
use display::DisplayMiniFB;

use crate::processor::cpu::*;
use crate::ppu::*;

const FRAME_TIME_TOTAL: Duration = Duration::from_micros(16_740);

fn main() {
    let rom = "roms/cpu_instrs.gb";
    // let rom = "roms/11-op a,(hl).gb";
    let cart = cart::Cart::load_from_file(rom).expect("Problem with ROM file");
    let mut system_memory_data = memory_gb::MemoryMap::allocate(cart);
    let system_memory = Rc::new(RefCell::new(memory_gb::MemoryMap::new(&mut system_memory_data)));
    let mut cpu = Cpu::new(system_memory.clone());
    let mut ppu = Ppu::new(system_memory.clone());
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
        if debt <= 0 && !cpu_locked {
            let payment = (cpu.run() * 4) as i16;
            debt += payment;
            if payment == 0 {
                cpu_locked = true;
            }
        }
        else {
            if cpu_locked {
                ppu.run();
                cpu_locked = false
            }
            else {
                debt -= ppu.run();
            }
        }
        
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
            // println!("{:x?}", color_buffer);
            // Clock in the time taken as late as possible for a decent sleep timing
            frame_time_end = Instant::now();
            let frame_time_elapsed = frame_time_end - frame_time_start;
            // println!("frame start {:?}, frame end {:?}, duration {:?}", frame_time_start, frame_time_end, frame_time_elapsed);
            if frame_time_elapsed < FRAME_TIME_TOTAL {
                sleep(FRAME_TIME_TOTAL - frame_time_elapsed);
            }
            display.update(&color_buffer);
            frame_time_start = Instant::now();
        }
    }
    
    // let mut i = 0;
    // loop {
    //     print!("{}\n", i);
    //     cpu.step();
    //     i += 1;
    // }
}
