use anyhow::{
    Context,
    bail,
};

use crate::WORD;

#[derive(Debug)]
pub enum Operation {
    Add(Operand, Operand),
    Sub(Operand, Operand),
    Mov(Operand, Operand),
}

impl TryFrom<&str> for Operation {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (operation, operands) = value
            .split_once(' ')
            .context(format!("Invalid operation {}", value))?;
        match operation {
            "add" => {
                let mut split_operands = operands.split(&[',', ' ']);
                let op0 = Operand::try_from(
                    split_operands
                        .next()
                        .context(format!("Invalid operands {}", operands))?,
                )?;
                let op1 = Operand::try_from(
                    split_operands
                        .next()
                        .context(format!("Invalid operands {}", operands))?,
                )?;
                Ok(Self::Add(op0, op1))
            }
            "sub" => {
                let mut split_operands = operands.split(&[',', ' ']);
                let op0 = Operand::try_from(
                    split_operands
                        .next()
                        .context(format!("Invalid operands {}", operands))?,
                )?;
                let op1 = Operand::try_from(
                    split_operands
                        .next()
                        .context(format!("Invalid operands {}", operands))?,
                )?;
                Ok(Self::Sub(op0, op1))
            }
            "mov" => {
                let mut split_operands = operands.split(&[',', ' ']);
                let op0 = Operand::try_from(
                    split_operands
                        .next()
                        .context(format!("Invalid operands {}", operands))?,
                )?;
                let op1 = Operand::try_from(
                    split_operands
                        .next()
                        .context(format!("Invalid operands {}", operands))?,
                )?;
                Ok(Self::Mov(op0, op1))
            }
            _ => bail!("Invalid operation: {value}"),
        }
    }
}

#[derive(Debug)]
pub struct Operand {
    pub ty: OperandInner,
    pub word: WORD,
    pub deref: bool,
}

impl Operand {
    pub fn can_store(&self) -> bool {
        self.deref || matches!(self.ty, OperandInner::Register)
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

        let ty = if *iter.peek().context(format!("Invalid operand: {}", value))? == '$' {
            iter.next();
            OperandInner::Register
        } else {
            OperandInner::Literal
        };

        let val: WORD = iter
            .collect::<String>()
            .parse()
            .context(format!("Invalid operand: {}", value))?;

        Ok(Self {
            ty,
            word: val,
            deref,
        })
    }
}

#[derive(Debug)]
pub enum OperandInner {
    Register,
    Literal,
}
