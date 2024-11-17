//! # Intel 8080 Space Invaders Emulator

/// Size of memory, including rom, ram and framebuffer (16kb)
pub const MEMORY_SIZE: usize = 0x4000;
/// Number of registers (B, C, D, E, H, L, F - flags, A - accumulator)
pub const NREGS: usize = 8;
/// Number of I/O ports
pub const NPORTS: usize = 8;
/// Display frames per second
pub const FPS: u32 = 60;
/// ~ 2 MHz CPU
pub const FREQ: u32 = 1_996_800;
/// Width of display in pixels
pub const DISPLAY_WIDTH: u32 = 224;
/// Height of display in pixels
pub const DISPLAY_HEIGHT: u32 = 256;

pub mod cpu;
pub mod emu;
pub mod utils;
