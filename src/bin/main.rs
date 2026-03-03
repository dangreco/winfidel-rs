#![no_std]
#![no_main]
#![deny(clippy::mem_forget)]
#![deny(clippy::large_stack_frames)]

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

use defmt::error;
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Duration;
use embassy_time::Timer;
use esp_hal::clock::CpuClock;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("panic: {:?}", defmt::Debug2Format(info));
    loop {}
}

#[esp_rtos::main]
async fn main(__: Spawner) -> ! {
    rtt_target::rtt_init_defmt!();
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Start RTOS
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_ints = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_ints.software_interrupt0);

    info!("ready");
    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}
