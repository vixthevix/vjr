
use embedded_graphics_core::{
    prelude::RawData,
    pixelcolor::{
        RgbColor, // Trait
        Rgb565 as RGB, // We are only really dealing with rgb565
        raw::RawU16
    },
};

use crate::video::pusher::Pusher;
use crate::video::scene::*;

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
pub struct Renderer<P> {
    buffer_a : [u8; DISPLAY_BUFFER_SIZE],
    buffer_b : [u8; DISPLAY_BUFFER_SIZE],
    pusher   : P,
}

impl<P: Pusher> Renderer<P> {
    pub fn new(pusher: P) -> Self {
        Self {
            buffer_a: [0_u8; DISPLAY_BUFFER_SIZE],
            buffer_b: [0_u8; DISPLAY_BUFFER_SIZE],
            pusher,
        }
    }

    pub fn draw_scene(&mut self, scene: &mut Scene) {
        let mut buffer_swap = true;
        let chunk_height = 16; // Pixels
        
        // Deconstruct self
        let Self { buffer_a, buffer_b, pusher } = self;

        for chunk in 0..15 {
            let mut cur_buffer = if buffer_swap { *buffer_a } else { *buffer_b };
            let cur_y = (chunk * chunk_height) as u16;
            
            // THIS LINE MUST BE FIXED
            Self::fill_bg(&mut cur_buffer, scene.bg_colour);

            unsafe {
                for &sprite in &scene.sprites {
                    (*sprite).draw(&mut cur_buffer, cur_y);
                }
            }

            pusher.push(&cur_buffer, cur_y);
            
            buffer_swap = !buffer_swap;
        }
    }
    
    // General function for filling in a background colour
    fn fill_bg(buffer: &mut [u8; DISPLAY_BUFFER_SIZE], bg_colour: RGB) {
        // We need the raw u16 value from the colour.
        let u16_container = RawU16::from(bg_colour);
        let colour = u16_container.into_inner();
        
        let colour_bytes = colour.to_ne_bytes();
        for pixel in buffer.chunks_exact_mut(2) {
            pixel.copy_from_slice(&colour_bytes);
        }
    }
}
