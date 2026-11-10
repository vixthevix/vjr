/*
Base implementation of Matrices.
4x4 for standard 2D + 3D
*/

use crate::trig::sine::{ sin };
use crate::trig::cosine::{ cos };

/*
Enum for specifying axis to rotate about.
*/
#[derive(Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug)]
pub struct Matrix {
    m:  [f32; 16],
}

impl Matrix {

    pub fn new(m: [f32; 16]) -> Self {
        Self { m }
    }

    pub fn identity() -> Self {
        Self {
            m: [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
            ]
        }
    }

    pub fn mult_matrices(m1: &Self, m2: &Self) -> Self {
        let mut out = Self::identity();
        
        for r in 0..4 {
            for c in 0..4 {
                out.m[r * 4 + c] =
                    // Multiplyng and adding column by row
                    m1.m[0 * 4 + c] * m2.m[r * 4 + 0] +
                    m1.m[1 * 4 + c] * m2.m[r * 4 + 1] +
                    m1.m[2 * 4 + c] * m2.m[r * 4 + 2] +
                    m1.m[3 * 4 + c] * m2.m[r * 4 + 3];
            }
        }

        out
    }

    pub fn mult_by_matrix(&mut self, other: &Self) {
        *self = Self::mult_matrices(self, other);
    }

    pub fn mult_by_scalar(&mut self, k: f32) {
        for val in self.m.iter_mut() {
            *val *= k;
        }
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        let mut t = Self::identity();
        

        t.m[0]  = x;
        t.m[5]  = y;
        t.m[10] = z;
    
        self.mult_by_matrix(&t);
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        let mut t = Self::identity();
        
        t.m[12] = x;
        t.m[13] = y;
        t.m[14] = z;
    
        self.mult_by_matrix(&t);
    }
    
    pub fn rotate(&mut self, angle: usize, axis: Axis) {
        let mut t = Self::identity();
        
        // Set up angles properly
        let cosine = cos(angle) as f32 / 4096.0;
        let sine = sin(angle) as f32 / 4096.0;
        
        match axis {
            Axis::X => {
                t.m[5]  = cosine;
                t.m[6]  = -sine;
                t.m[9]  = sine;
                t.m[10] = cosine;
            },
            Axis::Y => {
                t.m[0]  = cosine;
                t.m[8]  = sine;
                t.m[2]  = -sine;
                t.m[10] = cosine;
            },
            Axis::Z => {
                t.m[0]  = cosine;
                t.m[1]  = -sine;
                t.m[4]  = sine;
                t.m[5] = cosine;
            }
        }

        self.mult_by_matrix(&t);
    }

    // Method for applying self to a vertex
    pub fn apply_to_vertex(&self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        let w = 1.0; // Default
        
        // Apply matrix to each point
        let out_x = self.m[0]*x + self.m[4]*y + self.m[8]*z  + self.m[12]*w;
        let out_y = self.m[1]*x + self.m[5]*y + self.m[9]*z  + self.m[13]*w;
        let out_z = self.m[2]*x + self.m[6]*y + self.m[10]*z + self.m[14]*w;
        let out_w = self.m[3]*x + self.m[7]*y + self.m[11]*z + self.m[15]*w;

        // Perform perspective divide
        if out_w != 0.0 && out_w != 1.0 {
            (out_x / out_w, out_y / out_w, out_z / out_w)
        }
        else {
            (out_x, out_y, out_z)
        }

    }
}

