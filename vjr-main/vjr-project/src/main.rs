#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer, Ticker, Instant};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use log::info;

use esp_hal::gpio::Output;

use embassy_sync::{
    channel::{
        Channel,
        Receiver,
        Sender
    },
    blocking_mutex::raw::NoopRawMutex
};

use embassy_futures::*;

use static_cell::StaticCell;

// vjr modules
pub mod video;
pub mod trig;
pub mod matrix;

// vjr crates
use crate::{
    video::{
        display::*,
        sprite::*,
        renderer::*,
        pusher::*,
        scene::*,
    },
    matrix::{
        prelude::*,
        camera::*,
        projection::*,
    },
    trig::{
        sine::*,
    },
};



extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

/*
Example of wrapper for sending messages between async tasks.
*/
//enum ChannelMessage {
//    Even(u16),
//    Odd(u16),
//}

/*
Examples of async tasks, utilising wrapper messages
*/

//#[embassy_executor::task]
//async fn tx_even_task(tx: Sender<'static, NoopRawMutex, ChannelMessage, 5>) -> ! {
//    let mut count: u16 = 0;
//    loop {
//        tx.send(ChannelMessage::Even(count)).await;
//        Timer::after(Duration::from_millis(500)).await;
//        count += 2;
//    }
//}
//#[embassy_executor::task]
//async fn tx_odd_task(tx: Sender<'static, NoopRawMutex, ChannelMessage, 5>) -> ! {
//    let mut count: u16 = 1;
//    loop {
//        tx.send(ChannelMessage::Odd(count)).await;
//        Timer::after(Duration::from_millis(1000)).await;
//        count += 2;
//    }
//}
//
//#[embassy_executor::task]
//async fn rx_task(rx: Receiver<'static, NoopRawMutex, ChannelMessage, 5>) -> ! {
//    loop {
//        let msg = rx.receive().await;
//        match msg {
//            ChannelMessage::Even(n) => info!("Even received: {n}"),
//            ChannelMessage::Odd(n)  => info!("Odd received: {n}"),
//        }
//    }
//}


// Specific video buffer pusher for Home menu
struct HomePusher<'a> {
    display: *mut Display<'a>,
}

impl<'a> Pusher for HomePusher<'a> {
    fn prepare(&mut self) {
        unsafe {
            (*self.display).write_prepare();
        }
    }

    // async fn push(&mut self, buffer: &[u8], y_offset: u16) {
    async fn push(&mut self, buffer: &[u8]) {
        // Use SPI DMA to bush buffer to display.
        unsafe {
            // let display = self.display;
            // (*display).write_buffer(buffer, y_offset).await;
            (*self.display).write_buffer(buffer).await;

        }
    }

    fn finish(&mut self) {
        unsafe {
            (*self.display).write_finish();
        }
    }
} 

impl<'a> HomePusher<'a> {
    pub fn new(display: &mut Display<'a>) -> Self {
        Self {
            display: display as *mut Display
        }
    }
}


// Example Memory Pin for async task communication channel.
// static CHANNEL: StaticCell<Channel<NoopRawMutex, ChannelMessage, 5>> = StaticCell::new();

// Static display buffer for command.
display_cmd_buffer_new!(DISPLAY_CMD_BUFFER);

