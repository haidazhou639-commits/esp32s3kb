#![no_std]
#![no_main]

use esp_backtrace as _;
use rmk::macros::rmk_keyboard;

#[rmk_keyboard]
mod keyboard {}
