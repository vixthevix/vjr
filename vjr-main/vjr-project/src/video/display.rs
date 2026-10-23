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

use embedded_hal::spi::SpiDevice;

use embedded_hal_bus::spi::{
    ExclusiveDevice,
    NoDelay,
};


use esp_hal::{
    delay::Delay,
    time::Rate,
    dma::{
        DmaChannel,
    },
    spi::{
        master::{
            Config as SpiConfig,
            Spi,
            SpiDma,
            SpiMasterDmaChannel,
            AnySpi
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


// Enum for commands that can be communicated to the display with.
// Datasheet used: https://cdn-shop.adafruit.com/datasheets/ILI9341.pdf
#[repr(u8)]                   // C-style enum, each value can be treated as u8  
#[derive(Clone, Copy, Debug)] // Give Copy and Clone access, to match u8 repr
pub enum Command {
    Nop = 0x00,
    SoftwareReset = 0x01,
    ReadDisplayId = 0x04,
    ReadDisplayStatus = 0x09,
    ReadDisplayPower = 0x0A,
    ReadDisplay = 0x0B,
    ReadDisplayPixel = 0x0C,
    ReadDisplayImage = 0x0D,
    ReadDisplaySignal = 0x0E,
    ReadDisplayDiagnosis = 0x0F,
    SleepOn = 0x10,
    SleepOff = 0x11,
    PartialModeOn = 0x12,
    PartialModeOff = 0x13,
    DisplayInversionOff = 0x20,
    DisplayInversionOn = 0x21,
    GammaSet = 0x26,
    DisplayOff = 0x28,
    DisplayOn = 0x29,
    SetColumnCoords = 0x2A,
    SetRowCoords = 0x2B,
    MemoryWrite = 0x2C,
    ColourSet = 0x2D,
    MemoryRead = 0x2E,
    PartialAreaSet = 0x30,
    VerticalScrollAreaSet = 0x33,
    TearingEffectOff = 0x34,
    TearingEffectOn = 0x35,
    MemoryAccessCtrl = 0x36,
    VerticalScrollStartSet = 0x37,
    IdleOff = 0x38,
    IdleOn = 0x39,
    SetColourMode = 0x3A,
    MemoryWriteContinue = 0x3C,
    MemoryReadContinue = 0x3E,
    TearScanlineSet = 0x44,
    ScanlineGet = 0x45,
    BrightnessSet = 0x51,
    BrightnessGet = 0x52,
    CtrlDisplaySet = 0x53,
    CtrlDisplayGet = 0x54,
    AdaptiveBrightnessSet = 0x55,
    AdaptiveBrightnessGet = 0x56,
    CABCBrightnessGet = 0x5E,
    CABCBrightnessSet = 0x5F,
    ReadId1 = 0xDA,
    ReadId2 = 0xDB,
    ReadId3 = 0xDC,
    RGBInterfaceSignalCtrl = 0xB0,
    FramerateCtrlNormal = 0xB1,
    FramerateCtrlIdle = 0xB2,
    FramerateCtrlPartial = 0xB3,
    DisplayInversionCtrl = 0xB4,
    BlankingPorchCtrl = 0xB5,
    DisplayFunctionCtrl = 0xB6,
    EntryModeSet = 0xB7,
    BacklightCtrl1 = 0xB8,
    BacklightCtrl2 = 0xB9,
    BacklightCtrl3 = 0xBA,
    BacklightCtrl4 = 0xBB,
    BacklightCtrl5 = 0xBC,
    BacklightCtrl7 = 0xBE,
    BacklightCtrl8 = 0xBF,
    PowerCtrl1 = 0xC0,
    PowerCtrl2 = 0xC1,
    PowerCtrl3 = 0xC2,
    PowerCtrl4 = 0xC3,
    PowerCtrl5 = 0xC4,
    VcomCtrl1 = 0xC5,
    VcomCtrl2 = 0xC7,
    NVMemoryWrite = 0xD0,
    NVMemoryProtectionKey = 0xD1,
    NVMemoryStatusRead = 0xD2,
    ReadID4 = 0xD3,
    GammaCorrectionPlus = 0xE0,
    GammaCorrectionMinus = 0xE1,
    DigitalGammaCtrl1 = 0xE2,
    DigitalGammaCtrl2 = 0xE3,
    InterfaceCtrl = 0xF6,
    PowerCtrlA = 0xCB,
    PowerCtrlB = 0xCF,
    DriverInternalTimingCtrlA = 0xE8,
    DriverExternalTimingCtrlA = 0xE9,
    DriverTimingCtrlB = 0xEA,
    PowerOnSequenceCtrl = 0xED,
    EnableGammaCtrl3 = 0xF2,
    PumpRatioCtrl = 0xF7,
}


pub const DISPLAY_CMD_BUFFER_SIZE: usize = 16;

// Type aliases for clear code
pub type DisplaySpiBus<'a> = SpiDma<'a, esp_hal::Blocking>;
pub type DisplayCsPin <'a> = Output<'a>; 
pub type DisplaySpiDev<'a> = ExclusiveDevice<DisplaySpiBus<'a>, DisplayCsPin<'a>, NoDelay>;
pub type DisplayDcPin <'a> = Output<'a>;
pub type DisplayRstPin<'a> = Output<'a>;

// The Display Interface and final Screen types:
pub type DisplayInterface<'a> = SpiInterface<'a, DisplaySpiDev<'a>, DisplayDcPin<'a>>;
pub type DisplayScreen <'a>= mipidsi::Display<DisplayInterface<'a>, ILI9341Rgb565, DisplayRstPin<'a>>;

/*
General struct for the display.
For maximum control, we will implement commands ourselves.
*/
pub struct Display<'a> {
    interface: DisplaySpiDev<'a>, // The target for writing our SPI data to.
    data_mode: DisplayDcPin<'a>,  // Switching between Data and Command mode.

}

impl<'a> Display<'a> {
    pub fn new<C: DmaChannel + SpiMasterDmaChannel<'a, AnySpi<'a>>>(
        pinout     : PinOut<'a>,
        dma_channel: C,
        cmd_buffer : &'static mut [u8; DISPLAY_CMD_BUFFER_SIZE]
    ) -> Self {
        
        // Initialise the screen with mipidsi, then hand control over back to us.

        // SPI specific pins
        let spi = Spi::new(
            pinout.spi2,
            SpiConfig::default()
                .with_frequency(Rate::from_mhz(60))
                .with_mode(SpiMode::_0),
        )
        .unwrap()
        .with_sck(pinout.sck)
        .with_mosi(pinout.mosi)
        .with_dma(dma_channel);

        // Display specific pins
        let cs  = Output::new(pinout.cs , Level::Low, OutputConfig::default());
        let res = Output::new(pinout.res, Level::Low, OutputConfig::default());
        
        let mut dc  = Output::new(pinout.dc , Level::Low, OutputConfig::default());
        
        // Linking into display interface
        let mut spi_dev   = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
        let mut interface = SpiInterface::new(&mut spi_dev, &mut dc, cmd_buffer); 
        
        // Start up the screen with mipidsi driver, before handing off control to Display
        let mut delay = Delay::new();
        let _interface = Builder::new(ILI9341Rgb565, &mut interface)
            .reset_pin(res)
            .invert_colors(ColorInversion::Inverted) 
            .color_order(ColorOrder::Bgr)
            .init(&mut delay).unwrap();

        Display {
            interface: spi_dev,
            data_mode: dc
        }
    }
    
    /*
    Helper function for converting RGB888 into corresponding RGB565.
    */
    pub fn rgb(r: u8, g: u8, b: u8) -> RGB {
        RGB::new(
            r >> 3,
            g >> 2,
            b >> 3,
        )
    }
    
    fn execute(&mut self, command: Command, arguments: &[u8]) {
        // For command mode => dc = 0
        // For raw mode     => dc = 1

        self.data_mode.set_low();
        self.interface.write(&[command as u8]).unwrap();

        self.data_mode.set_high();
        self.interface.write(arguments).unwrap();

        self.data_mode.set_low(); // For safety
    }

    pub fn write_buffer(&mut self, buffer: &[u8], y_offset: u16) {
        // Define drawing region
        let x1: u16 = 0;
        let y1: u16 = y_offset;
        let x2: u16 = 319;          // 320 px display width (HARDCODED, CHANGE LATER)
        let y2: u16 = y_offset + 15; // 16  px buffer height (HARDCODED, CHANGE LATER)
        

        // To fill the screen, we have to set the column and row addresses.
        // Command 0x2a => Column Address Set
        // Command 0x2b => Page (Row) Address Set
        // Command 0x2c => Memory Write
        
        // Additionally, we need to split our coordinates into u16 values,
        // Due to arguments being u8 for raw data.
        
        let x1 = x1.to_be_bytes();
        let y1 = y1.to_be_bytes();
        let x2 = x2.to_be_bytes();
        let y2 = y2.to_be_bytes();
        
        // Column set
        self.execute(Command::SetColumnCoords, &[x1[0], x1[1], x2[0], x2[1]]);

        // Row set
        self.execute(Command::SetRowCoords, &[y1[0], y1[1], y2[0], y2[1]]);

        // Memory write
        self.execute(Command::MemoryWrite, buffer);
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
