use std::fmt::Display;

use anyhow::{
    Context,
    bail,
};

use crate::cfg::Word;

#[derive(Debug, macros::MachineCode)]
pub enum Operation {
    Add(Operand, Operand),
    Sub(Operand, Operand),
    Mov(Operand, Operand),
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let instruction = match self {
            Self::Add(op0, op1) => format!("add {op0},{op1}"),
            Self::Sub(op0, op1) => format!("sub {op0},{op1}"),
            Self::Mov(op0, op1) => format!("mov {op0},{op1}"),
        };
        write!(f, "{instruction}")
    }
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
            _ => bail!("Invalid operation: {value}"),
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug, PartialEq)]
pub enum OperandInner {
    Register(Register),
    Literal(Word),
}

impl OperandInner {
    fn value_bytes(&self) -> Vec<u8> {
        match self {
            Self::Register(reg) => vec![*reg as u8],
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

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
#[repr(u8)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
}

impl TryFrom<&str> for Register {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "r0" => Ok(Self::R0),
            "r1" => Ok(Self::R1),
            "r2" => Ok(Self::R2),
            "r3" => Ok(Self::R3),
            _ => bail!("Invalid register"),
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::R0 => "r0",
            Self::R1 => "r1",
            Self::R2 => "r2",
            Self::R3 => "r3",
        };

        write!(f, "{s}")
    }
}
