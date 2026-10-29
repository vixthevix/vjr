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
        Rgb565 as RGB, // We are only really dealing with rgb565
        raw::RawU16,
    },
    prelude::RawData
};

use crate::video::renderer::{
    DISPLAY_BUFFER_SIZE,
    DISPLAY_BUFFER_WIDTH,
    DISPLAY_BUFFER_HEIGHT,
};

use crate::trig::sine::{sin, sin_mult, ANGLE_360};
use crate::trig::cosine::{cos, cos_mult};

use heapless::Vec;

/*
Base trait for all sprites that can be drawn to the screen.
*/
pub trait Sprite {
    // fn can_draw(&self, top_left: Coord2D, bottom_right: Coord2D);
    fn draw(&self, buffer: & mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16);
}




/*
2D Coordinate struct
*/
#[derive(Debug)]
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
Texture enum for deciding between colour and loaded image.
*/
enum Texture {
    Colour(RGB, RGB),
    Image, // Unsure what values to attach right now.
}

/*
Ngon shape. A uniform shape with N vertices (Coord2D)
*/

const N_MAX: u16 = 64;

#[derive(Debug)]
pub struct Ngon {
    center: Coord2D,
    radius: u16,

    // To save storage, only include vertices and order them, rather than indices as well.
    vertices: Vec<Coord2D, { N_MAX as usize }>,

    outline: RGB,
    fill   : RGB,
}

impl Ngon {
    pub fn new(n: u16, center: Coord2D, radius: u16) -> Self {
        // Impose a maximum Ngon n size
        assert!(n <= N_MAX);

        // Temporarily store a max size vector, add only what we need, then cast to slice.
        let mut vertices: Vec<Coord2D, { N_MAX as usize }> = Vec::new(); 

        for i in 0..n {
            let angle: usize = (ANGLE_360 * i as usize) / n as usize;

            let cur_x: isize = center.x + cos_mult(angle, radius.into()) as isize;
            let cur_y: isize = center.y + sin_mult(angle, radius.into()) as isize;

            let _ = vertices.push(Coord2D { x: cur_x, y: cur_y });
        }

        Self {
            center,
            radius,
            vertices,
            outline: RGB::BLACK,
            fill: RGB::BLACK,
        }
    }
    
    pub fn with_outline(self, colour: RGB) -> Self {
        Self {
            outline: colour,
            ..self
        }
    }

    pub fn with_fill(self, colour: RGB) -> Self {
        Self {
            fill: colour,
            ..self
        }
    }
}

impl Sprite for Ngon {
    // Vertex drawing
//    fn draw(&self, buffer: &mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16) {
//          // Check bounds
//          let x1: isize = 0;
//          let y1: isize = y_offset as isize;
//          let x2: isize = DISPLAY_BUFFER_WIDTH as isize;
//          let y2: isize = y_offset as isize + DISPLAY_BUFFER_HEIGHT as isize;
//
//          // RGB565 means each px is two bytes.
//          let colour = RawU16::from(self.outline).into_inner();
//          let colour = colour.to_be_bytes();
//          
//          // Loop through vertices and draw points onto buffer.
//          for v in &self.vertices {
//              if x1 <= v.x && v.x < x2 && y1 <= v.y && v.y < y2 {
//                  // We can draw here.
//                  // x does not change, but y is offsetted back.
//                  let px_x = v.x;
//                  let px_y = v.y - y_offset as isize;
//
//                  // Buffer is 1D, and each px is two bytes.
//                  let index: usize = ((px_x + (DISPLAY_BUFFER_WIDTH as isize * px_y)) * 2) as usize;
//
//                  buffer[index]     = colour[0];
//                  buffer[index + 1] = colour[1];
//              }
//          }
//      }
    
