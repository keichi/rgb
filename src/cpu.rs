use mmu;

#[derive(Debug)]
pub struct CPU {
    mmu: mmu::MMU,
    pc: u16,
    sp: u16,
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
}

impl CPU {
    pub fn new(mmu: mmu::MMU) -> Self {
        CPU {
            mmu: mmu,
            pc: 0x100, // TODO skip boot ROM for now
            sp: 0,
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
        }
    }

    /// Read AF register
    fn af(&self) -> u16 {
        (self.a as u16) << 8 | self.f as u16
    }

    /// Write AF register
    fn set_af(&mut self, val: u16) {
        self.a = (val >> 8 & 0xff) as u8;
        self.f = (val & 0xff) as u8;
    }

    /// Read BC register
    fn bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    /// Write BC register
    fn set_bc(&mut self, val: u16) {
        self.b = (val >> 8 & 0xff) as u8;
        self.c = (val & 0xff) as u8;
    }

    /// Read DE register
    fn de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    /// Write DE register
    fn set_de(&mut self, val: u16) {
        self.d = (val >> 8 & 0xff) as u8;
        self.e = (val & 0xff) as u8;
    }

    /// Read HL register
    fn hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    /// Write HL register
    fn set_hl(&mut self, val: u16) {
        self.h = (val >> 8 & 0xff) as u8;
        self.l = (val & 0xff) as u8;
    }

    fn set_f_z(&mut self, z: bool) {
        self.f = (self.f & !(1 << 7)) | (u8::from(z) << 7);
    }

    fn f_z(&self) -> bool {
        (self.f >> 7) & 1 == 1
    }

    fn set_f_n(&mut self, n: bool) {
        self.f = (self.f & !(1 << 6)) | (u8::from(n) << 6);
    }

    fn f_n(&self) -> bool {
        (self.f >> 6) & 1 == 1
    }

    fn set_f_h(&mut self, h: bool) {
        self.f = (self.f & !(1 << 5)) | (u8::from(h) << 5);
    }

    fn f_h(&self) -> bool {
        (self.f >> 5) & 1 == 1
    }

    fn set_f_c(&mut self, c: bool) {
        self.f = (self.f & !(1 << 4)) | (u8::from(c) << 4);
    }

    fn f_c(&self) -> bool {
        (self.f >> 4) & 1 == 1
    }

    fn reg_to_string(idx: u8) -> String {
        match idx {
            0 => String::from("B"),
            1 => String::from("C"),
            2 => String::from("D"),
            3 => String::from("E"),
            4 => String::from("H"),
            5 => String::from("L"),
            6 => String::from("(HL)"),
            7 => String::from("A"),
            _ => panic!("Invalid operand index: {}", idx),
        }
    }

    fn reg16_to_string(idx: u8) -> String {
        match idx {
            0 => String::from("BC"),
            1 => String::from("DE"),
            2 => String::from("HL"),
            3 => String::from("SP"),
            _ => panic!("Invalid operand index: {}", idx),
        }
    }

    /// Write 8-bit operand
    fn write_r8(&mut self, idx: u8, val: u8) {
        match idx {
            0 => self.b = val,
            1 => self.c = val,
            2 => self.d = val,
            3 => self.e = val,
            4 => self.h = val,
            5 => self.l = val,
            6 => {
                let hl = self.hl();
                self.mmu.write(hl, val);
            }
            7 => self.a = val,
            _ => panic!("Invalid operand index: {}", idx),
        }
    }

    /// Read 8-bit operand
    fn read_r8(&mut self, idx: u8) -> u8 {
        match idx {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => self.mmu.read(self.hl()),
            7 => self.a,
            _ => panic!("Invalid operand index: {}", idx),
        }
    }

    /// Write 16-bit operand
    fn write_r16(&mut self, idx: u8, val: u16) {
        match idx {
            0 => self.set_bc(val),
            1 => self.set_de(val),
            2 => self.set_hl(val),
            3 => self.sp = val,
            _ => panic!("Invalid operand index: {}", idx),
        }
    }

    /// Read 16-bit operand
    fn read_r16(&mut self, idx: u8) -> u16 {
        match idx {
            0 => self.bc(),
            1 => self.de(),
            2 => self.hl(),
            3 => self.sp,
            _ => panic!("Invalid operand index: {}", idx),
        }
    }

    /// Read 8-bit immediate from memory
    fn read_d8(&mut self) -> u8 {
        let imm = self.mmu.read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        imm
    }

