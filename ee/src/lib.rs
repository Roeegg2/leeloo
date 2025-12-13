pub struct Cpu {
    gprs: [u128; 32],
    pc: u32,
    hi0: u64,
    hi1: u64,
    lo0: u64,
    lo1: u64,
    sa: u64,
    next_pc: u32,
}

impl Cpu {
    // Helper functions for instruction field extraction
    #[inline]
    const fn extract_rs(raw: u32) -> usize {
        ((raw >> 21) & 0b11111) as usize
    }

    #[inline]
    const fn extract_rt(raw: u32) -> usize {
        ((raw >> 16) & 0b11111) as usize
    }

    #[inline]
    const fn extract_rd(raw: u32) -> usize {
        ((raw >> 11) & 0b11111) as usize
    }

    #[inline]
    const fn extract_sa(raw: u32) -> u32 {
        (raw >> 6) & 0b11111
    }

    // Read GPR as 32-bit word (lower 32 bits)
    #[inline]
    fn read_gpr_word(&self, index: usize) -> u32 {
        self.gprs[index] as u32
    }

    // Write GPR as 32-bit word (sign-extended to 128 bits)
    #[inline]
    fn write_gpr_word(&mut self, index: usize, value: u32) {
        self.gprs[index] &= 0xffff_ff00;
        self.gprs[index] |= value as u128;
    }

    // Read GPR as 64-bit doubleword (lower 64 bits)
    #[inline]
    fn read_gpr_dword(&self, index: usize) -> u64 {
        self.gprs[index] as u64
    }

    // Write GPR as 64-bit doubleword (sign-extended to 128 bits)
    #[inline]
    fn write_gpr_dword(&mut self, index: usize, value: u64) {
        self.gprs[index] &= 0xffff_0000;
        self.gprs[index] |= value as u128;
    }

    // Read GPR as 128-bit quadword (full width)
    #[inline]
    fn read_gpr_qword(&self, index: usize) -> u128 {
        self.gprs[index]
    }

