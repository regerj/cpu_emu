use std::fmt::Display;

use anyhow::{
    Context,
    bail,
};
use macros::{
    DisplayOp,
    DisplayReg,
    TryFromStr,
    TryFromU16,
};

use crate::cfg::Word;

#[derive(Debug, DisplayOp, macros::MachineCode)]
pub enum Operation {
    Add(Operand, Operand),
    Sub(Operand, Operand),
    Mov(Operand, Operand),
    Jmp(Operand),
    Cmp(Operand, Operand),
    Jeq(Operand),
    Jne(Operand),
    Psh(Operand),
    Pop(Operand),
    End,
    Cal(Operand),
    Ret,
    Sys(Operand),
}

impl TryFrom<&str> for Operation {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (operation, operands) = value
            .split_once(' ')
            .context(format!("Invalid operation {}", value))?;
        match operation {
            "add" => {
                let mut split_operands = operands.split(',');
                let op0 = Operand::try_from(
                    split_operands
                        .next()
                        .context(format!("Invalid operands {}", operands))?
                        .trim(),
                )?;
                let op1 = Operand::try_from(
                    split_operands
                        .next()
                        .context(format!("Invalid operands {}", operands))?
                        .trim(),
                )?;
                Ok(Self::Add(op0, op1))
            }
            "sub" => {
                let mut split_operands = operands.split(',');
                let op0 = Operand::try_from(
                    split_operands
                        .next()
                        .context(format!("Invalid operands {}", operands))?
                        .trim(),
                )?;
                let op1 = Operand::try_from(
                    split_operands
                        .next()
                        .context(format!("Invalid operands {}", operands))?
                        .trim(),
                )?;
                Ok(Self::Sub(op0, op1))
            }
            "mov" => {
                let mut split_operands = operands.split(',');
                let op0 = Operand::try_from(
                    split_operands
                        .next()
                        .context(format!("Invalid operands {}", operands))?
                        .trim(),
                )?;
                let op1 = Operand::try_from(
                    split_operands
                        .next()
                        .context(format!("Invalid operands {}", operands))?
                        .trim(),
                )?;
                Ok(Self::Mov(op0, op1))
            }
            "jmp" => {
                let op0 = Operand::try_from(operands)?;
                Ok(Self::Jmp(op0))
            }
            "cmp" => {
                let mut split_operands = operands.split(',');
                let op0 = Operand::try_from(
                    split_operands
                        .next()
                        .context(format!("Invalid operands {}", operands))?
                        .trim(),
                )?;
                let op1 = Operand::try_from(
                    split_operands
                        .next()
                        .context(format!("Invalid operands {}", operands))?
                        .trim(),
                )?;
                Ok(Self::Cmp(op0, op1))
            }
            _ => bail!("Invalid operation: {value}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Operand {
    LValue(OperandInner),
    RValue(OperandInner),
}

impl Operand {
    pub fn inner(&self) -> &OperandInner {
        match self {
            Self::LValue(inner) => inner,
            Self::RValue(inner) => inner,
        }
    }

    pub fn is_immediate(&self) -> bool {
        if let Self::RValue(inner) = self
            && let OperandInner::Literal(_) = *inner
        {
            true
        } else {
            false
        }
    }

    fn value_bytes(&self) -> Vec<u8> {
        self.inner().value_bytes()
    }

    fn is_deref(&self) -> bool {
        match self {
            Self::LValue(..) => true,
            Self::RValue(..) => false,
        }
    }

    fn is_reg(&self) -> bool {
        match self.inner() {
            OperandInner::Register(..) => true,
            OperandInner::Literal(..) => false,
        }
    }

    fn __create(reg: bool, deref: bool, val: u16) -> Self {
        let inner = if reg {
            OperandInner::Register(Register::try_from(val).unwrap())
        } else {
            OperandInner::Literal(val)
        };

        if deref {
            Self::LValue(inner)
        } else {
            Self::RValue(inner)
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let star = match self {
            Self::LValue(..) => "*",
            _ => "",
        };

        let inner = self.inner();

        write!(f, "{star}{inner}")
    }
}

impl TryFrom<&str> for Operand {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut iter = value.chars().peekable();
        let deref = if *iter.peek().context(format!("Invalid operand: {}", value))? == '*' {
            iter.next();
            true
        } else {
            false
        };

        let inner = if *iter.peek().context(format!("Invalid operand: {}", value))? == '$' {
            iter.next();
            let name = iter.collect::<String>();
            OperandInner::Register(name.as_str().try_into()?)
        } else {
            let val: Word = iter
                .collect::<String>()
                .parse()
                .context(format!("Invalid operand: {}", value))?;
            OperandInner::Literal(val)
        };

        Ok(if deref {
            Self::LValue(inner)
        } else {
            Self::RValue(inner)
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OperandInner {
    Register(Register),
    Literal(Word),
}

impl OperandInner {
    fn value_bytes(&self) -> Vec<u8> {
        match self {
            Self::Register(reg) => vec![*reg as u8, 0],
            Self::Literal(word) => word.to_le_bytes().into(),
        }
    }
}

impl Display for OperandInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Register(name) => write!(f, "${name}"),
            Self::Literal(val) => write!(f, "{val}"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash, TryFromU16, TryFromStr, DisplayReg)]
#[repr(u8)]
pub enum Register {
    /// General Purpose Register 0
    R0,
    /// General Purpose Register 1
    R1,
    /// General Purpose Register 2
    R2,
    /// General Purpose Register 3
    R3,
    /// Instruction Pointer
    IP,
    /// Status Register
    ST,
    /// Stack base
    SB,
    /// Stack pointer
    SP,
    /// Interrupt table
    IT,
}
