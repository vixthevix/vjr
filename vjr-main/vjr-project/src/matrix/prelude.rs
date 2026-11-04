/*
Base implementation of Matrices.
4x4 for standard 2D + 3D
*/

use crate::trig::sine::{ sin };
use crate::trig::cosine::{ cos };

#[derive(Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug)]
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

    fn set(&mut self, index: usize, value: f32) {
        self.m[index] = value;
    }

    fn get(&self, index: usize) -> f32 {
        self.m[index]
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
        
        t.set(0,  x);
        t.set(5,  y);
        t.set(10, z);
    
        self.mult_by_matrix(&t);
    }

    pub fn translate(&mut self, x: isize, y: isize, z: isize) {
        let mut t = Self::identity();
        
        t.set(12, x as f32);
        t.set(13, y as f32);
        t.set(14, z as f32);
    
        self.mult_by_matrix(&t);
    }
    
    pub fn rotate(&mut self, angle: usize, axis: Axis) {
        let mut t = Self::identity();
        
        // Set up angles properly
        let cosine = cos(angle) as f32 / 4096.0;
        let sine = sin(angle) as f32 / 4096.0;
        
        match axis {
            Axis::X => {
                t.set(5, cosine);
                t.set(6, -sine);
                t.set(9, sine);
                t.set(10, cosine);
            },
            Axis::Y => {
                t.set(0, cosine);
                t.set(8, sine);
                t.set(2, -sine);
                t.set(10, cosine);
            },
            Axis::Z => {
                t.set(0, cosine);
                t.set(1, -sine);
                t.set(4, sine);
                t.set(5, cosine);
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

