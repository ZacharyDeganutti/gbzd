use std::cell::RefMut;
use std::mem;
use std::ops::Add;
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
// A Tile is 8x8 pixels
struct Tile {
    // Each pixel is 2 bits, 8 pixels per PixelLine
    lines: [Word; 8]
}

impl Tile {
    pub fn from_address(memory: &mut RefMut<MemoryMap>, address: Address) -> Tile {
        println!("Tile address {:x}", address);
        let lines: [Word; 8] = core::array::from_fn(|i| memory.read(address + (mem::size_of::<Word>() * i) as Address));
        Tile {
            lines
        }
    }

    pub fn color(&self, idx_x: u8, idx_y: u8) -> Option<Color> {
        if idx_x > 7 || idx_y > 7 {
            None
        }
        else {
            let data_word: Word = self.lines[idx_y as usize];
            let low_byte: Byte = (0xFF & data_word) as Byte; 
            let high_byte: Byte = (0xFF & (data_word >> 8)) as Byte;
            // Good chance this is all flipped around
            let mask: u8 = 1 << idx_x;
            Some(Color::from_bits((high_byte & mask) > 0, (low_byte & mask) > 0))
        }
    }
}

#[derive(Clone, Copy)]
enum Color {
    Blank,
    A,
    B,
    C
}

impl Color {
    pub fn from_bits(high: bool, low: bool) -> Color {
        match (high, low) {
            (false, false) => Color::Blank,
            (false, true) => Color::A,
            (true, false) => Color::B,
            (true, true) => Color::C
        }
    }

