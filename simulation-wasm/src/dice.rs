use crate::model::DiceFormula;
use rand::Rng;

pub fn evaluate(formula: &DiceFormula, dice_multiplier: u32) -> f64 {
    match formula {
        DiceFormula::Value(v) => *v,
        DiceFormula::Expr(s) => parse_and_roll(s, dice_multiplier),
    }
}

pub fn average(formula: &DiceFormula) -> f64 {
    match formula {
        DiceFormula::Value(v) => *v,
        DiceFormula::Expr(s) => parse_average(s),
    }
}

pub fn parse_average(expr: &str) -> f64 {
    // Similar to parse_and_roll but returns average
    let s = expr.replace(" ", "");
    let mut sum = 0.0;
    let mut current_term = String::new();
    let mut sign = 1.0;

    for c in s.chars() {
        if c == '+' || c == '-' {
            if !current_term.is_empty() {
                sum += sign * parse_term_average(&current_term);
                current_term.clear();
            }
            sign = if c == '+' { 1.0 } else { -1.0 };
        } else {
            current_term.push(c);
        }
    }
    if !current_term.is_empty() {
        sum += sign * parse_term_average(&current_term);
    }
    sum
}

fn parse_term_average(term: &str) -> f64 {
    // Strip bracket notation: "3[PB]" -> "3", "1d4[Bless]" -> "1d4"
    let cleaned_term = if let Some(bracket_pos) = term.find('[') {
        &term[..bracket_pos]
    } else {
        term
    };

    if cleaned_term.contains('d') {
        let parts: Vec<&str> = cleaned_term.split('d').collect();
        if parts.len() == 2 {
            let count = parts[0].parse::<i32>().unwrap_or(1);
            let count = if count == 0 && parts[0].is_empty() {
                1
            } else {
                count
            };
            let sides = parts[1].parse::<i32>().unwrap_or(6);

            // Average of 1dN is (N+1)/2
            return count as f64 * (sides as f64 + 1.0) / 2.0;
        }
    }
    cleaned_term.parse::<f64>().unwrap_or(0.0)
}

fn parse_and_roll(expr: &str, dice_multiplier: u32) -> f64 {
    // Very basic parser for now. Supports "XdY+Z", "XdY-Z", "XdY", "Z"
    // TODO: Implement full parser if needed (e.g. using a crate like `caith` or writing a recursive descent parser)

    // Remove whitespace
    let s = expr.replace(" ", "");

    // Handle simple addition/subtraction of terms
    // This is a naive implementation and won't handle order of operations correctly for mixed * and +
    // But D&D formulas are usually Sum of Terms.

    let mut sum = 0.0;
    let mut current_term = String::new();
    let mut sign = 1.0;

    for c in s.chars() {
        if c == '+' || c == '-' {
            if !current_term.is_empty() {
                sum += sign * parse_term(&current_term, dice_multiplier);
                current_term.clear();
            }
            sign = if c == '+' { 1.0 } else { -1.0 };
        } else {
            current_term.push(c);
        }
    }
    if !current_term.is_empty() {
        sum += sign * parse_term(&current_term, dice_multiplier);
    }

    sum
}

fn parse_term(term: &str, dice_multiplier: u32) -> f64 {
    // Strip bracket notation: "3[PB]" -> "3", "1d4[Bless]" -> "1d4"
    let cleaned_term = if let Some(bracket_pos) = term.find('[') {
        &term[..bracket_pos]
    } else {
        term
    };

    if cleaned_term.contains('d') {
        let parts: Vec<&str> = cleaned_term.split('d').collect();
        if parts.len() == 2 {
            let count = parts[0].parse::<i32>().unwrap_or(1); // "d8" -> count 1
            let count = if count == 0 && parts[0].is_empty() {
                1
            } else {
                count
            };

            let sides = parts[1].parse::<i32>().unwrap_or(6);

            let mut rng = rand::thread_rng();
            let mut total = 0.0;
            for _ in 0..(count * dice_multiplier as i32) {
                total += rng.gen_range(1..=sides) as f64;
            }
            return total;
        }
    }

    cleaned_term.parse::<f64>().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dice_parsing() {
        // Since it's random, we can't assert exact values easily, but we can check ranges or run multiple times
        let res = parse_and_roll("1d1+5", 1);
        assert_eq!(res, 6.0);

        let res = parse_and_roll("10", 1);
        assert_eq!(res, 10.0);
    }

    #[test]
    fn test_bracket_notation() {
        // Test simple bracket notation
        let res = parse_and_roll("3[PB]+5[STR]", 1);
        assert_eq!(res, 8.0);

        let res = parse_and_roll("10[Base]-5[SS]", 1);
        assert_eq!(res, 5.0);

        // Test single bracketed value
        let res = parse_and_roll("7[Modifier]", 1);
        assert_eq!(res, 7.0);
    }

    #[test]
    fn test_dice_with_brackets() {
        // Test dice notation with brackets
        let res = parse_and_roll("1d1[Bless]+3[Guidance]", 1);
        assert_eq!(res, 4.0);

        // Test complex formula
        let res = parse_and_roll("3[PB]+5[STR]+2[Weapon]-5[SS]", 1);
        assert_eq!(res, 5.0);
    }

    #[test]
    fn test_average_with_brackets() {
        // Test average calculation with brackets
        let res = parse_average("3[PB]+5[STR]");
        assert_eq!(res, 8.0);

        let res = parse_average("1d4[Bless]+2[Guidance]");
        assert_eq!(res, 4.5); // Average of 1d4 is 2.5, plus 2 = 4.5
    }
}
