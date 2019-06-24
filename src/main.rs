#![no_std]
#![no_main]

// pick a panicking behavior
//extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::Peripherals;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

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
        hprintln!(
            "Current measure {} slot {} Measured {:?}",
            self.position,
            self.current_slot(),
            current_measure
        )
        .unwrap();
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
    hprintln!("Starting").unwrap();
    let mut measurements = Grill::new();
    let peripherals = Peripherals::take().unwrap();
    let mut timer = peripherals.SYST;

    timer.set_clock_source(SystClkSource::Core);
    timer.set_reload(SECOND);
    timer.enable_counter();

    loop {
        measurements.measure();
        measurements.trigger_error();
        while !timer.has_wrapped() {}
    }
}
