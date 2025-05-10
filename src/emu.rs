//! Emulator implementation using SDL3 for I/O

use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use sdl3::{
    audio::{AudioSpec, AudioSpecWAV, AudioStreamOwner},
    event::Event,
    keyboard::{Keycode, Scancode},
    pixels::{Color, PixelFormat},
    rect::{Point, Rect},
    render::BlendMode,
    sys::pixels::{SDL_PixelFormat, SDL_PIXELFORMAT_ARGB8888},
};

use crate::{cpu::Cpu, DISPLAY_HEIGHT, DISPLAY_WIDTH, FPS, FREQ};

#[cfg(test)]
mod tests;

/// Options for the emulator
#[derive(Debug)]
pub struct Options {
    /// Scale of the display
    pub scale: u32,
    /// Foreground color
    pub color: u32,
    /// Background color
    pub background: u32,
    /// Color of top overlay
    pub top: u32,
    /// Color of bottom overlay
    pub bottom: u32,
}

type SoundState<'a> = (
    u8,
    u8,
    &'a str,
    Option<AudioStreamOwner>,
    Option<AudioSpecWAV>,
    bool,
);
/// The state of the emulator
pub struct Emu<'a> {
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
    /// SDL Canvas<Window>
    canvas: sdl3::render::Canvas<sdl3::video::Window>,
    /// SDL Event Pump
    event_pump: sdl3::EventPump,
    /// Sound channels
    sounds: [SoundState<'a>; 10],
}

const PIXEL_FORMAT: SDL_PixelFormat = SDL_PIXELFORMAT_ARGB8888;

