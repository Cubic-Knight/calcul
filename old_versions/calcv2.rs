enum LexerState {
    ExpectingNumber,
    ExpectingOperator,
    ReadingNumberFormat,
    ReadingNumber,
    ReadingOperator,
    IgnoreChar,
}

enum ParsingError {
    EndOfExpr,
    UnmatchedParens(char, usize),
    UnexpectedChar(char, usize)
}

struct Number {
    value: u32,
    neg: bool,
    base: u32
}

const OP_CHARS: &str = "+-*/%|&^><";
enum Op {
    Add, Sub,
    Mul, Div, Mod,
    And, Or, Xor,
    LShift, RShift
}

fn precedence(op: &Op) -> i32 {
    match op {
        Op::And => 1,
        Op::Or => 1,
        Op::Xor => 1,
        Op::LShift => 1,
        Op::RShift => 1,
        Op::Add => 2,
        Op::Sub => 2,
        Op::Mul => 3,
        Op::Div => 3,
        Op::Mod => 3
    }
}

fn apply_top(op_stack: &mut Vec<Op>, stack: &mut Vec<i32>) {
    let func = match op_stack.pop().unwrap() {
        Op::Add => |a, b| a+b,
        Op::Sub => |a, b| a-b,
        Op::Mul => |a, b| a*b,
        Op::Div => |a, b| a/b,
        Op::Mod => |a, b| a%b,
        Op::And => |a, b| a&b,
        Op::Or => |a, b| a|b,
        Op::Xor => |a, b| a^b,
        Op::LShift => |a, b| a<<b,
        Op::RShift => |a, b| a>>b
    };
    let b = stack.pop().unwrap();
    let a = stack.pop().unwrap();
    stack.push(func(a, b));
}

