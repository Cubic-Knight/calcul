#[cfg(test)]
mod tests;

mod fixed;
mod ops;
mod parser;

fn main() -> Result<(), String> {
    let expr = match std::env::args().skip(1).next() {
        Some(s) => s,
        None => return Err("No argument given.  USAGE: calcul.exe [expr]".to_string())
    };
    match parser::exec(&expr) {
        Ok(n) => println!("{}", n),
        Err((s, c, i)) => return Err(format!("{s} '{c}' at {i}"))
    };
    Ok(())
}
