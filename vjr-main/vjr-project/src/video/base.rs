/*
Base functionality for video module of vjr.
Contains structs and functions for initialising SPI video, setting up video buffers, and general video display.
*/


use esp_backtrace as _;

use esp_hal::gpio::{
    Level,
    Output,
    OutputConfig,
};

use embedded_hal_bus::spi::{
    ExclusiveDevice,
    NoDelay,
};

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


// Macros 

#[macro_export]
macro_rules! display_pinout_new {
    ($p:expr) => {
        PinOut {
            spi2: $p.SPI2  ,
            mosi: $p.GPIO23,
            sck:  $p.GPIO18,
            cs:   $p.GPIO15,
            res:  $p.GPIO4 ,
            dc:   $p.GPIO2 ,
        }
    };
}

#[macro_export]
macro_rules! display_buffer_new {
    ($b:ident) => {
        static $b: StaticCell<[u8; DISPLAY_BUFFER_SIZE]> = StaticCell::new();
    }
}

#[macro_export]
macro_rules! display_buffer_init {
    ($b:expr) => {
        $b.init([0_u8; DISPLAY_BUFFER_SIZE])
    }
}

#[macro_export]
macro_rules! display_cmd_buffer_new {
    ($b:ident) => {
        static $b: StaticCell<[u8; DISPLAY_CMD_BUFFER_SIZE]> = StaticCell::new();
    }
}

#[macro_export]
macro_rules! display_cmd_buffer_init {
    ($b:expr) => {
        $b.init([0_u8; DISPLAY_CMD_BUFFER_SIZE])
    }
}

pub const DISPLAY_BUFFER_SIZE    : usize = 10240;
pub const DISPLAY_CMD_BUFFER_SIZE: usize = 16;


// Type aliases for clear code
pub type DisplaySpiBus<'a> = Spi<'a, esp_hal::Blocking>;
pub type DisplayCsPin <'a> = Output<'a>; 
pub type DisplaySpiDev<'a>     = ExclusiveDevice<DisplaySpiBus<'a>, DisplayCsPin<'a>, NoDelay>;
pub type DisplayDcPin <'a> = Output<'a>;
pub type DisplayRstPin<'a> = Output<'a>;

// The Display Interface and final Screen types:
pub type DisplayInterface<'a> = SpiInterface<'a, DisplaySpiDev<'a>, DisplayDcPin<'a>>;
pub type DisplayScreen <'a>= mipidsi::Display<DisplayInterface<'a>, ILI9341Rgb565, DisplayRstPin<'a>>;

/*
General struct for the display.
*/
pub struct Display<'a> {
    buffer_a: &'static mut [u8; DISPLAY_BUFFER_SIZE],
    buffer_b: &'static mut [u8; DISPLAY_BUFFER_SIZE],

    interface: DisplayScreen<'a>, // The thing made by Builder::new()
}

impl<'a> Display<'a> {
    pub fn new(
        pinout: PinOut<'a>, 
        buffer_a: &'static mut [u8; DISPLAY_BUFFER_SIZE], 
        buffer_b: &'static mut [u8; DISPLAY_BUFFER_SIZE],
        cmd_buffer: &'static mut [u8; DISPLAY_CMD_BUFFER_SIZE]
    ) -> Self {
        
        // SPI specific pins
        let spi = Spi::new(
            pinout.spi2,
            SpiConfig::default()
                .with_frequency(Rate::from_mhz(60))
                .with_mode(SpiMode::_0),
        )
        .unwrap()
        .with_sck(pinout.sck)
        .with_mosi(pinout.mosi);

        // Display specific pins
        let cs  = Output::new(pinout.cs , Level::Low, OutputConfig::default());
        let dc  = Output::new(pinout.dc , Level::Low, OutputConfig::default());
        let res = Output::new(pinout.res, Level::Low, OutputConfig::default());
        
        // Linking into display interface
        let spi_dev   = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
        let interface = SpiInterface::new(spi_dev, dc, cmd_buffer); 
        
        let mut delay = Delay::new();
        let interface = Builder::new(ILI9341Rgb565, interface)
            .reset_pin(res)
            .invert_colors(ColorInversion::Inverted) 
            .color_order(ColorOrder::Bgr)
            .init(&mut delay).unwrap();

        Display {
            buffer_a,
            buffer_b,
            interface
        }
    }
    
    /*
    Helper function for converting RGB888 into corresponding RGB565.
    */
    pub fn rgb(r: u8, g: u8, b: u8) -> RGB {
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
    
    pub fn fill(&mut self, colour: RGB) {
        self.interface.clear(colour).unwrap();
    }

    

}

/*
Container struct for pinout
*/
pub struct PinOut<'a> {
    pub spi2: esp_hal::peripherals::SPI2  <'a>, 
    pub mosi: esp_hal::peripherals::GPIO23<'a>,
    pub sck : esp_hal::peripherals::GPIO18<'a>,
    pub cs  : esp_hal::peripherals::GPIO15<'a>,
    pub res : esp_hal::peripherals::GPIO4 <'a>,
    pub dc  : esp_hal::peripherals::GPIO2 <'a>,
}
