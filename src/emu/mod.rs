//! Emulator implementation using SDL2 for I/O

use sdl2::{
    pixels::{Color, PixelFormatEnum},
    render::BlendMode,
};

use crate::cpu::{Cpu, DISPLAY_HEIGHT, DISPLAY_WIDTH};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Options {
    pub scale: u8,
    pub color: u32,
    pub background: u32,
}

pub struct Emu {
    cpu: Cpu,
    scale: u8,
    color: u32,
    background: u32,
}

impl Emu {
    pub fn new(cpu: Cpu, options: Options) -> Self {
        Emu {
            cpu,
            scale: options.scale,
            color: options.color,
            background: options.background,
        }
    }

    pub fn run(&mut self) {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();

        let window = video
            .window(
                "Chip8 Emulator",
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
            self.cpu.step();
        }
    }
}
