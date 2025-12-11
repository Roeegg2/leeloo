use ee::Cpu;

fn main() {
    let mut cpu = Cpu::new();
    let bits = 0xa0000000_u32;
    cpu.exec(bits);
}
