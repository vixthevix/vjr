
use embedded_graphics_core::{
    pixelcolor::{
        Rgb565 as RGB // We are only really dealing with rgb565
    },
};



use heapless::Vec;

use crate::{
    video::sprite::*,
    matrix::projection::*,
};

/*
Struct for holding pointers to Sprites for easy display. 
*/
const SCENE_SPRITE_MAX: u16 = 64;
pub struct Scene {
    pub sprites: Vec<*mut dyn Sprite, { SCENE_SPRITE_MAX as usize }>,
    pub bg_colour: RGB,
    pub projection: Projection,
}

impl Scene {
    pub fn new(bg_colour: RGB, projection: Projection) -> Self {
        Self {
            sprites: Vec::new(),
            bg_colour,
            projection,
        }
    }
    
    /*
    Adds a Sprite to the Scene, and returns a handle (index) to that Sprite.
    */
    pub fn add(&mut self, sprite: *mut dyn Sprite) -> usize {
        let _ = self.sprites.push(sprite);
        
        self.sprites.len() - 1
    }
    
    pub fn add_at(&mut self, sprite: &'static mut dyn Sprite, index: usize) -> usize {
        let _ = self.sprites.insert(index, sprite as *mut dyn Sprite);

        index
    }

    pub fn pop(&mut self) -> u16 {
        self.sprites.pop();

        (self.sprites.len() - 1) as u16
    }
    
    pub fn pop_at(&mut self, index: usize) -> usize {
        self.sprites.remove(index);

        index
    }
    
    // Swap two Sprites around in a scene, changing drawing order.
    pub fn swap(&mut self, a: usize, b: usize) -> (usize, usize) {
        let temp = self.sprites[a];
        self.sprites[a] = self.sprites[b];
        self.sprites[b] = temp;

        (b, a)
    }

    pub fn shift_down(&mut self, index: usize, count: usize) -> usize {
        
        let mut new_index = index;

        for _ in 0..count {
            (new_index, _) = self.swap(new_index, new_index - 1);
        }

        new_index
    }
    
    pub fn shift_up(&mut self, index: usize, count: usize) -> usize {
        
        let mut new_index = index;

        for _ in 0..count {
            (new_index, _) = self.swap(new_index, new_index + 1);
        }

        new_index
    }

    pub fn sprite_count(&self) -> usize {
        self.sprites.len() as usize
    }
}
