#![no_main]
#![no_std]

//use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
//use cortex_m::peripheral::Peripherals;

use nucleo_f401re::{
    gpio::{gpioa::PA5, gpiob::PB8, gpioc::PC13, Edge, ExtiPin, Input, Output, PullDown, PushPull},
    prelude::*,
    stm32,
};

use embedded_infrared::IrReceiver;

use rtfm::{app, Instant};

const CPU_FREQ: u32 = 84_000_000;


#[app(device = nucleo_f401re::hal::stm32)]
const APP: () = {
    // Late resources
    static mut EXTI: stm32::EXTI = ();
    static mut BUTTON: PC13<Input<PullDown>> = ();
    static mut LED: PA5<Output<PushPull>> = ();
    static mut IR: IrReceiver<PB8<Input<PullDown>>> = ();

    #[init(schedule = [sample])]
    fn init() {
        // Cortex-M peripherals
        let _core: rtfm::Peripherals = core;

        // Device specific peripherals
        let mut device: stm32::Peripherals = device;

        // Configure PC13 (User Button) as an input
        let gpioc = device.GPIOC.split();
        let mut button = gpioc.pc13.into_pull_down_input();

        // Configure the led pin as an output
        let gpioa = device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        // Enable the clock for the SYSCFG
        device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

        // Enable interrupt on PC13
        button.make_interrupt_source(&mut device.SYSCFG);
        button.enable_interrupt(&mut device.EXTI);
        button.trigger_on_edge(&mut device.EXTI, Edge::RISING);

        let gpiob = device.GPIOB.split();
        let mut ir_pin = gpiob.pb8.into_pull_down_input();

        ir_pin.make_interrupt_source(&mut device.SYSCFG);
        ir_pin.enable_interrupt(&mut device.EXTI);
        ir_pin.trigger_on_edge(&mut device.EXTI, Edge::RISING);

        let recv = IrReceiver::new(ir_pin);

        // Setup the system clock
        let rcc = device.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(CPU_FREQ.hz()).freeze();

        let now = Instant::now();

        schedule.sample(now + 4200.cycles()).unwrap();

        hprintln!("init done").unwrap();

        EXTI = device.EXTI;
        LED = led;
        BUTTON = button;
        IR = recv;
    }

    #[idle(resources = [IR])]
    fn idle() -> ! {
        hprintln!("idle").unwrap();

        // The idle loop
        loop {
/*
            if resources.IR.done {
                hprintln!("buf: {}", resources.IR.buf);

                resources.IR.done = false;
            }
*/

        }
    }

    #[task(resources = [IR])]
    fn debug_print() {
        hprintln!("res: {}", resources.IR.buf).unwrap();
    }

    #[task(schedule = [sample], spawn = [debug_print], resources = [IR])]
    fn sample() {

        resources.IR.read_pin_state();

        if resources.IR.done {
            // spawn debug_print task
            spawn.debug_print().unwrap();
        }

        schedule
            .sample(scheduled + 4200.cycles())
            .unwrap();
    }


    #[interrupt(binds = EXTI9_5, resources = [EXTI, IR, LED])]
    fn on_ir_recv() {
        resources.IR.get_pin_ref().clear_interrupt_pending_bit(&mut resources.EXTI);
        resources.LED.toggle();
    }

    #[interrupt(binds = EXTI15_10, resources = [EXTI, LED, BUTTON])]
    fn on_button_press() {
        // Clear the interrupt
        resources.BUTTON.clear_interrupt_pending_bit(&mut resources.EXTI);
    }

    extern "C" {
        fn ADC();
    }

};

