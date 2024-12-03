//! # Intel 8080 Space Invaders Emulator

use std::ops::RangeInclusive;

/// Size of memory, including rom, ram and framebuffer (16kb)
pub const MEMORY_SIZE: usize = 0x4000; // ?
/// Memory total range
pub const MEMORY: RangeInclusive<usize> = 0..=0x3FFF;
/// ROM memory range
pub const ROM: RangeInclusive<usize> = 0..=0x1FFF;
/// RAM memory range
pub const RAM: RangeInclusive<usize> = 0x2000..=0x3FFF;
/// Stack pointer memory range (really should be no more than 16 levels), grows downward in memory
pub const STACK: RangeInclusive<usize> = 0x2301..=0x2400;
/// Framebuffer memory range
pub const FRAMEBUFFER: RangeInclusive<usize> = 0x2400..=0x3FFF;

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
