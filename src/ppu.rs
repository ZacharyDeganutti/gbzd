use std::rc::Rc;
use std::cell::RefCell;

use crate::memory_gb;
use crate::memory_gb::Address;
use crate::memory_gb::Byte;
use crate::memory_gb::MemoryBank;
use crate::memory_gb::BankType;
use crate::memory_gb::MemoryUnit;
use crate::memory_gb::Word;
use crate::memory_gb::MemoryRegion;
use crate::memory_gb::MemoryMap;

#[derive(Clone, Copy)]
struct OamEntry {
    y_pos: Byte,
    x_pos: Byte,
    tile_index: Byte,
    flags: Byte
}

#[derive(Clone, Copy)]
struct PixelLine {
    // Each pixel is 2 bits, 8 pixels per PixelLine
    pixels: u16
}

#[derive(Clone, Copy)]
// A Tile is 8x8 pixels
struct Tile {
    lines: [PixelLine; 8]
}

#[derive(Clone, Copy, PartialEq)]
enum RenderMode {
    OAMScan,
    PixelDraw,
    HBlank,
    VBlank
}

impl RenderMode {
    pub fn mode_number(&self) -> u8 {
        match self{
            RenderMode::OAMScan => 2,
            RenderMode::PixelDraw => 3,
            RenderMode::HBlank => 0,
            RenderMode::VBlank => 1
        }
    }
}

// TODO: Distant future optimization, representing pixels as u8 prior to handing off to drawing is a huge waste, make a packed format

struct PixelFifo {
    background_fifo: [u8; 16],
    object_fifo: [u8; 16]
}

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const DOTS_PER_LINE: u32 = 456;
// Denotes the start of VBlank
const HBLANK_END_DOTS: u32 = DOTS_PER_LINE * (SCREEN_HEIGHT as u32);
// Number of dots at which VBlank resets
const DOT_MAX: u32 = HBLANK_END_DOTS + 10;

const IF_REG_ADDR: Address = 0xFF0F;
const LCDC_ADDRESS: Address = 0xFF40;
const STAT_ADDRESS: Address = 0xFF41;
const LY_ADDRESS: Address = 0xFF44;
const LYC_ADDRESS: Address = 0xFF45;

pub struct Ppu<'a> {
    current_mode: RenderMode,
    current_dot: u32,
    screen_buffer: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
    oam_scan_results: Vec<OamEntry>,
    system_memory: Rc<RefCell<MemoryMap<'a>>>
}

impl<'a> Ppu<'a> {
    // Creates a PPU initialized to the tail end of VBLANK
    pub fn new(system_memory: Rc<RefCell<MemoryMap>>) -> Ppu {
        let cycles_per_second = 104826;
        let mut new_ppu = Ppu { 
            current_mode: RenderMode::VBlank,
            current_dot: DOT_MAX,
            screen_buffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            oam_scan_results: Vec::with_capacity(0),
            system_memory
        };
        new_ppu
    }

    pub fn run(&mut self) -> i16 {
        // Do some state transitions top level here so it happens after the cpu catches up
        self.update_render_state();

        let dots_spent = match self.current_mode {
            RenderMode::OAMScan => {
                // Scan the whole OAM in one shot since coroutines aren't 'real' yet
                // and I really don't want to implement that without those unless I really have to
                self.oam_scan_results = self.scan_oam();
                const OAM_SCAN_TIME: u32 = 80;
                self.current_dot += OAM_SCAN_TIME;
                OAM_SCAN_TIME as i16
            }
            RenderMode::PixelDraw => {
                // Actually granular timing is for nerds, let's just rip out whole modes at once
                // This could certainly make things funky within any line,
                // but SURELY this should be good enough and things will probably mostly shake out
                const PIXEL_DRAW_TIME: u32 = 172;
                self.current_dot += PIXEL_DRAW_TIME;
                PIXEL_DRAW_TIME as i16
            }
            RenderMode::HBlank => {
                const HBLANK_TIME: u32 = 204;
                self.current_dot += HBLANK_TIME;
                HBLANK_TIME as i16
            }
            RenderMode::VBlank => {
                const VBLANK_TIME: u32 = DOTS_PER_LINE;
                self.current_dot += VBLANK_TIME;
                VBLANK_TIME as i16
            }
        };
        dots_spent
    }