    /// Read 16-bit immediate from memory
    fn read_d16(&mut self) -> u16 {
        let imm = self.mmu.read16(self.pc);
        self.pc = self.pc.wrapping_add(2);

        imm
    }

    /// NOP
    fn nop(&mut self) {
        debug!("NOP");
    }

    /// LD r16, d16
    fn ld_r16_d16(&mut self, reg: u8) {
        let val = self.read_d16();

        debug!("LD {}, 0x{:04x}", Self::reg16_to_string(reg), val);

        self.write_r16(reg, val);
    }

    /// LD (d16), SP
    fn ld_ind_d16_sp(&mut self) {
        let addr = self.read_d16();

        debug!("LD (0x{:04x}), SP", addr);

        self.mmu.write16(addr, self.sp);
    }

    /// LD SP, HL
    fn ld_sp_hl(&mut self) {
        debug!("LD SP, HL");

        self.sp = self.hl();
    }

    /// ADD HL, r16
    fn add_hl_r16(&mut self, reg: u8) {
        debug!("ADD HL, {}", Self::reg16_to_string(reg));

        let hl = self.hl();
        let val = self.read_r16(reg);

        let half_carry = (hl & 0xfff) + (val & 0xfff) > 0xfff;
        let (res, carry) = hl.overflowing_add(val);
        self.set_hl(res);

        self.set_f_n(false);
        self.set_f_h(half_carry);
        self.set_f_c(carry);
    }

    fn _add_sp(&mut self, offset: i8) -> u16 {
        let val = offset as i16 as u16;

        let half_carry = (self.sp & 0x0f) + (val & 0x0f) > 0x0f;
        let carry = (self.sp & 0xff) + (val & 0xff) > 0xff;

        self.set_f_z(false);
        self.set_f_n(false);
        self.set_f_h(half_carry);
        self.set_f_c(carry);

        self.sp.wrapping_add(val)
    }

    /// ADD SP, d8
    fn add_sp_d8(&mut self) {
        let val = self.read_d8() as i8;

        debug!("ADD SP, {}", val);

        self.sp = self._add_sp(val);
    }

    /// LD HL, SP+d8
    fn ld_hl_sp_d8(&mut self) {
        let offset = self.read_d8() as i8;

        debug!("LD HL, SP{:+}", offset);

        let res = self._add_sp(offset);
        self.set_hl(res);
    }

    /// AND r8
    fn and_r8(&mut self, reg: u8) {
        debug!("AND {}", Self::reg_to_string(reg));

        let res = self.a & self.read_r8(reg);

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(true);
        self.set_f_c(false);
    }

    /// OR r8
    fn or_r8(&mut self, reg: u8) {
        debug!("OR {}", Self::reg_to_string(reg));

        let res = self.a | self.read_r8(reg);

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(false);
    }

    /// XOR r8
    fn xor_r8(&mut self, reg: u8) {
        debug!("XOR {}", Self::reg_to_string(reg));

        let res = self.a ^ self.read_r8(reg);

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(false);
    }

    /// CP r8
    fn cp_r8(&mut self, reg: u8) {
        debug!("CP {}", Self::reg_to_string(reg));

        let a = self.a;
        let val = self.read_r8(reg);

        self.set_f_z(a == val);
        self.set_f_n(true);
        self.set_f_h(a & 0x0f < val & 0x0f);
        self.set_f_c(a < val);
    }

    /// Decimal adjust register A
    fn daa(&mut self) {
        debug!("DAA");

        let mut a = self.a;

        if !self.f_n() {
            if self.f_c() || a > 0x99 {
                a = a.wrapping_add(0x60);
                self.set_f_c(true);
            }
            if self.f_h() || a & 0x0f > 0x09 {
                a = a.wrapping_add(0x06);
            }
        } else {
            if self.f_c() {
                a = a.wrapping_sub(0x60);
            }
            if self.f_h() {
                a = a.wrapping_sub(0x06);
            }
        }

        self.a = a;

        self.set_f_z(a == 0);
        self.set_f_h(false);
    }

    /// Complement A
    fn cpl(&mut self) {
        debug!("CPL");

        self.a = !self.a;
        self.set_f_n(true);
        self.set_f_h(true);
    }

    /// Complement carry flag
    fn ccf(&mut self) {
        debug!("CCF");

        self.set_f_n(false);
        self.set_f_h(false);

        let c = self.f_c();
        self.set_f_c(!c);
    }

