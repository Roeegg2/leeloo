pub struct Cpu {
    gprs: [u128; 32],
    pc: u32,
    hi: u128,
    lo: u128,
    sa: u64,
}

impl Cpu {
    const OPCODE_SPECIAL: u32 = 0b000000;

    const SPECIAL_FUNCT_SLL: u32 = 0b000000; // 0x00
    const SPECIAL_FUNCT_SRL: u32 = 0b000010; // 0x02
    const SPECIAL_FUNCT_SRA: u32 = 0b000011; // 0x03
    const SPECIAL_FUNCT_SLLV: u32 = 0b000100; // 0x04
    const SPECIAL_FUNCT_SRLV: u32 = 0b000110; // 0x06
    const SPECIAL_FUNCT_SRAV: u32 = 0b000111; // 0x07
    const SPECIAL_FUNCT_JR: u32 = 0b001000; // 0x08
    const SPECIAL_FUNCT_JALR: u32 = 0b001001; // 0x09
    const SPECIAL_FUNCT_MOVZ: u32 = 0b001010; // 0x0A
    const SPECIAL_FUNCT_MOVN: u32 = 0b001011; // 0x0B
    const SPECIAL_FUNCT_SYSCALL: u32 = 0b001100; // 0x0C
    const SPECIAL_FUNCT_BREAK: u32 = 0b001101; // 0x0D
    const SPECIAL_FUNCT_SYNC: u32 = 0b001111; // 0x0F
    const SPECIAL_FUNCT_MFHI: u32 = 0b010000; // 0x10
    const SPECIAL_FUNCT_MTHI: u32 = 0b010001; // 0x11
    const SPECIAL_FUNCT_MFLO: u32 = 0b010010; // 0x12
    const SPECIAL_FUNCT_MTLO: u32 = 0b010011; // 0x13
    const SPECIAL_FUNCT_DSLLV: u32 = 0b010100; // 0x14
    const SPECIAL_FUNCT_DSRLV: u32 = 0b010110; // 0x16
    const SPECIAL_FUNCT_DSRAV: u32 = 0b010111; // 0x17
    const SPECIAL_FUNCT_MULT: u32 = 0b011000; // 0x18
    const SPECIAL_FUNCT_MULTU: u32 = 0b011001; // 0x19
    const SPECIAL_FUNCT_DIV: u32 = 0b011010; // 0x1A
    const SPECIAL_FUNCT_DIVU: u32 = 0b011011; // 0x1B
    const SPECIAL_FUNCT_ADD: u32 = 0b100000; // 0x20
    const SPECIAL_FUNCT_ADDU: u32 = 0b100001; // 0x21
    const SPECIAL_FUNCT_SUB: u32 = 0b100010; // 0x22
    const SPECIAL_FUNCT_SUBU: u32 = 0b100011; // 0x23
    const SPECIAL_FUNCT_AND: u32 = 0b100100; // 0x24
    const SPECIAL_FUNCT_OR: u32 = 0b100101; // 0x25
    const SPECIAL_FUNCT_XOR: u32 = 0b100110; // 0x26
    const SPECIAL_FUNCT_NOR: u32 = 0b100111; // 0x27
    const SPECIAL_FUNCT_MFSA: u32 = 0b101000; // 0x28
    const SPECIAL_FUNCT_MTSA: u32 = 0b101001; // 0x29
    const SPECIAL_FUNCT_SLT: u32 = 0b101010; // 0x2A
    const SPECIAL_FUNCT_SLTU: u32 = 0b101011; // 0x2B
    const SPECIAL_FUNCT_DADD: u32 = 0b101100; // 0x2C
    const SPECIAL_FUNCT_DADDU: u32 = 0b101101; // 0x2D
    const SPECIAL_FUNCT_DSUB: u32 = 0b101110; // 0x2E
    const SPECIAL_FUNCT_DSUBU: u32 = 0b101111; // 0x2F
    const SPECIAL_FUNCT_TGE: u32 = 0b110000; // 0x30
    const SPECIAL_FUNCT_TGEU: u32 = 0b110001; // 0x31
    const SPECIAL_FUNCT_TLT: u32 = 0b110010; // 0x32
    const SPECIAL_FUNCT_TLTU: u32 = 0b110011; // 0x33
    const SPECIAL_FUNCT_TEQ: u32 = 0b110100; // 0x34
    const SPECIAL_FUNCT_TNE: u32 = 0b110110; // 0x36
    const SPECIAL_FUNCT_DSLL: u32 = 0b111000; // 0x38
    const SPECIAL_FUNCT_DSRL: u32 = 0b111010; // 0x3A
    const SPECIAL_FUNCT_DSRA: u32 = 0b111011; // 0x3B
    const SPECIAL_FUNCT_DSLL32: u32 = 0b111100; // 0x3C
    const SPECIAL_FUNCT_DSRL32: u32 = 0b111110; // 0x3E
    const SPECIAL_FUNCT_DSRA32: u32 = 0b111111; // 0x3F

    pub fn new() -> Self {
        Cpu {
            gprs: [0; 32],
            pc: 0,
            hi: 0,
            lo: 0,
            sa: 0,
        }
    }

    pub fn exec(&mut self, raw: u32) {
        let opcode = (raw >> 26) & 0b111111;
        match opcode {
            Self::OPCODE_SPECIAL => self.handle_special(raw),
            _ => unimplemented!("Opcode {:06b} not implemented", opcode),
        }
    }