    // Handles mode changes and updates the render buffer with pixel data at the tail of VBlank
    fn update_render_state(&mut self) {
        let mut memory = self.system_memory.borrow_mut();
        let mut start_vblank = false;
        let mut frame_done = false;
        self.current_mode = match self.current_mode {
            RenderMode::OAMScan => RenderMode::PixelDraw,
            RenderMode::PixelDraw => RenderMode::HBlank,
            RenderMode::HBlank => {
                // HBlank typically goes back to the next line's OAM scan
                // The exception is at the end of the 144th line where it goes to
                if self.current_dot >= HBLANK_END_DOTS {
                    start_vblank = true;
                    RenderMode::VBlank
                }
                else {
                    RenderMode::OAMScan
                }
            }
            RenderMode::VBlank => {
                // VBlank happens for 10 lines, until it hits the reset point
                if self.current_dot >= DOT_MAX {
                    frame_done = true;
                    RenderMode::OAMScan
                }
                else {
                    RenderMode::VBlank
                }
            }
        };

        // Increment LY whenever a new OAMScan is entered. Set 0 if a frame was just wrapped up
        let old_ly: Byte = memory.read(LY_ADDRESS);
        let ly = if self.current_mode == RenderMode::OAMScan {
            if frame_done {
                0
            }
            else {
                old_ly + 1
            }
        }
        else {
            old_ly
        };
        memory.write(ly, LY_ADDRESS);

        // Update the LY=LYC check and mode in the STAT register. 
        // Probably not enough to be accurate for CPU changes to LYC
        // Might be worth trapping LYC on the CPU to cover both ends
        let lyc: Byte = memory.read(LYC_ADDRESS);
        let ly_eq_lc = (if lyc == ly { 1 } else { 0 }) << 2;
        let mode_number = self.current_mode.mode_number();
        let old_stat: Byte = memory.read(STAT_ADDRESS);
        let stat = (old_stat & !(0x7)) | (ly_eq_lc | mode_number);
        memory.write(stat, STAT_ADDRESS);
        
        // Handle possible interrupts arising from VBlank or STAT
        let mut interrupt_flag: Byte = memory.read(IF_REG_ADDR);
        // TODO: Add all the other stat interrupt sources (mode based ones)
        interrupt_flag |= 0x2;
        if start_vblank {
            interrupt_flag |= 0x1;
        }
        memory.write(interrupt_flag, IF_REG_ADDR);

    }

    // Returns a vector of OAM entries sorted by reverse priority
    fn scan_oam(&mut self) -> Vec<OamEntry> {
        const MAX_OBJECTS_PER_LINE: usize = 10;
        const OAM_START: Address = 0xFE00;
        const OAM_END: Address = 0xFE9F + 1;
        const _TOTAL_OAM_SLOTS: u8 = 40;

        let mut line_objects_buffer: Vec<OamEntry> = Vec::with_capacity(MAX_OBJECTS_PER_LINE);

        let mut mem = self.system_memory.borrow_mut();

        let lcdc: Byte = mem.read(0xFF40);
        let ly: Byte = mem.read(0xFF44);

        let objects_are_tall = (lcdc & 0x4) > 0; 
        let (ly_padded, object_size) = if objects_are_tall { (ly, 16) } else { (ly + 8, 8) };

        for entry_address in (OAM_START..OAM_END).step_by(4) {
            let current_object = OamEntry {
                y_pos:      mem.read(entry_address),
                x_pos:      mem.read(entry_address + 1),
                tile_index: mem.read(entry_address + 2),
                flags:      mem.read(entry_address + 3)
            };

            // Check each object (up to max allowable) to see if they exist on this line
            // TODO: verify accuracy
            if (current_object.y_pos >= ly_padded) && (ly_padded < (current_object.y_pos + object_size)) {
                line_objects_buffer.push(current_object);
                if line_objects_buffer.len() >= MAX_OBJECTS_PER_LINE {
                    break;
                }
            }
        }
        // Object priority in the original gameboy requires a stable sorting of objects by x position
        line_objects_buffer.sort_by(|a, b| a.x_pos.partial_cmp(&b.x_pos).unwrap());
        // Order is reversed because we want to draw lower priority pixels first and potentially overwrite them with higher priority ones
        line_objects_buffer.reverse();
        line_objects_buffer
    }
}
