#![no_std]
#![no_main]

// pick a panicking behavior
//extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

use stm32f1xx_hal::{
    prelude::*,
    pac,
    timer::Timer,
};
use nb::block;

const MAXIMUM_TEMPERATURE: f32 = 60.0;
const MINIMUM_MEASUREMENTS: usize = 10;
const MILLISECOND: u32 = 8_000;
const SECOND: u32 = MILLISECOND * 1_000;

struct Grill {
    measures: [Option<f32>; MINIMUM_MEASUREMENTS],
    position: usize,
}

impl Grill {
    pub(crate) fn new() -> Self {
        Self {
            measures: [None; MINIMUM_MEASUREMENTS],
            position: 0,
        }
    }

    pub(crate) fn measure(&mut self) {
        let current_measure = Some(60.0);
        self.measures[self.current_slot()] = current_measure;
        self.increment_position();
    }

    pub(crate) fn has_minimum_measurements(&self) -> bool {
        self.measures.iter().all(|m| m.is_some())
    }

    pub(crate) fn current_average_temperature(&self) -> f32 {
        let total: f32 = self.measures.iter().map(|m| m.unwrap()).sum();
        total / MINIMUM_MEASUREMENTS as f32
    }

    pub(crate) fn grill_too_hot(&self) -> bool {
        self.current_average_temperature() >= MAXIMUM_TEMPERATURE
    }

    pub(crate) fn someone_present(&self) -> bool {
        false
    }

    pub(crate) fn should_trigger_error(&self) -> bool {
        if self.has_minimum_measurements() {
            self.grill_too_hot() && !self.someone_present()
        } else {
            false
        }
    }

    pub(crate) fn trigger_error(&self) {
        if self.should_trigger_error() {
            hprintln!("Trigger").unwrap();
        } else {
            hprintln!("No trigger").unwrap();
        }
    }

    fn increment_position(&mut self) -> usize {
        self.position = self.position.wrapping_add(1);
        self.position
    }

    fn current_slot(&self) -> usize {
        self.position % MINIMUM_MEASUREMENTS
    }
}

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store
    // the frozen frequencies in `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Acquire the GPIOC peripheral
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
    // in order to configure the port. For pins 0-7, crl should be passed instead.
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, 1.hz(), clocks);

    let a = 0x40011000 as *const i32;

    // Wait for the timer to trigger an update and change the state of the LED
    loop {
        block!(timer.wait()).unwrap();
        unsafe {hprintln!("High {:#b}", *a).unwrap();}
        led.set_high();
        block!(timer.wait()).unwrap();
        unsafe {hprintln!("Low  {:#b}", *a).unwrap();}
        led.set_low();
    }
}
