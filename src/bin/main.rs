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
    clock::CpuClock,
    interrupt::software::SoftwareInterruptControl,
    ledc::{self, LSGlobalClkSource, Ledc, LowSpeed, timer::TimerIFace},
    peripherals::Peripherals,
    time::Rate,
    timer::timg::TimerGroup,
};
use static_cell::StaticCell;
use winfidel_rs::drivers::led::Led;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("panic: {:?}", defmt::Debug2Format(info));
    loop {}
}

// ---------------------------------------------------------------------------
// Static cells for LEDC resources (must outlive the LED driver)
// ---------------------------------------------------------------------------

static LEDC: StaticCell<Ledc> = StaticCell::new();
static LEDC_TIMER: StaticCell<ledc::timer::Timer<'static, LowSpeed>> = StaticCell::new();

// ---------------------------------------------------------------------------
// Peripheral helpers
// ---------------------------------------------------------------------------

/// Read a single peripheral field out of the static `Peripherals` struct.
///
/// # Safety
///
/// Each field must be read **exactly once** across the entire program.
/// The caller is responsible for upholding this invariant.
unsafe fn take_peripheral<T>(field: &T) -> T {
    unsafe { core::ptr::read(field) }
}

/// Initialise the RGB LED driver from the static peripherals reference.
///
/// Consumes `LEDC`, `GPIO5`, `GPIO4`, and `GPIO3`.
fn init_led(peripherals: &'static mut Peripherals) -> Led<'static> {
    // SAFETY: each peripheral field is consumed exactly once.
    let ledc = LEDC.init(Ledc::new(unsafe { take_peripheral(&peripherals.LEDC) }));
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let timer = LEDC_TIMER.init(ledc.timer::<LowSpeed>(ledc::timer::Number::Timer0));
    timer
        .configure(ledc::timer::config::Config {
            duty: ledc::timer::config::Duty::Duty5Bit,
            clock_source: ledc::timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(24),
        })
        .unwrap();

    Led::new(
        timer,
        unsafe { take_peripheral(&peripherals.GPIO5) },
        unsafe { take_peripheral(&peripherals.GPIO4) },
        unsafe { take_peripheral(&peripherals.GPIO3) },
    )
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    rtt_target::rtt_init_defmt!();
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Move peripherals into a static cell so we can hand out &'static mut
    // references to individual fields. Each field is consumed exactly once
    // via `take_peripheral` — do not read the same field twice.
    static PERIPHERALS: StaticCell<Peripherals> = StaticCell::new();
    let peripherals = PERIPHERALS.init(peripherals);

    // SAFETY: TIMG0 and SW_INTERRUPT are each consumed exactly once here.
    let timg0 = TimerGroup::new(unsafe { take_peripheral(&peripherals.TIMG0) });
    let sw_ints =
        SoftwareInterruptControl::new(unsafe { take_peripheral(&peripherals.SW_INTERRUPT) });
    esp_rtos::start(timg0.timer0, sw_ints.software_interrupt0);

    let mut led = init_led(peripherals);

    info!("ready");
    led.color(0, 0, 100).unwrap();
    loop {
        led.on().unwrap();
        Timer::after(Duration::from_millis(500)).await;
        led.off().unwrap();
        Timer::after(Duration::from_millis(500)).await;
    }
}
