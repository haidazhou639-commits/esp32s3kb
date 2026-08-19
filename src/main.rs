#![no_std]
#![no_main]

use esp_backtrace as _;
use rmk::macros::rmk_keyboard;

type DisplayType = ::rmk::display::ssd1306::Ssd1306Async<
    ::display_interface_i2c::I2CInterface<
        ::esp_hal::i2c::master::I2c<'static, ::esp_hal::Async>,
    >,
    ::rmk::display::ssd1306::prelude::DisplaySize128x64,
    ::rmk::display::ssd1306::mode::BufferedGraphicsMode<
        ::rmk::display::ssd1306::prelude::DisplaySize128x64,
    >,
>;

#[embassy_executor::task]
async fn display_task(display: DisplayType) {
    let mut oled = ::rmk::display::DisplayProcessor::with_renderer(display, ::rmk::display::OledRenderer::default());
    oled.run().await;
}

#[rmk_keyboard]
mod keyboard {
    #[Override(chip_init)]
    fn init_chip() {
        ::esp_println::logger::init_logger_from_env();
        let p = ::esp_hal::init(::esp_hal::Config::default().with_cpu_clock(::esp_hal::clock::CpuClock::max()));
        ::esp_alloc::heap_allocator!(size: 72 * 1024);
        let timg0 = ::esp_hal::timer::timg::TimerGroup::new(p.TIMG0);
        ::esp_rtos::start(timg0.timer0, p.FROM_CPU_INTR0);
        let _trng_source = ::esp_hal::rng::TrngSource::new(p.RNG, p.ADC1);
        let connector = ::esp_radio::ble::controller::BleConnector::new(p.BT, Default::default()).unwrap();
        let ble_controller: ::bt_hci::controller::ExternalController<_, 64> = ::bt_hci::controller::ExternalController::new(connector);
        let ble_addr = [0x7e, 0xfe, 0x73, 0x05, 0x66, 0xe3];

        // Initialize I2C for display in Async mode
        let i2c = ::esp_hal::i2c::master::I2c::new(
            p.I2C0,
            ::esp_hal::i2c::master::Config::default()
        )
        .unwrap()
        .with_sda(p.GPIO10)
        .with_scl(p.GPIO9)
        .into_async();

        let display_interface = ::rmk::display::ssd1306::I2CDisplayInterface::new_custom_address(i2c, 0x3c);
        let display = ::rmk::display::ssd1306::Ssd1306Async::new(
            display_interface,
            ::rmk::display::ssd1306::prelude::DisplaySize128x64,
            ::rmk::display::ssd1306::prelude::DisplayRotation::Rotate0,
        ).into_buffered_graphics_mode();

        // Spawn display task
        _s.spawn(display_task(display).unwrap());
    }

    #[Override(usb)]
    fn init_usb() {
        static mut EP_MEMORY: [u8; 1024] = [0; 1024];
        let usb = ::esp_hal::usb::otg::Usb::new_fs(p.USB_FS, p.GPIO20, p.GPIO19);
        let usb_config = ::esp_hal::usb::otg::embassy_usb_device::Config::default();
        ::esp_hal::usb::otg::embassy_usb_device::Driver::new(usb, unsafe { &mut *core::ptr::addr_of_mut!(EP_MEMORY) }, usb_config)
    }
}