    fn handle_special(&mut self, raw: u32) {
        let funct = raw & 0b111111;
        match funct {
            Self::SPECIAL_FUNCT_SLL => self.do_sll(raw),
            Self::SPECIAL_FUNCT_SRL => self.do_srl(raw),
            Self::SPECIAL_FUNCT_SRA => self.do_sra(raw),
            Self::SPECIAL_FUNCT_SLLV => self.do_sllv(raw),
            Self::SPECIAL_FUNCT_SRLV => self.do_srlv(raw),
            Self::SPECIAL_FUNCT_SRAV => self.do_srav(raw),
            Self::SPECIAL_FUNCT_JR => self.do_jr(raw),
            Self::SPECIAL_FUNCT_JALR => self.do_jalr(raw),
            Self::SPECIAL_FUNCT_MOVZ => self.do_movz(raw),
            Self::SPECIAL_FUNCT_MOVN => self.do_movn(raw),
            Self::SPECIAL_FUNCT_SYSCALL => self.do_syscall(raw),
            Self::SPECIAL_FUNCT_BREAK => self.do_break(raw),
            Self::SPECIAL_FUNCT_SYNC => self.do_sync(raw),
            Self::SPECIAL_FUNCT_MFHI => self.do_mfhi(raw),
            Self::SPECIAL_FUNCT_MTHI => self.do_mthi(raw),
            Self::SPECIAL_FUNCT_MFLO => self.do_mflo(raw),
            Self::SPECIAL_FUNCT_MTLO => self.do_mtlo(raw),
            Self::SPECIAL_FUNCT_DSLLV => self.do_dsllv(raw),
            Self::SPECIAL_FUNCT_DSRLV => self.do_dsrlv(raw),
            Self::SPECIAL_FUNCT_DSRAV => self.do_dsrav(raw),
            Self::SPECIAL_FUNCT_MULT => self.do_mult(raw),
            Self::SPECIAL_FUNCT_MULTU => self.do_multu(raw),
            Self::SPECIAL_FUNCT_DIV => self.do_div(raw),
            Self::SPECIAL_FUNCT_DIVU => self.do_divu(raw),
            Self::SPECIAL_FUNCT_ADD => self.do_add(raw),
            Self::SPECIAL_FUNCT_ADDU => self.do_addu(raw),
            Self::SPECIAL_FUNCT_SUB => self.do_sub(raw),
            Self::SPECIAL_FUNCT_SUBU => self.do_subu(raw),
            Self::SPECIAL_FUNCT_AND => self.do_and(raw),
            Self::SPECIAL_FUNCT_OR => self.do_or(raw),
            Self::SPECIAL_FUNCT_XOR => self.do_xor(raw),
            Self::SPECIAL_FUNCT_NOR => self.do_nor(raw),
            Self::SPECIAL_FUNCT_MFSA => self.do_mfsa(raw),
            Self::SPECIAL_FUNCT_MTSA => self.do_mtsa(raw),
            Self::SPECIAL_FUNCT_SLT => self.do_slt(raw),
            Self::SPECIAL_FUNCT_SLTU => self.do_sltu(raw),
            Self::SPECIAL_FUNCT_DADD => self.do_dadd(raw),
            Self::SPECIAL_FUNCT_DADDU => self.do_daddu(raw),
            Self::SPECIAL_FUNCT_DSUB => self.do_dsub(raw),
            Self::SPECIAL_FUNCT_DSUBU => self.do_dsubu(raw),
            Self::SPECIAL_FUNCT_TGE => self.do_tge(raw),
            Self::SPECIAL_FUNCT_TGEU => self.do_tgeu(raw),
            Self::SPECIAL_FUNCT_TLT => self.do_tlt(raw),
            Self::SPECIAL_FUNCT_TLTU => self.do_tltu(raw),
            Self::SPECIAL_FUNCT_TEQ => self.do_teq(raw),
            Self::SPECIAL_FUNCT_TNE => self.do_tne(raw),
            Self::SPECIAL_FUNCT_DSLL => self.do_dsll(raw),
            Self::SPECIAL_FUNCT_DSRL => self.do_dsrl(raw),
            Self::SPECIAL_FUNCT_DSRA => self.do_dsra(raw),
            Self::SPECIAL_FUNCT_DSLL32 => self.do_dsll32(raw),
            Self::SPECIAL_FUNCT_DSRL32 => self.do_dsrl32(raw),
            Self::SPECIAL_FUNCT_DSRA32 => self.do_dsra32(raw),
            _ => unimplemented!("Function {:06b} not implemented", funct),
        }
    }

    fn do_sll(&mut self, raw: u32) {
        // SLL rd, rt, sa
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;
        let sa = ((raw >> 6) & 0b11111) as u32;

        // Get lower 32 bits of rt and shift left by sa
        let rt_value = self.gprs[rt] as u32;
        let result = rt_value << sa;

        // Sign-extend the 32-bit result to 128 bits
        self.gprs[rd] = result as i32 as i64 as i128 as u128;
    }

    fn do_srl(&mut self, raw: u32) {
        // SRL rd, rt, sa - Shift Right Logical
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;
        let sa = ((raw >> 6) & 0b11111) as u32;

        let rt_value = self.gprs[rt] as u32;
        let result = rt_value >> sa;

        self.gprs[rd] = result as i32 as i64 as i128 as u128;
    }

    fn do_sra(&mut self, raw: u32) {
        // SRA rd, rt, sa - Shift Right Arithmetic
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;
        let sa = ((raw >> 6) & 0b11111) as u32;

        let rt_value = self.gprs[rt] as i32;
        let result = rt_value >> sa;

        self.gprs[rd] = result as i64 as i128 as u128;
    }

    fn do_sllv(&mut self, raw: u32) {
        // SLLV rd, rt, rs - Shift Left Logical Variable
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let rt_value = self.gprs[rt] as u32;
        let sa = (self.gprs[rs] & 0b11111) as u32;
        let result = rt_value << sa;

        self.gprs[rd] = result as i32 as i64 as i128 as u128;
    }

    fn do_srlv(&mut self, raw: u32) {
        // SRLV rd, rt, rs - Shift Right Logical Variable
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let rt_value = self.gprs[rt] as u32;
        let sa = (self.gprs[rs] & 0b11111) as u32;
        let result = rt_value >> sa;

        self.gprs[rd] = result as i32 as i64 as i128 as u128;
    }

