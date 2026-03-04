#![no_std]
#![no_main]
#![deny(clippy::mem_forget)]
#![deny(clippy::large_stack_frames)]

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    clock::CpuClock, interrupt::software::SoftwareInterruptControl, peripherals::Peripherals,
    timer::timg::TimerGroup,
};
use static_cell::StaticCell;

use winfidel_rs::peripheral;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("panic: {:?}", defmt::Debug2Format(info));
    loop {}
}

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    rtt_target::rtt_init_defmt!();
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    static PERIPHERALS: StaticCell<Peripherals> = StaticCell::new();
    let peripherals = PERIPHERALS.init(peripherals);

    // SAFETY: TIMG0 and SW_INTERRUPT are each consumed exactly once here.
    let timg0 = TimerGroup::new(unsafe { peripheral::take(&peripherals.TIMG0) });
    let sw_ints =
        SoftwareInterruptControl::new(unsafe { peripheral::take(&peripherals.SW_INTERRUPT) });
    esp_rtos::start(timg0.timer0, sw_ints.software_interrupt0);

    let mut led = peripheral::led::onboard(peripherals);

    info!("ready");
    led.color(0, 0, 100).unwrap();
    loop {
        led.on().unwrap();
        Timer::after(Duration::from_millis(500)).await;
        led.off().unwrap();
        Timer::after(Duration::from_millis(500)).await;
    }
}
