use inv8080rs::{
    cpu::Cpu,
    emu::{Emu, Options},
};

fn main() {
    let program = std::fs::read("roms/invaders.rom").expect("could not read file");
    let mut emu = Emu::new(
        Cpu::new(program),
        Options {
            scale: 3,               // 256x224 -> 768x672 display size
            color: 0xff00ff00,      // ARGB8888 -> bright green
            background: 0xff000000, // ARGB8888 -> black
        },
    );

    emu.run();
}