    fn do_srav(&mut self, raw: u32) {
        // SRAV rd, rt, rs - Shift Right Arithmetic Variable
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let rt_value = self.gprs[rt] as i32;
        let sa = (self.gprs[rs] & 0b11111) as u32;
        let result = rt_value >> sa;

        self.gprs[rd] = result as i64 as i128 as u128;
    }

    fn do_jr(&mut self, raw: u32) {
        // JR rs - Jump Register
        let rs = ((raw >> 21) & 0b11111) as usize;
        self.pc = self.gprs[rs] as u32;
    }

    fn do_jalr(&mut self, raw: u32) {
        // JALR rd, rs - Jump And Link Register
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let target = self.gprs[rs] as u32;
        self.gprs[rd] = (self.pc + 8) as u128; // Save return address
        self.pc = target;
    }

    fn do_movz(&mut self, raw: u32) {
        // MOVZ rd, rs, rt - Move Conditional on Zero
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        if self.gprs[rt] == 0 {
            self.gprs[rd] = self.gprs[rs];
        }
    }

    fn do_movn(&mut self, raw: u32) {
        // MOVN rd, rs, rt - Move Conditional on Not Zero
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        if self.gprs[rt] != 0 {
            self.gprs[rd] = self.gprs[rs];
        }
    }

    fn do_syscall(&mut self, _raw: u32) {
        // SYSCALL - System Call
        // Trigger system call exception (implementation depends on system)
        panic!("SYSCALL instruction executed");
    }

    fn do_break(&mut self, _raw: u32) {
        // BREAK - Breakpoint
        // Trigger breakpoint exception
        panic!("BREAK instruction executed");
    }

    fn do_sync(&mut self, _raw: u32) {
        // SYNC - Synchronize Shared Memory
        // On EE, this is essentially a NOP for ordering memory operations
    }

    fn do_mfhi(&mut self, raw: u32) {
        // MFHI rd - Move From HI
        let rd = ((raw >> 11) & 0b11111) as usize;
        self.gprs[rd] = self.hi;
    }

    fn do_mthi(&mut self, raw: u32) {
        // MTHI rs - Move To HI
        let rs = ((raw >> 21) & 0b11111) as usize;
        self.hi = self.gprs[rs];
    }

    fn do_mflo(&mut self, raw: u32) {
        // MFLO rd - Move From LO
        let rd = ((raw >> 11) & 0b11111) as usize;
        self.gprs[rd] = self.lo;
    }

    fn do_mtlo(&mut self, raw: u32) {
        // MTLO rs - Move To LO
        let rs = ((raw >> 21) & 0b11111) as usize;
        self.lo = self.gprs[rs];
    }

    fn do_dsllv(&mut self, raw: u32) {
        // DSLLV rd, rt, rs - Doubleword Shift Left Logical Variable
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let rt_value = self.gprs[rt] as u64;
        let sa = (self.gprs[rs] & 0b111111) as u32;
        let result = rt_value << sa;

        self.gprs[rd] = result as i64 as i128 as u128;
    }

    fn do_dsrlv(&mut self, raw: u32) {
        // DSRLV rd, rt, rs - Doubleword Shift Right Logical Variable
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let rt_value = self.gprs[rt] as u64;
        let sa = (self.gprs[rs] & 0b111111) as u32;
        let result = rt_value >> sa;

        self.gprs[rd] = result as i64 as i128 as u128;
    }

    fn do_dsrav(&mut self, raw: u32) {
        // DSRAV rd, rt, rs - Doubleword Shift Right Arithmetic Variable
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let rt_value = self.gprs[rt] as i64;
        let sa = (self.gprs[rs] & 0b111111) as u32;
        let result = rt_value >> sa;

        self.gprs[rd] = result as i128 as u128;
    }

    fn do_mult(&mut self, raw: u32) {
        // MULT rs, rt - Multiply Word
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as i32;
        let rt_value = self.gprs[rt] as i32;
        let result = (rs_value as i64) * (rt_value as i64);

        self.lo = (result as i32) as i64 as i128 as u128;
        self.hi = ((result >> 32) as i32) as i64 as i128 as u128;
    }

    fn do_multu(&mut self, raw: u32) {
        // MULTU rs, rt - Multiply Unsigned Word
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as u32;
        let rt_value = self.gprs[rt] as u32;
        let result = (rs_value as u64) * (rt_value as u64);

        self.lo = (result as u32) as i32 as i64 as i128 as u128;
        self.hi = ((result >> 32) as u32) as i32 as i64 as i128 as u128;
    }

    fn do_div(&mut self, raw: u32) {
        // DIV rs, rt - Divide Word
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as i32;
        let rt_value = self.gprs[rt] as i32;

        if rt_value == 0 {
            // Division by zero - result is undefined, typically no exception on MIPS
            self.lo = if rs_value >= 0 { u128::MAX } else { 1 };
            self.hi = rs_value as i64 as i128 as u128;
        } else {
            let quotient = rs_value.wrapping_div(rt_value);
            let remainder = rs_value.wrapping_rem(rt_value);
            self.lo = quotient as i64 as i128 as u128;
            self.hi = remainder as i64 as i128 as u128;
        }
    }

    fn do_divu(&mut self, raw: u32) {
        // DIVU rs, rt - Divide Unsigned Word
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as u32;
        let rt_value = self.gprs[rt] as u32;

        if rt_value == 0 {
            self.lo = u128::MAX;
            self.hi = rs_value as i32 as i64 as i128 as u128;
        } else {
            let quotient = rs_value / rt_value;
            let remainder = rs_value % rt_value;
            self.lo = quotient as i32 as i64 as i128 as u128;
            self.hi = remainder as i32 as i64 as i128 as u128;
        }
    }