    /// Set carry flag
    fn scf(&mut self) {
        debug!("SCF");

        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(true);
    }

    fn _add(&mut self, val: u8) {
        let half_carry = (self.a & 0xf) + (val & 0xf) > 0xf;
        let (res, carry) = self.a.overflowing_add(val);

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(half_carry);
        self.set_f_c(carry);
    }

    fn add_r8(&mut self, reg: u8) {
        let val = self.read_r8(reg);

        debug!("ADD {}", Self::reg_to_string(reg));

        self._add(val);
    }

    fn adc_r8(&mut self, reg: u8) {
        let val = self.read_r8(reg);

        debug!("ADC {}", Self::reg_to_string(reg));

        self._adc(val);
    }

    fn sub_r8(&mut self, reg: u8) {
        let val = self.read_r8(reg);

        debug!("SUB {}", Self::reg_to_string(reg));

        self._sub(val);
    }

    fn sbc_r8(&mut self, reg: u8) {
        let val = self.read_r8(reg);

        debug!("SBC {}", Self::reg_to_string(reg));

        self._sbc(val);
    }

    /// ADD d8
    fn add_d8(&mut self) {
        let val = self.read_d8();

        debug!("ADD 0x{:02x}", val);

        self._add(val);
    }

    fn _sub(&mut self, val: u8) {
        let half_carry = (self.a & 0xf) < (val & 0xf);
        let (res, carry) = self.a.overflowing_sub(val);

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(true);
        self.set_f_h(half_carry);
        self.set_f_c(carry);
    }

    /// SUB d8
    fn sub_d8(&mut self) {
        let val = self.read_d8();

        debug!("SUB 0x{:02x}", val);

        self._sub(val);
    }

    fn _adc(&mut self, val: u8) {
        let c = if self.f_c() { 1 } else { 0 };

        let res = self.a.wrapping_add(val).wrapping_add(c);
        let half_carry = (self.a & 0xf) + (val & 0xf) + c > 0xf;
        let carry = (self.a as u16) + (val as u16) + (c as u16) > 0xff;

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(half_carry);
        self.set_f_c(carry);
    }

    /// ADC d8
    fn adc_d8(&mut self) {
        let val = self.read_d8();

        debug!("ADC 0x{:02x}", val);

        self._adc(val);
    }

    fn _sbc(&mut self, val: u8) {
        let c = if self.f_c() { 1 } else { 0 };

        let res = self.a.wrapping_sub(val).wrapping_sub(c);
        let half_carry = (self.a & 0xf) < (val & 0xf) + c;
        let carry = (self.a as u16) < (val as u16) + (c as u16);

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(true);
        self.set_f_h(half_carry);
        self.set_f_c(carry);
    }

    /// SBC d8
    fn sbc_d8(&mut self) {
        let val = self.read_d8();

        debug!("SBC 0x{:02x}", val);

        self._sbc(val);
    }

    /// AND d8
    fn and_d8(&mut self) {
        let val = self.read_d8();

        debug!("AND 0x{:02x}", val);

        let res = self.a & val;

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(true);
        self.set_f_c(false);
    }

    /// OR d8
    fn or_d8(&mut self) {
        let val = self.read_d8();

        debug!("OR 0x{:02x}", val);

        let res = self.a | val;

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(false);
    }

    /// XOR d8
    fn xor_d8(&mut self) {
        let val = self.read_d8();

        debug!("XOR 0x{:02x}", val);

        let res = self.a ^ val;

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(false);
    }

    /// CP d8
    fn cp_d8(&mut self) {
        let imm = self.read_d8();

        debug!("CP 0x{:02x}", imm);

        let a = self.a;

        self.set_f_z(a == imm);
        self.set_f_n(true);
        self.set_f_h(a & 0x0f < imm & 0x0f);
        self.set_f_c(a < imm);
    }

    fn ldi_hl_a(&mut self) {
        debug!("LD (HL+), A");

        let addr = self.hl();
        self.mmu.write(addr, self.a);
        let hl = self.hl();
        self.set_hl(hl.wrapping_add(1));
    }

    fn ldd_hl_a(&mut self) {
        debug!("LD (HL-), A");

        let addr = self.hl();
        self.mmu.write(addr, self.a);
        let hl = self.hl();
        self.set_hl(hl.wrapping_sub(1));
    }

