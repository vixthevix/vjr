
use embedded_graphics_core::{
    pixelcolor::{
        RgbColor, // Trait
        Rgb565 as RGB // We are only really dealing with rgb565
    },
};

use crate::video::pusher::Pusher;

pub const DISPLAY_BUFFER_SIZE: usize = 10240;

#[macro_export]
macro_rules! display_buffer_new {
    ($b:ident) => {
        static $b: StaticCell<[u8; DISPLAY_BUFFER_SIZE]> = StaticCell::new();
    }
}

#[macro_export]
macro_rules! display_buffer_init {
    ($b:expr) => {
        $b.init([0_u8; DISPLAY_BUFFER_SIZE])
    }
}

/*
Struct for holding video row buffers and displaying scenes.
*/
pub struct Renderer {
    buffer_a : [u8; DISPLAY_BUFFER_SIZE],
    buffer_b : [u8; DISPLAY_BUFFER_SIZE],
    bg_colour: RGB,
    pusher   : Pusher,
}

impl Renderer {
    pub fn new(pusher: Pusher) -> Self {
        Self {
            buffer_a: [u8; DISPLAY_BUFFER_SIZE],
            buffer_b: [u8; DISPLAY_BUFFER_SIZE],
            pusher,
        }
    }

    pub fn draw_scene(&mut self, scene: &mut Scene) {
        let mut buffer_swap = true;
        let chunk_height = 16; // Pixels
        
        for chunk in 0..15 {
            let cur_buffer = if buffer_swap { &mut self.buffer_a } else { &mut self.buffer_b };
            let cur_y = (chunk * chunk_height) as u16;
            
            cur_buffer.fill(scene.bg_colour);
            
            for sprite in &scene.sprites {
                sprite.draw(cur_buffer, cur_y);
            }

            self.pusher.push(cur_buffer, cur_y);
            
            buffer_swap = !buffer_swap;
        }
    }
}