    fn do_add(&mut self, raw: u32) {
        // ADD rd, rs, rt - Add with Overflow
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as i32;
        let rt_value = self.gprs[rt] as i32;

        match rs_value.checked_add(rt_value) {
            Some(result) => self.gprs[rd] = result as i64 as i128 as u128,
            None => panic!("ADD overflow exception"),
        }
    }

    fn do_addu(&mut self, raw: u32) {
        // ADDU rd, rs, rt - Add Unsigned (no overflow)
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as u32;
        let rt_value = self.gprs[rt] as u32;
        let result = rs_value.wrapping_add(rt_value);

        self.gprs[rd] = result as i32 as i64 as i128 as u128;
    }

    fn do_sub(&mut self, raw: u32) {
        // SUB rd, rs, rt - Subtract with Overflow
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as i32;
        let rt_value = self.gprs[rt] as i32;

        match rs_value.checked_sub(rt_value) {
            Some(result) => self.gprs[rd] = result as i64 as i128 as u128,
            None => panic!("SUB overflow exception"),
        }
    }

    fn do_subu(&mut self, raw: u32) {
        // SUBU rd, rs, rt - Subtract Unsigned (no overflow)
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as u32;
        let rt_value = self.gprs[rt] as u32;
        let result = rs_value.wrapping_sub(rt_value);

        self.gprs[rd] = result as i32 as i64 as i128 as u128;
    }

    fn do_and(&mut self, raw: u32) {
        // AND rd, rs, rt - Bitwise AND
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        self.gprs[rd] = self.gprs[rs] & self.gprs[rt];
    }

    fn do_or(&mut self, raw: u32) {
        // OR rd, rs, rt - Bitwise OR
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        self.gprs[rd] = self.gprs[rs] | self.gprs[rt];
    }

    fn do_xor(&mut self, raw: u32) {
        // XOR rd, rs, rt - Bitwise XOR
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        self.gprs[rd] = self.gprs[rs] ^ self.gprs[rt];
    }

    fn do_nor(&mut self, raw: u32) {
        // NOR rd, rs, rt - Bitwise NOR
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        self.gprs[rd] = !(self.gprs[rs] | self.gprs[rt]);
    }

    fn do_mfsa(&mut self, raw: u32) {
        // MFSA rd - Move From Shift Amount
        let rd = ((raw >> 11) & 0b11111) as usize;
        self.gprs[rd] = self.sa as u128;
    }

    fn do_mtsa(&mut self, raw: u32) {
        // MTSA rs - Move To Shift Amount
        let rs = ((raw >> 21) & 0b11111) as usize;
        self.sa = self.gprs[rs] as u64;
    }

    fn do_slt(&mut self, raw: u32) {
        // SLT rd, rs, rt - Set on Less Than
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as i64;
        let rt_value = self.gprs[rt] as i64;
        self.gprs[rd] = if rs_value < rt_value { 1 } else { 0 };
    }

    fn do_sltu(&mut self, raw: u32) {
        // SLTU rd, rs, rt - Set on Less Than Unsigned
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as u64;
        let rt_value = self.gprs[rt] as u64;
        self.gprs[rd] = if rs_value < rt_value { 1 } else { 0 };
    }

    fn do_dadd(&mut self, raw: u32) {
        // DADD rd, rs, rt - Doubleword Add with Overflow
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as i64;
        let rt_value = self.gprs[rt] as i64;

        match rs_value.checked_add(rt_value) {
            Some(result) => self.gprs[rd] = result as i128 as u128,
            None => panic!("DADD overflow exception"),
        }
    }

    fn do_daddu(&mut self, raw: u32) {
        // DADDU rd, rs, rt - Doubleword Add Unsigned (no overflow)
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as u64;
        let rt_value = self.gprs[rt] as u64;
        let result = rs_value.wrapping_add(rt_value);

        self.gprs[rd] = result as i64 as i128 as u128;
    }

    fn do_dsub(&mut self, raw: u32) {
        // DSUB rd, rs, rt - Doubleword Subtract with Overflow
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as i64;
        let rt_value = self.gprs[rt] as i64;

        match rs_value.checked_sub(rt_value) {
            Some(result) => self.gprs[rd] = result as i128 as u128,
            None => panic!("DSUB overflow exception"),
        }
    }

    fn do_dsubu(&mut self, raw: u32) {
        // DSUBU rd, rs, rt - Doubleword Subtract Unsigned (no overflow)
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as u64;
        let rt_value = self.gprs[rt] as u64;
        let result = rs_value.wrapping_sub(rt_value);

        self.gprs[rd] = result as i64 as i128 as u128;
    }

    fn do_tge(&mut self, raw: u32) {
        // TGE rs, rt - Trap if Greater or Equal
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as i64;
        let rt_value = self.gprs[rt] as i64;

        if rs_value >= rt_value {
            panic!("TGE trap exception");
        }
    }

    fn do_tgeu(&mut self, raw: u32) {
        // TGEU rs, rt - Trap if Greater or Equal Unsigned
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as u64;
        let rt_value = self.gprs[rt] as u64;

        if rs_value >= rt_value {
            panic!("TGEU trap exception");
        }
    }

    fn do_tlt(&mut self, raw: u32) {
        // TLT rs, rt - Trap if Less Than
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as i64;
        let rt_value = self.gprs[rt] as i64;

        if rs_value < rt_value {
            panic!("TLT trap exception");
        }
    }

    fn do_tltu(&mut self, raw: u32) {
        // TLTU rs, rt - Trap if Less Than Unsigned
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;

        let rs_value = self.gprs[rs] as u64;
        let rt_value = self.gprs[rt] as u64;

        if rs_value < rt_value {
            panic!("TLTU trap exception");
        }
    }

    fn do_teq(&mut self, raw: u32) {
        // TEQ rs, rt - Trap if Equal
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;

        if self.gprs[rs] == self.gprs[rt] {
            panic!("TEQ trap exception");
        }
    }