    fn ldi_a_hl(&mut self) {
        debug!("LD A, (HL+)");

        let addr = self.hl();
        self.a = self.mmu.read(addr);
        let hl = self.hl();
        self.set_hl(hl.wrapping_add(1));
    }

    fn ldd_a_hl(&mut self) {
        debug!("LD A, (HL-)");

        let addr = self.hl();
        self.a = self.mmu.read(addr);
        let hl = self.hl();
        self.set_hl(hl.wrapping_sub(1));
    }

    fn ld_ind_bc_a(&mut self) {
        debug!("LD (BC), A");

        let addr = self.bc();
        self.mmu.write(addr, self.a);
    }

    fn ld_ind_de_a(&mut self) {
        debug!("LD (DE), A");

        let addr = self.de();
        self.mmu.write(addr, self.a);
    }

    fn ld_a_ind_bc(&mut self) {
        debug!("LD A, (BC)");

        self.a = self.mmu.read(self.bc());
    }

    fn ld_a_ind_de(&mut self) {
        debug!("LD A, (DE)");

        self.a = self.mmu.read(self.de());
    }

    /// Test bit
    fn bit(&mut self, pos: u8, reg: u8) {
        debug!("BIT {}, {}", pos, Self::reg_to_string(reg));

        let z = (self.read_r8(reg) >> pos & 1) == 0;
        self.set_f_z(z);
        self.set_f_n(false);
        self.set_f_h(true);
    }

    /// Set bit
    fn set(&mut self, pos: u8, reg: u8) {
        debug!("SET {}, {}", pos, Self::reg_to_string(reg));

        let val = self.read_r8(reg);
        self.write_r8(reg, val | (1 << pos));
    }

    /// Reset bit
    fn res(&mut self, pos: u8, reg: u8) {
        debug!("RES {}, {}", pos, Self::reg_to_string(reg));

        let val = self.read_r8(reg);
        self.write_r8(reg, val & !(1 << pos));
    }

    fn _rl(&mut self, reg: u8) {
        let orig = self.read_r8(reg);
        let res = (orig << 1) | (if self.f_c() { 1 } else { 0 });
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(orig >> 7 & 1 == 1);
    }

    /// Rotate left through carry
    fn rl(&mut self, reg: u8) {
        debug!("RL {}", Self::reg_to_string(reg));

        self._rl(reg);
    }

    fn _rlc(&mut self, reg: u8) {
        let orig = self.read_r8(reg);
        let res = orig.rotate_left(1);
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(orig >> 7 & 1 == 1);
    }

    /// Rotate left
    fn rlc(&mut self, reg: u8) {
        debug!("RLC {}", Self::reg_to_string(reg));

        self._rlc(reg);
    }

    fn _rr(&mut self, reg: u8) {
        let orig = self.read_r8(reg);
        let res = (orig >> 1) | (if self.f_c() { 1 } else { 0 } << 7);
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(orig & 1 == 1);
    }

    /// Rotate right through carry
    fn rr(&mut self, reg: u8) {
        debug!("RR {}", Self::reg_to_string(reg));

        self._rr(reg);
    }

    fn _rrc(&mut self, reg: u8) {
        let orig = self.read_r8(reg);
        let res = orig.rotate_right(1);
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(orig & 1 == 1);
    }

    /// Rotate right
    fn rrc(&mut self, reg: u8) {
        debug!("RRC {}", Self::reg_to_string(reg));

        self._rrc(reg);
    }

    /// Shift left into carry
    fn sla(&mut self, reg: u8) {
        debug!("SLA {}", Self::reg_to_string(reg));

        let orig = self.read_r8(reg);
        let res = orig << 1;
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(orig & 0x80 > 0);
    }

    /// Shift right into carry
    fn sra(&mut self, reg: u8) {
        debug!("SRA {}", Self::reg_to_string(reg));

        let orig = self.read_r8(reg);
        let res = (orig >> 1) | (orig & 0x80);
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(orig & 1 > 0);
    }

    /// Swap low/hi-nibble
    fn swap(&mut self, reg: u8) {
        debug!("SWAP {}", Self::reg_to_string(reg));

        let orig = self.read_r8(reg);
        let res = ((orig & 0x0f) << 4) | ((orig & 0xf0) >> 4);
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(false);
    }

    /// Shift right through carry
    fn srl(&mut self, reg: u8) {
        debug!("SRL {}", Self::reg_to_string(reg));

        let orig = self.read_r8(reg);
        let res = orig >> 1;
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(orig & 1 == 1);
    }

