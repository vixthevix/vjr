/*
Fast cosine implementation
cos is 90 degrees ahead of sin, so we can use the SINE_TABLE In the same way.
*/

use crate::trig::sine::*;

pub fn cos(index: usize) -> i16 {
    SINE_TABLE[((index + ANGLE_90) & 0x0FFF) as usize]
}

pub fn cos_mult(index: usize, val: i32) -> i32 {
    let x = cos(index);
    let result: i32 = (x as i32 * val) >> 12;
    
    result
}