    fn do_tne(&mut self, raw: u32) {
        // TNE rs, rt - Trap if Not Equal
        let rs = ((raw >> 21) & 0b11111) as usize;
        let rt = ((raw >> 16) & 0b11111) as usize;

        if self.gprs[rs] != self.gprs[rt] {
            panic!("TNE trap exception");
        }
    }

    fn do_dsll(&mut self, raw: u32) {
        // DSLL rd, rt, sa - Doubleword Shift Left Logical
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;
        let sa = ((raw >> 6) & 0b11111) as u32;

        let rt_value = self.gprs[rt] as u64;
        let result = rt_value << sa;

        self.gprs[rd] = result as i64 as i128 as u128;
    }

    fn do_dsrl(&mut self, raw: u32) {
        // DSRL rd, rt, sa - Doubleword Shift Right Logical
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;
        let sa = ((raw >> 6) & 0b11111) as u32;

        let rt_value = self.gprs[rt] as u64;
        let result = rt_value >> sa;

        self.gprs[rd] = result as i64 as i128 as u128;
    }

    fn do_dsra(&mut self, raw: u32) {
        // DSRA rd, rt, sa - Doubleword Shift Right Arithmetic
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;
        let sa = ((raw >> 6) & 0b11111) as u32;

        let rt_value = self.gprs[rt] as i64;
        let result = rt_value >> sa;

        self.gprs[rd] = result as i128 as u128;
    }

    fn do_dsll32(&mut self, raw: u32) {
        // DSLL32 rd, rt, sa - Doubleword Shift Left Logical + 32
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;
        let sa = ((raw >> 6) & 0b11111) as u32;

        let rt_value = self.gprs[rt] as u64;
        let result = rt_value << (sa + 32);

        self.gprs[rd] = result as i64 as i128 as u128;
    }

    fn do_dsrl32(&mut self, raw: u32) {
        // DSRL32 rd, rt, sa - Doubleword Shift Right Logical + 32
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;
        let sa = ((raw >> 6) & 0b11111) as u32;

        let rt_value = self.gprs[rt] as u64;
        let result = rt_value >> (sa + 32);

        self.gprs[rd] = result as i64 as i128 as u128;
    }

