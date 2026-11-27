/*
Graphical functionality for video module of vjr.
Contains structs and functions for scenes and various kinds of sprites. 

BASIC SHAPES:
    Ngon
    Triangle
    Square
*/

use core::fmt::{
    Arguments,
    Write
};

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

use heapless::{
    Vec,
    String,
};

// use log::info;

/*
Base trait for all sprites that can be drawn to the screen.
*/
pub const SPRITE_CACHE_MAX: usize = N_MAX as usize * 4;
pub trait Sprite {
    fn gen_vertices(&self, cache: &mut [(i16, i16, u16, u16); SPRITE_CACHE_MAX], setup_matrix: &Matrix);
    fn draw(&self, vertices: &[(i16, i16, u16, u16); SPRITE_CACHE_MAX], buffer: &mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16);
}

/*
Default function for drawing a line between two points, using Bresenham's algorithm
*/
fn line_draw(p1: (i16, i16), p2: (i16, i16), outline: &[u8; 2], buffer: &mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16) {
    // Check bounds
    let x1: i16 = 0;
    let y1: i16 = y_offset as i16;
    let x2: i16 = DISPLAY_BUFFER_WIDTH as i16;
    let y2: i16 = y_offset as i16 + DISPLAY_BUFFER_HEIGHT as i16;

    // If both points (in the same row) out of scope, no line to be drawn
    if p1.1 < y1 && p2.1 < y1 || p1.1 > y2 && p2.1 > y2 { return; }

    let mut ax = p1.0;
    let mut ay = p1.1;
    let mut bx = p2.0;
    let mut by = p2.1;
     
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
            let px_y = y_t - y_offset as i16;

            // Buffer is 1D, and each px is two bytes.
            let index: usize = ((px_x + (DISPLAY_BUFFER_WIDTH as i16 * px_y)) * 2) as usize;

            buffer[index]     = outline[0];
            buffer[index + 1] = outline[1];
        }
        
        // Update integer error between y coordinates 
        ierror += 2 * (by - ay).abs();
        
        // Branchless updates for y (adding/subtracting 1) and error (taking into account x)
        y      += (if by > ay { 1 } else { -1 }) * (ierror > (bx - ax)) as i16;
        ierror -= (2 * (bx - ax))                * (ierror > (bx - ax)) as i16;
    }
}

/*
Default function for filling in a triangle with a colour
*/
fn triangle_fill(p1: (i16, i16), p2: (i16, i16), p3: (i16, i16), fill: &[u8; 2], buffer: &mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16) {
    // Check bounds
    let x1: i16 = 0;
    let y1: i16 = y_offset as i16;
    let x2: i16 = DISPLAY_BUFFER_WIDTH as i16;
    let y2: i16 = y_offset as i16 + DISPLAY_BUFFER_HEIGHT as i16;

    let mut a = Coord2D::new(p1.0.into(), p1.1.into());
    let mut b = Coord2D::new(p2.0.into(), p2.1.into());
    let mut c = Coord2D::new(p3.0.into(), p3.1.into());
    
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
            
            let px_y = y - y_offset as i16;
            let px_offset = DISPLAY_BUFFER_WIDTH as i16 * px_y;

            // Perform draw
            for x in x_start..x_end {
                // We can draw here.
                // x does not change, but y is offsetted back.
                let px_x = x;

                // Buffer is 1D, and each px is two bytes.
                let index: usize = ((px_x + px_offset) * 2) as usize;

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

            let px_y = y - y_offset as i16;
            let px_offset = DISPLAY_BUFFER_WIDTH as i16 * px_y;

            // Perform draw
            for x in x_start..x_end {
                // We can draw here.
                // x does not change, but y is offsetted back.
                let px_x = x;

                // Buffer is 1D, and each px is two bytes.
                let index: usize = ((px_x + px_offset) * 2) as usize;

                // Draw outline for first and last point
                buffer[index]     = fill[0];
                buffer[index + 1] = fill[1];
            }
        }
    }
}

