enum LexerState {
    ExpectingOperator,
    ExpectingNumber,
    ReadingNumber
}

enum Token {
    Add, Sub, Mul, Div,
    ParensOpen, ParensClose,
    Num(i32)
}

fn tok_to_ins(tok: &Token) -> Instruction {
    match tok {
        Token::Add => Instruction::Add,
        Token::Sub => Instruction::Sub,
        Token::Mul => Instruction::Mul,
        Token::Div => Instruction::Div,
        Token::Num(n) => Instruction::Num(*n),
        // this is never reached
        _ => Instruction::Add
    }
}

enum Instruction {
    Add, Sub, Mul, Div,
    Num(i32)
}

fn precedence(op: Option<&Instruction>) -> i32 {
    if let None = op {return 0;};
    match op.unwrap() {
        Instruction::Add => 1,
        Instruction::Sub => 1,
        Instruction::Mul => 2,
        Instruction::Div => 2,
        // this is never reached
        Instruction::Num(_) => 0
    }
}

enum ParsingError {
    EndOfExpr,
    UnbalancedParens,
    UnmatchedParens(usize),
    UnexpectedChar(char, usize)
}

fn parse(expr: &str) -> Result<Vec<Token>, ParsingError> {
    let mut ins = Vec::new();
    let mut curr = 0;
    let mut lexer_state = LexerState::ExpectingNumber;
    let mut parens_count = 0;
    for (index, c) in expr.chars().enumerate() {
        if let LexerState::ReadingNumber = lexer_state {
            match c.to_digit(10) {
                Some(d) => { curr = 10*curr + d as i32; },
                None => {
                    ins.push(Token::Num(curr));
                    lexer_state = LexerState::ExpectingOperator;
                }
            }
        }
        if c.is_whitespace() { continue; };
        match lexer_state {
            LexerState::ReadingNumber => {},
            LexerState::ExpectingOperator => {
                match c {
                    '+' => { ins.push(Token::Add); lexer_state = LexerState::ExpectingNumber; },
                    '-' => { ins.push(Token::Sub); lexer_state = LexerState::ExpectingNumber; },
                    '*' => { ins.push(Token::Mul); lexer_state = LexerState::ExpectingNumber; },
                    '/' => { ins.push(Token::Div); lexer_state = LexerState::ExpectingNumber; },
                    ')' => {
                        if parens_count == 0 { return Err(ParsingError::UnmatchedParens(index)); };
                        ins.push(Token::ParensClose);
                        parens_count -= 1;
                    },
                    other => return Err(ParsingError::UnexpectedChar(other, index))
                }
            },
            LexerState::ExpectingNumber => {
                match (c, c.to_digit(10)) {
                    (_, Some(d)) => {
                        curr = d as i32;
                        lexer_state = LexerState::ReadingNumber;
                    },
                    ('(', _) => {
                        ins.push(Token::ParensOpen);
                        parens_count += 1;
                    },
                    (other, _) => return Err(ParsingError::UnexpectedChar(other, index))
                };
            }
        };
    };
    match lexer_state {
        LexerState::ReadingNumber => { ins.push(Token::Num(curr)); },
        LexerState::ExpectingNumber => { return Err(ParsingError::EndOfExpr); },
        LexerState::ExpectingOperator => {}
    }
    if parens_count != 0 { return Err(ParsingError::UnbalancedParens); };
    Ok(ins)
}

fn to_rpn(ins: &Vec<Token>) -> Vec<Instruction> {
    let mut rpn_code = Vec::new();
    let mut op_stack = Vec::new();
    let mut op_counts = vec![0];
    for tok in ins {
        match tok {
            Token::Num(n) => rpn_code.push(Instruction::Num(*n)),
            Token::ParensOpen => op_counts.push(0),
            Token::ParensClose => {
                for _ in 0..op_counts.pop().unwrap() {
                    rpn_code.push( op_stack.pop().unwrap() );
                };
            },
            token => {
                let op = tok_to_ins(token);
                if *op_counts.last().unwrap() == 0 ||
                    precedence(Some(&op)) > precedence(op_stack.last())
                {
                    op_stack.push(op);
                    let count = op_counts.pop().unwrap();
                    op_counts.push(count + 1);
                } else {
                    for _ in 0..op_counts.pop().unwrap() {
                        rpn_code.push( op_stack.pop().unwrap() );
                    };
                    op_stack.push(op);
                    op_counts.push(1);
                };
            }
        };
    };
    for _ in 0..op_counts.pop().unwrap() {
        rpn_code.push( op_stack.pop().unwrap() );
    };
    rpn_code
}

fn stack_operation<T>(stack: &mut Vec<T>, func: fn(T, T) -> T) {
    let b = stack.pop().unwrap();
    let a = stack.pop().unwrap();
    stack.push(func(a, b))
}

fn exec(instructions: &Vec<Instruction>) -> i32 {
    let mut stack = Vec::new();
    for ins in instructions {
        match ins {
            Instruction::Num(n) => stack.push(*n),
            Instruction::Add => stack_operation(&mut stack, |a, b| a+b),
            Instruction::Sub => stack_operation(&mut stack, |a, b| a-b),
            Instruction::Mul => stack_operation(&mut stack, |a, b| a*b),
            Instruction::Div => stack_operation(&mut stack, |a, b| a/b),
        };
    };
    stack.pop().unwrap()
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
    let tokens = match parse(&expr) {
        Ok(v) => v,
        Err(e) => {
            match e {
                ParsingError::EndOfExpr => println!("Error: Incomplete expression"),
                ParsingError::UnbalancedParens => println!("Error: Unbalanced Parentheses"),
                ParsingError::UnmatchedParens(i) => println!("Error: Unmatched ')' at {}", i),
                ParsingError::UnexpectedChar(c, i) => println!("Error: Unexpected '{}' at {}", c, i)
            };
            return;
        }
    };
    let instructions = to_rpn(&tokens);
    let result = exec(&instructions);
    println!("{}", result)
}
