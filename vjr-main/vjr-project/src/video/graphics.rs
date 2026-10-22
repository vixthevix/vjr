/*
Graphical functionality for video module of vjr.
Contains structs and functions for scenes and various kinds of sprites. 

BASIC SHAPES:
    Triangle
    Square

Image sprite as well.

*/
use embedded_graphics_core::{
    pixelcolor::{
        RgbColor, // Trait
        Rgb565 as RGB // We are only really dealing with rgb565
    },
    // draw_target::DrawTarget, // Trait
};

use heapless::Vec;


use crate::video::base::*;

/*
Base trait for all sprites that can be drawn to the screen.
*/
pub trait Sprite {
    fn can_draw(&self, top_left: Coord2D, bottom_right: Coord2D);
    fn draw(&self, buffer: &'static mut [u8; DISPLAY_BUFFER_SIZE]);
}


/*
Struct for holding video row buffers and displaying scenes.
*/
pub struct Renderer {
    buffer_a: [u8; DISPLAY_BUFFER_SIZE],
    buffer_b: [u8; DISPLAY_BUFFER_SIZE],
}

impl Renderer {
    pub fn new() {}

    pub fn draw_scene(&mut self, scene: &mut Scene) {}
}

/*
Struct for looping through Sprites and drawing them.
*/
const SCENE_SPRITE_MAX: u16 = 64;
pub struct Scene {
    sprites: Vec<*mut dyn Sprite, { SCENE_SPRITE_MAX as usize }>
}

impl Scene {
    pub fn new(buffer_a: [u8; DISPLAY_BUFFER_SIZE], buffer_b: [u8; DISPLAY_BUFFER_SIZE] ) -> Self {
        Self {
            buffer_a,
            buffer_b,
            buffer_swap: true,
            sprites: Vec::new(),
        }
    }
    
    /*
    Adds a Sprite to the Scene, and returns a handle (index) to that Sprite.
    */
    pub fn add(&mut self, sprite: &'static mut dyn Sprite) -> usize {
        let _ = self.sprites.push(sprite as *mut dyn Sprite);
        
        self.sprites.len() - 1
    }
    
    pub fn add_at(&mut self, sprite: &'static mut dyn Sprite, index: usize) -> usize {
        let _ = self.sprites.insert(index, sprite as *mut dyn Sprite);

        index
    }

    pub fn pop(&mut self) -> u16 {
        self.sprites.pop();

        (self.sprites.len() - 1) as u16
    }
    
    pub fn pop_at(&mut self, index: usize) -> usize {
        self.sprites.remove(index);

        index
    }
    
    // Swap two Sprites around in a scene, changing drawing order.
    pub fn swap(&mut self, a: usize, b: usize) -> (usize, usize) {
        let temp = self.sprites[a];
        self.sprites[a] = self.sprites[b];
        self.sprites[b] = temp;

        (b, a)
    }

    pub fn shift_down(&mut self, index: usize, count: usize) -> usize {
        
        let mut new_index = index;

        for _ in 0..count {
            (new_index, _) = self.swap(new_index, new_index - 1);
        }

        new_index
    }
    
    pub fn shift_up(&mut self, index: usize, count: usize) -> usize {
        
        let mut new_index = index;

        for _ in 0..count {
            (new_index, _) = self.swap(new_index, new_index + 1);
        }

        new_index
    }

    pub fn sprite_count(&self) -> usize {
        self.sprites.len() as usize
    }
    
    // UNSAFE FUNCTION
    // Dereferences pointer to draw to screen.
    pub fn draw(&self, display: &mut Display) {
        
        for &sprite in &self.sprites {
            unsafe {
                (*sprite).draw();
            }
        }
    }
}

/*
2D Coordinate struct
*/
pub struct Coord2D {
    pub x: isize,
    pub y: isize,
}

impl Coord2D {
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
}

/*
3D Coordinate struct
*/
pub struct Coord3D {
    pub x: isize,
    pub y: isize,
    pub z: isize,
} 

impl Coord3D {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        Self { x, y, z }
    }
}