/*
Default function for filling in a triangle with an image texture
*/
fn triangle_texture(p1: (i16, i16, u16, u16), p2: (i16, i16, u16, u16), p3: (i16, i16, u16, u16), image: &Image, buffer: &mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16) {
    // Check bounds
    let x1: i16 = 0;
    let y1: i16 = y_offset as i16;
    let x2: i16 = DISPLAY_BUFFER_WIDTH as i16;
    let y2: i16 = y_offset as i16 + DISPLAY_BUFFER_HEIGHT as i16;
    
    // Cast texture coords to isize
    let t1 = (p1.2 as i16, p1.3 as i16);
    let t2 = (p2.2 as i16, p2.3 as i16);
    let t3 = (p3.2 as i16, p3.3 as i16);

    let mut a = (Coord2D::new(p1.0, p1.1), t1);
    let mut b = (Coord2D::new(p2.0, p2.1), t2);
    let mut c = (Coord2D::new(p3.0, p3.1), t3);

    // If all points are out of range, we do not fill.
    if a.0.y < y1 && b.0.y < y1 && c.0.y < y1 || a.0.y >= y2 && b.0.y >= y2 && c.0.y >= y2 { return; }
    
    // Sort vertices from highest to lowest based on y values.
    if a.0.y > b.0.y { core::mem::swap(&mut a, &mut b); }
    if a.0.y > c.0.y { core::mem::swap(&mut a, &mut c); }
    if b.0.y > c.0.y { core::mem::swap(&mut b, &mut c); }

    let total_height = c.0.y - a.0.y;

    // Get access to the texture
    let texture = &image.data;
    
    // Focus on filling top and bottom segments (split by b)
    
    if a.0.y != b.0.y { // Bottom half is not horizontally flat
        let segment_height = b.0.y - a.0.y; // From center line to bottom
        
        // Clamp y to row buffer border
        let y_start = a.0.y.max(y1) as i16;
        let y_end   = b.0.y.min(y2) as i16;

        // Draw horizontal lines with each y
        for y in y_start..y_end {
            // Get coordinate and texture lines for each edge
            let y_diff: i16 = (y - a.0.y) as i16;

            // a -> c line
            let x_ac = a.0.x + ((c.0.x - a.0.x) * y_diff) / total_height;  
            let u_ac = a.1.0 + ((c.1.0 - a.1.0) * y_diff) / total_height;
            let v_ac = a.1.1 + ((c.1.1 - a.1.1) * y_diff) / total_height;
            
            // a -> b line
            let x_ab = a.0.x + ((b.0.x - a.0.x) * y_diff) / segment_height;  
            let u_ab = a.1.0 + ((b.1.0 - a.1.0) * y_diff) / segment_height;
            let v_ab = a.1.1 + ((b.1.1 - a.1.1) * y_diff) / segment_height;
        
            
            // Determine min and max
            let (x_min, x_max, u_min, u_max, v_min, v_max) = if x_ac < x_ab {
                (x_ac, x_ab, u_ac as u32, u_ab as u32, v_ac as u32, v_ab as u32)
            }
            else {
                (x_ab, x_ac, u_ab as u32, u_ac as u32, v_ab as u32, v_ac as u32)
            };
            
            // To navigate texture, calculate gradient of U and V lines with respect to x
            // Since integer division, use bit shifting to account for errors
            
            let dx = (x_max - x_min) as i32;

            let du_dx = if dx > 0 { ((u_max - u_min) << 16) as i32 / dx } else { 0 };
            let dv_dx = if dx > 0 { ((v_max - v_min) << 16) as i32 / dx } else { 0 };
            
            // UV trackers, incremented by d/dx, with shifting taken into account for correction
            let mut u_cur = u_min << 16;
            let mut v_cur = v_min << 16;
            
            // Clamp x to row buffer border
            let x_start = x_min.max(x1);
            let x_end   = x_max.min(x2);

            let px_y = y - y_offset as i16;
            let px_offset = DISPLAY_BUFFER_WIDTH as i16 * px_y;

            // Perform draw
            for x in x_start..x_end {
                // Shift UV back to get actual coordinates
                let tex_x = (u_cur >> 16) as usize;
                let tex_y = (v_cur >> 16) as usize;
                
                // Clamp UV to image size
                let tex_x = tex_x.min(image.width  as usize - 1);
                let tex_y = tex_y.min(image.height as usize - 1);
                
                // Texture data is 1D, and each px is two bytes
                let tex_index = (tex_x + (image.width as usize * tex_y)) * 2;

                // x does not change, but y is offsetted back.
                let px_x = x;

                // Buffer is 1D, and each px is two bytes.
                let index: usize = ((px_x + px_offset) * 2) as usize;

                buffer[index]     = texture[tex_index];
                buffer[index + 1] = texture[tex_index + 1];

                // Increment current UV
                u_cur += du_dx as u32;
                v_cur += dv_dx as u32;
            }
        }
    }
    
    if b.0.y != c.0.y { // Top half is not horizontally flat
        let segment_height = c.0.y - b.0.y; // From center line to bottom
        
        // Clamp y to row buffer border
        let y_start = b.0.y.max(y1);
        let y_end   = c.0.y.min(y2);
        
        // Draw horizontal lines with each y
        for y in y_start..y_end {
            // Get coordinate and texture lines for each edge

            // a -> c line
            let x_ac = a.0.x + ((c.0.x - a.0.x) * (y - a.0.y)) / total_height;  
            let u_ac = a.1.0 + ((c.1.0 - a.1.0) * (y - a.0.y)) / total_height;
            let v_ac = a.1.1 + ((c.1.1 - a.1.1) * (y - a.0.y)) / total_height;
            
            // b -> c line
            let x_bc = b.0.x + ((c.0.x - b.0.x) * (y - b.0.y)) / segment_height;  
            let u_bc = b.1.0 + ((c.1.0 - b.1.0) * (y - b.0.y)) / segment_height;
            let v_bc = b.1.1 + ((c.1.1 - b.1.1) * (y - b.0.y)) / segment_height;
        
            
            // Determine min and max
            let (x_min, x_max, u_min, u_max, v_min, v_max) = if x_ac < x_bc {
                (x_ac, x_bc, u_ac as u32, u_bc as u32, v_ac as u32, v_bc as u32)
            }
            else {
                (x_bc, x_ac, u_bc as u32, u_ac as u32, v_bc as u32, v_ac as u32)
            };
            
            // To navigate texture, calculate gradient of U and V lines with respect to x
            // Since integer division, use bit shifting to account for errors
            
            let dx = (x_max - x_min) as i32;

            let du_dx = if dx > 0 { ((u_max - u_min) << 16) as i32 / dx } else { 0 };
            let dv_dx = if dx > 0 { ((v_max - v_min) << 16) as i32 / dx } else { 0 };
            
            // UV trackers, incremented by d/dx, with shifting taken into account for correction
            let mut u_cur = u_min << 16;
            let mut v_cur = v_min << 16;

            // Clamp x to row buffer border
            let x_start = x_min.max(x1);
            let x_end   = x_max.min(x2);

            let px_y = y - y_offset as i16;
            let px_offset = DISPLAY_BUFFER_WIDTH as i16 * px_y;

            // Perform draw
            for x in x_start..x_end {
                // Shift UV back to get actual coordinates
                let tex_x = (u_cur >> 16) as usize;
                let tex_y = (v_cur >> 16) as usize;
                
                // Clamp UV to image size
                let tex_x = tex_x.min(image.width  as usize - 1);
                let tex_y = tex_y.min(image.height as usize - 1);
                
                // Texture data is 1D, and each px is two bytes
                let tex_index = (tex_x + (image.width as usize * tex_y)) * 2;

                // x does not change, but y is offsetted back.
                let px_x = x;

                // Buffer is 1D, and each px is two bytes.
                let index: usize = ((px_x + px_offset) * 2) as usize;

                buffer[index]     = texture[tex_index];
                buffer[index + 1] = texture[tex_index + 1];

                // Increment current UV
                u_cur += du_dx as u32;
                v_cur += dv_dx as u32;
            }
        }
    }
}

