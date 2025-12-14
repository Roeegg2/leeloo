use ee::Cpu;

fn main() {
    let mut cpu = Cpu::new();
    let bits = 0xa0000000_u32;
    loop {
        // read bits
        cpu.update_pc();
        cpu.exec(bits);
    }
}