/*
Triangle shape.
*/
struct Triangle {
    p1: Coord2D,
    p2: Coord2D,
    p3: Coord2D,
    
    outline_colour: RGB,
    fill_colour   : RGB,
}

impl Triangle {
    // A triangle can be made in many different ways, so support them.

    pub fn new(p1: Coord2D, p2: Coord2D, p3: Coord2D) -> Self {
        Self { p1, p2, p3 , outline_colour: RGB::BLACK, fill_colour: RGB::BLACK}
    }

    pub fn new_equilateral(center: Coord2D, radius: usize) -> Self {        
        
        // Assume 0 degrees is pointing up, and we spin around 120 degrees each time.
        // Use linear algebra to rotate.
        // Sin 120 = 0.866
        // Cos 120 = -0.5
        //
        // cos x   -sin x
        // sin x    cos x
        //
        // x' = x cos t - y sin t
        // y' = x sin t + y cos t


        let original = Coord2D {
            x: 0,
            y: 0 - radius as isize,
        };

        let p1 = Coord2D {
            x: original.x + center.x,
            y: original.y + center.y,
        };

        let original = Coord2D {
            x: ((original.x as f32 * -0.5)  - (original.y as f32 * 0.866)) as isize,
            y: ((original.x as f32 * 0.866) + (original.y as f32 * -0.5))  as isize,
        };

        let p2 = Coord2D {
            x: original.x + center.x,
            y: original.y + center.y,
        };

        let original = Coord2D {
            x: ((original.x as f32 * -0.5)  - (original.y as f32 * 0.866)) as isize,
            y: ((original.x as f32 * 0.866) + (original.y as f32 * -0.5))  as isize,
        };

        let p3 = Coord2D {
            x: original.x + center.x,
            y: original.y + center.y,
        };

        Self { p1, p2, p3, outline_colour: RGB::BLACK, fill_colour: RGB::BLACK }
    }
    
    // Chain constructor for other parts of triangle
    pub fn with_outline(self, colour: RGB) -> Self {
        Self {
            outline_colour: colour,
            ..self
        }
    }

    pub fn with_fill(self, colour: RGB) -> Self {
        Self {
            fill_colour: colour,
            ..self
        }
    }
}


/*
Image sprite.
Supports VIMG only for now.
*/

struct Image {
    pub pos: Coord2D,
    pub width:  u16,
    pub height: u16,
    pub compression: u8,
    
    pub meta: &'static [u8], // Metadata for compression and such.
    pub data: &'static [u8], // Essentially, a reference to the image data in flash.

}

impl Image {
    pub fn new(pos: Coord2D, data: &'static [u8]) -> Self {
        
        // Read the data for the header and extract information from it.
        
        // Find 'VIMG' guard
        let guard = &data[0..4];
        if !(guard == b"VIMG") {
            panic!("Invalid VIMG, starting guard not present");
        }

        // Compression type stored in 1 byte
        let compression: u8 = data[4];

        // Width and height store in 2 bytes each.
        let width  = u16::from_be_bytes([data[5], data[6]]);
        let height = u16::from_be_bytes([data[7], data[8]]);


        // Copy over metadata about the VIMG
        // Metadata ends once VIMG end guard reached.
        let mut i: usize = 9;
        let mut meta_length: usize = 0;
        loop {
            let guard = &data[i..i+4];
            if guard == b"VIMG" {
                break;
            }
            meta_length += 1;
            i += 1;
        }
        
        let meta: &'static [u8] = &data[9..i];

        // i has start index of VIMG, so add 4
        i += 4;
        let raw_data: &'static [u8] = &data[i..];

        Self {
            pos,
            width,
            height,
            compression,

            meta,
            data: raw_data,
        }
    }
}