/*
2D Coordinate struct
*/
#[derive(Copy, Clone, Debug)]
pub struct Coord2D {
    pub x: i16,
    pub y: i16,
}

impl Coord2D {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

/*
3D Coordinate struct
*/
pub struct Coord3D {
    pub x: i16,
    pub y: i16,
    pub z: i16,
} 

impl Coord3D {
    pub fn new(x: i16, y: i16, z: i16) -> Self {
        Self { x, y, z }
    }
}

/*
Private 2D vertex struct.
Contains UV data.
*/
#[derive(Clone, Copy, Debug)]
struct Vertex2D {
    pub x: f32,
    pub y: f32,

    pub u: f32, 
    pub v: f32,
}

impl Vertex2D {
    pub fn new(x: f32, y: f32, u: f32, v: f32) -> Self {
        Self { x, y, u, v }
    } 
}

/*
Image container struct.
Stores references to data, along with metadata
*/
#[derive(Debug)]
struct Image<'a> {
    pub width:  u16,
    pub height: u16,
    pub compression: u8,
    
    pub meta: &'a [u8], // Metadata for compression and such.
    pub data: &'a [u8], // Essentially, a reference to the image data in flash.

}

impl<'a> Image<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        
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
        
        let meta: &'a [u8] = &data[9..i];

        // i has start index of VIMG, so add 4
        i += 4;
        let raw_data: &'a [u8] = &data[i..];

        Self {
            width,
            height,
            compression,

            meta,
            data: raw_data,
        }
    }
}

/*
Texture enum for deciding between colour and loaded image.
*/
#[derive(Debug)]
enum Texture<'a> {
    Colour(RGB, RGB),
    Image(Image<'a>), // Unsure what values to attach right now.
}

