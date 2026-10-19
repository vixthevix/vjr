/*
Base functionality for video module of vjr.
Contains structs and functions for initialising SPI video, setting up video buffers, and general video display.
*/


pub const DISPLAY_BUFFER_SIZE: usize = 10240;

// Macro to take ownership of peripherals
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

macro_rules! display_buffer_new {
    ($b:ident) => {
        static $b: StaticCell<[u8; DISPLAY_BUFFER_SIZE]> = StaticCell::new()
    }
}

macro_rules! display_buffer_init {
    ($b:expr) => {
        $b.init([0_u8; DISPLAY_BUFFER_SIZE])
    }
}


/*
General struct for the display.
*/
struct Display {
    pinout  : PinOut,
    
    buffer_a: &'static mut [u8; DISPLAY_BUFFER_SIZE],
    buffer_b: &'static mut [u8; DISPLAY_BUFFER_SIZE],
}

impl Display {
    fn new(pinout: Pinout, buffer_a: &'static mut [u8; DISPLAY_BUFFER_SIZE], buffer_b: &'static mut [u8: DISPLAY_BUFFER_SIZE]) -> Self {
        Display {
            pinout,
            buffer_a,
            buffer_b,
        }
    }
}

struct PinOut<'a> {
    pub spi2: esp_hal::peripherals::SPI2  <'a>, 
    pub mosi: esp_hal::peripherals::GPIO23<'a>,
    pub sck : esp_hal::peripherals::GPIO18<'a>,
    pub cs  : esp_hal::peripherals::GPIO15<'a>,
    pub res : esp_hal::peripherals::GPIO4 <'a>,
    pub dc  : esp_hal::peripherals::GPIO2 <'a>,
}
