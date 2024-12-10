//! Emulator implementation using SDL2 for I/O

use std::{
    collections::HashMap,
    thread::sleep,
    time::{Duration, Instant},
};

use sdl2::{
    audio::{AudioSpecDesired, AudioSpecWAV},
    event::Event,
    keyboard::{Keycode, Scancode},
    pixels::{Color, PixelFormatEnum},
    rect::Point,
    render::BlendMode,
};

use crate::{cpu::Cpu, utils::get_bit, DISPLAY_HEIGHT, DISPLAY_WIDTH, FPS, FREQ};

#[cfg(test)]
mod tests;

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
    /// Options
    options: Options,
    /// Display frames per second
    fps: u32,
    /// Frequency of CPU, number of cycles per second
    freq: u32,
    /// Emulator should quit
    quit: bool,
}

impl Emu {
    pub fn new(cpu: Cpu, options: Options) -> Self {
        Emu {
            cpu,
            options,
            fps: FPS,
            freq: FREQ,
            quit: false,
        }
    }

    pub fn run(&mut self) {
        let sdl2 = sdl2::init().expect("Could not initialize SDL");
        let mut canvas = sdl2
            .video()
            .expect("Could not initialize video")
            .window(
                "Intel 8080 Space Invaders Emulator",
                DISPLAY_WIDTH * self.options.scale as u32,
                DISPLAY_HEIGHT * self.options.scale as u32,
            )
            .position_centered()
            .build()
            .expect("Could not open window")
            .into_canvas()
            .build()
            .expect("Could not create canvas");

        // Support alpha blending
        canvas.set_blend_mode(BlendMode::Blend);

        let background_color = Color::RGBA(
            ((self.options.background & 0xff0000) >> 16) as u8,
            ((self.options.background & 0x00ff00) >> 8) as u8,
            (self.options.background & 0x0000ff) as u8,
            ((self.options.background & 0xff000000) >> 24) as u8,
        );

        let foreground_color = Color::RGBA(
            ((self.options.color & 0xff0000) >> 16) as u8,
            ((self.options.color & 0x00ff00) >> 8) as u8,
            (self.options.color & 0x0000ff) as u8,
            ((self.options.color & 0xff000000) >> 24) as u8,
        );

        // Create an overlay grid for pixelation effect as a texture
        let texture_creator = canvas.texture_creator();
        let mut grid = texture_creator
            .create_texture_target(
                PixelFormatEnum::ARGB8888,
                DISPLAY_WIDTH * self.options.scale as u32,
                DISPLAY_HEIGHT * self.options.scale as u32,
            )
            .expect("Could not create texture");
        grid.set_blend_mode(BlendMode::Blend);

        canvas
            .with_texture_canvas(&mut grid, |c| {
                let mut grid_color = background_color;
                grid_color.a = 0x48;
                c.set_draw_color(grid_color);
                // Draw horizontal lines
                for y in 0..(DISPLAY_HEIGHT * self.options.scale as u32) {
                    if y % (self.options.scale as u32) == 0 {
                        c.draw_line(
                            (0, y as i32),
                            ((self.options.scale as u32 * DISPLAY_WIDTH) as i32, y as i32),
                        )
                        .expect("Could not draw horizontal lines on texture");
                    }
                }
                // Draw vertical lines
                for x in 0..(DISPLAY_WIDTH * self.options.scale as u32) {
                    if x % (self.options.scale as u32) == 0 {
                        c.draw_line(
                            (x as i32, 0),
                            (
                                x as i32,
                                (self.options.scale as u32 * DISPLAY_HEIGHT) as i32,
                            ),
                        )
                        .expect("Could not draw vertical lines on texture");
                    }
                }
            })
            .expect("Could not draw on texture");

        // The logical size is set to the size of the display. It makes it possible to draw single pixels at the correct position and get a scaled display automatically
        canvas
            .set_logical_size(DISPLAY_WIDTH, DISPLAY_HEIGHT)
            .expect("Could not set a logical size for canvas");

        println!(
            "{:?}, default_pixel_format: {:?}, scale: {:?}, logical_size: {:?}, output_size: {:?}, render_target_supported: {:?}",
            canvas.info(),
            canvas.default_pixel_format(),
            canvas.scale(),
            canvas.logical_size(),
            canvas.output_size().expect("Could not get output size from canvas"),
            canvas.render_target_supported()
        );

        let audio_subsystem = sdl2.audio().expect("Could not initialize audio");

        let mut wav_specs: HashMap<&str, AudioSpecWAV> = HashMap::new();
        for w in ["shot", "hit"] {
            wav_specs.insert(
                w,
                AudioSpecWAV::load_wav(format!("roms/{}.wav", w)).expect("Could not load wav"),
            );
        }

        let audio_spec = AudioSpecDesired {
            channels: Some(wav_specs["shot"].channels),
            freq: Some(wav_specs["shot"].freq),
            samples: None,
        };

        let queue = audio_subsystem
            .open_queue(None, &audio_spec)
            .expect("Could not create audio queue");
        queue.resume();

        let mut events = sdl2.event_pump().expect("Could not get event pump");
        let cycles_per_frame = self.freq / self.fps;

        let mut playing = [false; 9];

        while !self.quit {
            let t = Instant::now();

            // Handle input/controls
            self.handle_input(&mut events);

            // Run correct number of cycles, generate interrupts etc
            self.run_cpu(cycles_per_frame);

            // Handle sound
            let port3 = self.cpu.get_bus_out(3);
            let _port5 = self.cpu.get_bus_out(5);

            if get_bit(port3, 1) {
                if !playing[1] {
                    playing[1] = true;
                    queue
                        .queue_audio(wav_specs["shot"].buffer())
                        .expect("Could not queue audio");
                }
            } else if playing[1] {
                playing[1] = false;
            }

            if get_bit(port3, 3) {
                if !playing[2] {
                    playing[2] = true;
                    queue
                        .queue_audio(wav_specs["hit"].buffer())
                        .expect("Could not queue audio");
                }
            } else if playing[1] {
                playing[2] = false;
            }

            // Handle display
            if self.cpu.get_display_update() {
                canvas.set_draw_color(background_color);
                canvas.clear();
                canvas.set_draw_color(foreground_color);

                for y in 0..DISPLAY_HEIGHT {
                    for x in 0..DISPLAY_WIDTH {
                        if self.cpu.display(x, y) {
                            canvas
                                .draw_point(Point::new(x as i32, y as i32))
                                .expect("Could not draw pixel on display");
                        }
                    }
                }

                // Copy grid texture on top to give a slight pixelated look
                canvas
                    .copy(&grid, None, None)
                    .expect("Could not copy texture to canvas");

                canvas.present();

                self.cpu.set_display_update(false); // Cpu will set this to true whenever something changes on screen
            }

            self.sleep_before_next_frame(t);
        }
    }

