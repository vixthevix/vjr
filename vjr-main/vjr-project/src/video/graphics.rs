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
        // RgbColor, // Trait
        Rgb565 as RGB // We are only really dealing with rgb565
    },
    draw_target::DrawTarget, // Trait
};

/*
Base trait for all sprites that can be drawn to the screen.
*/
pub trait Sprite {
    fn draw(&self);
}

/*
2D Coordinate struct
*/
pub struct Coord2D {
    pub x: isize,
    pub y: isize,
} 

/*
3D Coordinate struct
*/
pub struct Coord3D {
    pub x: isize,
    pub y: isize,
    pub z: isize,
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
    pub width:  usize,
    pub height: usize,
    pub data: &'static [u8], // Essentially, a reference to the image data in flash.
}

impl Image {
    pub fn new(pos: Coord2D, data: &'static [u8]) -> Self {
        
        // Read the data for the header and extract information from it.

        Self {
            pos,
            width: 0,
            height: 0,
            data
        }
    }
}