impl Emu<'_> {
    pub fn new(cpu: Cpu, options: Options) -> Self {
        let sdl = sdl3::init().expect("Could not initialize SDL");
        let video = sdl.video().expect("Could not initialize video");
        let mut canvas = video
            .window(
                "Intel 8080 Space Invaders Emulator",
                DISPLAY_WIDTH * options.scale,
                DISPLAY_HEIGHT * options.scale,
            )
            .position_centered()
            .build()
            .expect("Could not initialize window")
            .into_canvas();

        // Support alpha blending
        canvas.set_blend_mode(BlendMode::Blend);
        let audio = sdl.audio().expect("Could not initialize audio");

        let mut sounds: [SoundState; 10] = [
            (3, 0, "ufo", None, None, false),  // Ufo movement
            (3, 1, "shot", None, None, false), // Player shoots
            (3, 2, "die", None, None, false),  // Player dies
            (3, 3, "hit", None, None, false),  // Invader hit
            (3, 4, "xp", None, None, false),   // Extended play?
            // (3, 5, "amp"),  // Amp enable, turn on/off all sounds?
            (5, 0, "fleet1", None, None, false),  // Fleet 1
            (5, 1, "fleet2", None, None, false),  // Fleet 2
            (5, 2, "fleet1", None, None, false),  // Fleet 3
            (5, 3, "fleet2", None, None, false),  // Fleet 4
            (5, 4, "ufo_hit", None, None, false), // Fleet 4
        ];

        let audio_spec = AudioSpec {
            channels: Some(1),
            freq: Some(11025),
            format: Some(sdl3::audio::AudioFormat::U8),
        };

        let audio_device = audio
            .open_playback_device(&audio_spec)
            .expect("Could not open audio device");
        let stream1 = audio_device.open_device_stream(Some(&audio_spec)).unwrap();

        // for (_, _, w, queue, wav, _) in &mut sounds {
        //     *wav =Some(
        //         AudioSpecWAV::load_wav(format!("assets/{}.wav", w)).expect("Could not load wav"));
        //     let aso = audio_device.open_device_stream(Some(&audio_spec)).unwrap();
        //     *queue = Some(aso);
        // }

        let event_pump = sdl.event_pump().expect("Could not initialize event pump");
        Emu {
            cpu,
            options,
            fps: FPS,
            freq: FREQ,
            quit: false,
            canvas,
            event_pump,
            sounds,
        }
    }

    pub fn run(&mut self) {
        let pixel_format =
            PixelFormat::try_from(PIXEL_FORMAT).expect("Could not convert pixel format enum");

        let background_color = Color::from_u32(&pixel_format, self.options.background);
        let foreground_color = Color::from_u32(&pixel_format, self.options.color);
        let top_color = Color::from_u32(&pixel_format, self.options.top);
        let bottom_color = Color::from_u32(&pixel_format, self.options.bottom);

        // Create an overlay grid for pixelation effect as a texture
        let texture_creator = self.canvas.texture_creator();
        let mut grid_texture = texture_creator
            .create_texture_target(
                pixel_format,
                DISPLAY_WIDTH * self.options.scale,
                DISPLAY_HEIGHT * self.options.scale,
            )
            .expect("Could not create grid texture");
        grid_texture.set_blend_mode(BlendMode::Blend);
        grid_texture.set_scale_mode(sdl3::render::ScaleMode::Nearest);

        self.canvas
            .with_texture_canvas(&mut grid_texture, |c| {
                // Draw horizontal lines
                let mut grid_color = background_color;
                grid_color.a = 0x20;
                c.set_draw_color(grid_color);
                for y in 0..(DISPLAY_HEIGHT * self.options.scale) {
                    if y % (self.options.scale) == 0 {
                        c.draw_line(
                            (0, y as i32),
                            ((self.options.scale * DISPLAY_WIDTH) as i32, y as i32),
                        )
                        .expect("Could not draw horizontal lines on texture");
                    }
                }

                // Draw vertical lines
                grid_color.a = 0x7;
                c.set_draw_color(grid_color);
                for x in 0..(DISPLAY_WIDTH * self.options.scale) {
                    if x % (self.options.scale) == 0 {
                        c.draw_line(
                            (x as i32, 0),
                            (x as i32, (self.options.scale * DISPLAY_HEIGHT) as i32),
                        )
                        .expect("Could not draw vertical lines on texture");
                    }
                }
            })
            .expect("Could not draw on texture");

        let mut overlay_texture = texture_creator
            .create_texture_target(pixel_format, DISPLAY_WIDTH, DISPLAY_HEIGHT)
            .expect("Could not create game texture");
        overlay_texture.set_blend_mode(BlendMode::Mul);
        overlay_texture.set_scale_mode(sdl3::render::ScaleMode::Nearest);

        self.canvas
            .with_texture_canvas(&mut overlay_texture, |c| {
                c.set_draw_color(top_color);
                c.fill_rect(Rect::new(0, 32, DISPLAY_WIDTH, 32))
                    .expect("Could not fill top rect");
                c.set_draw_color(bottom_color);
                c.fill_rect(Rect::new(0, 184, DISPLAY_WIDTH, 56))
                    .expect("Could not fill bottom rect");
                c.fill_rect(Rect::new(16, 240, 120, 15))
                    .expect("Could not fill remaining ship area");
            })
            .expect("Could not draw overlay");

        let mut game_texture = texture_creator
            .create_texture_target(pixel_format, DISPLAY_WIDTH, DISPLAY_HEIGHT)
            .expect("Could not create game texture");
        game_texture.set_blend_mode(BlendMode::Blend);
        game_texture.set_scale_mode(sdl3::render::ScaleMode::Nearest);

        println!("{:?}", self.canvas.renderer_name);

        let cycles_per_frame = self.freq / self.fps;

        while !self.quit {
            let t = Instant::now();

            // Handle input/controls
            self.handle_input();

            // Run correct number of cycles, generate interrupts etc
            self.run_cpu(cycles_per_frame);

            // Handle sound
            // for (port, bit, _, queue, wav, playing) in &mut self.sounds {
            //     if get_bit(self.cpu.get_bus_out((*port).into()), *bit) {
            //         if !(*playing) {
            //             *playing = true;
            //             let q = queue.as_ref().expect("No audio queue for sound");
            //             let w = wav.as_ref().expect("No audio content for sound");
            //             q.queue_audio(w.buffer()).expect("Could not queue audio");
            //             q.resume();
            //         }
            //     } else if *playing {
            //         *playing = false;
            //     }
            // }

            // Handle display
            if self.cpu.get_display_update() {
                self.canvas
                    .with_texture_canvas(&mut game_texture, |c| {
                        c.set_draw_color(background_color);
                        c.clear();

                        for (color, range) in [(foreground_color, 0..DISPLAY_HEIGHT)] {
                            c.set_draw_color(color);
                            for y in range {
                                for x in 0..DISPLAY_WIDTH {
                                    if self.cpu.display(x, y) {
                                        c.draw_point(Point::new(x as i32, y as i32))
                                            .expect("Could not draw pixel on display");
                                    }
                                }
                            }
                        }
                    })
                    .expect("Could not render game frame");

                self.canvas
                    .copy(&game_texture, None, None)
                    .expect("Could not copy game texture to canvas");
                // Copy grid texture on top to give a slight pixelated look
                self.canvas
                    .copy(&grid_texture, None, None)
                    .expect("Could not copy grid texture to canvas");
                // Copy overlay texture at last
                self.canvas
                    .copy(&overlay_texture, None, None)
                    .expect("Could not copy overlay texture to canvas");

                self.canvas.present();

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
        for i in [1, 2] {
            let mut cycles: u32 = 0;

            while cycles < cycles_per_frame / 2 {
                cycles += self.cpu.step();
            }
            self.cpu.interrupt(i);
        }
    }

    fn handle_input(&mut self) {
        for event in self.event_pump.poll_iter() {
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
                    if let Some((port, bit)) = Self::keymap(scancode) {
                        self.cpu.set_bus_in_bit(port, bit, true);
                    }
                }
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => {
                    if let Some((port, bit)) = Self::keymap(scancode) {
                        self.cpu.set_bus_in_bit(port, bit, false);
                    }
                }
                _ => {}
            }
        }
    }

    /// Match MAME controls somewhat
    fn keymap(scancode: Scancode) -> Option<(usize, u8)> {
        match scancode {
            Scancode::T => Some((2, 2)),     // Tilt
            Scancode::_5 => Some((1, 0)),    // Add Credit
            Scancode::_1 => Some((1, 2)),    // P1 Start
            Scancode::_2 => Some((1, 1)),    // P2 Start
            Scancode::LCtrl => Some((1, 4)), // P1 Fire
            Scancode::Left => Some((1, 5)),  // P1 Left
            Scancode::Right => Some((1, 6)), // P1 Right
            Scancode::A => Some((2, 4)),     // P2 Fire
            Scancode::D => Some((2, 5)),     // P2 Left
            Scancode::G => Some((2, 6)),     // P2 Right
            _ => None,
        }
    }
}
