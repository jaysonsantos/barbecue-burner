#![no_std]

use core::{
    fmt::Debug,
    sync::atomic::{AtomicUsize, Ordering},
};
use defmt::{info, Write};
use defmt_rtt as _;
use embedded_hal::blocking::delay::*;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use panic_probe as _; // global logger

pub mod grill;
pub mod thermometer;

#[defmt::timestamp]
fn timestamp() -> u64 {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n as u64
}

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
