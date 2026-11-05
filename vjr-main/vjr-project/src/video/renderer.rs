
use embedded_graphics_core::{
    prelude::RawData,
    pixelcolor::{
        Rgb565 as RGB, // We are only really dealing with rgb565
        raw::RawU16
    },
};

use crate::{
    video::{
        pusher::Pusher,
        scene::*,
    },
    matrix::{
        camera::Camera,
        prelude::*,
    },
};

pub const DISPLAY_BUFFER_SIZE: usize = 10240;

pub const DISPLAY_BUFFER_WIDTH : usize = 320;
pub const DISPLAY_BUFFER_HEIGHT: usize = 16;

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

    pub fn draw_scene(&mut self, scene: &mut Scene, camera: &Camera) {
        let mut buffer_swap = true;
        
        // Deconstruct self to access mutable references
        let Self { buffer_a, buffer_b, pusher } = self;
        
        // Create matrix to pass in
        let m = Matrix::mult_matrices(&scene.projection.get_matrix(), &camera.get_matrix());

        for chunk in 0..20 {
            let mut cur_buffer = if buffer_swap { *buffer_a } else { *buffer_b };
            let cur_y = (chunk * DISPLAY_BUFFER_HEIGHT) as u16;
            
            Self::fill_bg(&mut cur_buffer, scene.bg_colour);

            unsafe {
                for &sprite in &scene.sprites {
                    (*sprite).draw(&mut cur_buffer, cur_y, &m);
                }
            }
            
            // Draw pixels onto screen
            pusher.push(&cur_buffer, cur_y);
            
            // Double buffering swap
            buffer_swap = !buffer_swap;
        }
    }
    
    // General function for filling in a background colour
    fn fill_bg(buffer: &mut [u8; DISPLAY_BUFFER_SIZE], bg_colour: RGB) {
        // We need the raw u16 value from the colour.
        let u16_container = RawU16::from(bg_colour);
        let colour = u16_container.into_inner();
        
        let colour = (colour << 8) | (colour >> 8);
        // UNSAFE
        // Rust slices have the 'fill' method, which does what you think it does.
        // However, colour is a u16, not a u8, so we have to perform type casting here.

        let u16_buffer = unsafe {
            core::slice::from_raw_parts_mut(     // Take a subslice with pointer and stride
                buffer.as_mut_ptr() as *mut u16, // Casted pointer
                DISPLAY_BUFFER_SIZE / 2          // Half our size (8 * 2 = 16)
            )
        };

        // Now we can safely all fill for the fastest filling possible.
        u16_buffer.fill(colour);
    }
}
