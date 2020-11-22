use core::{fmt::Debug, ops::Shl};
use defmt::info;

const READ_POWER_SUPPLY: u8 = 0xb4;
const READ_ROM: u8 = 0x33;

use embedded_hal::{
    blocking::delay::DelayUs,
    digital::v2::{InputPin, OutputPin},
};
use stm32f1xx_hal::gpio::{gpioa::CRL, gpioa::PA2, Dynamic};

pub struct Thermometer<D>
where
    D: DelayUs<u32>,
{
    pin: PA2<Dynamic>,
    crl: CRL,
    delay: D,
}

impl<D> Thermometer<D>
where
    D: DelayUs<u32>,
{
    pub fn new(pin: PA2<Dynamic>, crl: CRL, delay: D) -> Self {
        Self { pin, crl, delay }
    }

    pub fn measure(&mut self) -> i32 {
        self.reset();
        todo!();
    }

    fn set_input(&mut self) {
        self.pin.make_pull_up_input(&mut self.crl);
    }

    fn set_output(&mut self) {
        self.pin.make_open_drain_output(&mut self.crl);
    }

    fn reset(&mut self) {
        self.set_output();
        self.pin.set_low().unwrap();

        self.delay.delay_us(480);
        self.set_input();

        // assert!(self.pin.is_high().unwrap());

        self.delay.delay_us(70);
        assert!(self.pin.is_low().unwrap());

        self.delay.delay_us(410)
    }

    pub fn is_parasite_mode(&mut self) -> bool {
        self.reset();
        self.write_byte(READ_POWER_SUPPLY);

        self.start_read_slot();
        self.delay.delay_us(1);
        self.pin.is_low().unwrap()
    }

    pub fn read_rom(&mut self) -> u64 {
        self.reset();
        self.write_byte(READ_ROM);
        let rom_data = self.read_u64();
        info!("Rom data {:u64}", rom_data);
        assert!(rom_data >> 56 as u8 == 0x28);
        todo!();
    }

    fn write_byte(&mut self, byte: u8) {
        // info!("Writing bytes");
        for shift in 0..8 {
            self.write_bit(((byte >> shift) & 0x1) == 1);
        }
    }

    fn write_bit(&mut self, bit: bool) {
        if bit {
            self.write_bit_1();
        } else {
            self.write_bit_0();
        }
    }

    fn write_bit_1(&mut self) {
        self.set_output();
        self.pin.set_low().unwrap();
        self.delay.delay_us(10);
        self.set_input();
        self.delay.delay_us(1);
        assert!(self.pin.is_high().unwrap());
        self.delay.delay_us(59);
    }

    fn write_bit_0(&mut self) {
        self.set_output();
        self.pin.set_low().unwrap();
        self.delay.delay_us(60);
        self.set_input();
        self.delay.delay_us(1);
        assert!(self.pin.is_high().unwrap());
    }

    fn start_read_slot(&mut self) {
        // Read time
        self.set_output();
        self.pin.set_low().unwrap();
        self.delay.delay_us(1);
        self.set_input();
    }

    fn read_u64(&mut self) -> u64 {
        let mut value: u64 = 0;

        for shift in 0..64 {
            let bit = self.read_bit();
            value = bit as u64 | (value << shift);
            self.delay.delay_us(1);
        }
        value
    }

    fn read_bit(&mut self) -> u8 {
        self.start_read_slot();
        self.delay.delay_us(9);
        let bit = if self.pin.is_high().unwrap() { 1 } else { 0 };
        self.delay.delay_us(50);
        bit
    }
}
