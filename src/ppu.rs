use std::cell::RefMut;
use std::mem;
use std::rc::Rc;
use std::cell::RefCell;

use crate::memory_gb::Address;
use crate::memory_gb::Byte;
use crate::memory_gb::ByteExt;
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

    pub fn color_index(&self, idx_x: u8, idx_y: u8) -> Option<ColorIndex> {
        if idx_x > 7 || idx_y > 7 {
            None
        }
        else {
            let data_word: Word = self.lines[idx_y as usize];
            let low_byte: Byte = (0xFF & data_word) as Byte; 
            let high_byte: Byte = (0xFF & (data_word >> 8)) as Byte;
            // Good chance this is all flipped around
            let mask: u8 = 0x80 >> idx_x;
            Some(ColorIndex::from_bits((high_byte & mask) > 0, (low_byte & mask) > 0))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ColorIndex {
    Blank,
    One,
    Two,
    Three
}

impl ColorIndex {
    pub fn from_bits(high: bool, low: bool) -> ColorIndex {
        match (high, low) {
            (false, false) => ColorIndex::Blank,
            (false, true) => ColorIndex::One,
            (true, false) => ColorIndex::Two,
            (true, true) => ColorIndex::Three
        }
    }

    pub fn from_value(value: u8) -> Option<ColorIndex> {
        match value {
            0 => Some(ColorIndex::Blank),
            1 => Some(ColorIndex::One),
            2 => Some(ColorIndex::Two),
            3 => Some(ColorIndex::Three),
            _ => None
        }
    }

    pub fn to_value(&self) -> u8 {
        match self {
            ColorIndex::Blank => 0,
            ColorIndex::One => 1,
            ColorIndex::Two => 2,
            ColorIndex::Three => 3
        }
    }

    pub fn apply_palette(&self, palette: Byte) -> Color {
        let color_number = self.to_value();
        let shade = (palette >> (color_number * 2)) & 0x3;
        Color::from_value(shade).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Color {
    A,
    B,
    C,
    D
}

impl Color {
    pub fn from_bits(high: bool, low: bool) -> Color {
        match (high, low) {
            (false, false) => Color::A,
            (false, true) => Color::B,
            (true, false) => Color::C,
            (true, true) => Color::D
        }
    }

    pub fn from_value(value: u8) -> Option<Color> {
        match value {
            0 => Some(Color::A),
            1 => Some(Color::B),
            2 => Some(Color::C),
            3 => Some(Color::D),
            _ => None
        }
    }

    pub fn to_value(&self) -> u8 {
        match self {
            Color::A => 0,
            Color::B => 1,
            Color::C => 2,
            Color::D => 3
        }
    }

    pub fn is_blank_color(&self, palette: Byte) -> bool {
        let blank_color = palette & 0x3;
        self.to_value() == blank_color
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
const DISPLAY_BUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const DOTS_PER_LINE: u32 = 456;
// Denotes the start of VBlank
const VBLANK_START_DOTS: u32 = DOTS_PER_LINE * (SCREEN_HEIGHT as u32);
// Number of dots at which VBlank resets
const DOT_MAX: u32 = VBLANK_START_DOTS + (10 * DOTS_PER_LINE);
// Number of dots taken in an OAM Scan
const OAM_SCAN_TIME: u32 = 80;
// Number of dots taken in a pixel draw
const PIXEL_DRAW_TIME: u32 = 172;
// Number of dots taken in HBlank
const HBLANK_TIME: u32 = 204;
// Denotes the start of HBlank on any given line
const PIXEL_DRAW_END_DOTS: u32 = OAM_SCAN_TIME + PIXEL_DRAW_TIME;

const TILE_WIDTH: u8 = 8;
const TILEMAP_WH: u16 = 256;

const IF_REG_ADDR: Address = 0xFF0F;
const LCDC_ADDRESS: Address = 0xFF40;
const STAT_ADDRESS: Address = 0xFF41;
const SCY_ADDRESS: Address = 0xFF42;
const SCX_ADDRESS: Address = 0xFF43;
const LY_ADDRESS: Address = 0xFF44;
const LYC_ADDRESS: Address = 0xFF45;
const BGP_ADDRESS: Address = 0xFF47;
const OBP0_ADDRESS: Address = 0xFF48;
const OPB1_ADDRESS: Address = 0xFF49;
const WY_ADDRESS: Address = 0xFF4A;
const WX_ADDRESS: Address = 0xFF4B;


pub struct Ppu<'a> {
    current_mode: RenderMode,
    current_dot: u32,
    // Double buffer with a back and front
    display_buffer: [Color; DISPLAY_BUFFER_SIZE * 2],
    front_buffer_base: usize,
    back_buffer_base: usize,
    oam_scan_results: Vec<OamEntry>,
    internal_window_line_counter: u16,
    frame_ready: bool,
    system_memory: Rc<RefCell<MemoryMap<'a>>>
}

impl<'a> Ppu<'a> {
    // Creates a PPU initialized to the tail end of VBLANK
    pub fn new(system_memory: Rc<RefCell<MemoryMap>>) -> Ppu {
        let new_ppu = Ppu { 
            current_mode: RenderMode::VBlank,
            current_dot: DOT_MAX,
            display_buffer: [Color::A; DISPLAY_BUFFER_SIZE * 2],
            front_buffer_base: 0,
            back_buffer_base: DISPLAY_BUFFER_SIZE,
            oam_scan_results: Vec::with_capacity(0),
            internal_window_line_counter: 0,
            frame_ready: false,
            system_memory
        };
        new_ppu
    }
    
    pub fn frame_is_ready(&mut self) -> bool {
        let ready = self.frame_ready;
        self.frame_ready = false;
        ready
    }

    pub fn display_handle(&self) -> Vec<Color> {
        (&self.display_buffer[self.front_buffer_base .. (DISPLAY_BUFFER_SIZE + self.front_buffer_base)]).to_vec()
    }

    pub fn run(&mut self) -> i16 {
        let running = {
            let mut memory = self.system_memory.borrow_mut();
            let lcdc: Byte = memory.read(LCDC_ADDRESS);
            (lcdc & (1 << 7)) > 0
        };
        // If the LCD is disabled, refresh all the state and boot back control
        /*
        if !running {
            self.current_mode = RenderMode::VBlank;
            self.current_dot = DOT_MAX;
            self.front_buffer_base = 0;
            self.frame_ready = false;
            self.internal_window_line_counter = 0;
            return 1
        }
        */
        let dots_spent = match self.current_mode {
            RenderMode::OAMScan => {
                // Scan the whole OAM in one shot since coroutines aren't 'real' yet
                // and I really don't want to implement that without those unless I really have to
                self.oam_scan_results.clear();
                
                const OAM_DOT_GRANULARITY: u32 = OAM_SCAN_TIME/40;
                self.current_dot += OAM_DOT_GRANULARITY;
                if (self.current_dot % DOTS_PER_LINE) >= OAM_SCAN_TIME {
                    self.oam_scan_results = self.scan_oam();
                    // println!("oam_scan_results length {}", self.oam_scan_results.len());
                }
                (OAM_DOT_GRANULARITY) as i16
            }
            RenderMode::PixelDraw => {
                // Actually granular timing is for nerds, let's just rip out whole modes at once
                // This could certainly make things funky within any line,
                // but SURELY this should be good enough and things will probably mostly shake out
                const PIXEL_DRAW_GRANULARITY: u32 = PIXEL_DRAW_TIME/4;
                let line_number = self.current_dot / DOTS_PER_LINE;
                self.current_dot += PIXEL_DRAW_GRANULARITY;
                // If we're onscreen and at the end of the pixel drawing mode, write the pixels into the buffer
                if line_number < SCREEN_HEIGHT as u32 {
                    if (self.current_dot % DOTS_PER_LINE) >= PIXEL_DRAW_END_DOTS {
                        self.draw_line(line_number);
                    }
                }
                PIXEL_DRAW_GRANULARITY as i16
            }
            RenderMode::HBlank => {
                const HBLANK_GRANULARITY: u32 = HBLANK_TIME/17;
                self.current_dot += HBLANK_GRANULARITY;
                HBLANK_GRANULARITY as i16
            }
            RenderMode::VBlank => {
                if self.current_dot == DOT_MAX - DOTS_PER_LINE {
                    self.swap_buffers();
                    //self.output_screen();
                    self.internal_window_line_counter = 0;
                }
                const VBLANK_TIME: u32 = DOTS_PER_LINE/19;
                self.current_dot += VBLANK_TIME;
                VBLANK_TIME as i16
            }
        };
        // Do some state transitions top level here so it happens after the cpu catches up
        self.update_render_state();
        dots_spent
    }

    fn swap_buffers(&mut self) {
        let tmp: usize = self.front_buffer_base;
        self.front_buffer_base = self.back_buffer_base;
        self.back_buffer_base = tmp;
        self.frame_ready = true;
    }

    // Handles mode changes and updates the render buffer with pixel data at the tail of VBlank
    fn update_render_state(&mut self) {
        let mut memory = self.system_memory.borrow_mut();
        let mut ly_eq_lyc = false;
        let mut start_oam_scan = false;
        let mut start_hblank = false;
        let mut start_vblank = false;

        let ly = (self.current_dot / DOTS_PER_LINE) as u8;
        let lyc: Byte = memory.read(LYC_ADDRESS);

        if (self.current_dot % DOTS_PER_LINE) == 0 {
            // We have this separate flag to check for a rising edge on this condition
            ly_eq_lyc = ly == lyc;
            // println!("ly {}, {}", ly, ly_eq_lyc);
        }

        self.current_mode = match self.current_mode {
            RenderMode::OAMScan => {
                if (self.current_dot % DOTS_PER_LINE) >= OAM_SCAN_TIME {
                    RenderMode::PixelDraw 
                } else { 
                    RenderMode::OAMScan 
                }
            }
            RenderMode::PixelDraw => {
                if (self.current_dot % DOTS_PER_LINE) >= PIXEL_DRAW_END_DOTS {
                    start_hblank = true;
                    RenderMode::HBlank
                }
                else {
                    RenderMode::PixelDraw
                }
            } 
            RenderMode::HBlank => {
                // HBlank is over when a new line is reached
                if (self.current_dot % DOTS_PER_LINE) == 0 {
                    if self.current_dot >= VBLANK_START_DOTS {
                        // At the end of the 144th line HBlank goes to VBlank
                        start_vblank = true;
                        RenderMode::VBlank
                    }
                    else {
                        // HBlank typically goes back to the next line's OAM scan
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
                    self.current_dot = 0;
                    start_oam_scan = true;
                    RenderMode::OAMScan
                }
                else {
                    RenderMode::VBlank
                }
            }
        };
        memory.write(ly, LY_ADDRESS);

        // Update the LY=LYC check and mode in the STAT register. 
        // Probably not enough to be accurate for CPU changes to LYC
        // Might be worth trapping LYC on the CPU to cover both ends

        //println!("ly {}", ly);
        let ly_eq_lyc_flag = (if lyc == ly { 1 } else { 0 }) << 2;
        let mode_number_flag = self.current_mode.mode_number();
        let old_stat: Byte = memory.read(STAT_ADDRESS);
        let stat = (old_stat & !(0x7)) | (ly_eq_lyc_flag | mode_number_flag);
        memory.write(stat, STAT_ADDRESS);
        
        // Handle possible interrupts arising from VBlank or STAT
        let mut interrupt_flag: Byte = memory.read(IF_REG_ADDR);
        // Check stat interrupt enables and set the stat interrupt flag if enabled mode changes occur
        if (start_oam_scan && (stat & (1 << 5)) > 0) 
            || (ly_eq_lyc && (stat & (1 << 6)) > 0)
            || (start_vblank && (stat & (1 << 4)) > 0) 
            || (start_hblank && (stat & (1 << 3)) > 0) 
        {
            interrupt_flag |= 0x2;
        }
        
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

        let lcdc: Byte = mem.read(LCDC_ADDRESS);
        let ly: Byte = mem.read(LY_ADDRESS);

        let objects_are_tall = (lcdc & (1 << 2)) > 0; 
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
        let bg_palette: Byte = mem.read(BGP_ADDRESS);
        let obj_palette_0: Byte = mem.read(OBP0_ADDRESS);
        let obj_palette_1: Byte = mem.read(OPB1_ADDRESS);
        let lcdc: Byte = mem.read(LCDC_ADDRESS);
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
            let wx: Byte = mem.read::<Byte>(WX_ADDRESS).saturating_sub(7);

            let mut drew_inside_window: bool = false;
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
                    let screen_pos_x = (pixel.wrapping_sub(wx as u16)) % TILEMAP_WH;
                    let screen_pos_y = (self.internal_window_line_counter) % TILEMAP_WH;
                    drew_inside_window = true;
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
                    // Impossible to overflow/underflow Address with the TDO value range, so we can just unwrap here
                    ((tile_data_base_address as i32) + (tile_data_offset * mem::size_of::<Tile>() as i32)).try_into().unwrap()
                };
                let tile = Tile::from_address(&mut mem, tile_data_address);
                let color = tile.color_index(tile_pos_x, tile_pos_y);
                // Always draw to the back buffer
                let pixel_index = self.back_buffer_base + SCREEN_WIDTH*(line_number as usize) + (pixel as usize);
                self.display_buffer[pixel_index] = color.unwrap().apply_palette(bg_palette);
            }
            if drew_inside_window {
                self.internal_window_line_counter += 1;
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
            for pixel in 0..(SCREEN_WIDTH as u16) {
                for object in &self.oam_scan_results {
                    // get the palette that this object is using
                    let obj_palette = if (object.flags & (1 << 4)) == 0 { obj_palette_0 } else { obj_palette_1 };
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
                        let color_index = tile.color_index(flip_adjusted_x, flip_adjusted_y % 8).unwrap();
                        // Blank is transparent, and should allow the background or lower priority objects to shine through
                        // No reason to draw blanks
                        if color_index != ColorIndex::Blank {
                            let pixel_index = self.back_buffer_base + SCREEN_WIDTH*(line_number as usize) + (pixel as usize);
                            // But otherwise, we draw it if objects have priority or the background is just a blank pixel
                            if ((object.flags & (1 << 7)) == 0) || (self.display_buffer[pixel_index].is_blank_color(bg_palette)) {
                                self.display_buffer[pixel_index] = color_index.apply_palette(obj_palette);
                            }
                        }
                    }
                }
            }
        }
    }
}