fn exec(expr: &str) -> Result<i32, ParsingError> {
    let mut lexer_state = LexerState::ExpectingNumber;
    let mut stack = Vec::new();
    let mut op_stack = Vec::new();
    let mut op_counts = vec![0];
    let mut parens_indices = Vec::new();
    let mut num = Number{ value: 0, neg: false, base: 10 };
    let mut curr_op = ' ';
    for (index, c) in expr.chars().enumerate() {
        match lexer_state {
            LexerState::ReadingNumberFormat => {
                match c {
                    '+' => (),
                    '-' => num.neg = !num.neg,
                    'b' => {
                        num.base = 2;
                        lexer_state = LexerState::ReadingNumber;
                    },
                    'o' => {
                        num.base = 8;
                        lexer_state = LexerState::ReadingNumber;
                    },
                    'x' => {
                        num.base = 16;
                        lexer_state = LexerState::ReadingNumber;
                    },
                    other => {
                        match other.to_digit(num.base) {
                            Some(d) => {
                                num.value = d;
                                lexer_state = LexerState::ReadingNumber;
                            },
                            None => {
                                let number = match num.neg {
                                    false => num.value as i32,
                                    true => -(num.value as i32)
                                };
                                stack.push(number);
                                lexer_state = LexerState::ExpectingOperator;
                            }
                        };
                    }
                };
            },
            LexerState::ReadingNumber => {
                match c.to_digit(num.base) {
                    Some(d) => num.value = num.base*num.value + d,
                    None => {
                        let number = match num.neg {
                            false => num.value as i32,
                            true => -(num.value as i32)
                        };
                        stack.push(number);
                        lexer_state = LexerState::ExpectingOperator;
                    }
                }
            },
            LexerState::ReadingOperator => {
                let (op, two_chars) = match (curr_op, c) {
                    ('+', _) => (Op::Add, false),
                    ('-', _) => (Op::Sub, false),
                    ('*', _) => (Op::Mul, false),
                    ('/', _) => (Op::Div, false),
                    ('%', _) => (Op::Mod, false),
                    ('&', _) => (Op::And, false),
                    ('|', _) => (Op::Or, false),
                    ('^', _) => (Op::Xor, false),
                    ('<', '<') => (Op::LShift, true),
                    ('>', '>') => (Op::RShift, true),
                    _ => return Err(ParsingError::UnexpectedChar(c, index))
                };
                if *op_counts.last().unwrap() == 0 ||
                    precedence(&op) > precedence(op_stack.last().unwrap())
                {
                    op_stack.push(op);
                    let count = op_counts.pop().unwrap();
                    op_counts.push(count + 1);
                } else {
                    for _ in 0..op_counts.pop().unwrap() {
                        apply_top(&mut op_stack, &mut stack);
                    };
                    op_stack.push(op);
                    op_counts.push(1);
                };
                match two_chars {
                    false => lexer_state = LexerState::ExpectingNumber,
                    true => lexer_state = LexerState::IgnoreChar
                }
            },
            _ => ()
        };
        if c.is_whitespace() { continue; };
        match lexer_state {
            LexerState::IgnoreChar => {
                lexer_state = LexerState::ExpectingNumber
            },
            LexerState::ExpectingOperator => {
                if c == ')' {
                    if op_counts.len() == 1 { return Err(ParsingError::UnmatchedParens(c, index)) };
                    for _ in 0..op_counts.pop().unwrap() {
                        apply_top(&mut op_stack, &mut stack);
                    };
                } else if OP_CHARS.contains(c) {
                    curr_op = c;
                    lexer_state = LexerState::ReadingOperator;
                } else {
                    return Err(ParsingError::UnexpectedChar(c, index));
                };
            },
            LexerState::ExpectingNumber => {
                match c {
                    '0' => {
                        num = Number{ value: 0, neg: false, base: 10 };
                        lexer_state = LexerState::ReadingNumberFormat;
                    }
                    '+' => {
                        num = Number{ value: 0, neg: false, base: 10 };
                        lexer_state = LexerState::ReadingNumberFormat;
                    }
                    '-' => {
                        num = Number{ value: 0, neg: true, base: 10 };
                        lexer_state = LexerState::ReadingNumberFormat;
                    }
                    '(' => {
                        op_counts.push(0);
                        parens_indices.push(index);
                    },
                    other => {
                        match c.to_digit(10) {
                            Some(d) => {
                                num = Number{ value: d, neg: false, base: 10 };
                                lexer_state = LexerState::ReadingNumber;
                            },
                            None => return Err(ParsingError::UnexpectedChar(other, index))
                        }
                    }
                };
            },
            _ => ()
        };
    };
    match lexer_state {
        LexerState::ReadingNumberFormat => {
            let number = match num.neg {
                false => num.value as i32,
                true => -(num.value as i32)
            };
            stack.push(number);
        },
        LexerState::ReadingNumber => {
            let number = match num.neg {
                false => num.value as i32,
                true => -(num.value as i32)
            };
            stack.push(number);
        },
        LexerState::ReadingOperator => return Err(ParsingError::EndOfExpr),
        LexerState::IgnoreChar => return Err(ParsingError::EndOfExpr),
        LexerState::ExpectingNumber => return Err(ParsingError::EndOfExpr),
        LexerState::ExpectingOperator => ()
    };
    if op_counts.len() > 1 {
        let idx = op_counts.len() - 2;
        return Err(ParsingError::UnmatchedParens('(', parens_indices[idx]))
    };
    for _ in 0..op_counts.pop().unwrap() {
        apply_top(&mut op_stack, &mut stack);
    };
    Ok(stack.pop().unwrap())
}

fn main() {
    let expr = match std::env::args().skip(1).next() {
        Some(s) => s,
        None => {
            println!("Error: No argument");
            println!("Usage: calcul.exe [EXPRESSION]");
            return;
        }
    };
    let result = match exec(&expr) {
        Ok(n) => n,
        Err(e) => {
            match e {
                ParsingError::EndOfExpr => println!("Error: Incomplete expression"),
                ParsingError::UnmatchedParens(c, i) => println!("Error: Unmatched '{}' at {}", c, i),
                ParsingError::UnexpectedChar(c, i) => println!("Error: Unexpected '{}' at {}", c, i)
            };
            return;
        }
    };
    println!("{}", result);
}