    fn sleep_before_next_frame(&mut self, instant_at_start_of_frame: Instant) {
        let sleep_duration = (1_000_000_000_i64 / self.fps as i64)
            - instant_at_start_of_frame.elapsed().as_nanos() as i64;

        if sleep_duration >= 0 {
            sleep(Duration::new(0, sleep_duration as u32));
        }
    }

    fn run_cpu(&mut self, cycles_per_frame: u32) {
        let mut cycles: u32 = 0;

        let mut halfway = false;
        while cycles < cycles_per_frame {
            cycles += self.cpu.step();
            // Interrupts should happen in the middle of frame and at the end
            if !halfway && (cycles > cycles_per_frame / 2) {
                cycles += self.cpu.interrupt(1);
                halfway = true;
            }
        }
        self.cpu.interrupt(2);
    }

    fn handle_input(&mut self, events: &mut sdl2::EventPump) {
        for event in events.poll_iter() {
            match event {
                // Quit
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => self.quit = true,
                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => {
                    if let Some((port, bit)) = self.keymap(scancode) {
                        self.cpu.set_bus_in_bit(port, bit, true);
                    }
                }
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => {
                    if let Some((port, bit)) = self.keymap(scancode) {
                        self.cpu.set_bus_in_bit(port, bit, false);
                    }
                }
                _ => {}
            }
        }
    }

    /// Match MAME controls somewhat
    fn keymap(&self, scancode: Scancode) -> Option<(usize, u8)> {
        match scancode {
            Scancode::T => Some((2, 2)),     // Tilt
            Scancode::Num5 => Some((1, 0)),  // Credit
            Scancode::Num1 => Some((1, 2)),  // P1 Start
            Scancode::Num2 => Some((1, 1)),  // P2 Start
            Scancode::LCtrl => Some((1, 4)), // P1 Fire
            Scancode::Left => Some((1, 5)),  // P1 Left
            Scancode::Right => Some((1, 6)), // P1 Left
            Scancode::A => Some((2, 4)),     // P2 Fire
            Scancode::D => Some((2, 5)),     // P2 Left
            Scancode::G => Some((2, 6)),     // P3 Left
            _ => None,
        }
    }
}
