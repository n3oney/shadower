use std::str::FromStr;

// TODO: Possibly move into some struct constructed with the width and height, since these will
// never change.

pub fn parse_math(math: String, width: i32, height: i32) -> f32 {
    let max = width.max(height) as f32;
    let min = width.min(height) as f32;
    let math_with_keywords = math
        .replace("width", &width.to_string())
        .replace("height", &height.to_string())
        .replace("max", &max.to_string())
        .replace("min", &min.to_string());
    match eval(&math_with_keywords) {
        Ok(result) => result,
        Err(_) => panic!("Invalid math expression: {math}"),
    }
}

pub fn eval(expr: &str) -> Result<f32, String> {
    let mut tokens = expr.chars().filter(|c| !c.is_whitespace()).peekable();
    let result = read_expression(&mut tokens)?;
    if tokens.peek().is_some() {
        return Err(format!("Unexpected character: {}", tokens.next().unwrap()));
    }
    Ok(result)
}

pub fn read_expression<I>(tokens: &mut std::iter::Peekable<I>) -> Result<f32, String>
where
    I: Iterator<Item = char>,
{
    let mut lhs = read_term(tokens)?;
    while tokens.peek().is_some() {
        match tokens.peek().unwrap() {
            '+' | '-' => {
                let operator = tokens.next().unwrap();
                let rhs = read_term(tokens)?;
                lhs = match operator {
                    '+' => lhs + rhs,
                    '-' => lhs - rhs,
                    _ => unreachable!(),
                };
            }
            _ => break,
        }
    }
    Ok(lhs)
}

pub fn read_term<I>(tokens: &mut std::iter::Peekable<I>) -> Result<f32, String>
where
    I: Iterator<Item = char>,
{
    let mut lhs = read_operand(tokens)?;
    while tokens.peek().is_some() {
        match tokens.peek().unwrap() {
            '*' | '/' => {
                let operator = tokens.next().unwrap();
                let rhs = read_operand(tokens)?;
                lhs = match operator {
                    '*' => lhs * rhs,
                    '/' => lhs / rhs,
                    _ => unreachable!(),
                };
            }
            _ => break,
        }
    }
    Ok(lhs)
}

pub fn read_operand<I>(tokens: &mut std::iter::Peekable<I>) -> Result<f32, String>
where
    I: Iterator<Item = char>,
{
    if tokens.peek().is_none() {
        return Err("Unexpected end of input".to_owned());
    }
    match tokens.next().unwrap() {
        '(' => {
            let result = read_expression(tokens)?;
            if tokens.next() != Some(')') {
                return Err("Expected ')'".to_owned());
            }
            Ok(result)
        }
        c => {
            let mut num = String::new();
            num.push(c);
            while let Some(&c) = tokens.peek() {
                match c {
                    '0'..='9' | '.' => {
                        num.push(c);
                        tokens.next();
                    }
                    _ => break,
                }
            }
            f32::from_str(&num).map_err(|_| format!("Invalid number: {}", num))
        }
    }
}
