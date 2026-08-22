#![no_std]
#![no_main]

use esp_backtrace as _;
use rmk::macros::rmk_keyboard;
use rmk::core_traits::Runnable;

type DisplayType = ::rmk::display::ssd1306::Ssd1306Async<
    ::display_interface_i2c::I2CInterface<
        ::esp_hal::i2c::master::I2c<'static, ::esp_hal::Async>,
    >,
    ::rmk::display::ssd1306::prelude::DisplaySize128x64,
    ::rmk::display::ssd1306::mode::BufferedGraphicsModeAsync<
        ::rmk::display::ssd1306::prelude::DisplaySize128x64,
    >,
>;

struct KeyboardLayoutRenderer;

impl ::rmk::display::DisplayRenderer<::embedded_graphics::pixelcolor::BinaryColor> for KeyboardLayoutRenderer {
    fn render<D: ::embedded_graphics::prelude::DrawTarget<Color = ::embedded_graphics::pixelcolor::BinaryColor>>(
        &mut self,
        ctx: &::rmk::display::RenderContext,
        display: &mut D,
    ) {
        use ::embedded_graphics::{
            prelude::*,
            primitives::{Line, PrimitiveStyle, Rectangle},
            text::{Baseline, Text},
            mono_font::{ascii::FONT_6X10, MonoTextStyle},
            pixelcolor::BinaryColor,
        };

        // Clear the screen
        display.clear(BinaryColor::Off).ok();

        // 1. Draw top header separator (y = 15)
        let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
        Line::new(Point::new(0, 15), Point::new(127, 15))
            .into_styled(line_style)
            .draw(display)
            .ok();

        // 2. Draw header in yellow zone
        let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        Text::with_baseline("3x3 MACROPAD", Point::new(4, 3), text_style, Baseline::Top)
            .draw(display)
            .ok();

        // Show BLE connection status in the top right
        {
            let conn_text = match ctx.ble_status.state {
                ::rmk_types::ble::BleState::Advertising => "Pair",
                ::rmk_types::ble::BleState::Connected => "Conn",
                ::rmk_types::ble::BleState::Inactive => "USB",
            };
            let mut status_buf: ::rmk::heapless::String<12> = ::rmk::heapless::String::new();
            let _ = ::core::fmt::write(&mut status_buf, format_args!("P{}:{}", ctx.ble_status.profile, conn_text));
            let x = 124 - (status_buf.len() as i32 * 6);
            Text::with_baseline(&status_buf, Point::new(x, 3), text_style, Baseline::Top)
                .draw(display)
                .ok();
        }

        // 3. Draw 3x3 Key Grid in the blue area (y: 16..63, center x: 28..100)
        Rectangle::new(Point::new(28, 16), Size::new(72, 48))
            .into_styled(line_style)
            .draw(display)
            .ok();

        // Vertical separators
        Line::new(Point::new(52, 16), Point::new(52, 63))
            .into_styled(line_style)
            .draw(display)
            .ok();
        Line::new(Point::new(76, 16), Point::new(76, 63))
            .into_styled(line_style)
            .draw(display)
            .ok();

        // Horizontal separators
        Line::new(Point::new(28, 32), Point::new(100, 32))
            .into_styled(line_style)
            .draw(display)
            .ok();
        Line::new(Point::new(28, 48), Point::new(100, 48))
            .into_styled(line_style)
            .draw(display)
            .ok();

        // Layer labels mapping to the current active layer
        let labels = match ctx.layer {
            1 => [
                ["Ply", "V+", "Nxt"],
                ["Mut", "V-", "Prv"],
                ["", "", "L1"],
            ],
            _ => [
                ["7", "8", "9"],
                ["4", "5", "6"],
                ["1", "2", "L1"],
            ],
        };

        for row in 0..3 {
            for col in 0..3 {
                let label = labels[row][col];
                if !label.is_empty() {
                    let char_width = 6;
                    let label_width = label.len() as i32 * char_width;
                    let x_offset = (24 - label_width) / 2;
                    let x = 28 + (col as i32) * 24 + x_offset;
                    let y = 16 + (row as i32) * 16 + 3;
                    Text::with_baseline(label, Point::new(x, y), text_style, Baseline::Top)
                        .draw(display)
                        .ok();
                }
            }
        }

        // 4. Draw Info panel on the left (x: 0..28)
        Text::with_baseline("Lyr", Point::new(4, 22), text_style, Baseline::Top)
            .draw(display)
            .ok();
        
        let mut lyr_buf: ::rmk::heapless::String<4> = ::rmk::heapless::String::new();
        let _ = ::core::fmt::write(&mut lyr_buf, format_args!("{}", ctx.layer));
        let lyr_x = (28 - (lyr_buf.len() as i32 * 6)) / 2;
        Text::with_baseline(&lyr_buf, Point::new(lyr_x, 38), text_style, Baseline::Top)
            .draw(display)
            .ok();

        // 5. Draw Info panel on the right (x: 100..128)
        Text::with_baseline("WPM", Point::new(104, 22), text_style, Baseline::Top)
            .draw(display)
            .ok();

        let mut wpm_buf: ::rmk::heapless::String<4> = ::rmk::heapless::String::new();
        let _ = ::core::fmt::write(&mut wpm_buf, format_args!("{}", ctx.wpm));
        let wpm_x = 100 + (28 - (wpm_buf.len() as i32 * 6)) / 2;
        Text::with_baseline(&wpm_buf, Point::new(wpm_x, 38), text_style, Baseline::Top)
            .draw(display)
            .ok();
    }
}

#[embassy_executor::task]
async fn display_task(display: DisplayType) {
    let mut oled = ::rmk::display::DisplayProcessor::with_renderer(display, KeyboardLayoutRenderer);
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