    // Write GPR as 128-bit quadword (full width)
    #[inline]
    fn write_gpr_qword(&mut self, index: usize, value: u128) {
        self.gprs[index] = value;
    }

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
            next_pc: 4,
            lo0: 0,
            hi0: 0,
            lo1: 0,
            hi1: 0,
            sa: 0,
        }
    }

    #[inline]
    pub fn update_pc(&mut self) {
        self.pc = self.next_pc;
        self.next_pc += 4;
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
        // SLL rd, rt, sa - Shift Left Logical
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);
        let sa = Self::extract_sa(raw);

        let result = (self.read_gpr_word(rt) as i32) << sa;
        self.write_gpr_dword(rd, result as i64 as u64);
    }

    fn do_srl(&mut self, raw: u32) {
        // SRL rd, rt, sa - Shift Right Logical
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);
        let sa = Self::extract_sa(raw);

        let result = self.read_gpr_word(rt) >> sa;
        self.write_gpr_dword(rd, result as i32 as i64 as u64);
    }

    fn do_sra(&mut self, raw: u32) {
        // SRA rd, rt, sa - Shift Right Arithmetic
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);
        let sa = Self::extract_sa(raw);

        let result = (self.read_gpr_word(rt) as i32) >> sa;
        self.write_gpr_dword(rd, result as i64 as u64);
    }

    fn do_sllv(&mut self, raw: u32) {
        // SLLV rd, rt, rs - Shift Left Logical Variable
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        let sa = self.read_gpr_word(rs) & 0b11111;
        let result = (self.read_gpr_word(rt) as i32) << sa;
        self.write_gpr_dword(rd, result as i64 as u64);
    }

    fn do_srlv(&mut self, raw: u32) {
        // SRLV rd, rt, rs - Shift Right Logical Variable
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        let sa = self.read_gpr_word(rs) & 0b11111;
        let result = (self.read_gpr_word(rt) as u32) >> sa;
        self.write_gpr_dword(rd, result as i32 as i64 as u64);
    }

    fn do_srav(&mut self, raw: u32) {
        // SRAV rd, rt, rs - Shift Right Arithmetic Variable
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        let sa = self.read_gpr_word(rs) & 0b11111;
        let result = (self.read_gpr_word(rt) as i32) >> sa;
        self.write_gpr_dword(rd, result as i64 as u64);
    }

    fn do_jr(&mut self, raw: u32) {
        // JR rs - Jump Register
        let rs = Self::extract_rs(raw);
        // TODO: check rs alignment (should be checked during the regular fetch though)
        // NOTE: technically this should 'read_gpr_dword', but on the PS2 EE the bit width of PC is 32, so...
        self.next_pc = self.read_gpr_word(rs);
    }

    fn do_jalr(&mut self, raw: u32) {
        // JALR rd, rs - Jump And Link Register
        let rs = Self::extract_rs(raw);
        let rd = Self::extract_rd(raw);
        // TODO: check rs alignment (should be checked during the regular fetch though)
        // TODO: make sure rs != rd

        self.write_gpr_dword(rd, (self.pc + 8) as u64);
        self.next_pc = self.read_gpr_word(rs);
    }

    fn do_movz(&mut self, raw: u32) {
        // MOVZ rd, rs, rt - Move Conditional on Zero
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        if self.read_gpr_dword(rt) == 0 {
            self.write_gpr_dword(rd, self.read_gpr_dword(rs));
        }
    }

    fn do_movn(&mut self, raw: u32) {
        // MOVN rd, rs, rt - Move Conditional on Not Zero
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        if self.read_gpr_dword(rt) != 0 {
            self.write_gpr_dword(rd, self.read_gpr_dword(rs));
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

    // SKIPPED THIS
    fn do_sync(&mut self, _raw: u32) {
        // SYNC - Synchronize Shared Memory
        // On EE, this is essentially a NOP for ordering memory operations
    }

    fn do_mfhi(&mut self, raw: u32) {
        // MFHI rd - Move From HI
        let rd = Self::extract_rd(raw);
        // TODO: need to make sure that the 2 preceding instructions don't modify HI0
        self.write_gpr_dword(rd, self.hi0);
    }

    fn do_mthi(&mut self, raw: u32) {
        // MTHI rs - Move To HI
        let rs = Self::extract_rs(raw);
        // TODO: need to make sure that the 2 following instructions don't modify HI0
        self.hi0 = self.read_gpr_dword(rs);
    }

    fn do_mflo(&mut self, raw: u32) {
        // MFLO rd - Move From LO
        let rd = Self::extract_rd(raw);
        // TODO: need to make sure that the 2 preceding instructions don't modify HI0
        self.write_gpr_dword(rd, self.lo0);
    }

    fn do_mtlo(&mut self, raw: u32) {
        // MTLO rs - Move To LO
        let rs = Self::extract_rs(raw);
        // TODO: need to make sure that the 2 following instructions don't modify HI0
        self.lo0 = self.read_gpr_dword(rs);
    }

    fn do_dsllv(&mut self, raw: u32) {
        // DSLLV rd, rt, rs - Doubleword Shift Left Logical Variable
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        let sa = self.read_gpr_word(rs) & 0b111111;
        let result = self.read_gpr_dword(rt) << sa;
        self.write_gpr_dword(rd, result);
    }

    fn do_dsrlv(&mut self, raw: u32) {
        // DSRLV rd, rt, rs - Doubleword Shift Right Logical Variable
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        let sa = self.read_gpr_word(rs) & 0b111111;
        let result = self.read_gpr_dword(rt) >> sa;
        self.write_gpr_dword(rd, result);
    }

    fn do_dsrav(&mut self, raw: u32) {
        // DSRAV rd, rt, rs - Doubleword Shift Right Arithmetic Variable
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        let sa = self.read_gpr_qword(rs) & 0b111111;
        let result = (self.read_gpr_dword(rt) as i64) >> sa;
        self.write_gpr_dword(rd, result as u64);
    }

    fn do_mult(&mut self, raw: u32) {
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);

        let a = self.read_gpr_word(rs) as i32 as i64;
        let b = self.read_gpr_word(rt) as i32 as i64;
        let result = a * b;

        self.lo0 = result as u32 as u64;
        self.hi0 = (result >> 32) as u32 as u64;
    }

    fn do_multu(&mut self, raw: u32) {
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);

        let a = self.read_gpr_word(rs) as u64;
        let b = self.read_gpr_word(rt) as u64;
        let result = a * b;

        self.lo0 = result as u32 as u64;
        self.hi0 = result >> 32;
    }

    fn do_div(&mut self, raw: u32) {
        // DIV rs, rt - Divide Word
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);

        let rs_value = self.read_gpr_word(rs) as i32 as i64;
        let rt_value = self.read_gpr_word(rt) as i32 as i64;

        if rt_value == 0 {
            // in this case, lo0 and hi0 should be in an "undefined state". I think a NOP will do the job
            return;
        } else {
            self.lo0 = rs_value.wrapping_div(rt_value) as u64;
            self.hi0 = rs_value.wrapping_rem(rt_value) as u64;
        }
    }

    fn do_divu(&mut self, raw: u32) {
        // DIVU rs, rt - Divide Unsigned Word
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);

        let rs_value = self.read_gpr_word(rs) as u64;
        let rt_value = self.read_gpr_word(rt) as u64;

        if rt_value == 0 {
            // in this case, lo0 and hi0 should be in an "undefined state". I think a NOP will do the job
            return;
        } else {
            self.lo0 = rs_value / rt_value;
            self.hi0 = rs_value % rt_value;
        }
    }

    fn do_add(&mut self, raw: u32) {
        // ADD rd, rs, rt - Add with Overflow
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        let rs_value = self.read_gpr_word(rs) as i32;
        let rt_value = self.read_gpr_word(rt) as i32;

        match rs_value.checked_add(rt_value) {
            Some(result) => self.write_gpr_dword(rd, result as i64 as u64),
            None => panic!("ADD overflow exception"),
        }
    }

    fn do_addu(&mut self, raw: u32) {
        // ADDU rd, rs, rt - Add Unsigned (no overflow)
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        let result = self.read_gpr_word(rs).wrapping_add(self.read_gpr_word(rt));
        self.write_gpr_dword(rd, result as u64);
    }

    fn do_sub(&mut self, raw: u32) {
        // SUB rd, rs, rt - Subtract with Overflow
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        let rs_value = self.read_gpr_word(rs) as i32;
        let rt_value = self.read_gpr_word(rt) as i32;

        match rs_value.checked_sub(rt_value) {
            Some(result) => self.write_gpr_dword(rd, result as i64 as u64),
            None => panic!("SUB overflow exception"),
        }
    }

    fn do_subu(&mut self, raw: u32) {
        // SUBU rd, rs, rt - Subtract Unsigned (no overflow)
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        let result = self.read_gpr_word(rs).wrapping_sub(self.read_gpr_word(rt));
        self.write_gpr_dword(rd, result as u64);
    }

    fn do_and(&mut self, raw: u32) {
        // AND rd, rs, rt - Bitwise AND
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        self.write_gpr_dword(rd, self.read_gpr_dword(rs) & self.read_gpr_dword(rt));
    }

    fn do_or(&mut self, raw: u32) {
        // OR rd, rs, rt - Bitwise OR
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        self.write_gpr_dword(rd, self.read_gpr_dword(rs) | self.read_gpr_dword(rt));
    }

    fn do_xor(&mut self, raw: u32) {
        // XOR rd, rs, rt - Bitwise XOR
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        self.write_gpr_dword(rd, self.read_gpr_dword(rs) ^ self.read_gpr_dword(rt));
    }

    fn do_nor(&mut self, raw: u32) {
        // NOR rd, rs, rt - Bitwise NOR
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        self.write_gpr_dword(rd, !(self.read_gpr_dword(rs) | self.read_gpr_dword(rt)));
    }

    fn do_mfsa(&mut self, raw: u32) {
        // MFSA rd - Move From Shift Amount
        let rd = Self::extract_rd(raw);
        self.write_gpr_dword(rd, self.sa as u64);
    }

    fn do_mtsa(&mut self, raw: u32) {
        // MTSA rs - Move To Shift Amount
        let rs = Self::extract_rs(raw);
        self.sa = self.read_gpr_dword(rs);
    }

    fn do_slt(&mut self, raw: u32) {
        // SLT rd, rs, rt - Set on Less Than
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        let result = if (self.read_gpr_dword(rs) as i64) < (self.read_gpr_dword(rt) as i64) {
            1
        } else {
            0
        };
        self.write_gpr_dword(rd, result);
    }

    fn do_sltu(&mut self, raw: u32) {
        // SLTU rd, rs, rt - Set on Less Than Unsigned
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        let result = if self.read_gpr_dword(rs) < self.read_gpr_dword(rt) {
            1
        } else {
            0
        };
        self.write_gpr_dword(rd, result);
    }

    fn do_dadd(&mut self, raw: u32) {
        // DADD rd, rs, rt - Doubleword Add with Overflow
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        let rs_value = self.read_gpr_dword(rs) as i64;
        let rt_value = self.read_gpr_dword(rt) as i64;

        match rs_value.checked_add(rt_value) {
            Some(result) => self.write_gpr_dword(rd, result as u64),
            None => panic!("DADD overflow exception"),
        }
    }

    fn do_daddu(&mut self, raw: u32) {
        // DADDU rd, rs, rt - Doubleword Add Unsigned (no overflow)
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        let result = self
            .read_gpr_dword(rs)
            .wrapping_add(self.read_gpr_dword(rt));
        self.write_gpr_dword(rd, result);
    }

    fn do_dsub(&mut self, raw: u32) {
        // DSUB rd, rs, rt - Doubleword Subtract with Overflow
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        let rs_value = self.read_gpr_dword(rs) as i64;
        let rt_value = self.read_gpr_dword(rt) as i64;

        match rs_value.checked_sub(rt_value) {
            Some(result) => self.write_gpr_dword(rd, result as u64),
            None => panic!("DSUB overflow exception"),
        }
    }

    fn do_dsubu(&mut self, raw: u32) {
        // DSUBU rd, rs, rt - Doubleword Subtract Unsigned (no overflow)
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);

        let result = self
            .read_gpr_dword(rs)
            .wrapping_sub(self.read_gpr_dword(rt));
        self.write_gpr_dword(rd, result);
    }

    fn do_tge(&mut self, raw: u32) {
        // TGE rs, rt - Trap if Greater or Equal
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);

        if (self.read_gpr_dword(rs) as i64) >= (self.read_gpr_dword(rt) as i64) {
            panic!("TGE trap exception");
        }
    }

    fn do_tgeu(&mut self, raw: u32) {
        // TGEU rs, rt - Trap if Greater or Equal Unsigned
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);

        if self.read_gpr_dword(rs) >= self.read_gpr_dword(rt) {
            panic!("TGEU trap exception");
        }
    }

    fn do_tlt(&mut self, raw: u32) {
        // TLT rs, rt - Trap if Less Than
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);

        if (self.read_gpr_dword(rs) as i64) < (self.read_gpr_dword(rt) as i64) {
            panic!("TLT trap exception");
        }
    }

    fn do_tltu(&mut self, raw: u32) {
        // TLTU rs, rt - Trap if Less Than Unsigned
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);

        if self.read_gpr_dword(rs) < self.read_gpr_dword(rt) {
            panic!("TLTU trap exception");
        }
    }

    fn do_teq(&mut self, raw: u32) {
        // TEQ rs, rt - Trap if Equal
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);

        if self.read_gpr_qword(rs) == self.read_gpr_qword(rt) {
            panic!("TEQ trap exception");
        }
    }

    fn do_tne(&mut self, raw: u32) {
        // TNE rs, rt - Trap if Not Equal
        let rs = Self::extract_rs(raw);
        let rt = Self::extract_rt(raw);

        if self.read_gpr_qword(rs) != self.read_gpr_qword(rt) {
            panic!("TNE trap exception");
        }
    }

    fn do_dsll(&mut self, raw: u32) {
        // DSLL rd, rt, sa - Doubleword Shift Left Logical
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);
        let sa = Self::extract_sa(raw);

        let result = self.read_gpr_dword(rt) << (sa & 0b11111);
        self.write_gpr_dword(rd, result);
    }

    fn do_dsrl(&mut self, raw: u32) {
        // DSRL rd, rt, sa - Doubleword Shift Right Logical
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);
        let sa = Self::extract_sa(raw);

        let result = self.read_gpr_dword(rt) >> (sa & 0b11111);
        self.write_gpr_dword(rd, result);
    }

    fn do_dsra(&mut self, raw: u32) {
        // DSRA rd, rt, sa - Doubleword Shift Right Arithmetic
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);
        let sa = Self::extract_sa(raw);

        let result = (self.read_gpr_dword(rt) as i64) >> (sa & 0b11111);
        self.write_gpr_dword(rd, result as u64);
    }

    fn do_dsll32(&mut self, raw: u32) {
        // DSLL32 rd, rt, sa - Doubleword Shift Left Logical + 32
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);
        let sa = Self::extract_sa(raw);

        let result = self.read_gpr_dword(rt) << ((sa & 0b11111) + 32);
        self.write_gpr_dword(rd, result);
    }

    fn do_dsrl32(&mut self, raw: u32) {
        // DSRL32 rd, rt, sa - Doubleword Shift Right Logical + 32
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);
        let sa = Self::extract_sa(raw);

        let result = self.read_gpr_dword(rt) >> ((sa & 0b11111) + 32);
        self.write_gpr_dword(rd, result);
    }

    fn do_dsra32(&mut self, raw: u32) {
        // DSRA32 rd, rt, sa - Doubleword Shift Right Arithmetic + 32
        let rt = Self::extract_rt(raw);
        let rd = Self::extract_rd(raw);
        let sa = Self::extract_sa(raw);

        let result = (self.read_gpr_dword(rt) as i64) >> ((sa & 0b11111) + 32);
        self.write_gpr_dword(rd, result as u64);
    }
}