/*
Ngon shape. A uniform shape with N vertices (Coord2D)
*/
pub const N_MAX: u16 = 64 + 1;
#[derive(Debug)]
pub struct Ngon<'a> {
    center: Coord2D,
    radius: u16,
    rotation: usize,
    
    // Used for tracking orbits about a pivot, for rotate_around
    orbit_pivot: Option<Coord2D>, // Option since does not always apply, also used as check
    orbit_rotation: usize,         

    // To save storage, only include vertices and order them, rather than indices as well.
    vertices: Vec<Vertex2D, { N_MAX as usize }>,

    matrix: Matrix,
    
    texture: Option<Texture<'a>>,
}

impl<'a> Ngon<'a> {
    pub fn new(n: u16, center: Coord2D, radius: u16) -> Self {
        // Impose a maximum Ngon n size
        assert!(n < N_MAX);
        
        let mut vertices: Vec<Vertex2D, { N_MAX as usize }> = Vec::new(); 
        
        // Store vertices as floats for base, before matrix multiplications.
        for i in 0..n {
            let angle: usize = (ANGLE_360 * i as usize) / n as usize;

            let cur_x = cos_mult(angle, 4096) as f32 / 4096.0;
            let cur_y = sin_mult(angle, 4096) as f32 / 4096.0;

            let _ = vertices.push(Vertex2D::new(cur_x, cur_y, 0.0, 0.0));
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
            texture: None, 
        }
    }

    pub fn with_colour(self, fill: RGB, outline: RGB) -> Self {
        Self {
            texture: Some(Texture::Colour(fill, outline)),
            ..self
        }
    }

    pub fn with_texture(mut self, data: &'a [u8]) -> Self {
        // Wrap in Image and calculate UV coordinates
        let image = Image::new(data);
        
        for vertex in self.vertices.iter_mut() {
            // Base on x coordinates, already mapped to [-1, 1]
            // So map to [0, 1]
            vertex.u = (vertex.x + 1.0) * 0.5;
            vertex.v = (vertex.y + 1.0) * 0.5;
        }

        Self {
            texture: Some(Texture::Image(image)),
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

impl<'a> Sprite for Ngon<'a> {
    fn gen_vertices(&self, cache: &mut [(i16, i16, u16, u16); SPRITE_CACHE_MAX], setup_matrix: &Matrix) {
        let m = Matrix::mult_matrices(&setup_matrix, &self.matrix); 
        let mut i = 0;
        
        // Let the first vertex be the center of the screen
        let (cx, cy) = m.apply_to_vertex_2d(0.0, 0.0);

        let mut u: u16 = 0;
        let mut v: u16 = 0;
        if let Some(Texture::Image(image)) = &self.texture {
            u = image.width  as u16 / 2;
            v = image.height as u16 / 2; 
        }

        let c = (cx, cy, u, v);
        cache[0] = c;
        i += 1;
        
        // rest of vertices
        for vertex in &self.vertices {
            let (x, y) = m.apply_to_vertex_2d(vertex.x, vertex.y);
            
            let mut u: u16 = 0;
            let mut v: u16 = 0;

            if let Some(Texture::Image(image)) = &self.texture {
                // Mapped to [0.0, 1.0], so return to ([0, IMAGE_WIDTH], [0, IMAGE_HEIGHT])
                u = (vertex.u * image.width  as f32) as u16; 
                v = (vertex.v * image.height as f32) as u16; 
            }
            
            cache[i] = (x, y, u, v);
            i += 1;
        }
    }

    fn draw(&self, vertices: &[(i16, i16, u16, u16); SPRITE_CACHE_MAX], buffer: &mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16) {
        // let m = Matrix::mult_matrices(&setup_matrix, &self.matrix); 
        
        let c = vertices[0];
        let n = self.vertices.len();
        
        // Draw, according to texture
        match &self.texture {
            &Some(Texture::Colour(fill, outline)) => {
                // RGB565 means each px is two bytes.
                let outline = RawU16::from(outline).into_inner().to_be_bytes();
                let fill    = RawU16::from(fill)   .into_inner().to_be_bytes();

                // Fill draw, with scanline rasterization
                for i in 1..n {
                    // Grab current two points and center (3 points for triangle)
                    let a = vertices[i];
                    let b = vertices[i + 1];
                    triangle_fill((a.0, a.1), (b.0, b.1), (c.0, c.1), &fill, buffer, y_offset);
                }

                let a = vertices[n];
                let b = vertices[1];
                triangle_fill((a.0, a.1), (b.0, b.1), (c.0, c.1), &fill, buffer, y_offset);
                
                // Outline draw with Bresenham's algorithm
                for i in 1..n {
                    // Grab current two vertices (with loop around)
                    let p1 = vertices[i];
                    let p2 = vertices[i + 1];
                    line_draw((p1.0, p1.1), (p2.0, p2.1), &outline, buffer, y_offset);
                }

                let p1 = vertices[n];
                let p2 = vertices[1];
                line_draw((p1.0, p1.1), (p2.0, p2.1), &outline, buffer, y_offset);
            },

            Some(Texture::Image(image)) => {
                // Fill draw, with scanline texture rasterization
                for i in 1..n {
                    // Grab current two points and center (3 points for triangle)
                    let a = vertices[i];
                    let b = vertices[i + 1];
                    triangle_texture(a, b, c, &image, buffer, y_offset);
                }

                let a = vertices[n];
                let b = vertices[1];
                triangle_texture(a, b, c, &image, buffer, y_offset);
            },
            _ => panic!("Ngon texture not defined properly")
        }
    }
}

/*
Triangle shape.
*/
pub struct Triangle<'a> {
    // Change to floating point to apply matrix transformations to 
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),

    // Store the current center of the triangle, for easy manipulation
    centroid: (f32, f32),
    
    matrix: Matrix,

    texture: Option<Texture<'a>>,
}

impl<'a> Triangle<'a> {
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

