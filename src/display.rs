use std::str::FromStr;

use minifb::{Icon, ScaleMode, Window, WindowOptions};

pub struct DisplayMiniFB {
    pub width: usize,
    pub height: usize,
    window: Window
}

impl DisplayMiniFB {
    pub fn new() -> Self {
        const WIDTH: usize = 160;
        const HEIGHT: usize = 144;

        let mut window = Window::new(
            "GBZD - :^)",
            WIDTH,
            HEIGHT,
            WindowOptions {
                resize: true,
                scale_mode: ScaleMode::UpperLeft,
                ..WindowOptions::default()
            },
        )
        .expect("Unable to create the window");
        
        // window.set_target_fps(60);
        window.limit_update_rate(None);

        window.set_icon(Icon::from_str("images/ziti_icon.ico").unwrap());

        DisplayMiniFB {
            width: WIDTH,
            height: HEIGHT,
            window
        }
    }

    pub fn update(&mut self, color_buffer: &Vec<u32>) {
        self.window.update_with_buffer(color_buffer, self.width, self.height).unwrap();
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }
}
