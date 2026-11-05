
/*
Trait for interracting with a row buffer to draw pixels.
*/
pub trait Pusher {
    fn push(&mut self, buffer: &[u8], y_offset: u16);
}