    fn draw(&self, buffer: &mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16) {
        let x1: isize = 0;
        let y1: isize = y_offset as isize;
        let x2: isize = DISPLAY_BUFFER_WIDTH as isize;
        let y2: isize = y_offset as isize + DISPLAY_BUFFER_HEIGHT as isize;

        // RGB565 means each px is two bytes.
        let colour = RawU16::from(self.outline).into_inner();
        let colour = colour.to_be_bytes();
        
        for i in 0..self.vertices.len() {
            let p1 = &self.vertices[i];
            let p2 = &self.vertices[(i + 1) % self.vertices.len()];
            
            if (p1.y < y1 && p2.y < y1 || p1.y > y2 && p2.y > y2) { continue; }

            let mut ax = p1.x;
            let mut ay = p1.y;
            let mut bx = p2.x;
            let mut by = p2.y;
            
            let steep: bool = (ax - bx).abs() < (ay - by).abs();
            if steep {
               // Swap x and y, for transpose
               let temp = ax;
               ax = ay;
               ay = temp;
               
               let temp = bx;
               bx = by;
               by = temp;
            }

            if ax > bx { // Make left to right
               let temp = ax;
               ax = bx;
               bx = temp;
               
               let temp = ay;
               ay = by;
               by = temp;
            }

            let mut y = ay;
            let mut ierror = 0;
            for x in ax..=bx {
                
                let mut x_t = if steep { y } else { x };
                let mut y_t = if steep { x } else { y };

                if (x1 <= x_t && x_t < x2 && y1 <= y_t && y_t < y2) {
                    // We can draw here.
                    // x does not change, but y is offsetted back.
                    let px_x = x_t;
                    let px_y = y_t - y_offset as isize;

                    // Buffer is 1D, and each px is two bytes.
                    let index: usize = ((px_x + (DISPLAY_BUFFER_WIDTH as isize * px_y)) * 2) as usize;

                    buffer[index]     = colour[0];
                    buffer[index + 1] = colour[1];
                }
                
                ierror += 2 * (by - ay).abs();
                
                y      += (if by > ay { 1 } else { -1 }) * (ierror > (bx - ax)) as isize;
                ierror -= (2 * (bx - ax))                * (ierror > (bx - ax)) as isize;

            }

        }        
    }

    // Line drawing
//    fn draw(&self, buffer: &mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16) {
//        // Check bounds
//        let x1: isize = 0;
//        let y1: isize = y_offset as isize;
//        let x2: isize = DISPLAY_BUFFER_WIDTH as isize;
//        let y2: isize = y_offset as isize + DISPLAY_BUFFER_HEIGHT as isize;
//
//        // RGB565 means each px is two bytes.
//        let outline = RawU16::from(self.outline).into_inner().to_be_bytes();
//
//        // We will be using Bresenham's line drawing.
//        // Need each pair of close points.
//        
//        for i in 0..self.vertices.len() {
//            let p1 = self.vertices[i];
//            let p2 = self.vertices[(i + 1) % self.vertices.len()];
//
//            // If both points are out of range, we do not draw this line.
//            if !(p1.y < y1 && p2.y < y1 || p1.y > y2 && p2.y > y2) {
//                // We can draw a line here.
//                // Normal Bresenham involves a t between 0 and 1, a float.
//                // Instead, we can inflate to high values and then divide down.
//                let p1 = Coord2D::new(p1.x << 8, p1.y << 8);
//                let p2 = Coord2D::new(p2.x << 8, p2.y << 8);
//                let x_dif = p2.x - p1.x;
//                let y_dif = p2.y - p1.y;
//                for t in 0..256 {
//                     let x = (p1.x + (x_dif * t)) >> 8;
//                     let y = ((p1.y + (y_dif * t)) >> 8) - y_offset;
//
//                     if (y < y1 || y > y2) { continue; }
//
//                     // Buffer is 1D, and each px is two bytes.
//                     let index: usize = ((x + (DISPLAY_BUFFER_WIDTH as isize * y)) * 2) as usize;
//                     buffer[index]     = outline[0];
//                     buffer[index + 1] = outline[1];
//                }
//            }
//        }
//    }
}

/*
Triangle shape.
*/
pub struct Triangle {
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

pub struct Image {
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
        loop {
            let guard = &data[i..i+4];
            if guard == b"VIMG" {
                break;
            }
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
