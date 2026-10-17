#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use log::info;

use embassy_sync::{
    channel::{
        Channel,
        Receiver,
        Sender
    },
    blocking_mutex::raw::NoopRawMutex
};

use static_cell::StaticCell;

use esp_hal::gpio::{
    Level,
    Output,
    OutputConfig,
};

use embedded_hal_bus::spi::ExclusiveDevice;

use esp_hal::{
    delay::Delay,
    time::Rate,
    spi::{
        master::{
            Config as SpiConfig,
            Spi,
        },
        Mode as SpiMode,
    },
};

use mipidsi::{
    interface::SpiInterface,
    options::{
        ColorInversion, // Corrects colour
        ColorOrder,     // RGB <-> BGR
    }, 
    Builder,
    models::ILI9341Rgb565 // Fairly sure this is the right one
};

use embedded_graphics_core::{
    pixelcolor::{
        // RgbColor, // Trait
        Rgb565 as RGB // We are only really dealing with rgb565
    },
    draw_target::DrawTarget, // Trait
};

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

/*
Wrapper for messages sent across channels.
This is standard for embedded channels.
*/
enum ChannelMessage {
    Even(u16),
    Odd(u16),
}

#[embassy_executor::task]
async fn print_task() {
    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task] 
async fn led_task(mut led: esp_hal::gpio::Output<'static>) -> ! {
    loop {
        led.toggle();
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn tx_even_task(tx: Sender<'static, NoopRawMutex, ChannelMessage, 5>) -> ! {
    let mut count: u16 = 0;
    loop {
        tx.send(ChannelMessage::Even(count)).await;
        Timer::after(Duration::from_millis(500)).await;
        count += 2;
    }
}
#[embassy_executor::task]
async fn tx_odd_task(tx: Sender<'static, NoopRawMutex, ChannelMessage, 5>) -> ! {
    let mut count: u16 = 1;
    loop {
        tx.send(ChannelMessage::Odd(count)).await;
        Timer::after(Duration::from_millis(1000)).await;
        count += 2;
    }
}

#[embassy_executor::task]
async fn rx_task(rx: Receiver<'static, NoopRawMutex, ChannelMessage, 5>) -> ! {
    loop {
        let msg = rx.receive().await;
        match msg {
            ChannelMessage::Even(n) => info!("Even received: {n}"),
            ChannelMessage::Odd(n)  => info!("Odd received: {n}"),
        }
    }
}


/*
Helper function for converting RGB888 into corresponding RGB565.
*/
fn rgb(r: u8, g: u8, b: u8) -> RGB {
    // Ratios between 888 and 565
    let r_k: f32 = 0.122;
    let g_k: f32 = 0.247;
    let b_k: f32 = 0.122;
    
    // Apply transformation
    let r_t: f32 = r as f32 * r_k;
    let g_t: f32 = g as f32 * g_k;
    let b_t: f32 = b as f32 * b_k;
    
    // Cast to byte
    let r_byte: u8 = r_t as u8; 
    let g_byte: u8 = g_t as u8; 
    let b_byte: u8 = b_t as u8; 
    

    RGB::new(
        r_byte,
        g_byte,
        b_byte
    )
}


// Memory Pin for async task communication channel.
static CHANNEL: StaticCell<Channel<NoopRawMutex, ChannelMessage, 5>> = StaticCell::new();



#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // generator version: 1.4.0
    // generator parameters: -o esp32 -o unstable-hal -o embassy -o alloc -o wifi -o log -o neovim-classic -o vscode -o esp-backtrace -o esp

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    

    // Initialise the SPI display

    // SPI specific pins
    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(60))
            .with_mode(SpiMode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO18)
    .with_mosi(peripherals.GPIO23);
    
    // Display specific pins
    let cs  = Output::new(peripherals.GPIO15, Level::Low, OutputConfig::default());
    let dc  = Output::new(peripherals.GPIO2 , Level::Low, OutputConfig::default());
    let res = Output::new(peripherals.GPIO4 , Level::Low, OutputConfig::default());
    
    // Display pixel buffer 
    let mut buffer = [0_u8; 512];

    // Linking into display interface
    let spi_dev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let interface = SpiInterface::new(spi_dev, dc, &mut buffer);
    
    let mut delay = Delay::new();
    let mut display = Builder::new(ILI9341Rgb565, interface)
        .reset_pin(res)
        .invert_colors(ColorInversion::Inverted) 
        .color_order(ColorOrder::Bgr)
        .init(&mut delay).unwrap();
    
    display.clear(rgb(220, 20, 60)).unwrap();

    

    // peripherals gives us access to the GPIO pins
//    let led = esp_hal::gpio::Output::new(
//        peripherals.GPIO23,                    // D23 on dev board
//        esp_hal::gpio::Level::Low,             // Voltage
//        esp_hal::gpio::OutputConfig::default() // Config used
//    );

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    info!("Embassy initialized!");

    let _wifi_controller =
        esp_radio::wifi::WifiController::new(peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");
    let _wifi_interface = esp_radio::wifi::Interface::station();

    // TODO: Spawn some tasks
    let _ = spawner;

    // With Embassy, run two async tasks.
    // Printing to serial, and blinking the LED. (DONE)
    // Setting up a RX/TX channel between two tasks.
    
    let channel = CHANNEL.init(Channel::new());

    let rx = channel.receiver();
    let tx = channel.sender();
    
    let tx_even = tx.clone();
    let tx_odd  = tx.clone();

    // spawner.spawn(print_task() .unwrap());
    // spawner.spawn(led_task(led).unwrap());
    
    spawner.spawn(tx_even_task(tx_even).unwrap());
    spawner.spawn(tx_odd_task(tx_odd)  .unwrap());
    spawner.spawn(rx_task(rx)          .unwrap());
    

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.2.2/examples
}
