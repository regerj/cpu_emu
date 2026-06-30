use std::{io::BufRead, ops::Deref};

use anyhow::{Result, bail};
use bitfield_struct::{bitenum, bitfield};

use crate::{cfg::Word, ops::asm};

#[derive(Default)]
pub struct MachineCode {
    inner: Vec::<u8>,
}

impl MachineCode {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn concat(&mut self, other: Self) {
        self.inner.extend_from_slice(&other);
    }
}

impl Deref for MachineCode {
    type Target = Vec::<u8>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub trait ToMachineCode {
    fn assemble(self) -> MachineCode;
}

pub trait FromMachineCode {
}

impl ToMachineCode for asm::Register {
    fn assemble(self) -> MachineCode {
        MachineCode { inner: vec![self as u8] }
    }
}

impl ToMachineCode for Word {
    fn assemble(self) -> MachineCode {
        let mut ret = MachineCode { inner: vec![] };
        for i in 0..size_of::<Word>() {
            ret.inner.push((self >> (i * 8)) as u8);
        }

        ret
    }
}

impl ToMachineCode for &asm::OperandInner {
    fn assemble(self) -> MachineCode {
        match self {
            asm::OperandInner::Register(reg) => reg.assemble(),
            asm::OperandInner::Literal(val) => val.assemble(),
        }
    }
}

impl ToMachineCode for asm::Operation {
    fn assemble(self) -> MachineCode {
        let mut operation = Operation::new();
        operation.configure_op(&self);
        let (arg0, arg1) = match self {
            Self::Add(op0, op1) => (op0, op1),
            Self::Sub(op0, op1) => (op0, op1),
            Self::Mov(op0, op1) => (op0, op1),
        };

        let mut meta = Metadata::new();
        meta.configure_arg0(&arg0);
        meta.configure_arg1(&arg1);

        operation.set_meta(meta);

        let mut assembled = MachineCode { inner: Vec::new() };

        assembled.concat(operation.into_bits().assemble());
        let bits = operation.into_bits();
        for i in 0..size_of_val(&bits) {
            assembled.inner.push((bits >> (i * 8)) as u8);
        }

        // Next assemble the arg values
        let arg0_mc = arg0.inner().assemble();
        let arg1_mc = arg1.inner().assemble();

        assembled.concat(arg0_mc);
        assembled.concat(arg1_mc);

        assembled
    }
}

/// Trait defining that an item can be disassembled back into its assembly representation.
///
/// This should be implemented on consuming reader types like `BufReader` or similar.
pub trait Disassembleable {
    /// The target of the disassembly.
    type ASM;

    /// Attempt to disassemble the type. This may fail in the event of invalid machine code.
    fn try_disassemble(self) -> Result<Vec<Self::ASM>>;
}

impl<T: BufRead> Disassembleable for T {
    type ASM = asm::Operation;
    fn try_disassemble(mut self) -> Result<Vec<Self::ASM>> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        let op = Operation::from_bits(u16::from_le_bytes(buf));

        let mut args = vec![];

        if op.meta().arg0().is_present() {
        }

        match op.op() {
            Instruction::Add | Instruction::Sub | Instruction::Mov => {

            }
            Instruction::Inv => bail!("Invalid instruction"),
        }
        
        unimplemented!()
    }
}

#[bitfield(u16)]
struct Operation {
    #[bits(8)]
    op: Instruction,
    #[bits(8)]
    meta: Metadata,
}

impl Operation {
    fn configure_op(&mut self, op: &asm::Operation) -> &mut Self {
        let op = match op {
            asm::Operation::Add(..) => Instruction::Add,
            asm::Operation::Sub(..) => Instruction::Sub,
            asm::Operation::Mov(..) => Instruction::Mov,
        };

        self.set_op(op);
        self
    }
}

#[bitfield(u8)]
struct Metadata {
    arg0_deref: bool,
    #[bits(2)]
    arg0: ArgSpec,
    arg1_deref: bool,
    #[bits(2)]
    arg1: ArgSpec,
    #[bits(2)]
    __: u8,
}

impl Metadata {
    fn configure_arg0(&mut self, arg: &asm::Operand) -> &mut Self {
        let inner = match arg {
            asm::Operand::LValue(inner) => {
                self.set_arg0_deref(true);
                inner
            }
            asm::Operand::RValue(inner) => {
                self.set_arg0_deref(false);
                inner
            }
        };

        let spec = match inner {
            asm::OperandInner::Register(..) => ArgSpec::Register,
            asm::OperandInner::Literal(..) => ArgSpec::Literal,
        };

        self.set_arg0(spec);
        self
    }

    fn configure_arg1(&mut self, arg: &asm::Operand) -> &mut Self {
        let inner = match arg {
            asm::Operand::LValue(inner) => {
                self.set_arg1_deref(true);
                inner
            }
            asm::Operand::RValue(inner) => {
                self.set_arg1_deref(false);
                inner
            }
        };

        let spec = match inner {
            asm::OperandInner::Register(..) => ArgSpec::Register,
            asm::OperandInner::Literal(..) => ArgSpec::Literal,
        };

        self.set_arg1(spec);
        self
    }
}

#[bitenum]
#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
enum ArgSpec {
    #[fallback]
    NotPresent,
    Literal,
    Register,
}

impl ArgSpec {
    fn is_present(&self) -> bool {
        *self == Self::NotPresent
    }
}

#[bitenum]
#[repr(u8)]
#[derive(Debug)]
enum Instruction {
    Add,
    Sub,
    Mov,
    #[fallback]
    Inv,
}
