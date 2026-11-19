
/*
Trait for interracting with a row buffer to draw pixels.
*/

use core::future::Future;

pub trait Pusher {
    fn prepare(&mut self);
    // async fn push(&mut self, buffer: &[u8], y_offset: u16);
    async fn push(&mut self, buffer: &[u8]);
    fn finish(&mut self);
}
