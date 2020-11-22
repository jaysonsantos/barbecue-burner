#![no_std]
#![no_main]

use core::{convert::Infallible, fmt::Error};

use barbecue_burner::{exit, thermometer::Thermometer};
use cortex_m_rt::entry;

use nb::block;
use stm32f1xx_hal::{delay::Delay, pac, prelude::*, time::Hertz, timer::Timer};

use defmt::info;
use embedded_hal::blocking::delay::*;
use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::prelude::*;

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
    let clocks = rcc
        .cfgr
        .use_hse(16.mhz())
        .sysclk(72.mhz())
        .freeze(&mut flash.acr);
    info!("HCLK: {:u32}", clocks.hclk().0);

    // Acquire the GPIO peripherals that we'll use
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    // Configure gpio B pin 12 as a push-pull output. The `crh` register is passed to the function
    // in order to configure the port. For pins 0-7, crl should be passed instead.
    let mut led = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);
    let mut delay = Delay::new(cp.SYST, clocks);
    // 1.hz();
    let mut a2 = gpioa.pa2.into_dynamic(&mut gpioa.crl);
    // let a2 = gpioa.pa2.into_pull_up_input(&mut gpioa.crl);

    let mut thermometer = Thermometer::new(a2, gpioa.crl, delay);

    loop {
        let is_parasite_mode = thermometer.is_parasite_mode();
        info!("Parasite mode? {:bool}", is_parasite_mode);
        let rom = thermometer.read_rom();
        let rom = exit();
    }
}
