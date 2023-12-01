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
                .filter_map(|line| line.ok()));

            println!("Sum of calibration values: {}", sum);
        }

        {
            let sum = calibration_sum_textual(BufReader::new(File::open(path)?)
                .lines()
                .filter_map(|line| line.ok()));

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
    let mut digits = Vec::new();

    digits.extend(line.match_indices(char::is_numeric)
        .map(|(index, slice)| (index, (slice.as_bytes()[0] - b'0') as u32)));

    for (digit_string, digit_value) in [
        ("one", 1),
        ("two", 2),
        ("three", 3),
        ("four", 4),
        ("five", 5),
        ("six", 6),
        ("seven", 7),
        ("eight", 8),
        ("nine", 9)] {

        digits.extend(line.match_indices(digit_string)
            .map(|(index, _)| (index, digit_value)));
    }

    digits.sort_by_key(|(index, _)| *index);

    if let (Some((_, first)), Some((_, last))) = (digits.first(), digits.last()) {
        first * 10 + last
    } else {
        0
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
