use anyhow::{
    Context,
    Result,
};
use macros::MachineCode;

#[derive(MachineCode, PartialEq, Debug)]
enum Operation {
    Foo,
    Add(Operand, Operand),
    Sub(Operand, Operand),
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Operand {
    deref: bool,
    reg: bool,
    val: u16,
}

impl Operand {
    pub fn __create(reg: bool, deref: bool, val: u16) -> Self {
        Self { deref, reg, val }
    }

    pub fn is_deref(&self) -> bool {
        self.deref
    }

    pub fn is_reg(&self) -> bool {
        self.reg
    }

    pub fn value_bytes(&self) -> Vec<u8> {
        self.val.to_le_bytes().to_vec()
    }
}

#[test]
fn test_compile() {
    assert_eq!(Operation::Foo.compile(), vec![0, 0]);
    assert_eq!(
        Operation::Add(
            Operand {
                deref: false,
                reg: true,
                val: 64,
            },
            Operand {
                deref: true,
                reg: false,
                val: 0x1337,
            }
        )
        .compile(),
        vec![1, 6, 64, 0, 0x37, 0x13]
    );

    assert_eq!(
        Operation::Sub(
            Operand {
                deref: true,
                reg: true,
                val: 64,
            },
            Operand {
                deref: true,
                reg: false,
                val: 0x1337,
            }
        )
        .compile(),
        vec![2, 7, 64, 0, 0x37, 0x13]
    );
}

#[test]
fn test_consume() {
    let bytes: [u8; _] = [1, 6, 64, 0, 0x37, 0x13, 2, 7, 64, 0, 0x37, 0x13];
    let mut byte_iter = bytes.into_iter();

    assert_eq!(
        Operation::consume(&mut byte_iter).unwrap().unwrap(),
        Operation::Add(
            Operand {
                deref: false,
                reg: true,
                val: 64
            },
            Operand {
                deref: true,
                reg: false,
                val: 0x1337
            }
        )
    );

    assert_eq!(
        Operation::consume(&mut byte_iter).unwrap().unwrap(),
        Operation::Sub(
            Operand {
                deref: true,
                reg: true,
                val: 64,
            },
            Operand {
                deref: true,
                reg: false,
                val: 0x1337,
            }
        )
    );
}

#[test]
fn test_num_bytes() {
    assert_eq!(Mneumonics::Foo.num_bytes(), 2);
}
