use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        {
            let sum = calibration_sum(BufReader::new(File::open(path)?)
                .lines()
                .map_while(Result::ok));

            println!("Sum of calibration values: {}", sum);
        }

        {
            let sum = calibration_sum_textual(BufReader::new(File::open(path)?)
                .lines()
                .map_while(Result::ok));

            println!("Sum of calibration values with text interpretation: {}", sum);
        }

        Ok(())
    } else {
        Err("Usage: day01 INPUT_FILE_PATH".into())
    }
}

fn calibration_value(line: &str) -> u32 {
    if let (Some(first), Some(last)) = (line.find(char::is_numeric), line.rfind(char::is_numeric)) {
        let bytes = line.as_bytes();

        let first_digit = bytes[first] - b'0';
        let last_digit = bytes[last] - b'0';

        first_digit as u32 * 10 + last_digit as u32
    } else {
        0
    }
}

fn calibration_sum(lines: impl Iterator<Item = String>) -> u32 {
    lines.map(|line| calibration_value(line.as_str())).sum()
}

fn calibration_value_textual(line: &str) -> u32 {
    let chars: Vec<char> = line.chars().collect();

    let first_digit = (0..chars.len())
        .map(|offset| &chars[offset..])
        .filter_map(to_digit_textual)
        .next();

    let last_digit = (0..chars.len())
        .rev()
        .map(|offset| &chars[offset..])
        .filter_map(to_digit_textual)
        .next();

    if let (Some(first), Some(last)) = (first_digit, last_digit) {
        first * 10 + last
    } else {
        0
    }
}

fn to_digit_textual(slice: &[char]) -> Option<u32> {
    match slice {
        [n @ '0'..='9', ..] => {
            let mut bytes = [0; 1];
            n.encode_utf8(&mut bytes);

            Some((bytes[0] - b'0') as u32)
        }
        ['o', 'n', 'e', ..] => Some(1),
        ['t', 'w', 'o', ..] => Some(2),
        ['t', 'h', 'r', 'e', 'e', ..] => Some(3),
        ['f', 'o', 'u', 'r', ..] => Some(4),
        ['f', 'i', 'v', 'e', ..] => Some(5),
        ['s', 'i', 'x', ..] => Some(6),
        ['s', 'e', 'v', 'e', 'n', ..] => Some(7),
        ['e', 'i', 'g', 'h', 't', ..] => Some(8),
        ['n', 'i', 'n', 'e', ..] => Some(9),
        _ => None,
    }
}

fn calibration_sum_textual(lines: impl Iterator<Item = String>) -> u32 {
    lines.map(|line| calibration_value_textual(line.as_str())).sum()
}

#[cfg(test)]
mod test {
    use indoc::indoc;
    use super::*;

    #[test]
    fn test_calibration_value() {
        assert_eq!(12, calibration_value("1abc2"));
        assert_eq!(38, calibration_value("pqr3stu8vwx"));
        assert_eq!(15, calibration_value("a1b2c3d4e5f"));
        assert_eq!(77, calibration_value("treb7uchet"));
    }

    #[test]
    fn test_calibration_sum() {
        let lines = indoc! {"
            1abc2
            pqr3stu8vwx
            a1b2c3d4e5f
            treb7uchet
        "};

        assert_eq!(142, calibration_sum(lines.lines().map(String::from)));
    }

    #[test]
    fn test_calibration_value_textual() {
        assert_eq!(29, calibration_value_textual("two1nine"));
        assert_eq!(83, calibration_value_textual("eightwothree"));
        assert_eq!(13, calibration_value_textual("abcone2threexyz"));
        assert_eq!(24, calibration_value_textual("xtwone3four"));
        assert_eq!(42, calibration_value_textual("4nineeightseven2"));
        assert_eq!(14, calibration_value_textual("zoneight234"));
        assert_eq!(76, calibration_value_textual("7pqrstsixteen"));
    }

    #[test]
    fn test_calibration_sum_textual() {
        let lines = indoc! {"
            two1nine
            eightwothree
            abcone2threexyz
            xtwone3four
            4nineeightseven2
            zoneight234
            7pqrstsixteen
        "};

        assert_eq!(281, calibration_sum_textual(lines.lines().map(String::from)));
    }
}
