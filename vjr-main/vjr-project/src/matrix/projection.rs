/*
Projection implementation for graphical pipeline
*/

use crate::matrix::prelude::*;
use crate::trig::*;

pub struct Projection {
    matrix: Matrix,
    depth: f32,
}

impl Projection {
    /*
    Constructs a Projection for a 2D display.
    Uses this algorithm: https://en.wikipedia.org/wiki/Orthographic_projection
    */
    pub fn new_2D(depth: f32) -> Self {

        let left: f32 = 0.0;
        let right: f32 = 320.0; // HARDCODED
        let top: f32 = 0.0;
        let bottom: f32 = 240.0; // HARDCODED

        let vnear = depth * -1.0;
        let vfar  = depth;

        let matrix = Matrix::new(
            [
                2.0 / (right - left), 0.0, 0.0, 0.0,
                0.0, 2.0 / (top - bottom), 0.0, 0.0,
                0.0, 0.0, -2.0 / (vfar - vnear), 0.0,

                -(right + left) / (right - left),
                -(top + bottom) / (top - bottom),
                -(vfar + vnear) / (vfar - vnear),
                1.0
            ]
        );
        
        Self {
            matrix,
            depth,
        }
    }

    pub fn get_matrix(&self) -> &Matrix {
        &self.matrix
    }
}
