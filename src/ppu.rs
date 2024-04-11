use std::cell::RefMut;
use std::mem;
use std::ops::Add;
use std::rc::Rc;
use std::cell::RefCell;

use crate::memory_gb;
use crate::memory_gb::Address;
use crate::memory_gb::Byte;
use crate::memory_gb::ByteExt;
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
enum ObjectIntersection {
    // provides inner x coordinate, inner y coordinate, and object height
    Coordinate(u8, u8, u8),
    None
}

#[derive(Clone, Copy)]
// A Tile is 8x8 pixels
struct Tile {
    // Each pixel is 2 bits, 8 pixels per PixelLine
    lines: [Word; 8]
}

impl Tile {
    pub fn from_address(memory: &mut RefMut<MemoryMap>, address: Address) -> Tile {
        // println!("Tile address {:x}", address);
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
            let mask: u8 = 0x80 >> idx_x;
            Some(Color::from_bits((high_byte & mask) > 0, (low_byte & mask) > 0))
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
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
// Number of dots taken in an OAM Scan
const OAM_SCAN_TIME: u32 = 80;
// Number of dots taken in HBLANK
const HBLANK_TIME: u32 = 204;

const TILE_WIDTH: u8 = 8;
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
    internal_window_line_counter: u16,
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
            internal_window_line_counter: 0,
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

                self.oam_scan_results.clear();
                self.oam_scan_results = self.scan_oam();
                println!("oam_scan_results length {}", self.oam_scan_results.len());
                
                self.current_dot += OAM_SCAN_TIME;
                (OAM_SCAN_TIME) as i16
            }
            RenderMode::PixelDraw => {
                // Actually granular timing is for nerds, let's just rip out whole modes at once
                // This could certainly make things funky within any line,
                // but SURELY this should be good enough and things will probably mostly shake out
                let line_number = self.current_dot / 456;
                // println!("Line number: {}", line_number);
                if line_number < SCREEN_HEIGHT as u32 {
                    self.draw_line(line_number);
                }
                const PIXEL_DRAW_TIME: u32 = 172;
                self.current_dot += PIXEL_DRAW_TIME;
                PIXEL_DRAW_TIME as i16
            }
            RenderMode::HBlank => {
                const HBLANK_GRANULARITY: u32 = HBLANK_TIME;
                self.current_dot += HBLANK_GRANULARITY;
                HBLANK_GRANULARITY as i16
            }
            RenderMode::VBlank => {
                self.output_screen();
                self.internal_window_line_counter = 0;
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
        println!("----------------");
    } 