const APPLE_VIMG: &[u8]    = include_bytes!("../images/appl.vimg");
// const GOLDSHIP_VIMG: &[u8] = include_bytes!("../images/goldship_optimized.vimg");
const CSM_VIMG: &[u8] = include_bytes!("../images/csm_movie.vimg");

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // generator version: 1.4.0
    // generator parameters: -o esp32 -o unstable-hal -o embassy -o alloc -o wifi -o log -o neovim-classic -o vscode -o esp-backtrace -o esp
    
    // Initialise USB logger for communication with PC
    esp_println::logger::init_logger_from_env();
    
    // Set up ESP32 pins
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    
    // Allocate a heap (unused)
    // esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);
    
    // Initialise timer interrupt for async task switching
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    info!("Embassy initialized!");
    
    // Set up Wi-Fi (unused)
    //let _wifi_controller =
    //    esp_radio::wifi::WifiController::new(peripherals.WIFI, Default::default())
    //        .expect("Failed to initialize Wi-Fi controller");
    //let _wifi_interface = esp_radio::wifi::Interface::station();
    
    /*
    Example code for setting up async tasks.
    */

    // TODO: Spawn some tasks
    let _ = spawner;
    
    // let _channel = CHANNEL.init(Channel::new());

    // let _rx = _channel.receiver();
    // let _tx = _channel.sender();
    //
    // let _tx_even = _tx.clone();
    // let _tx_odd  = _tx.clone();
    
    //spawner.spawn(tx_even_task(tx_even).unwrap());
    //spawner.spawn(tx_odd_task(tx_odd)  .unwrap());
    //spawner.spawn(rx_task(rx)          .unwrap());
    
    
    ///////////////////////////////////////////////
    // VJR SETUP                                //
    ///////////////////////////////////////////////
    
    // Display setup
    let pinout      = display_pinout_new!(peripherals);
    let cmd_buffer  = display_cmd_buffer_init!(DISPLAY_CMD_BUFFER);
    let dma_channel = peripherals.DMA_SPI3;
    let mut display = Display::new(pinout, dma_channel, cmd_buffer);
    
    // Direct communication with screen
    let pusher = HomePusher::new(&mut display);
    
    let mut renderer = Renderer::new(pusher);
    
    // Construct our projection and camera
    let projection = Projection::new_2D(1.0);
    let camera     = Camera::new(0, 0, -50);
    
    // Setup a scene
    let mut home_scene = Scene::new(Display::rgb(0, 0, 0), projection);
    
    // Setup some shapes
    let mut ngon = Ngon::new(32, Coord2D::new(160, 60), 45)
        .with_colour(
            Display::rgb(220, 20, 60),
            Display::rgb(255, 255, 255)
        );

    let mut triangle = Ngon::new(3, Coord2D::new(40, 30), 20)
        .with_colour(
            Display::rgb(255, 255, 255),
            Display::rgb(220, 20, 60) 
        );
    
    // Proper triangle struct
    let mut lt_moving_coord = Coord2D::new(75, 30);
    let mut long_triangle = Triangle::new(
        lt_moving_coord,
        Coord2D::new(100, 69),
        Coord2D::new(100, 90)
    )
        .with_colour(
            Display::rgb(100, 30, 210),
            Display::rgb(210, 30, 100)
        );
    
    // Proper quad struct
    let mut quad = Quad::new(Coord2D::new(67, 67), 10, 100)
        .with_colour(
            Display::rgb(45, 167, 1),
            Display::rgb(100, 1, 240)
        );
    
    // Proper image displayed
    let mut image = Quad::new(Coord2D::new(200, 200), 67, 67)
        // .with_texture(GOLDSHIP_VIMG);
        .with_texture(APPLE_VIMG);
    
    let mut circle_image = Ngon::new(16, Coord2D::new(100, 100), 50)
        //.with_texture(GOLDSHIP_VIMG);
        .with_texture(CSM_VIMG);

    // Add shapes to scene
    home_scene.add(&mut ngon);
    home_scene.add(&mut triangle);
    home_scene.add(&mut long_triangle);
    home_scene.add(&mut quad);
    home_scene.add(&mut image);
    home_scene.add(&mut circle_image);

    info!("{:#?}", ngon);
    
    // Rotational data
    let angle = ANGLE_90 >> 3; // ~10 degrees
    let counter_angle = ANGLE_360 - (angle >> 2);
    let pivot = Coord2D::new(160, 120);
     
    // Game ticker for 60 FPS
    let mut ticks = Ticker::every(Duration::from_hz(60));
    
    // Used for measuring FPS
    let mut timer = Instant::now();

    // Main loop
    loop {
        loop { // Home menu
            triangle.rotate_by(angle);
            circle_image.rotate_around(pivot, counter_angle);
            circle_image.rotate_by(angle);

            lt_moving_coord.x = (lt_moving_coord.x + 5) % 300;
            long_triangle.move_vertex_to(1, lt_moving_coord);
            
            renderer.draw_scene(&mut home_scene, &camera).await;
            
            let fps = 1e6 as f32 / timer.elapsed().as_micros() as f32;
            timer = Instant::now();
            info!("FPS: {fps}");
             ticks.next().await;
        }
        
//        loop { // Background BIOS
//            
//        }
    
        // Home button pressed signal received, returning to start of loop.
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.2.2/examples
}
