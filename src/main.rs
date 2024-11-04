use inv8080rs::cpu::Cpu;

fn main() {
    let program = std::fs::read("roms/invaders.rom").expect("could not read file");
    let mut cpu = Cpu::new(program);

    loop {
        cpu.step();
    }
}
