#![no_main]
#![no_std]

use cortex_m_rt::entry;
//use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use cortex_m::peripheral::Peripherals;

use nucleo_f401re::{delay::Delay, prelude::*, stm32};

use embedded_infrared::IrReceiver;

#[entry]
fn main() -> ! {
    // The Stm32 peripherals
    let device = stm32::Peripherals::take().unwrap();
    let cp = Peripherals::take().unwrap();

    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    let gpiob = device.GPIOB.split();
    let scl = gpiob
        .pb8
        .into_floating_input();

    let mut recv = IrReceiver::new(scl);

    let mut delay = Delay::new(cp.SYST, clocks);

    loop {
        recv.read_pin_state();

        delay.delay_us(50_u16);
    }
}
