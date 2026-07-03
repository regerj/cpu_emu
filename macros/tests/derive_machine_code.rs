use macros::MachineCode;

#[derive(MachineCode)]
enum Operation {
    Foo,
    Add(Operand, Operand),
    Sub(Operand, Operand),
}

struct Operand {
    deref: bool,
    reg: bool,
}

impl Operand {
    pub fn is_deref(&self) -> bool {
        self.deref
    }

    pub fn is_reg(&self) -> bool {
        self.reg
    }

    pub fn value_bytes(&self) -> Vec<u8> {
        if self.is_reg() {
            vec![64]
        } else {
            vec![13, 37]
        }
    }
}

#[test]
fn test_derive_machine_code() {
    assert_eq!(Operation::Foo.compile(), vec![0, 0]);
    assert_eq!(
        Operation::Add(
            Operand {
                deref: false,
                reg: true
            },
            Operand {
                deref: true,
                reg: false
            }
        )
        .compile(),
        vec![1, 6, 64, 13, 37]
    );
    assert_eq!(
        Operation::Sub(
            Operand {
                deref: true,
                reg: true
            },
            Operand {
                deref: true,
                reg: false
            }
        )
        .compile(),
        vec![2, 7, 64, 13, 37]
    );
}
