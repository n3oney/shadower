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
        Err(err) => panic!("Invalid math expression: {math}\n{err:#?}"),
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
    match tokens.peek().unwrap() {
        'c' => {
            if tokens.take(5).collect::<String>() != "clamp" {
                return Err("Expected 'clamp'".to_owned());
            }
            if tokens.next() != Some('(') {
                return Err("Expected '('".to_owned());
            }

            while let Some(token) = tokens.peek() {
                if token == &' ' {
                    tokens.next();
                    continue;
                }

                break;
            }

            let min = if tokens.peek().ok_or("Unfinished clamp function")? == &'_' {
                tokens.next();
                f32::MIN
            } else {
                read_expression(tokens.by_ref())?
            };

            if tokens.next() != Some(',') {
                return Err("Expected ','".to_owned());
            }
            let value = read_expression(tokens.by_ref())?;
            if tokens.next() != Some(',') {
                return Err("Expected ','".to_owned());
            }

            while let Some(token) = tokens.peek() {
                if token == &' ' {
                    tokens.next();
                    continue;
                }

                break;
            }

            let max = if tokens.peek().ok_or("Unfinished clamp function")? == &'_' {
                tokens.next();
                f32::MAX
            } else {
                read_expression(tokens.by_ref())?
            };

            if tokens.next() != Some(')') {
                return Err("Expected ')'".to_owned());
            }
            Ok(value.max(min).min(max))
        }
        '(' => {
            tokens.next(); // consume the opening parenthesis
            let result = read_expression(tokens)?;
            if tokens.next() != Some(')') {
                return Err("Expected ')'".to_owned());
            }
            Ok(result)
        }
        c => {
            let mut num = String::new();
            num.push(*c);
            tokens.next(); // consume the character we just peeked
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

#[cfg(test)]
mod tests {
    use super::parse_math;

    const WIDTH: i32 = 100;
    const HEIGHT: i32 = 200;

    const WIDTHF: f32 = 100.0;
    const HEIGHTF: f32 = 200.0;

    // Helper functions for better tests readability
    fn max() -> f32 {
        WIDTHF.max(HEIGHTF)
    }

    fn min() -> f32 {
        WIDTHF.min(HEIGHTF)
    }

    #[test]
    fn addition() {
        assert_eq!(parse_math("9 + 10".to_owned(), WIDTH, HEIGHT), 19.0)
    }

    #[test]
    fn subtraction() {
        assert_eq!(parse_math("10 - 9".to_owned(), WIDTH, HEIGHT), 1.0)
    }

    #[test]
    fn multiplication() {
        assert_eq!(parse_math("9 * 10".to_owned(), WIDTH, HEIGHT), 90.0)
    }

    #[test]
    fn division() {
        assert_eq!(parse_math("21 / 3".to_owned(), WIDTH, HEIGHT), 7.0)
    }

    #[test]
    fn sizes() {
        assert_eq!(
            parse_math("width + 10".to_owned(), WIDTH, HEIGHT),
            WIDTHF + 10.0
        );
        assert_eq!(
            parse_math("height + 10".to_owned(), WIDTH, HEIGHT),
            HEIGHTF + 10.0
        );
    }

    #[test]
    fn minmax() {
        assert_eq!(
            parse_math("min + 10".to_owned(), WIDTH, HEIGHT),
            min() + 10.0
        );
        assert_eq!(
            parse_math("max + 10".to_owned(), WIDTH, HEIGHT),
            max() + 10.0
        );
    }

    #[test]
    fn brackets() {
        assert_eq!(parse_math("2 * (20 + 10)".to_owned(), WIDTH, HEIGHT), 60.0);
    }

    #[test]
    fn operation_order() {
        assert_eq!(parse_math("2 + 2 * 2".to_owned(), WIDTH, HEIGHT), 6.0);
    }

    #[test]
    fn clamp() {
        assert_eq!(
            parse_math("clamp(1, 2 + 2 * 2, 4)".to_owned(), WIDTH, HEIGHT),
            4.0
        );
        assert_eq!(
            parse_math("clamp(1, 2 + 2 * 2, _)".to_owned(), WIDTH, HEIGHT),
            6.0
        );
        assert_eq!(
            parse_math("clamp(_, -8, -10)".to_owned(), WIDTH, HEIGHT),
            -10.0
        );
    }

    #[test]
    fn everything() {
        fn clamp(min: f32, val: f32, max: f32) -> f32 {
            val.max(min).min(max)
        }

        assert_eq!(
            parse_math(
                "(5 + 1) * 2 + 2 * 2 / clamp(width, 300 / min, height) - max".to_owned(),
                WIDTH,
                HEIGHT
            ),
            (5.0 + 1.0) * 2.0 + 2.0 * 2.0 / clamp(WIDTHF, 300.0 / min(), HEIGHTF) - max()
        )
    }

    #[test]
    fn twenty_one() {
        assert_ne!(parse_math("9 + 10".to_owned(), WIDTH, HEIGHT), 21.0)
    }
}