        Self { p1, p2, p3, centroid, matrix, texture: None}
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
            y: 0 - radius as i16,
        };

        let p1 = Coord2D {
            x: original.x + center.x,
            y: original.y + center.y,
        };

        let original = Coord2D {
            x: ((original.x as f32 * -0.5)  - (original.y as f32 * 0.866)) as i16,
            y: ((original.x as f32 * 0.866) + (original.y as f32 * -0.5))  as i16,
        };

        let p2 = Coord2D {
            x: original.x + center.x,
            y: original.y + center.y,
        };

        let original = Coord2D {
            x: ((original.x as f32 * -0.5)  - (original.y as f32 * 0.866)) as i16,
            y: ((original.x as f32 * 0.866) + (original.y as f32 * -0.5))  as i16,
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

        Self { p1, p2, p3, centroid, matrix, texture: None }
    }
    
    pub fn with_colour(self, fill: RGB, outline: RGB) -> Self {
        Self {
            texture: Some(Texture::Colour(fill, outline)),
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

impl<'a> Sprite for Triangle<'a> {
    fn gen_vertices(&self, cache: &mut [(i16, i16, u16, u16); SPRITE_CACHE_MAX], setup_matrix: &Matrix) {
        let m = Matrix::mult_matrices(&setup_matrix, &self.matrix); 
        
        let (x, y) = m.apply_to_vertex_2d(self.p1.0, self.p1.1);
        let p1 = Coord2D::new(x, y);
        
        let (x, y) = m.apply_to_vertex_2d(self.p2.0, self.p2.1);
        let p2 = Coord2D::new(x, y);

        let (x, y) = m.apply_to_vertex_2d(self.p3.0, self.p3.1);
        let p3 = Coord2D::new(x, y);
        
        cache[0] = (p1.x, p1.y, 0, 0);
        cache[1] = (p2.x, p2.y, 0, 0);
        cache[2] = (p3.x, p3.y, 0, 0);
    }

    fn draw(&self, vertices: &[(i16, i16, u16, u16); SPRITE_CACHE_MAX], buffer: & mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16) {
        // Draw, according to texture
        
        let p1 = vertices[0];
        let p2 = vertices[1];
        let p3 = vertices[2];

        match self.texture {
            Some(Texture::Colour(fill, outline)) => {
                // RGB565 means each px is two bytes.
                let outline = RawU16::from(outline).into_inner().to_be_bytes();
                let fill    = RawU16::from(fill)   .into_inner().to_be_bytes();

                let p1 = (p1.0, p1.1);
                let p2 = (p2.0, p2.1);
                let p3 = (p3.0, p3.1);

                triangle_fill(p1, p2, p3, &fill, buffer, y_offset);

                line_draw(p1, p2, &outline, buffer, y_offset);
                line_draw(p2, p3, &outline, buffer, y_offset);
                line_draw(p3, p1, &outline, buffer, y_offset);
            },
            _ => panic!("Triangle texture not defined properly")
        }
    }
}


/*
Quad(rilateral) sprite.
*/
pub struct Quad<'a> {
    // Change to floating point to apply matrix transformations to 
    p1: Vertex2D,
    p2: Vertex2D,
    p3: Vertex2D,
    p4: Vertex2D,

    // Store the current center of the quad, for easy manipulation
    centroid: (f32, f32),

    width: f32,
    height: f32,
    
    matrix: Matrix,
    
    texture: Option<Texture<'a>>,
}

impl<'a> Quad<'a> {
    pub fn new(center: Coord2D, width: usize, height: usize) -> Self {
        // Normalize points, such that the centroid is (0, 0)
        let centroid: (f32, f32) = (
            center.x as f32,
            center.y as f32,
        );

        // Calculate where each point should be, in a regular rectangle-like fashion
        let width: f32 =  width as f32;
        let height: f32 = height as f32;

        // Center arround (0, 0) with side length 1
        //
        // p4      p2
        //
        // p3      p1
        
        //let p1: (f32, f32) = (0.5 ,  0.5);
        //let p2: (f32, f32) = (0.5 , -0.5);
        //let p3: (f32, f32) = (-0.5,  0.5);
        //let p4: (f32, f32) = (-0.5, -0.5);

        let p1: Vertex2D = Vertex2D::new(0.5 ,  0.5, 1.0, 1.0);
        let p2: Vertex2D = Vertex2D::new(0.5 , -0.5, 1.0, 0.0);
        let p3: Vertex2D = Vertex2D::new(-0.5,  0.5, 0.0, 1.0);
        let p4: Vertex2D = Vertex2D::new(-0.5, -0.5, 0.0, 0.0);

        // Set up matrix
        let mut matrix: Matrix = Matrix::identity();
        matrix.translate(centroid.0, centroid.1, 0.0);
        matrix.scale(width, height, 1.0);

        Self { p1, p2, p3, p4, centroid, width, height, matrix, texture: None }
    }

    pub fn new_from_corners(p1: Coord2D, p2: Coord2D) -> Self {
        let width:  f32 = (p1.x - p2.x).abs() as f32;
        let height: f32 = (p1.y - p2.y).abs() as f32;

        let centroid: (f32, f32) = (
            (p1.x + p2.x) as f32 / 2.0,
            (p1.y + p2.y) as f32 / 2.0,
        );

        // Center arround (0, 0) with side length 1
        // let p1: (f32, f32) = (0.5 ,  0.5);
        // let p2: (f32, f32) = (0.5 , -0.5);
        // let p3: (f32, f32) = (-0.5,  0.5);
        // let p4: (f32, f32) = (-0.5, -0.5);

        let p1: Vertex2D = Vertex2D::new(0.5 ,  0.5, 1.0, 1.0);
        let p2: Vertex2D = Vertex2D::new(0.5 , -0.5, 1.0, 0.0);
        let p3: Vertex2D = Vertex2D::new(-0.5,  0.5, 0.0, 1.0);
        let p4: Vertex2D = Vertex2D::new(-0.5, -0.5, 0.0, 0.0);

        // Set up matrix
        let mut matrix: Matrix = Matrix::identity();
        matrix.translate(centroid.0, centroid.1, 0.0);
        matrix.scale(width, height, 1.0);

        Self { p1, p2, p3, p4, centroid, width, height, matrix, texture: None }
    }
    
    pub fn with_colour(self, fill: RGB, outline: RGB) -> Self {
        Self {
            texture: Some(Texture::Colour(fill, outline)),
            ..self
        }
    }

    pub fn with_texture(self, data: &'a [u8]) -> Self {
        let image = Image::new(data);
        Self {
            texture: Some(Texture::Image(image)),
            ..self
        }
    }

}

impl<'a> Sprite for Quad<'a> {
    fn gen_vertices(&self, cache: &mut [(i16, i16, u16, u16); SPRITE_CACHE_MAX], setup_matrix: &Matrix) {
        let m = Matrix::mult_matrices(&setup_matrix, &self.matrix); 
        
        let (x, y) = m.apply_to_vertex_2d(self.p1.x, self.p1.y);
        let mut p1 = (x, y, 0 as u16, 0 as u16);
        
        let (x, y) = m.apply_to_vertex_2d(self.p2.x, self.p2.y);
        let mut p2 = (x, y, 0 as u16, 0 as u16);

        let (x, y) = m.apply_to_vertex_2d(self.p3.x, self.p3.y);
        let mut p3 = (x, y, 0 as u16, 0 as u16);
        
        let (x, y) = m.apply_to_vertex_2d(self.p4.x, self.p4.y);
        let mut p4 = (x, y, 0 as u16, 0 as u16);

        if let Some(Texture::Image(image)) = &self.texture {
            p1.2 = (self.p1.u * image.width as f32)  as u16;
            p1.3 = (self.p1.v * image.height as f32) as u16;
            p2.2 = (self.p2.u * image.width as f32)  as u16;
            p2.3 = (self.p2.v * image.height as f32) as u16;
            p3.2 = (self.p3.u * image.width as f32)  as u16;
            p3.3 = (self.p3.v * image.height as f32) as u16;
            p4.2 = (self.p4.u * image.width as f32)  as u16;
            p4.3 = (self.p4.v * image.height as f32) as u16;
        }
        
        cache[0] = p1;
        cache[1] = p2;
        cache[2] = p3;
        cache[3] = p4;
    }

    fn draw(&self, vertices: &[(i16, i16, u16, u16); SPRITE_CACHE_MAX], buffer: &mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16) {
        
        let p1 = vertices[0];
        let p2 = vertices[1];
        let p3 = vertices[2];
        let p4 = vertices[3];

        // Draw, according to texture
        match &self.texture {
            &Some(Texture::Colour(fill, outline)) => {
                // RGB565 means each px is two bytes.
                let outline = RawU16::from(outline).into_inner().to_be_bytes();
                let fill    = RawU16::from(fill)   .into_inner().to_be_bytes();

                let p1 = (p1.0, p1.1);
                let p2 = (p2.0, p2.1);
                let p3 = (p3.0, p3.1);
                let p4 = (p4.0, p4.1);

                triangle_fill(p1, p2, p3, &fill, buffer, y_offset);
                triangle_fill(p4, p2, p3, &fill, buffer, y_offset);

                line_draw(p1, p2, &outline, buffer, y_offset);
                line_draw(p2, p4, &outline, buffer, y_offset);
                line_draw(p4, p3, &outline, buffer, y_offset);
                line_draw(p3, p1, &outline, buffer, y_offset);
            },

            Some(Texture::Image(image)) => {
                triangle_texture(p1, p2, p3, &image, buffer, y_offset);
                triangle_texture(p4, p2, p3, &image, buffer, y_offset);
            },
            _ => panic!("Quad texture not defined properly")
        }
    }
}

pub struct Font<'a> {
    fontsheet: Image<'a>,
    glyph_width: u16,
    glyph_height: u16,
    characters: [(u16, u16); 95],
}

impl<'a> Font<'a> {
    pub fn new(fontsheet: &'a [u8]) -> Self {
        // VFONT adds a header to a VIMG
        // Structure:
        // VFONT (5 bytes, guard)
        // glyph width  (2 bytes)
        // glyph height (2 bytes)
        // glyphs per column (2 bytes)
        // glyphs per row    (2 bytes)
        // VFONT (5 bytes, guard)
        
        // Read the data for the header and extract information from it.
        
        // Find 'VFONT' guard
        let guard = &fontsheet[0..=4];
        if !(guard == b"VFONT") {
            panic!("Invalid VFONT, starting guard not present");
        }

        // Width and height store in 2 bytes each.
        let width  = u16::from_be_bytes([fontsheet[5], fontsheet[6]]);
        let height = u16::from_be_bytes([fontsheet[7], fontsheet[8]]);
        
        // Glyphs per row/column
        let glyphs_per_col = u16::from_be_bytes([fontsheet[9], fontsheet[10]]);
        let glyphs_per_row = u16::from_be_bytes([fontsheet[11], fontsheet[12]]);
        
        // Ensure final guard present
        let guard = &fontsheet[13..=17];
        if !(guard == b"VFONT") {
            panic!("Invalid VFONT, ending guard not present");
        }
        
        // We can wrap the rest of the VFONT in an Image, since it is just a VIMG
        let fontsheet = Image::new(&fontsheet[18..]);
        
        // Construct UV values for each font
        let mut characters = [(0 as u16, 0 as u16); 95];
        let mut i = 0;
        for x in 0..glyphs_per_col {
            for y in 0..glyphs_per_row {
               let u: u16 = (x * width).into(); 
               let v: u16 = (y * height).into(); 
               characters[i] = (u, v);
               i += 1;
            
                if i >= 95 { break; }
            }

            if i >= 95 { break; }
        }


        Self {
            fontsheet,
            glyph_width: width,
            glyph_height: height,
            characters,
        }
    }

