/*
Graphical functionality for video module of vjr.
Contains structs and functions for scenes and various kinds of sprites. 

BASIC SHAPES:
    Ngon
    Triangle
    Square
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
    fn draw(&self, buffer: &mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16, setup_matrix: &Matrix);
}

/*
Default function for drawing a line between two points, using Bresenham's algorithm
*/
fn line_draw(p1: Coord2D, p2: Coord2D, outline: &[u8; 2], buffer: &mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16) {
    // Check bounds
    let x1: isize = 0;
    let y1: isize = y_offset as isize;
    let x2: isize = DISPLAY_BUFFER_WIDTH as isize;
    let y2: isize = y_offset as isize + DISPLAY_BUFFER_HEIGHT as isize;

    // If both points (in the same row) out of scope, no line to be drawn
    if p1.y < y1 && p2.y < y1 || p1.y > y2 && p2.y > y2 { return; }

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
    
    let mut y = ay;     // Start drawing at a
    let mut ierror = 0; // Track pixel error to correct line direction
    
    for x in ax..=bx {
        
        // Allow steepness to correct error
        let x_t = if steep { y } else { x };
        let y_t = if steep { x } else { y };

        if x1 <= x_t && x_t < x2 && y1 <= y_t && y_t < y2 { // Inside row buffer
            // We can draw here.
            // x does not change, but y is offsetted back.
            let px_x = x_t;
            let px_y = y_t - y_offset as isize;

            // Buffer is 1D, and each px is two bytes.
            let index: usize = ((px_x + (DISPLAY_BUFFER_WIDTH as isize * px_y)) * 2) as usize;

            buffer[index]     = outline[0];
            buffer[index + 1] = outline[1];
        }
        
        // Update integer error between y coordinates 
        ierror += 2 * (by - ay).abs();
        
        // Branchless updates for y (adding/subtracting 1) and error (taking into account x)
        y      += (if by > ay { 1 } else { -1 }) * (ierror > (bx - ax)) as isize;
        ierror -= (2 * (bx - ax))                * (ierror > (bx - ax)) as isize;
    }
}