    fn do_dsra32(&mut self, raw: u32) {
        // DSRA32 rd, rt, sa - Doubleword Shift Right Arithmetic + 32
        let rt = ((raw >> 16) & 0b11111) as usize;
        let rd = ((raw >> 11) & 0b11111) as usize;
        let sa = ((raw >> 6) & 0b11111) as u32;

        let rt_value = self.gprs[rt] as i64;
        let result = rt_value >> (sa + 32);

        self.gprs[rd] = result as i128 as u128;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create R-type instruction
    fn make_r_type(rs: u32, rt: u32, rd: u32, sa: u32, funct: u32) -> u32 {
        (rs << 21) | (rt << 16) | (rd << 11) | (sa << 6) | funct
    }

    // Helper function to create instruction with just funct
    fn make_special(funct: u32) -> u32 {
        funct
    }

    #[test]
    fn test_sll() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x12345678;

        // SLL $2, $1, 4
        let instr = make_r_type(0, 1, 2, 4, Cpu::SPECIAL_FUNCT_SLL);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[2] as i32, 0x23456780_u32 as i32);
    }

    #[test]
    fn test_srl() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x80000000;

        // SRL $2, $1, 4
        let instr = make_r_type(0, 1, 2, 4, Cpu::SPECIAL_FUNCT_SRL);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[2] as u32, 0x08000000);
    }

    #[test]
    fn test_sra() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x80000000_u32 as i32 as i64 as i128 as u128;

        // SRA $2, $1, 4
        let instr = make_r_type(0, 1, 2, 4, Cpu::SPECIAL_FUNCT_SRA);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[2] as i32, 0xF8000000_u32 as i32);
    }

    #[test]
    fn test_sllv() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x12345678;
        cpu.gprs[3] = 8;

        // SLLV $2, $1, $3
        let instr = make_r_type(3, 1, 2, 0, Cpu::SPECIAL_FUNCT_SLLV);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[2] as i32, 0x34567800_u32 as i32);
    }

    #[test]
    fn test_srlv() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x80000000;
        cpu.gprs[3] = 4;

        // SRLV $2, $1, $3
        let instr = make_r_type(3, 1, 2, 0, Cpu::SPECIAL_FUNCT_SRLV);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[2] as u32, 0x08000000);
    }

    #[test]
    fn test_srav() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x80000000_u32 as i32 as i64 as i128 as u128;
        cpu.gprs[3] = 4;

        // SRAV $2, $1, $3
        let instr = make_r_type(3, 1, 2, 0, Cpu::SPECIAL_FUNCT_SRAV);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[2] as i32, 0xF8000000_u32 as i32);
    }

    #[test]
    fn test_jr() {
        let mut cpu = Cpu::new();
        cpu.gprs[31] = 0x1000;
        cpu.pc = 0x2000;

        // JR $31
        let instr = make_r_type(31, 0, 0, 0, Cpu::SPECIAL_FUNCT_JR);
        cpu.exec(instr);

        assert_eq!(cpu.pc, 0x1000);
    }

    #[test]
    fn test_jalr() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x5000;
        cpu.pc = 0x2000;

        // JALR $31, $1
        let instr = make_r_type(1, 0, 31, 0, Cpu::SPECIAL_FUNCT_JALR);
        cpu.exec(instr);

        assert_eq!(cpu.pc, 0x5000);
        assert_eq!(cpu.gprs[31], 0x2008);
    }

    #[test]
    fn test_movz() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x12345678;
        cpu.gprs[2] = 0; // condition is zero
        cpu.gprs[3] = 0xDEADBEEF;

        // MOVZ $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_MOVZ);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3], 0x12345678);
    }

    #[test]
    fn test_movz_no_move() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x12345678;
        cpu.gprs[2] = 1; // condition is not zero
        cpu.gprs[3] = 0xDEADBEEF;

        // MOVZ $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_MOVZ);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3], 0xDEADBEEF);
    }

    #[test]
    fn test_movn() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x12345678;
        cpu.gprs[2] = 1; // condition is not zero
        cpu.gprs[3] = 0xDEADBEEF;

        // MOVN $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_MOVN);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3], 0x12345678);
    }

    #[test]
    fn test_movn_no_move() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x12345678;
        cpu.gprs[2] = 0; // condition is zero
        cpu.gprs[3] = 0xDEADBEEF;

        // MOVN $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_MOVN);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3], 0xDEADBEEF);
    }

    #[test]
    fn test_sync() {
        let mut cpu = Cpu::new();

        // SYNC (should do nothing)
        let instr = make_special(Cpu::SPECIAL_FUNCT_SYNC);
        cpu.exec(instr);

        // Just verify it doesn't crash
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn test_mfhi() {
        let mut cpu = Cpu::new();
        cpu.hi = 0x12345678ABCDEF00;

        // MFHI $1
        let instr = make_r_type(0, 0, 1, 0, Cpu::SPECIAL_FUNCT_MFHI);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[1], 0x12345678ABCDEF00);
    }

    #[test]
    fn test_mthi() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0xFEEDFACECAFEBABE;

        // MTHI $1
        let instr = make_r_type(1, 0, 0, 0, Cpu::SPECIAL_FUNCT_MTHI);
        cpu.exec(instr);

        assert_eq!(cpu.hi, 0xFEEDFACECAFEBABE);
    }

    #[test]
    fn test_mflo() {
        let mut cpu = Cpu::new();
        cpu.lo = 0x12345678ABCDEF00;

        // MFLO $1
        let instr = make_r_type(0, 0, 1, 0, Cpu::SPECIAL_FUNCT_MFLO);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[1], 0x12345678ABCDEF00);
    }

    #[test]
    fn test_mtlo() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0xFEEDFACECAFEBABE;

        // MTLO $1
        let instr = make_r_type(1, 0, 0, 0, Cpu::SPECIAL_FUNCT_MTLO);
        cpu.exec(instr);

        assert_eq!(cpu.lo, 0xFEEDFACECAFEBABE);
    }

    #[test]
    fn test_dsllv() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x123456789ABCDEF0;
        cpu.gprs[3] = 8;

        // DSLLV $2, $1, $3
        let instr = make_r_type(3, 1, 2, 0, Cpu::SPECIAL_FUNCT_DSLLV);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[2] as i64, 0x3456789ABCDEF000_u64 as i64);
    }

    #[test]
    fn test_dsrlv() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x8000000000000000_u64 as i64 as i128 as u128;
        cpu.gprs[3] = 4;

        // DSRLV $2, $1, $3
        let instr = make_r_type(3, 1, 2, 0, Cpu::SPECIAL_FUNCT_DSRLV);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[2] as u64, 0x0800000000000000);
    }

    #[test]
    fn test_dsrav() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x8000000000000000_u64 as i64 as i128 as u128;
        cpu.gprs[3] = 4;

        // DSRAV $2, $1, $3
        let instr = make_r_type(3, 1, 2, 0, Cpu::SPECIAL_FUNCT_DSRAV);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[2] as i64, 0xF800000000000000_u64 as i64);
    }

    #[test]
    fn test_mult() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x00001000_u32 as i32 as i64 as i128 as u128;
        cpu.gprs[2] = 0x00002000_u32 as i32 as i64 as i128 as u128;

        // MULT $1, $2
        let instr = make_r_type(1, 2, 0, 0, Cpu::SPECIAL_FUNCT_MULT);
        cpu.exec(instr);

        assert_eq!(cpu.lo as i32, 0x02000000);
        assert_eq!(cpu.hi as i32, 0);
    }

    #[test]
    fn test_mult_negative() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = (-10_i32) as i64 as i128 as u128;
        cpu.gprs[2] = (20_i32) as i64 as i128 as u128;

        // MULT $1, $2
        let instr = make_r_type(1, 2, 0, 0, Cpu::SPECIAL_FUNCT_MULT);
        cpu.exec(instr);

        assert_eq!(cpu.lo as i32, -200);
        assert_eq!(cpu.hi as i32, -1);
    }

    #[test]
    fn test_multu() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0xFFFFFFFF;
        cpu.gprs[2] = 0x00000002;

        // MULTU $1, $2
        let instr = make_r_type(1, 2, 0, 0, Cpu::SPECIAL_FUNCT_MULTU);
        cpu.exec(instr);

        assert_eq!(cpu.lo as u32, 0xFFFFFFFE);
        assert_eq!(cpu.hi as u32, 0x00000001);
    }

    #[test]
    fn test_div() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 100_i32 as i64 as i128 as u128;
        cpu.gprs[2] = 7_i32 as i64 as i128 as u128;

        // DIV $1, $2
        let instr = make_r_type(1, 2, 0, 0, Cpu::SPECIAL_FUNCT_DIV);
        cpu.exec(instr);

        assert_eq!(cpu.lo as i32, 14); // quotient
        assert_eq!(cpu.hi as i32, 2); // remainder
    }

    #[test]
    fn test_div_negative() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = (-100_i32) as i64 as i128 as u128;
        cpu.gprs[2] = 7_i32 as i64 as i128 as u128;

        // DIV $1, $2
        let instr = make_r_type(1, 2, 0, 0, Cpu::SPECIAL_FUNCT_DIV);
        cpu.exec(instr);

        assert_eq!(cpu.lo as i32, -14); // quotient
        assert_eq!(cpu.hi as i32, -2); // remainder
    }

    #[test]
    fn test_divu() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 100;
        cpu.gprs[2] = 7;

        // DIVU $1, $2
        let instr = make_r_type(1, 2, 0, 0, Cpu::SPECIAL_FUNCT_DIVU);
        cpu.exec(instr);

        assert_eq!(cpu.lo as u32, 14); // quotient
        assert_eq!(cpu.hi as u32, 2); // remainder
    }

    #[test]
    fn test_add() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 100_i32 as i64 as i128 as u128;
        cpu.gprs[2] = 200_i32 as i64 as i128 as u128;

        // ADD $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_ADD);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3] as i32, 300);
    }

    #[test]
    #[should_panic(expected = "ADD overflow exception")]
    fn test_add_overflow() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = i32::MAX as i64 as i128 as u128;
        cpu.gprs[2] = 1_i32 as i64 as i128 as u128;

        // ADD $3, $1, $2 - should overflow
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_ADD);
        cpu.exec(instr);
    }

    #[test]
    fn test_addu() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 100;
        cpu.gprs[2] = 200;

        // ADDU $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_ADDU);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3] as i32, 300);
    }

    #[test]
    fn test_addu_no_overflow() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = i32::MAX as u32 as u128;
        cpu.gprs[2] = 1;

        // ADDU $3, $1, $2 - wraps without exception
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_ADDU);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3] as i32, i32::MIN);
    }

    #[test]
    fn test_sub() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 300_i32 as i64 as i128 as u128;
        cpu.gprs[2] = 100_i32 as i64 as i128 as u128;

        // SUB $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_SUB);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3] as i32, 200);
    }

    #[test]
    #[should_panic(expected = "SUB overflow exception")]
    fn test_sub_overflow() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = i32::MIN as i64 as i128 as u128;
        cpu.gprs[2] = 1_i32 as i64 as i128 as u128;

        // SUB $3, $1, $2 - should overflow
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_SUB);
        cpu.exec(instr);
    }

    #[test]
    fn test_subu() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 300;
        cpu.gprs[2] = 100;

        // SUBU $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_SUBU);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3] as i32, 200);
    }

    #[test]
    fn test_and() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0xF0F0F0F0;
        cpu.gprs[2] = 0xFF00FF00;

        // AND $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_AND);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3], 0xF000F000);
    }

    #[test]
    fn test_or() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0xF0F0F0F0;
        cpu.gprs[2] = 0x0F0F0F0F;

        // OR $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_OR);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3], 0xFFFFFFFF);
    }

    #[test]
    fn test_xor() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0xF0F0F0F0;
        cpu.gprs[2] = 0xFF00FF00;

        // XOR $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_XOR);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3], 0x0FF00FF0);
    }

    #[test]
    fn test_nor() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0xF0F0F0F0;
        cpu.gprs[2] = 0x0F0F0F0F;

        // NOR $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_NOR);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3], !0xFFFFFFFF_u128);
    }

    #[test]
    fn test_mfsa() {
        let mut cpu = Cpu::new();
        cpu.sa = 0x12345678ABCDEF00;

        // MFSA $1
        let instr = make_r_type(0, 0, 1, 0, Cpu::SPECIAL_FUNCT_MFSA);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[1], 0x12345678ABCDEF00);
    }

    #[test]
    fn test_mtsa() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0xFEEDFACECAFEBABE;

        // MTSA $1
        let instr = make_r_type(1, 0, 0, 0, Cpu::SPECIAL_FUNCT_MTSA);
        cpu.exec(instr);

        assert_eq!(cpu.sa, 0xFEEDFACECAFEBABE);
    }

    #[test]
    fn test_slt() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = (-10_i64) as i128 as u128;
        cpu.gprs[2] = 20_i64 as i128 as u128;

        // SLT $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_SLT);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3], 1);
    }

    #[test]
    fn test_slt_false() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 20_i64 as i128 as u128;
        cpu.gprs[2] = (-10_i64) as i128 as u128;

        // SLT $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_SLT);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3], 0);
    }

    #[test]
    fn test_sltu() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 10;
        cpu.gprs[2] = 20;

        // SLTU $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_SLTU);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3], 1);
    }

    #[test]
    fn test_sltu_false() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 20;
        cpu.gprs[2] = 10;

        // SLTU $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_SLTU);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3], 0);
    }

    #[test]
    fn test_dadd() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 100_i64 as i128 as u128;
        cpu.gprs[2] = 200_i64 as i128 as u128;

        // DADD $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_DADD);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3] as i64, 300);
    }

    #[test]
    #[should_panic(expected = "DADD overflow exception")]
    fn test_dadd_overflow() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = i64::MAX as i128 as u128;
        cpu.gprs[2] = 1_i64 as i128 as u128;

        // DADD $3, $1, $2 - should overflow
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_DADD);
        cpu.exec(instr);
    }

    #[test]
    fn test_daddu() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 100;
        cpu.gprs[2] = 200;

        // DADDU $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_DADDU);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3] as i64, 300);
    }

    #[test]
    fn test_dsub() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 300_i64 as i128 as u128;
        cpu.gprs[2] = 100_i64 as i128 as u128;

        // DSUB $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_DSUB);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3] as i64, 200);
    }

    #[test]
    #[should_panic(expected = "DSUB overflow exception")]
    fn test_dsub_overflow() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = i64::MIN as i128 as u128;
        cpu.gprs[2] = 1_i64 as i128 as u128;

        // DSUB $3, $1, $2 - should overflow
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_DSUB);
        cpu.exec(instr);
    }

    #[test]
    fn test_dsubu() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 300;
        cpu.gprs[2] = 100;

        // DSUBU $3, $1, $2
        let instr = make_r_type(1, 2, 3, 0, Cpu::SPECIAL_FUNCT_DSUBU);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[3] as i64, 200);
    }

    #[test]
    #[should_panic(expected = "TGE trap exception")]
    fn test_tge_triggers() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 20_i64 as i128 as u128;
        cpu.gprs[2] = 10_i64 as i128 as u128;

        // TGE $1, $2
        let instr = make_r_type(1, 2, 0, 0, Cpu::SPECIAL_FUNCT_TGE);
        cpu.exec(instr);
    }

    #[test]
    fn test_tge_no_trap() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 5_i64 as i128 as u128;
        cpu.gprs[2] = 10_i64 as i128 as u128;

        // TGE $1, $2
        let instr = make_r_type(1, 2, 0, 0, Cpu::SPECIAL_FUNCT_TGE);
        cpu.exec(instr);

        // Should not panic
        assert_eq!(cpu.gprs[1] as i64, 5);
    }

    #[test]
    #[should_panic(expected = "TGEU trap exception")]
    fn test_tgeu_triggers() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 20;
        cpu.gprs[2] = 10;

        // TGEU $1, $2
        let instr = make_r_type(1, 2, 0, 0, Cpu::SPECIAL_FUNCT_TGEU);
        cpu.exec(instr);
    }

    #[test]
    #[should_panic(expected = "TLT trap exception")]
    fn test_tlt_triggers() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 5_i64 as i128 as u128;
        cpu.gprs[2] = 10_i64 as i128 as u128;

        // TLT $1, $2
        let instr = make_r_type(1, 2, 0, 0, Cpu::SPECIAL_FUNCT_TLT);
        cpu.exec(instr);
    }

    #[test]
    #[should_panic(expected = "TLTU trap exception")]
    fn test_tltu_triggers() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 5;
        cpu.gprs[2] = 10;

        // TLTU $1, $2
        let instr = make_r_type(1, 2, 0, 0, Cpu::SPECIAL_FUNCT_TLTU);
        cpu.exec(instr);
    }

    #[test]
    #[should_panic(expected = "TEQ trap exception")]
    fn test_teq_triggers() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 10;
        cpu.gprs[2] = 10;

        // TEQ $1, $2
        let instr = make_r_type(1, 2, 0, 0, Cpu::SPECIAL_FUNCT_TEQ);
        cpu.exec(instr);
    }

    #[test]
    fn test_teq_no_trap() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 10;
        cpu.gprs[2] = 20;

        // TEQ $1, $2
        let instr = make_r_type(1, 2, 0, 0, Cpu::SPECIAL_FUNCT_TEQ);
        cpu.exec(instr);

        // Should not panic
        assert_eq!(cpu.gprs[1], 10);
    }

    #[test]
    #[should_panic(expected = "TNE trap exception")]
    fn test_tne_triggers() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 10;
        cpu.gprs[2] = 20;

        // TNE $1, $2
        let instr = make_r_type(1, 2, 0, 0, Cpu::SPECIAL_FUNCT_TNE);
        cpu.exec(instr);
    }

    #[test]
    fn test_tne_no_trap() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 10;
        cpu.gprs[2] = 10;

        // TNE $1, $2
        let instr = make_r_type(1, 2, 0, 0, Cpu::SPECIAL_FUNCT_TNE);
        cpu.exec(instr);

        // Should not panic
        assert_eq!(cpu.gprs[1], 10);
    }

    #[test]
    fn test_dsll() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x123456789ABCDEF0;

        // DSLL $2, $1, 4
        let instr = make_r_type(0, 1, 2, 4, Cpu::SPECIAL_FUNCT_DSLL);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[2] as i64, 0x23456789ABCDEF00_u64 as i64);
    }

    #[test]
    fn test_dsrl() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x8000000000000000_u64 as i64 as i128 as u128;

        // DSRL $2, $1, 4
        let instr = make_r_type(0, 1, 2, 4, Cpu::SPECIAL_FUNCT_DSRL);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[2] as u64, 0x0800000000000000);
    }

    #[test]
    fn test_dsra() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x8000000000000000_u64 as i64 as i128 as u128;

        // DSRA $2, $1, 4
        let instr = make_r_type(0, 1, 2, 4, Cpu::SPECIAL_FUNCT_DSRA);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[2] as i64, 0xF800000000000000_u64 as i64);
    }

    #[test]
    fn test_dsll32() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x12345678;

        // DSLL32 $2, $1, 0 (shifts by 32)
        let instr = make_r_type(0, 1, 2, 0, Cpu::SPECIAL_FUNCT_DSLL32);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[2] as i64, 0x1234567800000000_u64 as i64);
    }

    #[test]
    fn test_dsll32_with_sa() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x12345678;

        // DSLL32 $2, $1, 4 (shifts by 36)
        let instr = make_r_type(0, 1, 2, 4, Cpu::SPECIAL_FUNCT_DSLL32);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[2] as i64, 0x2345678000000000_u64 as i64);
    }

    #[test]
    fn test_dsrl32() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x1234567800000000_u64 as i64 as i128 as u128;

        // DSRL32 $2, $1, 0 (shifts by 32)
        let instr = make_r_type(0, 1, 2, 0, Cpu::SPECIAL_FUNCT_DSRL32);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[2] as u64, 0x12345678);
    }

    #[test]
    fn test_dsra32() {
        let mut cpu = Cpu::new();
        cpu.gprs[1] = 0x8000000000000000_u64 as i64 as i128 as u128;

        // DSRA32 $2, $1, 0 (shifts by 32)
        let instr = make_r_type(0, 1, 2, 0, Cpu::SPECIAL_FUNCT_DSRA32);
        cpu.exec(instr);

        assert_eq!(cpu.gprs[2] as i64, 0xFFFFFFFF80000000_u64 as i64);
    }

    #[test]
    #[should_panic(expected = "SYSCALL instruction executed")]
    fn test_syscall() {
        let mut cpu = Cpu::new();
        let instr = make_special(Cpu::SPECIAL_FUNCT_SYSCALL);
        cpu.exec(instr);
    }

    #[test]
    #[should_panic(expected = "BREAK instruction executed")]
    fn test_break() {
        let mut cpu = Cpu::new();
        let instr = make_special(Cpu::SPECIAL_FUNCT_BREAK);
        cpu.exec(instr);
    }
}
