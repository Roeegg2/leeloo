use ee::Cpu;

fn main() {
    let mut cpu = Cpu::new();
    let bits = 0xa0000000_u32;
    loop {
        // read bits
        cpu.update_pc();
        cpu.exec(bits);
    }

    // let val: u64 = 0xffff_0000_0000_0000;
    // println!("this is org val: {val}");
    // println!("this is CAST val: {}", val as i64);
}
