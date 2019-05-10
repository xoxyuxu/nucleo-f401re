#![no_main]
#![no_std]
#![allow(deprecated)]
#![allow(unused_imports)]

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
    // Res
    static mut CLOCK: u32 = 0;

    // Late resources
    static mut EXTI: stm32::EXTI = ();
    static mut BUTTON: PC13<Input<PullDown>> = ();
    static mut LED: PA5<Output<PushPull>> = ();
    //static mut IR: IrReceiver<PB8<Input<PullDown>>> = ();

    static mut INPIN: PB8<Input<PullDown>> = ();
    static mut IRRECV: IrRecv = ();

    #[init(schedule = [sample])]
    fn init() {
        // Cortex-M peripherals
        let _core: rtfm::Peripherals = core;

        // Device specific peripherals
        let mut device: stm32::Peripherals = device;

        // Enable the clock for the SYSCFG
        device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

        // Configure PC13 (User Button) as an input
        let gpioc = device.GPIOC.split();
        let mut button = gpioc.pc13.into_pull_down_input();

        // Configure the led pin as an output
        let gpioa = device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        // Enable interrupt on PC13
        button.make_interrupt_source(&mut device.SYSCFG);
        button.enable_interrupt(&mut device.EXTI);
        button.trigger_on_edge(&mut device.EXTI, Edge::RISING);

        let gpiob = device.GPIOB.split();
        let mut ir_pin = gpiob.pb8.into_pull_down_input();

        ir_pin.make_interrupt_source(&mut device.SYSCFG);
        ir_pin.enable_interrupt(&mut device.EXTI);
        ir_pin.trigger_on_edge(&mut device.EXTI, Edge::RISING_FALLING);
        //let recv = IrReceiver::new(ir_pin);

        // Setup the system clock
        let rcc = device.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(CPU_FREQ.hz()).freeze();

        let now = Instant::now();

        schedule.sample(now + 4200.cycles()).unwrap();

        hprintln!("init done").unwrap();

        EXTI = device.EXTI;
        LED = led;
        BUTTON = button;
        INPIN = ir_pin;
        IRRECV = IrRecv::new();
        //IR = recv;
    }

    #[idle]
    fn idle() -> ! {
        hprintln!("idle").unwrap();

        // The idle loop
        loop { }
    }

    #[task(resources = [IRRECV])]
    fn debug_print() {
        let ir: &IrRecv = &resources.IRRECV;

        for ts in &ir.buf {
            hprintln!("{}", ts).unwrap();
        }
    }

    #[task(schedule = [sample], spawn = [debug_print], resources = [CLOCK])]
    fn sample() {

        *resources.CLOCK += 1;

        //resources.IR.read_pin_state();

        schedule
            .sample(scheduled + 4200.cycles())
            .unwrap();
    }


    #[interrupt(binds = EXTI9_5, resources = [EXTI, INPIN, LED, CLOCK, IRRECV])]
    fn on_ir_recv() {

        let level = resources.INPIN.is_low();
        // assert!(this != prev level)
        resources.INPIN.clear_interrupt_pending_bit(&mut resources.EXTI);

        let mut recv = resources.IRRECV;
        let mut clk = resources.CLOCK;

        if recv.is_init() {
            *clk = 0;
        }

        let time = *clk;

        // Add event from IR PIN
        recv.log_event(time, level);

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

#[derive(PartialEq)]
pub enum State {
    Init,
    Capture,

    Done,
}


const BUF_LEN: usize = 32;

pub struct IrRecv {
    pub buf: [u32; BUF_LEN],
    pub cur: usize,
    pub state: State,
}

impl IrRecv {

    pub fn new() -> Self {
        Self {
            buf: [0; BUF_LEN],
            cur: 0,
            state: State::Init,
        }
    }

    /// Feed the state machine
    pub fn log_event(&mut self, time: u32, _level: bool) {

        match self.state {
            State::Init => {
                self.state = State::Capture;
            },
            _ => (),
        };

        if self.cur >= BUF_LEN {
            self.state = State::Done;
            return;
        }

        self.buf[self.cur] = time;
        self.cur += 1;
    }

    pub fn is_init(&self) -> bool {
        self.state == State::Init
    }
}
