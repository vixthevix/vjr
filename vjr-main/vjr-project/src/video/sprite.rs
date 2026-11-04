/*
Graphical functionality for video module of vjr.
Contains structs and functions for scenes and various kinds of sprites. 

BASIC SHAPES:
    Ngon
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

use crate::{
    trig::{
        sine::{ sin_mult, ANGLE_360 },
        cosine::{ cos_mult },
    },
    matrix::prelude::*,
};

use heapless::Vec;

use log::info;

/*
Base trait for all sprites that can be drawn to the screen.
*/
pub trait Sprite {
    // fn can_draw(&self, top_left: Coord2D, bottom_right: Coord2D);
    fn draw(&self, buffer: & mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16, setup_matrix: Matrix);
}


/*
2D Coordinate struct
*/
#[derive(Copy, Clone, Debug)]
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
#[warn(dead_code)]
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
    vertices: Vec<(f32, f32), { N_MAX as usize }>,

    matrix: Matrix,

    outline: RGB,
    fill   : RGB,
}

impl Ngon {
    pub fn new(n: u16, center: Coord2D, radius: u16) -> Self {
        // Impose a maximum Ngon n size
        assert!(n <= N_MAX);

        // Temporarily store a max size vector, add only what we need, then cast to slice.
        let mut vertices: Vec<(f32, f32), { N_MAX as usize }> = Vec::new(); 

        for i in 0..n {
            let angle: usize = (ANGLE_360 * i as usize) / n as usize;

            // let cur_x: isize = cos_mult(angle, radius.into()) as isize;
            // let cur_y: isize = sin_mult(angle, radius.into()) as isize;
            
            let cur_x = cos_mult(angle, 4096) as f32 / 4096.0;
            let cur_y = sin_mult(angle, 4096) as f32 / 4096.0;

            let _ = vertices.push((cur_x, cur_y));
        }

        // Initialise matrix
        let mut matrix = Matrix::identity();
        matrix.translate(center.x, center.y, 0);
        matrix.scale(radius as f32, radius as f32, 1.0);

        Self {
            center,
            radius,
            vertices,
            matrix,
            outline: RGB::BLACK,
            fill   : RGB::BLACK,
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
    
    // Rotation around center
    pub fn rotate_by(&mut self, angle: usize) {
        self.matrix.rotate(angle, Axis::Z);
    }
    
    // Rotation around pivot
    pub fn rotate_around(&mut self, pivot: Coord2D, angle: usize) {
        // Build a matrix representing the entire transformation
        let mut t = Matrix::identity();
        t.translate(pivot.x, pivot.y, 0);   // Shift to position
        t.rotate(angle, Axis::Z);           // Rotate
        t.translate(-pivot.x, -pivot.y, 0); // Shift to origin
        
        // Apply the transformation
        self.matrix = Matrix::mult_matrices(&t, &self.matrix);
    }

}

impl Sprite for Ngon {
    // Fill drawing
    fn draw(&self, buffer: &mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16, setup_matrix: Matrix) {
        // Check bounds
        let x1: isize = 0;
        let y1: isize = y_offset as isize;
        let x2: isize = DISPLAY_BUFFER_WIDTH as isize;
        let y2: isize = y_offset as isize + DISPLAY_BUFFER_HEIGHT as isize;

        // RGB565 means each px is two bytes.
        let outline = RawU16::from(self.outline).into_inner().to_be_bytes();
        let fill    = RawU16::from(self.fill)   .into_inner().to_be_bytes();
        
        // We must apply our matrix transformations to our vertices.
        let m = Matrix::mult_matrices(&setup_matrix, &self.matrix); 
        let mut vertices = [Coord2D { x: 0, y: 0 }; N_MAX as usize];
        let mut i = 0;
        for vertex in &self.vertices {
            let (x_t, y_t, _) = m.apply_to_vertex(vertex.0, vertex.1, 0.0);

            // Mapped to [-1, 1], so return to ([0, 360], [0, 240])
            let x: isize = ((x_t + 1.0) * 0.5 * 320.0) as isize; 
            let y: isize = ((1.0 - y_t) * 0.5 * 240.0) as isize; // Grows downwards
            
            vertices[i] = Coord2D::new(x, y);
            i += 1;
            // info!("Coord {i}: ({x}, {y})");
        }

        // Get new center
        let (cx_t, cy_t, _) = m.apply_to_vertex(0.0, 0.0, 0.0);
        let screen_cx = ((cx_t + 1.0) * 0.5 * 320.0) as isize;
        let screen_cy = ((1.0 - cy_t) * 0.5 * 240.0) as isize;
        let screen_center = Coord2D::new(screen_cx, screen_cy);

        // We will be using Scanline Rasterization.
        // Need each pair of close points, along with center of triangle.
        
        // Fill draw
        for i in 0..self.vertices.len() {
            let mut a = vertices[i];
            let mut b = vertices[(i + 1) % self.vertices.len()];
            let mut c = screen_center;
            
            // If all points are out of range, we do not fill.
            if a.y < y1 && b.y < y1 && c.y < y1 || a.y >= y2 && b.y >= y2 && c.y >= y2 { continue; }
            
            // Sort vertices from highest to lowest based on y values.
            if a.y > b.y { core::mem::swap(&mut a, &mut b); }
            if a.y > c.y { core::mem::swap(&mut a, &mut c); }
            if b.y > c.y { core::mem::swap(&mut b, &mut c); }

            let total_height = c.y - a.y;
            
            // Focus on filling top and bottom segments (split by b)
            
            if a.y != b.y { // Bottom half is not horizontally flat
                let segment_height = b.y - a.y; // From center line to bottom
                
                // Clamp y, to ensure lines existing in the current chunk are drawn only
                let y_start = a.y.max(y1);
                let y_end   = b.y.min(y2);

                // Draw horizontal lines with each y
                for y in y_start..y_end {
                    let mut x_min = a.x + ((c.x - a.x) * (y - a.y)) / total_height;   // a -> c line
                    let mut x_max = a.x + ((b.x - a.x) * (y - a.y)) / segment_height; // a -> b line 
                    // Ensure x_min is smaller
                    if x_min > x_max { core::mem::swap(&mut x_min, &mut x_max) }

                    // Clamp x
                    let x_start = x_min.max(x1);
                    let x_end   = x_max.min(x2);

                    // Perform draw
                    for x in x_start..x_end {
                        // We can draw here.
                        // x does not change, but y is offsetted back.
                        let px_x = x;
                        let px_y = y - y_offset as isize;

                        // Buffer is 1D, and each px is two bytes.
                        let index: usize = ((px_x + (DISPLAY_BUFFER_WIDTH as isize * px_y)) * 2) as usize;

                        buffer[index]     = fill[0];
                        buffer[index + 1] = fill[1];
                    }
                }
            }
            
            if b.y != c.y { // Top half is not horizontally flat
                let segment_height = c.y - b.y; // From center line to bottom
                
                // Clamp y, to ensure lines existing in the current chunk are drawn only
                let y_start = b.y.max(y1);
                let y_end   = c.y.min(y2);
                
                // Draw horizontal lines with each y
                for y in y_start..y_end {
                    let mut x_min = a.x + ((c.x - a.x) * (y - a.y)) / total_height;   // a -> c line
                    let mut x_max = b.x + ((c.x - b.x) * (y - b.y)) / segment_height; // a -> b line 
                    // Ensure x_min is smaller
                    if x_min > x_max { core::mem::swap(&mut x_min, &mut x_max) }
                    if x_min > x_max {
                        let temp = x_min;
                        x_min = x_max;
                        x_max = temp;
                    }
                    
                    // Clamp x
                    let x_start = x_min.max(x1);
                    let x_end   = x_max.min(x2);

                    // Perform draw
                    for x in x_start..x_end {
                        // We can draw here.
                        // x does not change, but y is offsetted back.
                        let px_x = x;
                        let px_y = y - y_offset as isize;

                        // Buffer is 1D, and each px is two bytes.
                        let index: usize = ((px_x + (DISPLAY_BUFFER_WIDTH as isize * px_y)) * 2) as usize;

                        // Draw outline for first and last point
                        buffer[index]     = fill[0];
                        buffer[index + 1] = fill[1];
                    }
                }
            }
        }
        
        // Outline draw
        for i in 0..self.vertices.len() {
            let p1 = vertices[i];
            let p2 = vertices[(i + 1) % self.vertices.len()];
            
            if p1.y < y1 && p2.y < y1 || p1.y > y2 && p2.y > y2 { continue; }

            let mut ax = p1.x;
            let mut ay = p1.y;
            let mut bx = p2.x;
            let mut by = p2.y;
            
            let steep: bool = (ax - bx).abs() < (ay - by).abs();
            if steep {
                // Swap x and y, for transpose
                core::mem::swap(&mut ax, &mut ay);
                core::mem::swap(&mut bx, &mut by);
            }

            if ax > bx { // Make left to right
                core::mem::swap(&mut ax, &mut bx);
                core::mem::swap(&mut ay, &mut by);
            }

            let mut y = ay;
            let mut ierror = 0;
            for x in ax..=bx {
                
                let x_t = if steep { y } else { x };
                let y_t = if steep { x } else { y };

                if x1 <= x_t && x_t < x2 && y1 <= y_t && y_t < y2 {
                    // We can draw here.
                    // x does not change, but y is offsetted back.
                    let px_x = x_t;
                    let px_y = y_t - y_offset as isize;

                    // Buffer is 1D, and each px is two bytes.
                    let index: usize = ((px_x + (DISPLAY_BUFFER_WIDTH as isize * px_y)) * 2) as usize;

                    buffer[index]     = outline[0];
                    buffer[index + 1] = outline[1];
                }
                
                ierror += 2 * (by - ay).abs();

                y      += (if by > ay { 1 } else { -1 }) * (ierror > (bx - ax)) as isize;
                ierror -= (2 * (bx - ax))                * (ierror > (bx - ax)) as isize;
            }
        }        
    }
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
