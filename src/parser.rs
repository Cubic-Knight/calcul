use crate::fixed::Fixed;
use crate::ops::{OP_CHARS, Op, precedence, apply_top};

enum LexerState {
    ExpectingNumber,
    ExpectingOperator,
    ReadingNumberSign,
    ReadingNumberFormat,
    ReadingNumberWhole,
    ReadingNumberDecimal,
    ReadingNumberTooLong,
    ReadingOperator,
    IgnoreThatChar
}
struct Number {
    whole: u32,
    decimal: u64,
    neg: bool,

    base: u32,
    decimal_count: u32
}
const DEFAULT: Number = Number { whole: 0, decimal: 0, neg: false, base: 10, decimal_count: 0 };

fn decimal_part_is_full(num: &Number) -> bool {
    // decimal part is full when (here they are hardcoded for the 4 usable bases):
    //   log2(num.base) * num.decimal_count >= 32
    match (num.base, num.decimal_count) {
        (2, 32) => true,
        (8, 11) => true,
        (10, 10) => true,
        (16, 8) => true,
        (_, _) => false
    }
}

fn push(num: Number, stack: &mut Vec<Fixed>) {
    let fixed_repr = match num.base {
        2  => ((num.whole as u64) << 32) + (num.decimal << (33 -     num.decimal_count) >> 1),
        8  => ((num.whole as u64) << 32) + (num.decimal << (33 - 3 * num.decimal_count) >> 1),
        16 => ((num.whole as u64) << 32) + (num.decimal << (33 - 4 * num.decimal_count) >> 1),
        10 => {
            let whole_part = (num.whole as u64) << 32;
            let decimal_part = num.decimal * 10u64.pow(10 - num.decimal_count);
            let decimal_part = (decimal_part << 22) / 9765625;
            whole_part + decimal_part
        }
        _  => panic!()
    };
    let final_num = match num.neg {
        false => fixed_repr as i64,
        true => -(fixed_repr as i64)
    };
    stack.push( Fixed::from_i64(final_num) );
}