/*
Default function for filling in a triangle
*/
fn triangle_fill(p1: Coord2D, p2: Coord2D, p3: Coord2D, fill: &[u8; 2], buffer: &mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16) {
    // Check bounds
    let x1: isize = 0;
    let y1: isize = y_offset as isize;
    let x2: isize = DISPLAY_BUFFER_WIDTH as isize;
    let y2: isize = y_offset as isize + DISPLAY_BUFFER_HEIGHT as isize;

    let mut a = p1;
    let mut b = p2;
    let mut c = p3;
    
    // If all points are out of range, we do not fill.
    if a.y < y1 && b.y < y1 && c.y < y1 || a.y >= y2 && b.y >= y2 && c.y >= y2 { return; }
    
    // Sort vertices from highest to lowest based on y values.
    if a.y > b.y { core::mem::swap(&mut a, &mut b); }
    if a.y > c.y { core::mem::swap(&mut a, &mut c); }
    if b.y > c.y { core::mem::swap(&mut b, &mut c); }

    let total_height = c.y - a.y;
    
    // Focus on filling top and bottom segments (split by b)
    
    if a.y != b.y { // Bottom half is not horizontally flat
        let segment_height = b.y - a.y; // From center line to bottom
        
        // Clamp y to row buffer border
        let y_start = a.y.max(y1);
        let y_end   = b.y.min(y2);

        // Draw horizontal lines with each y
        for y in y_start..y_end {
            let mut x_min = a.x + ((c.x - a.x) * (y - a.y)) / total_height;   // a -> c line
            let mut x_max = a.x + ((b.x - a.x) * (y - a.y)) / segment_height; // a -> b line 
            // Ensure x_min is smaller
            if x_min > x_max { core::mem::swap(&mut x_min, &mut x_max) }

            // Clamp x to row buffer border
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
        
        // Clamp y to row buffer border
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
            
            // Clamp x to row buffer border
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
    rotation: usize,
    
    // Used for tracking orbits about a pivot, for rotate_around
    orbit_pivot: Option<Coord2D>, // Option since does not always apply, also used as check
    orbit_rotation: usize,         

    // To save storage, only include vertices and order them, rather than indices as well.
    vertices: Vec<(f32, f32), { N_MAX as usize }>,

    matrix: Matrix,
    
    // To be replaced by Texture enum
    outline: RGB,
    fill   : RGB,
}

impl Ngon {
    pub fn new(n: u16, center: Coord2D, radius: u16) -> Self {
        // Impose a maximum Ngon n size
        assert!(n <= N_MAX);
        
        let mut vertices: Vec<(f32, f32), { N_MAX as usize }> = Vec::new(); 
        
        // Store vertices as floats for base, before matrix multiplications.
        for i in 0..n {
            let angle: usize = (ANGLE_360 * i as usize) / n as usize;

            let cur_x = cos_mult(angle, 4096) as f32 / 4096.0;
            let cur_y = sin_mult(angle, 4096) as f32 / 4096.0;

            let _ = vertices.push((cur_x, cur_y));
        }
        
        // Default rotation
        let rotation: usize = 0;
        
        // Default orbit
        let orbit_pivot: Option<Coord2D> = None;
        let orbit_rotation: usize = 0;

        // Initialise matrix
        let mut matrix = Matrix::identity();
        matrix.translate(center.x as f32, center.y as f32, 0.0);
        matrix.rotate(rotation, Axis::Z);
        matrix.scale(radius as f32, radius as f32, 1.0);

        Self {
            center,
            radius,
            rotation,
            orbit_pivot,
            orbit_rotation,
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
    
    // Updates matrix with stored information from scratch, prevents floating point math error 
    fn update_matrix(&mut self) {
        let mut m = Matrix::identity();
        
        if let Some(pivot) = self.orbit_pivot { // If orbiting
            // Encapsulate the transformation for an orbit
            m.translate(pivot.x as f32, pivot.y as f32, 0.0);       // Set at the pivot
            m.rotate(self.orbit_rotation, Axis::Z); // Rotate around it
            m.translate(-pivot.x as f32, -pivot.y as f32, 0.0);     // Set back to origin
        }

        // Perform Translate Rotate Scale update
        m.translate(self.center.x as f32, self.center.y as f32, 0.0);
        m.rotate(self.rotation, Axis::Z);
        m.scale(self.radius as f32, self.radius as f32, 1.0);

        self.matrix = m
    }
    
    // Rotation around center
    pub fn rotate_by(&mut self, angle: usize) { 
        self.rotation = (self.rotation + angle) & 0x0FFF;
        self.update_matrix();
    }
    
    // Rotation around pivot
    pub fn rotate_around(&mut self, pivot: Coord2D, angle: usize) {
        // Update orbit values
        self.orbit_pivot = Some(pivot);
        self.orbit_rotation = (self.orbit_rotation + angle) & 0x0FFF;
        self.update_matrix();
    }

}

impl Sprite for Ngon {
    // Fill drawing
    fn draw(&self, buffer: &mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16, setup_matrix: &Matrix) {
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
        }

        // Get new center
        let (cx_t, cy_t, _) = m.apply_to_vertex(0.0, 0.0, 0.0);
        let screen_cx = ((cx_t + 1.0) * 0.5 * 320.0) as isize;
        let screen_cy = ((1.0 - cy_t) * 0.5 * 240.0) as isize;
        let screen_center = Coord2D::new(screen_cx, screen_cy);

        
        // Fill draw, with scanline rasterization
        for i in 0..self.vertices.len() {
            // Grab current two points and center (3 points for triangle)
            let mut a = vertices[i];
            let mut b = vertices[(i + 1) % self.vertices.len()];
            let mut c = screen_center;

            triangle_fill(a, b, c, &fill, buffer, y_offset);
        }
        
        // Outline draw with Bresenham's algorithm
        for i in 0..self.vertices.len() {
            // Grab current two vertices (with loop around)
            let p1 = vertices[i];
            let p2 = vertices[(i + 1) % self.vertices.len()];
            
            line_draw(p1, p2, &outline, buffer, y_offset);
        }
    }
}

/*
Triangle shape.
*/
pub struct Triangle {
    // Change to floating point to apply matrix transformations to 
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),

    // Store the current center of the triangle, for easy manipulation
    centroid: (f32, f32),
    
    matrix: Matrix,

    outline: RGB,
    fill   : RGB,
}

impl Triangle {
    // A triangle can be made in many different ways, so support them.
    pub fn new(p1: Coord2D, p2: Coord2D, p3: Coord2D) -> Self {
        // Normalize points, such that the centroid is (0, 0)
        let centroid: (f32, f32) = (
            (p1.x + p2.x + p3.x) as f32 / 3.0, 
            (p1.y + p2.y + p3.y) as f32 / 3.0, 
        );

        let p1: (f32, f32) = (p1.x as f32 - centroid.0, p1.y as f32 - centroid.1);
        let p2: (f32, f32) = (p2.x as f32 - centroid.0, p2.y as f32 - centroid.1);
        let p3: (f32, f32) = (p3.x as f32 - centroid.0, p3.y as f32 - centroid.1);

        // Set up matrix
        let mut matrix: Matrix = Matrix::identity();
        matrix.translate(centroid.0, centroid.1, 0.0);

        Self { p1, p2, p3, centroid, matrix, outline: RGB::BLACK, fill: RGB::BLACK}
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

        // With that, we can set up proper Triangle

        let centroid: (f32, f32) = (
            center.x as f32,
            center.y as f32
        );

        let p1: (f32, f32) = (p1.x as f32 - centroid.0, p1.y as f32 - centroid.1);
        let p2: (f32, f32) = (p2.x as f32 - centroid.0, p2.y as f32 - centroid.1);
        let p3: (f32, f32) = (p3.x as f32 - centroid.0, p3.y as f32 - centroid.1);

        // Set up matrix
        let mut matrix: Matrix = Matrix::identity();
        matrix.translate(centroid.0, centroid.1, 0.0);

        Self { p1, p2, p3, centroid, matrix, outline: RGB::BLACK, fill: RGB::BLACK }
    }
    
    // Chain constructor for other parts of triangle
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
    
    pub fn move_vertex_to(&mut self, vertex: u8, coord: Coord2D) {
        // Get coordinates before centroid application
        let mut p1 = (self.p1.0 + self.centroid.0, self.p1.1 + self.centroid.1);
        let mut p2 = (self.p2.0 + self.centroid.0, self.p2.1 + self.centroid.1);
        let mut p3 = (self.p3.0 + self.centroid.0, self.p3.1 + self.centroid.1);

        // Apply coord to a chosen vertex
        let coord = (coord.x as f32, coord.y as f32);
        match vertex {
            1 => p1 = coord,
            2 => p2 = coord,
            3 => p3 = coord,
            _ => return,     // Add an error warning here or something
        }
        
        // Recalculate centroid, points and matrix
        self.centroid = (
            (p1.0 + p2.0 + p3.0) as f32 / 3.0, 
            (p1.1 + p2.1 + p3.1) as f32 / 3.0, 
        );

        self.p1 = (p1.0 as f32 - self.centroid.0, p1.1 as f32 - self.centroid.1);
        self.p2 = (p2.0 as f32 - self.centroid.0, p2.1 as f32 - self.centroid.1);
        self.p3 = (p3.0 as f32 - self.centroid.0, p3.1 as f32 - self.centroid.1);

        // Set up matrix
        self.matrix = Matrix::identity();
        self.matrix.translate(self.centroid.0, self.centroid.1, 0.0);
    }
    
    pub fn move_vertex_by(&mut self, vertex: u8, coord: Coord2D) {
        // Get coordinates before centroid application
        let mut p1 = (self.p1.0 + self.centroid.0, self.p1.1 + self.centroid.1);
        let mut p2 = (self.p2.0 + self.centroid.0, self.p2.1 + self.centroid.1);
        let mut p3 = (self.p3.0 + self.centroid.0, self.p3.1 + self.centroid.1);

        // Apply coord to a chosen vertex
        let coord = (coord.x as f32, coord.y as f32);
        match vertex {
            1 => p1 = (p1.0 + coord.0, p1.1 + coord.1),
            2 => p2 = (p2.0 + coord.0, p2.1 + coord.1),
            3 => p3 = (p3.0 + coord.0, p3.1 + coord.1),
            _ => return,     // Add an error warning here or something
        }
        
        // Recalculate centroid, points and matrix
        self.centroid = (
            (p1.0 + p2.0 + p3.0) as f32 / 3.0, 
            (p1.1 + p2.1 + p3.1) as f32 / 3.0, 
        );

        self.p1 = (p1.0 as f32 - self.centroid.0, p1.1 as f32 - self.centroid.1);
        self.p2 = (p2.0 as f32 - self.centroid.0, p2.1 as f32 - self.centroid.1);
        self.p3 = (p3.0 as f32 - self.centroid.0, p3.1 as f32 - self.centroid.1);

        // Set up matrix
        self.matrix = Matrix::identity();
        self.matrix.translate(self.centroid.0, self.centroid.1, 0.0);
    }
}

impl Sprite for Triangle {
    fn draw(&self, buffer: & mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16, setup_matrix: &Matrix) {
        // Pass points through matrix transformation
        
        let m = Matrix::mult_matrices(&setup_matrix, &self.matrix); 

        let (x_t, y_t, _) = m.apply_to_vertex(self.p1.0, self.p1.1, 0.0);
        // Mapped to [-1, 1], so return to ([0, 360], [0, 240])
        let x: isize = ((x_t + 1.0) * 0.5 * 320.0) as isize; 
        let y: isize = ((1.0 - y_t) * 0.5 * 240.0) as isize; // Grows downwards
        let p1 = Coord2D::new(x, y);
        
        let (x_t, y_t, _) = m.apply_to_vertex(self.p2.0, self.p2.1, 0.0);
        // Mapped to [-1, 1], so return to ([0, 360], [0, 240])
        let x: isize = ((x_t + 1.0) * 0.5 * 320.0) as isize; 
        let y: isize = ((1.0 - y_t) * 0.5 * 240.0) as isize; // Grows downwards
        let p2 = Coord2D::new(x, y);

        let (x_t, y_t, _) = m.apply_to_vertex(self.p3.0, self.p3.1, 0.0);
        // Mapped to [-1, 1], so return to ([0, 360], [0, 240])
        let x: isize = ((x_t + 1.0) * 0.5 * 320.0) as isize; 
        let y: isize = ((1.0 - y_t) * 0.5 * 240.0) as isize; // Grows downwards
        let p3 = Coord2D::new(x, y);

        // RGB565 means each px is two bytes.
        let outline = RawU16::from(self.outline).into_inner().to_be_bytes();
        let fill    = RawU16::from(self.fill)   .into_inner().to_be_bytes();
        
        triangle_fill(p1, p2, p3, &fill, buffer, y_offset);

        line_draw(p1, p2, &outline, buffer, y_offset);
        line_draw(p2, p3, &outline, buffer, y_offset);
        line_draw(p3, p1, &outline, buffer, y_offset);
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