    // Handles mode changes and updates the render buffer with pixel data at the tail of VBlank
    fn update_render_state(&mut self) {
        let mut memory = self.system_memory.borrow_mut();
        let mut start_oam_scan = false;
        let mut start_hblank = false;
        let mut start_vblank = false;
        let mut frame_done = false;
        let last_mode = self.current_mode;
        self.current_mode = match self.current_mode {
            RenderMode::OAMScan => {
                if (self.current_dot % DOTS_PER_LINE) >= 80 { RenderMode::PixelDraw } else { RenderMode::OAMScan }
            }
            RenderMode::PixelDraw => {
                start_hblank = true;
                RenderMode::HBlank
            } 
            RenderMode::HBlank => {
                // HBlank typically goes back to the next line's OAM scan
                // The exception is at the end of the 144th line where it goes to VBlank
                // HBlank is over when a new line is reached
                if (self.current_dot % DOTS_PER_LINE) == 0 {
                    if self.current_dot >= HBLANK_END_DOTS {
                        start_vblank = true;
                        RenderMode::VBlank
                    }
                    else {
                        start_oam_scan = true;
                        RenderMode::OAMScan
                    }
                }
                else {
                    RenderMode::HBlank
                }
            }
            RenderMode::VBlank => {
                // VBlank happens for 10 lines, until it hits the reset point
                if self.current_dot >= DOT_MAX {
                    frame_done = true;
                    self.current_dot = 0;
                    start_oam_scan = true;
                    RenderMode::OAMScan
                }
                else {
                    RenderMode::VBlank
                }
            }
        };

        // Increment LY whenever a new OAMScan is entered. Set 0 if a frame was just wrapped up
        let old_ly: Byte = memory.read(LY_ADDRESS);
        let ly = if (self.current_mode == RenderMode::OAMScan) && (last_mode != RenderMode::OAMScan) {
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
        //let ly_eq_lc = (if (lyc == ly) && (old_ly != ly) { 1 } else { 0 }) << 2;
        let ly_eq_lc = (if lyc == ly { 1 } else { 0 }) << 2;
        let mode_number = self.current_mode.mode_number();
        let old_stat: Byte = memory.read(STAT_ADDRESS);
        let stat = (old_stat & !(0x7)) | (ly_eq_lc | mode_number);
        memory.write(stat, STAT_ADDRESS);
        
        // Handle possible interrupts arising from VBlank or STAT
        let mut interrupt_flag: Byte = memory.read(IF_REG_ADDR);
        // Check stat interrupt enables and set the stat interrupt flag if enabled mode changes occur
        if (start_oam_scan && (stat & (1 << 5)) > 0) 
            || ((ly_eq_lc > 0) && (stat & (1 << 6)) > 0)
            || (start_vblank && (stat & (1 << 4)) > 0) 
            || (start_hblank && (stat & (1 << 3)) > 0) 
        {
            interrupt_flag |= 0x2;
        }
        
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
        // Pad LY because objects exist in a space beginning 16 lines before the screen. Convert LY to that space for easy comparisons
        let ly_padded = ly + 16;
        let object_size = if objects_are_tall { 2 * TILE_WIDTH } else { TILE_WIDTH };

        for entry_address in (OAM_START..OAM_END).step_by(4) {
            let current_object = OamEntry {
                y_pos:      mem.read(entry_address),
                x_pos:      mem.read(entry_address + 1),
                tile_index: mem.read(entry_address + 2),
                flags:      mem.read(entry_address + 3)
            };
            // Check each object (up to max allowable) to see if they exist on this line
            if (ly_padded >= current_object.y_pos ) && (ly_padded < (current_object.y_pos + (object_size))) {
                line_objects_buffer.push(current_object);
                if line_objects_buffer.len() >= MAX_OBJECTS_PER_LINE {
                    break;
                }
            }
        }
        // Object priority in the original gameboy requires a stable sorting of objects by x position
        line_objects_buffer.sort_by(|a, b| b.x_pos.partial_cmp(&a.x_pos).unwrap());
        // Order is reversed because we want to draw lower priority pixels first and potentially overwrite them with higher priority ones
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
        // let lcdc: Byte = 0b1010001;
        // println!("Draw Line Start");
        println!("lcdc: {:b}", lcdc);
        let viewport = Self::viewport_of(mem.read(SCX_ADDRESS), mem.read(SCY_ADDRESS));
        // Background/Window enabled, so draw them
        if (lcdc & (1 << 0)) > 0 {
            let tile_data_base_address: Address = if (lcdc & (1 << 4)) > 0 {
                0x8000
            }
            else {
                0x9000
            };
            let window_map_base_address: Address = if (lcdc & (1 << 6)) > 0 { 0x9C00 } else { 0x9800 };
            let background_map_base_address: Address = if (lcdc & (1 << 3)) > 0 { 0x9C00 } else { 0x9800 };

            // Grab window coordinates for this line
            let wy: Byte = mem.read(WY_ADDRESS);
            let wx: Byte = mem.read::<Byte>(WX_ADDRESS).wrapping_sub(7);

            for pixel in 0..(SCREEN_WIDTH as u16) {

                // Grab the background tile map address by default, otherwise grab the window tile map when
                // The window is enabled, AND
                // We're inside the window coordinates
                
                let (in_window, map_base_address) = if ((lcdc & (1 << 5)) > 0) && (line_number >= wy as u32) && (pixel >= (wx as u16)) {
                    
                    (true, window_map_base_address)
                }
                else {
                    (false, background_map_base_address)
                };

                let (tile_index, tile_pos_x, tile_pos_y) = if in_window {
                    let screen_pos_x = (pixel.wrapping_add(viewport.0.0.wrapping_sub(wx as u16))) % TILEMAP_WH;
                    let screen_pos_y = ((line_number as u16).wrapping_sub(wy as u16)) % TILEMAP_WH;
                    ((screen_pos_y/8)*32 + (screen_pos_x/8), (screen_pos_x % 8) as u8, (screen_pos_y % 8) as u8)
                }
                else {
                    // 32x32 tiles in map, quantize screen position to an 8x8 tile, flatten to linear buffer layout
                    let screen_pos_x = (pixel + viewport.0.0) % TILEMAP_WH;
                    let screen_pos_y = (line_number as u16 + viewport.0.1) % TILEMAP_WH;
                    ((screen_pos_y/8)*32 + (screen_pos_x/8), (screen_pos_x % 8) as u8, (screen_pos_y % 8) as u8)
                };

                let tile_map_address: Address = map_base_address + tile_index;

                let tile_data_address = if tile_data_base_address == 0x8000 {
                    // Get address of actual data
                    let tile_data_offset = mem.read::<Byte>(tile_map_address) as Address;
                    tile_data_base_address + (tile_data_offset * mem::size_of::<Tile>() as Address)
                }
                else {
                    let tile_data_offset = mem.read::<Byte>(tile_map_address).interpret_as_signed() as i32;
                    // print!("tdo: {}, ", (tile_data_offset));
                    // Impossible to overflow/underflow Address with the TDO value range, so we can just unwrap here
                    ((tile_data_base_address as i32) + (tile_data_offset * mem::size_of::<Tile>() as i32)).try_into().unwrap()
                };
                let tile = Tile::from_address(&mut mem, tile_data_address);
                let color = tile.color(tile_pos_x, tile_pos_y);
                // print!("color: ({}), addr: {:x} / ", color.unwrap() as u8, tile_data_address);
                self.screen_buffer[SCREEN_WIDTH*(line_number as usize) + (pixel as usize)] = color.unwrap();
            }
        }

        // Objects enabled, so draw them 
        if (lcdc & (1 << 1)) > 0 {
            // object-pixel intersection test
            let obj_intersect = |pix_obj_x: u8, pix_obj_line: u8, obj: &OamEntry| -> ObjectIntersection {
                // First check LCDC to see how tall objects are configured to be
                let obj_height = if (lcdc & (1 << 2)) > 0 { 2 * TILE_WIDTH } else { TILE_WIDTH };
                // Then look if the current object space pixel coordinate is inside the given object
                if (pix_obj_x >= obj.x_pos) && (pix_obj_x < (obj.x_pos + TILE_WIDTH)) && (pix_obj_line >= obj.y_pos) && (pix_obj_line < (obj.y_pos + obj_height)) {
                    // println!("HIT: x: {}, y: {}", pix_obj_x, pix_obj_line);
                    ObjectIntersection::Coordinate(pix_obj_x - obj.x_pos, pix_obj_line - obj.y_pos, obj_height)
                }
                else {
                    ObjectIntersection::None
                }
            };
            for object in &self.oam_scan_results {
                // println!("x: {}, y: {}", object.x_pos, object.y_pos);
            }
            for pixel in 0..(SCREEN_WIDTH as u16) {
                // if line_number == 0 { println!("X pixel: {}", pixel) } ;
                for object in &self.oam_scan_results {
                    // do a 'fake' shift of the pixel coordinates to account for the leftward 8 pixels of offscreen padding
                    let pixel_shifted = pixel as u8 + TILE_WIDTH;
                    // likewise for the line number and upper offscreen padding
                    let line_number_shifted = line_number as u8 + (2 * TILE_WIDTH);

                    // Overwrite the pixel based on the object's flags if we're inside the object 
                    if let ObjectIntersection::Coordinate(interior_x, interior_y, obj_height) = obj_intersect(pixel_shifted, line_number_shifted, object) {
                        let obj_data_base_address: Address = 0x8000;
                        let flip_adjusted_x = if (object.flags & (1 << 5)) > 0 { (TILE_WIDTH - 1) - interior_x } else { interior_x };
                        let flip_adjusted_y = if (object.flags & (1 << 6)) > 0 { (obj_height - 1) - interior_y } else { interior_y };
                        // See OAM byte 2 tile index documentation for details. Behavior is funky for 8x16 objects
                        let tile_index = if (lcdc & (1 << 2)) > 0 {
                            if ( flip_adjusted_y / TILE_WIDTH ) == 0 { object.tile_index & 0xFE } else { object.tile_index | 0x1 }
                        }
                        else { 
                            object.tile_index 
                        };
                        // Look where object data is stored. Add the tile index for this object. If we are in the lower part of the object, look at the next tile instead
                        let tile_data_address: Address = obj_data_base_address + (tile_index as Address * mem::size_of::<Tile>() as Address);
                        let tile = Tile::from_address(&mut mem, tile_data_address);
                        let color = tile.color(flip_adjusted_x, flip_adjusted_y % 8).unwrap();
                        // Blank is transparent, and should allow the background or lower priority objects to shine through
                        // No reason to draw blanks
                        if color != Color::Blank {
                            let pixel_index = SCREEN_WIDTH*(line_number as usize) + (pixel as usize);
                            // But otherwise, we draw it if objects have priority or the background is just a blank pixel
                            if ((object.flags & (1 << 7)) == 0) || (self.screen_buffer[pixel_index] == Color::Blank) {
                                self.screen_buffer[pixel_index] = color;
                            }
                        }
                    }
                }
            }
        }
    }
}
