#![feature(io_error_other)]
#![feature(is_sorted)]
#![feature(fn_traits)]
#![feature(const_fn_floating_point_arithmetic)]

mod app;
mod calibration_module;
mod camera_module;
mod csv;
mod fitting;
mod log;
mod spectrum_module;
mod tracer_module;

pub use app::SpeckApp;

pub const SMALLEST_WAVELENGTH: u16 = 380;
pub const LARGEST_WAVELENGTH: u16 = 750;
