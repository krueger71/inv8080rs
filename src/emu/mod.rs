//! Emulator implementation using SDL2 for I/O

use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use sdl2::{
    event::Event,
    keyboard::{Keycode, Scancode},
    pixels::{Color, PixelFormatEnum},
    render::BlendMode,
};

use crate::cpu::Cpu;

#[cfg(test)]
mod tests;

pub const DISPLAY_WIDTH: u32 = 256;
pub const DISPLAY_HEIGHT: u32 = 224;
pub const FPS: u32 = 60;
pub const FREQ: u32 = 1_996_800_000;

/// Options for the emulator
#[derive(Debug)]
pub struct Options {
    /// Scale of the display
    pub scale: u8,
    /// Foreground color
    pub color: u32,
    /// Background color
    pub background: u32,
}

/// The state of the emulator
pub struct Emu {
    /// CPU-model
    cpu: Cpu,
    /// Scale of the display
    scale: u8,
    /// Foreground color of display
    color: u32,
    /// Background color of display
    background: u32,
    /// Display frames per second
    fps: u32,
    /// Frequency of CPU, number of cycles per second
    freq: u32,
}

impl Emu {
    pub fn new(cpu: Cpu, options: Options) -> Self {
        Emu {
            cpu,
            scale: options.scale,
            color: options.color,
            background: options.background,
            fps: FPS,
            freq: FREQ,
        }
    }

    pub fn run(&mut self) {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();

        let window = video
            .window(
                "Intel 8080 Space Invaders Emulator",
                DISPLAY_WIDTH * self.scale as u32,
                DISPLAY_HEIGHT * self.scale as u32,
            )
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .accelerated()
            .build()
            .unwrap();

        // Support alpha blending
        canvas.set_blend_mode(BlendMode::Blend);

        let background_color = Color::RGBA(
            ((self.background & 0xff0000) >> 16) as u8,
            ((self.background & 0x00ff00) >> 8) as u8,
            (self.background & 0x0000ff) as u8,
            ((self.background & 0xff000000) >> 24) as u8,
        );

        let foreground_color = Color::RGBA(
            ((self.color & 0xff0000) >> 16) as u8,
            ((self.color & 0x00ff00) >> 8) as u8,
            (self.color & 0x0000ff) as u8,
            ((self.color & 0xff000000) >> 24) as u8,
        );

        // Create a grid as a texture
        let texture_creator = canvas.texture_creator();
        let mut grid = texture_creator
            .create_texture_target(
                PixelFormatEnum::ARGB8888,
                DISPLAY_WIDTH * self.scale as u32,
                DISPLAY_HEIGHT * self.scale as u32,
            )
            .unwrap();
        grid.set_blend_mode(BlendMode::Blend);

        canvas
            .with_texture_canvas(&mut grid, |c| {
                let mut grid_color = background_color;
                grid_color.a = 0x1d;
                c.set_draw_color(grid_color);
                // Draw horizontal lines
                for y in 0..(DISPLAY_HEIGHT * self.scale as u32) {
                    if y % (self.scale as u32) == 0 {
                        c.draw_line(
                            (0, y as i32),
                            ((self.scale as u32 * DISPLAY_WIDTH) as i32, y as i32),
                        )
                        .unwrap();
                    }
                }
                // Draw vertical lines
                for x in 0..(DISPLAY_WIDTH * self.scale as u32) {
                    if x % (self.scale as u32) == 0 {
                        c.draw_line(
                            (x as i32, 0),
                            (x as i32, (self.scale as u32 * DISPLAY_HEIGHT) as i32),
                        )
                        .unwrap();
                    }
                }
            })
            .unwrap();

        // The logical size is set to the size of the Chip8 display. It makes it possible to draw single pixels at the correct position and get a scaled display automatically
        canvas
            .set_logical_size(DISPLAY_WIDTH, DISPLAY_HEIGHT)
            .unwrap();

        println!(
            "{:?}, default_pixel_format: {:?}, scale: {:?}, logical_size: {:?}, output_size: {:?}, render_target_supported: {:?}",
            canvas.info(),
            canvas.default_pixel_format(),
            canvas.scale(),
            canvas.logical_size(),
            canvas.output_size().unwrap(),
            canvas.render_target_supported()
        );

        let audio_subsystem = sdl.audio().unwrap();

        println!(
            "{} {:?}",
            audio_subsystem.current_audio_driver(),
            audio_subsystem
        );

        let mut events = sdl.event_pump().unwrap();

        'main: loop {
            let t = Instant::now();

            // Handle input
            for event in events.poll_iter() {
                match event {
                    // Quit
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'main,
                    Event::KeyDown {
                        scancode: Some(scancode),
                        ..
                    } => {
                        if let Some(keycode) = self.keymap(scancode) {
                            // Handle the key down
                            #[cfg(debug_assertions)]
                            eprintln!("Key {:0x} down", keycode);
                        }
                    }
                    Event::KeyUp {
                        scancode: Some(scancode),
                        ..
                    } => {
                        if let Some(keycode) = self.keymap(scancode) {
                            // Handle the key up
                            #[cfg(debug_assertions)]
                            eprintln!("Key {:0x} up", keycode);
                        }
                    }
                    _ => {}
                }
            }

            // Run correct number of cycles, generate interrupts etc
            self.cpu.step();

            // Copy grid texture on top (could be configurable)
            canvas.copy(&grid, None, None).unwrap();

            canvas.present();

            let sleep_duration =
                (1_000_000_000_i64 / self.fps as i64) - t.elapsed().as_nanos() as i64;

            #[cfg(debug_assertions)]
            eprintln!("Sleeping {} ns", sleep_duration);

            if sleep_duration >= 0 {
                sleep(Duration::new(0, sleep_duration as u32));
            }
        }
    }

    fn keymap(&self, scancode: Scancode) -> Option<usize> {
        match scancode {
            Scancode::Num1 => Some(1),
            Scancode::Num2 => Some(2),
            Scancode::Num3 => Some(3),
            Scancode::Num4 => Some(0xC),
            Scancode::Q => Some(4),
            Scancode::W => Some(5),
            Scancode::E => Some(6),
            Scancode::R => Some(0xD),
            Scancode::A => Some(7),
            Scancode::S => Some(8),
            Scancode::D => Some(9),
            Scancode::F => Some(0xE),
            Scancode::Z => Some(0xA),
            Scancode::X => Some(0),
            Scancode::C => Some(0xB),
            Scancode::V => Some(0xF),
            _ => None,
        }
    }
}
