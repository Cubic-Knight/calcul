use crate::fixed::Fixed;

pub const OP_CHARS: &str = "+-*/%|&^><";
pub enum Op {
    Add, Sub,
    Mul, Div, Mod,
    And, Or, Xor,
    Shl, Shr
}

pub fn precedence(op: &Op) -> i32 {
    match op {
        Op::And => 1,
        Op::Or => 1,
        Op::Xor => 1,
        Op::Shl => 1,
        Op::Shr => 1,
        Op::Add => 2,
        Op::Sub => 2,
        Op::Mul => 3,
        Op::Div => 3,
        Op::Mod => 3
    }
}

/// This function pops two elements out of a stack, then applies an operation
/// depending on the operator that was popped out of the op_stack
/// The result of the operation is then pushed back onto the stack
pub fn apply_top(op_stack: &mut Vec<Op>, stack: &mut Vec<Fixed>) {
    let func = match op_stack.pop().unwrap() {
        Op::Add => |a, b| a+b,
        Op::Sub => |a, b| a-b,
        Op::Mul => |a, b| a*b,
        Op::Div => |a, b| a/b,
        Op::Mod => |a, b| a%b,
        Op::And => |a, b| a&b,
        Op::Or  => |a, b| a|b,
        Op::Xor => |a, b| a^b,
        Op::Shl => |a, b| a<<b,
        Op::Shr => |a, b| a>>b
    };
    let b = stack.pop().unwrap();
    let a = stack.pop().unwrap();
    stack.push(func(a, b));
}