    pub fn get_glyph_dimensions(&self) -> (u16, u16) {
        (self.glyph_width, self.glyph_height)
    }
    
    pub fn get_fontsheet(&self) ->  &Image<'a> {
        &self.fontsheet
    }

    pub fn get_character(&self, c: u8) -> Option<(u16, u16)> {
        if c > 95 {
            return None;
        }

        Some(self.characters[c as usize])
    }
}



pub struct Text<'a> {
    pub raw: String<16>,

    font: &'a Font<'a>,
    size: f32,
    colour: RGB,
    position: Coord2D,
    rotation: usize,
    x_spacing: u8,
    y_spacing: u8,
    matrix: Matrix,
}

impl<'a> Text<'a> {
    pub fn new(font: &'a Font<'a>, position: Coord2D) -> Self {
        let matrix = Matrix::identity();

        Self {
            raw: String::new(),
            font,
            size: 1.0,
            colour: RGB::BLACK,
            position,
            rotation: 0,
            x_spacing: 0,
            y_spacing: 0,
            matrix
        }
    }
    
    // Updates matrix with stored information from scratch, prevents floating point math error 
    pub fn update_matrix(&mut self) {
        let mut m = Matrix::identity();
        
        // Perform Translate Rotate Scale update
        m.rotate(self.rotation, Axis::Z);
        m.scale(self.size as f32, self.size as f32, 1.0);

        self.matrix = m;
    }

