use std::collections::HashMap;

use common::{
    cfg::Word,
    isa::{
        Mneumonics,
        Operand,
        OperandInner,
        Operation,
        Register,
    },
};

use crate::lexer::Token;

const MNEUMONIC_SIZE: Word = 1;
const METADATA_SIZE: Word = 1;
const LITERAL_SIZE: Word = 2;
const REGISTER_SIZE: Word = 2;

/// Evaluate a stack of tokens assumably constituting a whole and valid operation.
///
/// This will drain the stack, and will panic in the event that the stack does not constitute an
/// entire valid operation.
fn evaluate_token_stack(stack: &mut Vec<Token>, symbol_table: &HashMap<String, Word>) -> Operation {
    let mut tokens = stack.drain(0..);

    let Some(Token::Mneumonic(mneumonic)) = tokens.next() else {
        panic!("Invalid token stack");
    };

    let mneumonic = Mneumonics::try_from(mneumonic.as_str()).expect("Invalid mneumonic");
    let mut args = Vec::new();

    while let Some(token) = tokens.next() {
        let arg = match token {
            Token::Deref => {
                Operand::LValue(match tokens.next().expect("Unexpected end of tokens") {
                    Token::LabelInvoc(name) => {
                        let addr = symbol_table
                            .get(&name)
                            .unwrap_or_else(|| panic!("Invalid label: {name}"));
                        OperandInner::Literal(*addr as Word)
                    }
                    Token::Register(name) => OperandInner::Register(
                        Register::try_from(name.as_str()).expect("Invalid register name"),
                    ),
                    Token::Immediate(value) => {
                        OperandInner::Literal(value.parse().expect("Invalid immediate"))
                    }
                    _ => panic!("Unexpected token in stack"),
                })
            }
            Token::LabelInvoc(name) => Operand::RValue(OperandInner::Literal(
                *symbol_table.get(&name).expect("Invalid label"),
            )),
            Token::Register(name) => Operand::RValue(OperandInner::Register(
                Register::try_from(name.as_str()).expect("Invalid register name"),
            )),
            Token::Immediate(val) => Operand::RValue(OperandInner::Literal(
                val.parse().expect("Invalid immediate"),
            )),
            _ => panic!("Unexpected token in stack"),
        };
        args.push(arg);

        // Consume comma (if not end)
        let (Some(Token::Comma) | None) = tokens.next() else {
            panic!("Operands must be seperated by commas");
        };
    }

    match mneumonic {
        Mneumonics::Add => Operation::Add(args[0], args[1]),
        Mneumonics::Sub => Operation::Sub(args[0], args[1]),
        Mneumonics::Mov => Operation::Mov(args[0], args[1]),
        Mneumonics::Jmp => Operation::Jmp(args[0]),
        Mneumonics::Cmp => Operation::Cmp(args[0], args[1]),
        Mneumonics::Jeq => Operation::Jeq(args[0]),
        Mneumonics::Jne => Operation::Jne(args[0]),
        Mneumonics::Psh => Operation::Psh(args[0]),
        Mneumonics::Pop => Operation::Pop(args[0]),
    }
}

pub fn parse(tokens: Vec<Token>) -> Vec<Operation> {
    let mut label_table: HashMap<String, u16> = HashMap::new();

    // First pass, construct label table, does not validate syntax at all
    tokens.iter().fold(0xF000, |mut lc, token| {
        match token {
            Token::LabelDecl(name) => {
                label_table.insert(name.clone(), lc);
            }
            Token::Register(_) => lc += REGISTER_SIZE,
            // Labels are transposed to immediate addresses during parsing
            Token::Immediate(_) | Token::LabelInvoc(_) => lc += LITERAL_SIZE,
            Token::Mneumonic(_) => lc += MNEUMONIC_SIZE + METADATA_SIZE,
            _ => {}
        }
        lc
    });

    // Second pass, perform actual parsing
    let mut stack = Vec::new();
    let mut ret = Vec::new();
    for token in tokens {
        match stack.last() {
            None => match token {
                Token::LabelDecl(_) | Token::Comment(_) => continue,
                Token::Mneumonic(_) => stack.push(token),
                _ => panic!("Invalid token at beginning of line"),
            },
            Some(Token::Comma) => match token {
                Token::Register(_) | Token::Immediate(_) | Token::LabelInvoc(_) | Token::Deref => {
                    stack.push(token);
                }
                _ => panic!("Invalid token {token:?} following comma"),
            },
            Some(Token::Mneumonic(_)) => match token {
                Token::Register(_) | Token::Immediate(_) | Token::LabelInvoc(_) | Token::Deref => {
                    stack.push(token);
                }
                Token::Mneumonic(_) => {
                    ret.push(evaluate_token_stack(&mut stack, &label_table));
                    stack.push(token);
                }
                Token::LabelDecl(_) => {
                    ret.push(evaluate_token_stack(&mut stack, &label_table));
                }
                Token::Comment(_) => {}
                _ => panic!("Invalid token {token:?} following comma"),
            },
            Some(Token::Register(_)) | Some(Token::Immediate(_)) | Some(Token::LabelInvoc(_)) => {
                match token {
                    Token::Comma => stack.push(token),
                    Token::Mneumonic(_) => {
                        ret.push(evaluate_token_stack(&mut stack, &label_table));
                        stack.push(token);
                    }
                    Token::LabelDecl(_) => {
                        ret.push(evaluate_token_stack(&mut stack, &label_table));
                    }
                    Token::Comment(_) => {}
                    _ => panic!("Invalid token {token:?} following comma"),
                }
            }
            Some(Token::Deref) => match token {
                Token::Register(_) | Token::Immediate(_) | Token::LabelInvoc(_) => {
                    stack.push(token)
                }
                _ => panic!("Invalid token {token:?} following comma"),
            },
            _ => unimplemented!(),
        }
    }

    ret.push(evaluate_token_stack(&mut stack, &label_table));

    ret
}