pub fn exec(expr: &str) -> Result<Fixed, (&'static str, char, usize)> {
    let mut lexer_state = LexerState::ExpectingNumber;
    let mut num = Number { ..DEFAULT };
    let mut curr_op = ' ';
    let mut stack = Vec::new();
    let mut op_stack = Vec::new();
    let mut op_counts = vec![0];
    let mut parens_indices = Vec::new();

    for (index, c) in expr.chars().enumerate() {
        match lexer_state {
            LexerState::ReadingNumberSign => {
                match c {
                    '+' => (),
                    '-' => num.neg = !num.neg,
                    '.' => lexer_state = LexerState::ReadingNumberDecimal,
                    '0' => lexer_state = LexerState::ReadingNumberFormat,
                    d @ '1'..='9' => {
                        num.whole = d as u32 - 0x30;
                        lexer_state = LexerState::ReadingNumberWhole;
                    }
                    other => return Err(("Unexpected", other, index))
                };
            },
            LexerState::ReadingNumberFormat => {
                match c {
                    'b' => {
                        num.base = 2;
                        lexer_state = LexerState::ReadingNumberWhole;
                    },
                    'o' => {
                        num.base = 8;
                        lexer_state = LexerState::ReadingNumberWhole;
                    },
                    'x' => {
                        num.base = 16;
                        lexer_state = LexerState::ReadingNumberWhole;
                    }
                    '_' => lexer_state = LexerState::ReadingNumberWhole,
                    '.' => lexer_state = LexerState::ReadingNumberDecimal,
                    d @ '0'..='9' => {
                        num.whole = d as u32 - 0x30;
                        lexer_state = LexerState::ReadingNumberWhole;
                    },
                    _ => {
                        push(num, &mut stack);
                        num = Number { ..DEFAULT };
                        lexer_state = LexerState::ExpectingOperator;
                    }
                };
            },
            LexerState::ReadingNumberWhole => {
                match c {
                    '_' => (),
                    '.' => lexer_state = LexerState::ReadingNumberDecimal,
                    d if d.to_digit(num.base).is_some() => {
                        num.whole = num.whole * num.base + d.to_digit(num.base).unwrap();
                    },
                    _ => {
                        push(num, &mut stack);
                        num = Number { ..DEFAULT };
                        lexer_state = LexerState::ExpectingOperator;
                    }
                };
            },
            LexerState::ReadingNumberDecimal => {
                match c {
                    '_' => (),
                    d if d.to_digit(num.base).is_some() => {
                        let digit = d.to_digit(num.base).unwrap() as u64;
                        num.decimal = num.decimal * num.base as u64 + digit;
                        num.decimal_count += 1;
                        if decimal_part_is_full(&num) {
                            lexer_state = LexerState::ReadingNumberTooLong;
                        };
                    },
                    _ => {
                        push(num, &mut stack);
                        num = Number { ..DEFAULT };
                        lexer_state = LexerState::ExpectingOperator;
                    }
                };
            },
            LexerState::ReadingNumberTooLong => {
                match c {
                    '_' => (),
                    d if d.to_digit(num.base).is_some() => (),
                    _ => {
                        push(num, &mut stack);
                        num = Number { ..DEFAULT };
                        lexer_state = LexerState::ExpectingOperator;
                    }
                };
            },
            LexerState::ReadingOperator => {
                let (op, is_two_chars) = match (curr_op, c) {
                    ('+', _) => (Op::Add, false),
                    ('-', _) => (Op::Sub, false),
                    ('*', _) => (Op::Mul, false),
                    ('/', _) => (Op::Div, false),
                    ('%', _) => (Op::Mod, false),
                    ('&', _) => (Op::And, false),
                    ('|', _) => (Op::Or, false),
                    ('^', _) => (Op::Xor, false),
                    ('<', '<') => (Op::Shl, true),
                    ('>', '>') => (Op::Shr, true),
                    _ => return Err(("Unexpected", c, index))
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
                match is_two_chars {
                    false => lexer_state = LexerState::ExpectingNumber,
                    true => lexer_state = LexerState::IgnoreThatChar
                }
            },
            LexerState::ExpectingNumber
            | LexerState::ExpectingOperator
            | LexerState::IgnoreThatChar
            => ()
        };
        if c.is_whitespace() { continue; }
        match lexer_state {
            LexerState::IgnoreThatChar => lexer_state = LexerState::ExpectingNumber,
            LexerState::ExpectingOperator => {
                if c == ')' {
                    if op_counts.len() == 1 { return Err(("Unmatched", c, index)) };
                    for _ in 0..op_counts.pop().unwrap() {
                        apply_top(&mut op_stack, &mut stack);
                    };
                } else if OP_CHARS.contains(c) {
                    curr_op = c;
                    lexer_state = LexerState::ReadingOperator;
                } else {
                    return Err(("Unexpected", c, index))
                };
            },
            LexerState::ExpectingNumber => {
                match c {
                    '(' => {
                        op_counts.push(0);
                        parens_indices.push(index);
                    },
                    '-' => {
                        num.neg = true;
                        lexer_state = LexerState::ReadingNumberSign;
                    },
                    '+' => lexer_state = LexerState::ReadingNumberSign,
                    '0' => lexer_state = LexerState::ReadingNumberFormat,
                    '.' => lexer_state = LexerState::ReadingNumberDecimal,
                    d @ '1'..='9' => {
                        num.whole = d as u32 - 0x30;
                        lexer_state = LexerState::ReadingNumberWhole;
                    },
                    other => return Err(("Unexpected", other, index))
                };
            },
            LexerState::ReadingNumberSign
            | LexerState::ReadingNumberFormat
            | LexerState::ReadingNumberWhole
            | LexerState::ReadingNumberDecimal
            | LexerState::ReadingNumberTooLong
            | LexerState::ReadingOperator
            => ()
        };
    };
    match lexer_state {
        LexerState::ExpectingOperator => (),

        LexerState::ReadingNumberFormat
        | LexerState::ReadingNumberWhole
        | LexerState::ReadingNumberDecimal
        | LexerState::ReadingNumberTooLong
        => push(num, &mut stack),

        LexerState::IgnoreThatChar
        | LexerState::ExpectingNumber
        | LexerState::ReadingNumberSign
        | LexerState::ReadingOperator
        => return Err(("Unexpected end of expression", '\'', expr.len()))
    };
    if op_counts.len() > 1 {
        let idx = op_counts.len() - 2;
        return Err(("Unmatched", '(', parens_indices[idx]))
    };
    for _ in 0..op_counts.pop().unwrap() {
        apply_top(&mut op_stack, &mut stack);
    };
    Ok(stack.pop().unwrap())
}
