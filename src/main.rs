use inv8080rs::{
    cpu::Cpu,
    emu::{Emu, Options},
};

fn main() {
    let program = std::fs::read("assets/invaders.rom").expect("could not read file");
    let mut emu = Emu::new(
        Cpu::new(program),
        Options {
            scale: 3, // scale width and height by
            color: 0xffffffff,
            background: 0xff000000,
            top: 0xffff0000,
            bottom: 0xff00ff00,
        },
    );

    emu.run();
}