    fn _jp(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn jp_nz_d8(&mut self) {
        let addr = self.read_d16();

        debug!("JP NZ, 0x{:04x}", addr);

        if !self.f_z() {
            self._jp(addr);
        }
    }

    fn jp_nc_d8(&mut self) {
        let addr = self.read_d16();

        debug!("JP NC, 0x{:04x}", addr);

        if !self.f_c() {
            self._jp(addr);
        }
    }

    fn jp_z_d8(&mut self) {
        let addr = self.read_d16();

        debug!("JP Z, 0x{:04x}", addr);

        if self.f_z() {
            self._jp(addr);
        }
    }

    fn jp_c_d8(&mut self) {
        let addr = self.read_d16();

        debug!("JP C, 0x{:04x}", addr);

        if self.f_c() {
            self._jp(addr);
        }
    }

    /// Unconditional jump to d16
    fn jp_d16(&mut self) {
        let address = self.read_d16();

        debug!("JP 0x{:04x}", address);

        self.pc = address;
    }

    /// Unconditional jump to HL
    fn jp_hl(&mut self) {
        debug!("JP (HL)");

        self.pc = self.hl();
    }

    /// Jump to pc+d8 if not Z
    fn jr_nz_d8(&mut self) {
        let offset = self.read_d8() as i8;

        debug!("JR NZ, {}", offset);

        if !self.f_z() {
            self._jr(offset);
        }
    }

    /// Jump to pc+d8 if not C
    fn jr_nc_d8(&mut self) {
        let offset = self.read_d8() as i8;

        debug!("JR NC, {}", offset);

        if !self.f_c() {
            self._jr(offset);
        }
    }

    /// Jump to pc+d8 if Z
    fn jr_z_d8(&mut self) {
        let offset = self.read_d8() as i8;

        debug!("JR Z, {}", offset);

        if self.f_z() {
            self._jr(offset);
        }
    }

    /// Jump to pc+d8 if C
    fn jr_c_d8(&mut self) {
        let offset = self.read_d8() as i8;

        debug!("JR C, {}", offset);

        if self.f_c() {
            self._jr(offset);
        }
    }

    fn _jr(&mut self, offset: i8) {
        self.pc = self.pc.wrapping_add(offset as u16);
    }

    /// Jump to pc+d8
    fn jr_d8(&mut self) {
        let offset = self.read_d8() as i8;

        debug!("JR {}", offset);

        self._jr(offset);
    }

    fn ld_io_d8_a(&mut self) {
        let offset = self.read_d8() as u16;
        let addr = 0xff00 | offset;

        debug!("LD (0xff00+0x{:02x}), A", offset);

        self.mmu.write(addr, self.a);
    }

    fn ld_a_io_d8(&mut self) {
        let offset = self.read_d8() as u16;
        let addr = 0xff00 | offset;

        debug!("LD A, (0xff00+0x{:02x})", offset);

        self.a = self.mmu.read(addr);
    }

    fn ld_io_c_a(&mut self) {
        let addr = 0xff00 | self.c as u16;

        debug!("LD (0xff00+C), A");

        self.mmu.write(addr, self.a);
    }

    fn ld_a_io_c(&mut self) {
        let addr = 0xff00 | self.c as u16;

        debug!("LD A, (0xff00+C)");

        self.a = self.mmu.read(addr);
    }

    /// LD r8, d8
    fn ld_r8_d8(&mut self, reg: u8) {
        let imm = self.read_d8();

        debug!("LD {}, 0x{:02x}", Self::reg_to_string(reg), imm);

        self.write_r8(reg, imm);
    }

    /// INC r8
    fn inc_r8(&mut self, reg: u8) {
        debug!("INC {}", Self::reg_to_string(reg));

        let orig = self.read_r8(reg);
        let res = orig.wrapping_add(1);
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_h(orig & 0x0f == 0x0f);
        self.set_f_n(false);
    }

    /// DEC r8
    fn dec_r8(&mut self, reg: u8) {
        debug!("DEC {}", Self::reg_to_string(reg));

        let orig = self.read_r8(reg);
        let res = orig.wrapping_sub(1);
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_h(orig & 0x0f == 0x00);
        self.set_f_n(true);
    }

    /// LD r8, r8
    fn ld_r8_r8(&mut self, reg1: u8, reg2: u8) {
        debug!(
            "LD {}, {}",
            Self::reg_to_string(reg1),
            Self::reg_to_string(reg2)
        );

        let val = self.read_r8(reg2);
        self.write_r8(reg1, val);
    }

    fn _call(&mut self, addr: u16) {
        self.sp = self.sp.wrapping_sub(2);
        self.mmu.write16(self.sp, self.pc);
        self.pc = addr;
    }

    /// CALL d16
    fn call_d16(&mut self) {
        let addr = self.read_d16();

        debug!("CALL 0x{:04x}", addr);

        self._call(addr);
    }

    /// CALL NZ, d16
    fn call_nz_d16(&mut self) {
        let addr = self.read_d16();

        debug!("CALL NZ, 0x{:04x}", addr);

        if !self.f_z() {
            self._call(addr);
        }
    }

    /// CALL NC, d16
    fn call_nc_d16(&mut self) {
        let addr = self.read_d16();

        debug!("CALL NC, 0x{:04x}", addr);

        if !self.f_c() {
            self._call(addr);
        }
    }

    /// CALL Z, d16
    fn call_z_d16(&mut self) {
        let addr = self.read_d16();

        debug!("CALL Z, 0x{:04x}", addr);

        if self.f_z() {
            self._call(addr);
        }
    }

    /// CALL C, d16
    fn call_c_d16(&mut self) {
        let addr = self.read_d16();

        debug!("CALL C, 0x{:04x}", addr);

        if self.f_c() {
            self._call(addr);
        }
    }

    fn rst(&mut self, addr: u8) {
        debug!("RST 0x{:02x}", addr);
        self._call(addr as u16);
    }

    fn _ret(&mut self) {
        self.pc = self.mmu.read16(self.sp);
        self.sp = self.sp.wrapping_add(2);
    }

    /// RET
    fn ret(&mut self) {
        debug!("RET");

        self._ret();
    }

    /// RET NZ
    fn ret_nz(&mut self) {
        debug!("RET NZ");

        if !self.f_z() {
            self._ret();
        }
    }

    /// RET NC
    fn ret_nc(&mut self) {
        debug!("RET NC");

        if !self.f_c() {
            self._ret();
        }
    }

    /// RET Z
    fn ret_z(&mut self) {
        debug!("RET Z");

        if self.f_z() {
            self._ret();
        }
    }

    /// RET C
    fn ret_c(&mut self) {
        debug!("RET C");

        if self.f_c() {
            self._ret();
        }
    }

    /// PUSH BC
    fn push_bc(&mut self) {
        debug!("PUSH BC");

        self.sp = self.sp.wrapping_sub(2);
        let val = self.bc();
        self.mmu.write16(self.sp, val);
    }

    /// PUSH DE
    fn push_de(&mut self) {
        debug!("PUSH DE");

        self.sp = self.sp.wrapping_sub(2);
        let val = self.de();
        self.mmu.write16(self.sp, val);
    }

    /// PUSH HL
    fn push_hl(&mut self) {
        debug!("PUSH HL");

        self.sp = self.sp.wrapping_sub(2);
        let val = self.hl();
        self.mmu.write16(self.sp, val);
    }

    /// PUSH AF
    fn push_af(&mut self) {
        debug!("PUSH AF");

        self.sp = self.sp.wrapping_sub(2);
        let val = self.af();
        self.mmu.write16(self.sp, val);
    }

    /// POP BC
    fn pop_bc(&mut self) {
        debug!("POP BC");

        let val = self.mmu.read16(self.sp);
        self.set_bc(val);
        self.sp = self.sp.wrapping_add(2);
    }

    /// POP DE
    fn pop_de(&mut self) {
        debug!("POP DE");

        let val = self.mmu.read16(self.sp);
        self.set_de(val);
        self.sp = self.sp.wrapping_add(2);
    }

    /// POP HL
    fn pop_hl(&mut self) {
        debug!("POP HL");

        let val = self.mmu.read16(self.sp);
        self.set_hl(val);
        self.sp = self.sp.wrapping_add(2);
    }

    /// POP AF
    fn pop_af(&mut self) {
        debug!("POP AF");

        // lower nibble of F is always zero
        let val = self.mmu.read16(self.sp) & 0xfff0;
        self.set_af(val);
        self.sp = self.sp.wrapping_add(2);
    }

    fn rlca(&mut self) {
        debug!("RLCA");

        self._rlc(7);
        self.set_f_z(false);
    }

    fn rla(&mut self) {
        debug!("RLA");

        self._rl(7);
        self.set_f_z(false);
    }

    fn rrca(&mut self) {
        debug!("RLRA");

        self._rrc(7);
        self.set_f_z(false);
    }

    fn rra(&mut self) {
        debug!("RRA");

        self._rr(7);
        self.set_f_z(false);
    }

    fn inc_r16(&mut self, reg: u8) {
        debug!("INC {}", Self::reg16_to_string(reg));

        let val = self.read_r16(reg);
        self.write_r16(reg, val.wrapping_add(1));
    }

    fn dec_r16(&mut self, reg: u8) {
        debug!("DEC {}", Self::reg16_to_string(reg));

        let val = self.read_r16(reg);
        self.write_r16(reg, val.wrapping_sub(1));
    }

    fn ld_ind_d16_a(&mut self) {
        let addr = self.read_d16();

        debug!("LD (0x{:04x}), A", addr);

        self.mmu.write(addr, self.a);
    }

    fn ld_a_ind_d16(&mut self) {
        let addr = self.read_d16();

        debug!("LD A, (0x{:04x})", addr);

        self.a = self.mmu.read(addr);
    }

    /// Disable interrupt
    fn di(&mut self) {
        debug!("DI");

        // TODO disable interrupt
    }

    /// Enable interrupt
    fn ei(&mut self) {
        debug!("EI");

        // TODO enable interrupt
    }

    /// Enable interrupt and return
    fn reti(&mut self) {
        debug!("RETI");

        // TODO enable interrupt

        self._ret();
    }

    /// Prefixed instructions
    fn prefix(&mut self) {
        let opcode = self.read_d8();
        let pos = opcode >> 3 & 0x7;
        let reg = opcode & 0x7;

        match opcode {
            0x00...0x07 => self.rlc(reg),
            0x08...0x0f => self.rrc(reg),
            0x10...0x17 => self.rl(reg),
            0x18...0x1f => self.rr(reg),
            0x20...0x27 => self.sla(reg),
            0x28...0x2f => self.sra(reg),
            0x30...0x37 => self.swap(reg),
            0x38...0x3f => self.srl(reg),
            0x40...0x7f => self.bit(pos, reg),
            0x80...0xbf => self.res(pos, reg),
            0xc0...0xff => self.set(pos, reg),
            _ => panic!("Unimplemented opcode 0xcb 0x{:x}", opcode),
        }
    }

    pub fn step(&mut self) {
        let opcode = self.read_d8();
        let reg = opcode & 7;
        let reg2 = opcode >> 3 & 7;

        match opcode {
            // NOP
            0x00 => self.nop(),

            // LD r16, d16
            0x01 => self.ld_r16_d16(0),
            0x11 => self.ld_r16_d16(1),
            0x21 => self.ld_r16_d16(2),
            0x31 => self.ld_r16_d16(3),

            // LD (d16), SP
            0x08 => self.ld_ind_d16_sp(),

            // LD SP, HL
            0xf9 => self.ld_sp_hl(),

            // LD A, (r16)
            0x02 => self.ld_ind_bc_a(),
            0x12 => self.ld_ind_de_a(),
            0x0a => self.ld_a_ind_bc(),
            0x1a => self.ld_a_ind_de(),

            // PUSH r16
            0xc5 => self.push_bc(),
            0xd5 => self.push_de(),
            0xe5 => self.push_hl(),
            0xf5 => self.push_af(),

            // POP r16
            0xc1 => self.pop_bc(),
            0xd1 => self.pop_de(),
            0xe1 => self.pop_hl(),
            0xf1 => self.pop_af(),

            // Conditional absolute jump
            0xc2 => self.jp_nz_d8(),
            0xd2 => self.jp_nc_d8(),
            0xca => self.jp_z_d8(),
            0xda => self.jp_c_d8(),

            // Unconditional absolute jump
            0xc3 => self.jp_d16(),
            0xe9 => self.jp_hl(),

            // Conditional relative jump
            0x20 => self.jr_nz_d8(),
            0x30 => self.jr_nc_d8(),
            0x28 => self.jr_z_d8(),
            0x38 => self.jr_c_d8(),

            // Unconditional relative jump
            0x18 => self.jr_d8(),

            // Bit rotate on A
            0x07 => self.rlca(),
            0x17 => self.rla(),
            0x0f => self.rrca(),
            0x1f => self.rra(),

            // Arithmethic/logical operation on 16-bit register
            0x09 => self.add_hl_r16(0),
            0x19 => self.add_hl_r16(1),
            0x29 => self.add_hl_r16(2),
            0x39 => self.add_hl_r16(3),
            0xe8 => self.add_sp_d8(),
            0xf8 => self.ld_hl_sp_d8(),

            // Arithmethic/logical operation on 8-bit register
            0x80...0x87 => self.add_r8(reg),
            0x88...0x8f => self.adc_r8(reg),
            0x90...0x97 => self.sub_r8(reg),
            0x98...0x9f => self.sbc_r8(reg),
            0xa0...0xa7 => self.and_r8(reg),
            0xb0...0xb7 => self.or_r8(reg),
            0xa8...0xaf => self.xor_r8(reg),
            0xb8...0xbf => self.cp_r8(reg),

            // DAA
            0x27 => self.daa(),

            // CPL
            0x2f => self.cpl(),

            // SCF, CCF
            0x37 => self.scf(),
            0x3f => self.ccf(),

            // Arithmethic/logical operation on A
            0xc6 => self.add_d8(),
            0xd6 => self.sub_d8(),
            0xe6 => self.and_d8(),
            0xf6 => self.or_d8(),
            0xce => self.adc_d8(),
            0xde => self.sbc_d8(),
            0xee => self.xor_d8(),
            0xfe => self.cp_d8(),

            // LDI, LDD
            0x22 => self.ldi_hl_a(),
            0x32 => self.ldd_hl_a(),
            0x2a => self.ldi_a_hl(),
            0x3a => self.ldd_a_hl(),

            // LD IO port
            0xe0 => self.ld_io_d8_a(),
            0xf0 => self.ld_a_io_d8(),
            0xe2 => self.ld_io_c_a(),
            0xf2 => self.ld_a_io_c(),

            // LD r8, d8
            0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e | 0x36 | 0x3e => self.ld_r8_d8(reg2),

            // INC r8
            0x04 | 0x0c | 0x14 | 0x1c | 0x24 | 0x2c | 0x34 | 0x3c => self.inc_r8(reg2),

            // DEC r8
            0x05 | 0x0d | 0x15 | 0x1d | 0x25 | 0x2d | 0x35 | 0x3d => self.dec_r8(reg2),

            // LD r8, r8
            0x40...0x75 | 0x77...0x7f => self.ld_r8_r8(reg2, reg),

            // LD (d16), A
            0xea => self.ld_ind_d16_a(),

            // LD A, (d16)
            0xfa => self.ld_a_ind_d16(),

            // INC, DEC r16
            0x03 => self.inc_r16(0),
            0x13 => self.inc_r16(1),
            0x23 => self.inc_r16(2),
            0x33 => self.inc_r16(3),
            0x0b => self.dec_r16(0),
            0x1b => self.dec_r16(1),
            0x2b => self.dec_r16(2),
            0x3b => self.dec_r16(3),

            // Unconditional call
            0xcd => self.call_d16(),

            // Conditional call
            0xc4 => self.call_nz_d16(),
            0xd4 => self.call_nc_d16(),
            0xcc => self.call_z_d16(),
            0xdc => self.call_c_d16(),

            // Unconditional ret
            0xc9 => self.ret(),

            // Conditional ret
            0xc0 => self.ret_nz(),
            0xd0 => self.ret_nc(),
            0xc8 => self.ret_z(),
            0xd8 => self.ret_c(),

            // RETI
            0xd9 => self.reti(),

            // RST
            0xc7 => self.rst(0x00),
            0xcf => self.rst(0x08),
            0xd7 => self.rst(0x10),
            0xdf => self.rst(0x18),
            0xe7 => self.rst(0x20),
            0xef => self.rst(0x28),
            0xf7 => self.rst(0x30),
            0xff => self.rst(0x38),

            // DI, EI
            0xf3 => self.di(),
            0xfb => self.ei(),

            // CB prefixed
            0xcb => self.prefix(),
            _ => panic!("Unimplemented opcode 0x{:x}", opcode),
        }
    }

    #[allow(dead_code)]
    pub fn dump(&self) {
        println!("CPU State:");
        println!("PC: 0x{:04x}  SP: 0x{:04x}", self.pc, self.sp);
        println!("AF: 0x{:04x}  BC: 0x{:04x}", self.af(), self.bc());
        println!("DE: 0x{:04x}  HL: 0x{:04x}", self.de(), self.hl());
    }
}
