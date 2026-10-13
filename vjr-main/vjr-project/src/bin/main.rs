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
    
    // peripherals gives us access to the GPIO pins
    let led = esp_hal::gpio::Output::new(
        peripherals.GPIO23,                    // D23 on dev board
        esp_hal::gpio::Level::Low,             // Voltage
        esp_hal::gpio::OutputConfig::default() // Config used
    );

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
    spawner.spawn(led_task(led).unwrap());
    
    spawner.spawn(tx_even_task(tx_even).unwrap());
    spawner.spawn(tx_odd_task(tx_odd)  .unwrap());
    spawner.spawn(rx_task(rx)          .unwrap());
    

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.2.2/examples
}
