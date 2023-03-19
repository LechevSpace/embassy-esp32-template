#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Executor;
use embassy_time::{Duration, Timer};

use esp_backtrace as _;

use hal::{
    clock::ClockControl,
    embassy,
    gpio::{Gpio8, Output, PushPull},
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
    Rtc, IO,
};

use static_cell::StaticCell;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[hal::entry]
fn main() -> ! {
    esp_println::println!("Init!");
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    // Disable watchdog timers
    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    #[cfg(feature = "embassy-time-systick")]
    embassy::init(
        &clocks,
        hal::systimer::SystemTimer::new(peripherals.SYSTIMER),
    );

    #[cfg(feature = "embassy-time-timg0")]
    embassy::init(&clocks, timer_group0.timer0);

    // Set GPIO5 as an output, and set its state high initially.
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let led = io.pins.gpio8.into_push_pull_output();

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(blink(led)).ok();
        spawner.spawn(run()).ok();
    });
}

#[embassy_executor::task]
async fn blink(mut led: Gpio8<Output<PushPull>>) {
    loop {
        led.set_high().expect("Should set High");
        esp_println::println!("async LED ON");
        Timer::after(Duration::from_millis(100)).await;

        led.set_low().expect("Should set Low");
        esp_println::println!("async LED OFF");
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[embassy_executor::task]
async fn run() {
    loop {
        esp_println::println!("Bing!");
        Timer::after(Duration::from_millis(5_000)).await;
    }
}
