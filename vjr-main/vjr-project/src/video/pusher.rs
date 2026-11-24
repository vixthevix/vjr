
/*
Trait for interracting with a row buffer to draw pixels.
*/

//use core::future::Future;

#[allow(async_fn_in_trait)]
pub trait Pusher {
    fn prepare(&mut self);
    async fn push(&mut self, buffer: &[u8]);
    //fn push(&mut self, buffer: &[u8]) -> impl Future<Output = ()> + Send;
    fn finish(&mut self);
}
