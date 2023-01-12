use inv8080rs::Cpu;

fn main() {
    let program = std::fs::read("roms/invaders.rom").expect("could not read file");
    let size = program.len();
    let mut cpu = Cpu::new(program);

    for _ in 0..size {
        cpu.step();
    }

    println!("Exiting");
}
