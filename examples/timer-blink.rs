#![no_main]
#![no_std]
#![allow(deprecated)]

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nucleo_f401re::{
    delay::Delay,
    prelude::*,
    stm32::{self, TIM2},
    stm32::interrupt,
    timer::{Timer, Event},
    Interrupt,
    gpio::{
        PushPull, Output,
        gpioa::{PA5},
    },
};
use panic_semihosting as _;


static mut GLED: Option<PA5<Output<PushPull>>> = None;


#[entry]
fn main() -> ! {
    let device = stm32::Peripherals::take().unwrap();
    let mut core = Peripherals::take().unwrap();

    device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    let gpioa = device.GPIOA.split();
    let led = gpioa.pa5.into_push_pull_output();

    unsafe {
        GLED.replace(led);
    }

    let gpiob = device.GPIOB.split();
    let _irpin = gpiob.pb8.into_floating_input();

    // Setup the timer for 50us operation
    // 50 us = 20_000 Hz
    let mut timer2: Timer<TIM2> = Timer::tim2(device.TIM2, 20_000.hz(), clocks);
    // Interrupt
    timer2.listen(Event::TimeOut);

    // Enable the external interrupt
    core.NVIC.enable(Interrupt::TIM2);

    loop { }
}

#[interrupt]
fn TIM2() {
    static mut COUNT: u32 = 0;

    // Clear the interrupt
    unsafe {
        (*stm32::TIM2::ptr()).sr.modify(|_, w| w.tif().set_bit());
        //(*stm32::EXTI::ptr()).pr.modify(|_, w| w.pr13().set_bit());
    }

    unsafe {
        GLED.as_mut().map(|led| led.toggle());
    }


    // `COUNT` has type `&mut u32` and it's safe to use
    *COUNT += 1;
}




/*
struct InputCompareTimer {
    tim: TIM2,
    _prescaler: u16,
    _period: u32,
}


impl InputCompareTimer {
    /// Creates a new InputCompareTimer
    pub fn new(tim: TIM2) -> Self {
        Self {
            tim,
            _prescaler: 0,
            _period: 0,
        }
    }

    fn _polarity(&mut self) {

    }

    // 13.3.5 Input capture mode

    pub fn setup(&self) {
        // Disable
        self.tim.cr1.modify(|_, w| w.cen().clear_bit());

        // Clear
        self.tim.cnt.reset();

        //TODO: Should be 'ccmr1_input', but that doesn't work
        //      for some reason. Investigate.
        self.tim.ccmr1_output.modify(|_r, w| unsafe {w.cc1s().bits(1)});
        // Configure filter
        //TODO: Same problem as above. Could use bits() as a workaround

        // Use the internal clock
        self.tim.smcr.modify(|_r, w| w.sms().disabled());

        // Setup the prescaler
        self.tim.psc.modify(|_r, w| unsafe { w.psc().bits(16_000) });

        // Period
        self.tim.arr.modify(|_r, w| unsafe {w.bits(100_000)});

        // Channels


        // Edges (RM0368 p. 361)
        self.tim.ccer.modify(|_r, w|
                    w
                    // Rising edge, non inverting
                    .cc1p().clear_bit()
                    .cc1np().clear_bit()
                    // Capture output enable
                    .cc1e().set_bit()
                    );

        // Enable interrupt
        self.tim.dier.modify(|_r, w|
                    // Capture compare channel 1 interupt enable
                    w.cc1ie().enabled());


    }
}
*/
