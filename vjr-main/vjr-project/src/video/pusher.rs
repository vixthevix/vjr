


use crate::video::renderer::DISPLAY_BUFFER_SIZE;

pub trait Pusher {
    fn push(&mut self, buffer: &[u8; DISPLAY_BUFFER_SIZE], y_offset: u16);
}
