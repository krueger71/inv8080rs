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
            color: 0xffeeeeee,      // ARGB8888 -> CRT phosphor green (33ff00), "white" (eeeeee)
            background: 0xff111111, // ARGB8888 -> almost black (111111)
            top: 0xffff0000,
            bottom: 0xff33ff00,
        },
    );

    emu.run();
}
