/*
Camera implementation for graphical pipeline
*/

use crate::matrix::prelude::*;

pub struct Camera {
    matrix: Matrix,
    x: isize,
    y: isize,
    z: isize,
}

impl Camera {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        let mut matrix = Matrix::identity();
        
        // Set in reverse direction, due to moving pixels in the opposite direction
        matrix.translate(-x as f32, -y as f32, -z as f32);

        Self {
            matrix,
            x,
            y,
            z,
        }
    }

    pub fn move_by(&mut self, x_offset: isize, y_offset: isize, z_offset: isize) {
        self.x += x_offset;
        self.y += y_offset;
        self.z += z_offset;
        self.matrix.translate(-x_offset as f32, -y_offset as f32, -z_offset as f32);
    }

    pub fn move_to(&mut self, x: isize, y: isize, z: isize) {
        self.move_by(self.x - x, self.y - y, self.z - z);
    }
    
    pub fn get_matrix(&self) -> &Matrix {
        &self.matrix
    }
}
