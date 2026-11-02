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
        
        let mut row_offset: usize = 0;
        
        for row in 0..4 {
            row_offset = row * 4;
            for column in 0..4 {
                out.set(
                    row_offset + column,
                    
                    (m1.get(row_offset + 0) * m2.get(column + 0)) +
                    (m1.get(row_offset + 1) * m2.get(column + 4)) +
                    (m1.get(row_offset + 2) * m2.get(column + 8)) +
                    (m1.get(row_offset + 3) * m2.get(column + 12))
                );
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
}