    pub fn with_size(self, size: f32) -> Self {
        let mut new = Self {
            size,
            ..self
        };

        new.update_matrix();

        new
    }
    
    pub fn set_size(&mut self, size: f32) {
        self.size = size;
        self.update_matrix();
    }
    
    pub fn with_colour(self, colour: RGB) -> Self {
        let mut new = Self {
            colour,
            ..self
        };

        new
    }
    
    pub fn set_colour(&mut self, colour: RGB) {
        self.colour = colour;
    }

    pub fn set_str(&mut self, string: &str) {
        self.raw.clear();
        let _ = self.raw.push_str(string);
    }
    
    pub fn set_fmt(&mut self, args: Arguments<'a>) {
        self.raw.clear();
        let _ = self.raw.write_fmt(args);
    }
}

#[macro_export]
macro_rules! text_set {
    ($target:expr, $($arg:tt)*) => {
        $target.set_fmt(format_args!($($arg)*))
    };
}

impl<'a> Sprite for Text<'a> {
    fn gen_vertices(&self, cache: &mut [(i16, i16, u16, u16); SPRITE_CACHE_MAX as usize], setup_matrix: &Matrix) {
        let m = Matrix::mult_matrices(&setup_matrix, &self.matrix); 
        
        // We fill our cache with (x,y,u,v) for each character, based on its position and font
        // Newlines represent a change in y
        
        let mut i: usize = 0;
        let mut cur_coord = self.position;
        let (glyph_width, glyph_height) = self.font.get_glyph_dimensions();
        for c in self.raw.chars() {
            // Pass c to font to get UV values
            let uv = self.font.get_character(c as u8);
            
            if let Some((u, v)) = uv {
                // Transform XY, and store to cache
                let (x, y) = m.apply_to_vertex_2d(cur_coord.x as f32, cur_coord.y as f32);
                cache[i] = (x, y, u, v);
                i += 1;
                
                let (x, y) = m.apply_to_vertex_2d((cur_coord.x + glyph_width as i16) as f32, cur_coord.y as f32);
                cache[i] = (x, y, u + glyph_width, v);
                i += 1;
                
                let (x, y) = m.apply_to_vertex_2d(cur_coord.x as f32, (cur_coord.y + glyph_height as i16) as f32);
                cache[i] = (x, y, u, v + glyph_height);
                i += 1;

                let (x, y) = m.apply_to_vertex_2d((cur_coord.x + glyph_width as i16) as f32, (cur_coord.y + glyph_height as i16) as f32);
                cache[i] = (x, y, u + glyph_width, v + glyph_height);
                i += 1;
            }

            // Increment XY as necessary
            match c {
                '\n' => {
                    cur_coord.x = self.position.x;
                    cur_coord.y += (glyph_height + self.y_spacing as u16) as i16;
                },
                _ => {
                    cur_coord.x += (glyph_width + self.x_spacing as u16) as i16;
                },
            }
        }

    }

    fn draw(&self, vertices: &[(i16, i16, u16, u16); SPRITE_CACHE_MAX], buffer: & mut [u8; DISPLAY_BUFFER_SIZE], y_offset: u16) {
        // Loop per character in the string
        // Should use enumerate to get character and index to check if newline
        let mut index: usize = 0;
        for (i, c) in self.raw.chars().enumerate() {
            if c == '\n' || c == ' ' { continue; }

            let p1 = vertices[(index * 4) + 0];
            let p2 = vertices[(index * 4) + 1];
            let p3 = vertices[(index * 4) + 2];
            let p4 = vertices[(index * 4) + 3];

            triangle_texture(p1, p2, p3, self.font.get_fontsheet(), buffer, y_offset);
            triangle_texture(p4, p2, p3, self.font.get_fontsheet(), buffer, y_offset);
            
            index += 1;
        }
    }
}