    pub fn to_value(&self) -> u8 {
        match self {
            Color::Blank => 0,
            Color::A => 1,
            Color::B => 2,
            Color::C => 3
        }
    }
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

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const DOTS_PER_LINE: u32 = 456;
// Denotes the start of VBlank
const HBLANK_END_DOTS: u32 = DOTS_PER_LINE * (SCREEN_HEIGHT as u32);
// Number of dots at which VBlank resets
const DOT_MAX: u32 = HBLANK_END_DOTS + 10;

const TILEMAP_WH: u16 = 256;

const IF_REG_ADDR: Address = 0xFF0F;
const LCDC_ADDRESS: Address = 0xFF40;
const STAT_ADDRESS: Address = 0xFF41;
const SCY_ADDRESS: Address = 0xFF42;
const SCX_ADDRESS: Address = 0xFF43;
const LY_ADDRESS: Address = 0xFF44;
const LYC_ADDRESS: Address = 0xFF45;
const WY_ADDRESS: Address = 0xFF4A;
const WX_ADDRESS: Address = 0xFF4B;

pub struct Ppu<'a> {
    current_mode: RenderMode,
    current_dot: u32,
    screen_buffer: [Color; SCREEN_WIDTH * SCREEN_HEIGHT],
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
            screen_buffer: [Color::Blank; SCREEN_WIDTH * SCREEN_HEIGHT],
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
                let line_number = self.current_dot / 456;
                println!("{}", line_number);
                if line_number < SCREEN_HEIGHT as u32 {
                    self.draw_line(line_number);
                }
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
                self.output_screen();
                const VBLANK_TIME: u32 = DOTS_PER_LINE;
                self.current_dot += VBLANK_TIME;
                VBLANK_TIME as i16
            }
        };
        dots_spent
    }

    fn output_screen(&self) {
        self.debug_print()
    }

    fn debug_print(&self) {
        // Convert the color buffer to a sufficiently pretty string of unicode block values
        let stringed_screen = self.screen_buffer.iter()
        .map(|color| {
            match color {
                Color::Blank => 0x2588,   // solid shade
                Color::A => 0x2593,     // dark shade
                Color::B => 0x2592,     // medium shade
                Color::C => 0x2591,     // light shade
            }
        })
        .fold(String::with_capacity(SCREEN_HEIGHT*SCREEN_WIDTH), |mut screen_string, next_codepoint| {
            screen_string.push(std::char::from_u32(next_codepoint).unwrap());
            screen_string
        });
        
        for i in 0..SCREEN_HEIGHT {
            let printstr: String = stringed_screen.chars().skip(i*SCREEN_WIDTH).take(SCREEN_WIDTH).collect();
            println!("{}", printstr);
        }
        
        //println!("{}", stringed_screen);
        println!("----------------");
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
                // The exception is at the end of the 144th line where it goes to VBlank
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
                    self.current_dot = 0;
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
        // println!("ppu if A: {}", interrupt_flag);
        interrupt_flag |= 0x2;
        if start_vblank {
            interrupt_flag |= 0x1;
        }
        // println!("ppu if B: {}", interrupt_flag);
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

        let lcdc: Byte = mem.read(LCDC_ADDRESS);
        let ly: Byte = mem.read(LY_ADDRESS);

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

    // ((top left), (bottom right)) xy coordinate pairs
    fn viewport_of(scx: Byte, scy: Byte) -> ((u16, u16), (u16, u16)) {
        ((
            scx as u16,
            scy as u16
        ),
        (
            ((scx.wrapping_add(159)) as u16 % TILEMAP_WH),
            ((scy.wrapping_add(143)) as u16 % TILEMAP_WH) 
        ))
    }

    fn draw_line(&mut self, line_number: u32) {
        let mut mem = self.system_memory.borrow_mut();
        let lcdc: Byte = mem.read(LCDC_ADDRESS);
        println!("Draw Line Start");
        println!("lcdc: {:b}", lcdc);
        let viewport = Self::viewport_of(mem.read(SCX_ADDRESS), mem.read(SCY_ADDRESS));
        // Background/Window enabled
        if (lcdc & 0x1) > 0 {
            let tile_data_base_address: Address = if (lcdc & (1 << 4)) > 0 {
                0x8000
            }
            else {
                0x9000
            };
            let background_map_base_address: Address = if (lcdc & 0x8) > 0 { 0x9C00 } else { 0x9800 };

            for pixel in 0..(SCREEN_WIDTH as u16) {
                let screen_pos_x = (pixel + viewport.0.0) % TILEMAP_WH;
                let screen_pos_y = (line_number as u16 + viewport.0.1) % TILEMAP_WH;
                // 32x32 tiles in map, quantize screen position to an 8x8 tile, flatten to linear buffer layout
                let tile_index = (screen_pos_y/8)*32 + (screen_pos_x/8);
                print!("{} ", tile_index);
                let tile_map_address: Address = background_map_base_address + tile_index;
                // Get address of actual data
                let tile_data_offset = mem.read::<Byte>(tile_map_address) as Address;
                // Indexing is wrong, probably mixing up tile map and tile data mentally
                let tile_data_address = if tile_data_base_address == 0x8000 {
                    tile_data_base_address + (tile_data_offset * mem::size_of::<Tile>() as Address)
                }
                else {
                    // index into the higher region in the 0-127 range and lower region in the 128-255 range :(
                    (tile_data_base_address - (0x800 * (tile_data_offset >> 7)))  + ((tile_data_offset & 0x7F) * mem::size_of::<Tile>() as Address)
                };
                let tile = Tile::from_address(&mut mem, tile_data_address);
                let tile_pos_x = (screen_pos_x % 8) as u8;
                let tile_pos_y = (screen_pos_y % 8) as u8;
                let color = tile.color(tile_pos_x, tile_pos_y);
                self.screen_buffer[SCREEN_WIDTH*(line_number as usize) + (pixel as usize)] = color.unwrap_or(Color::Blank);
            }

            // Window enable, TODO probably need to move this up somewhere and integrate it with background tiles
            if (lcdc & (1 << 5)) > 0 {
                let window_map_address: Address = if (lcdc & (1)) > 0 { 0x9C00 } else { 0x9800 };

            }
        }
        // Objects enabled
        if (lcdc & 0x2) > 0 {
            
        }
    }
}
