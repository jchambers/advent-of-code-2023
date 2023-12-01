use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let sum = calibration_sum(BufReader::new(File::open(path)?)
            .lines()
            .filter_map(|line| line.ok()));

        println!("Sum of calibration values: {}", sum);

        Ok(())
    } else {
        Err("Usage: day01 INPUT_FILE_PATH".into())
    }
}

fn calibration_value(line: &str) -> u32 {
    if let (Some(first), Some(last)) = (line.find(char::is_numeric), line.rfind(char::is_numeric)) {
        let bytes = line.as_bytes();

        let first_digit = bytes[first] - '0' as u8;
        let last_digit = bytes[last] - '0' as u8;

        first_digit as u32 * 10 + last_digit as u32
    } else {
        0
    }
}

fn calibration_sum(lines: impl Iterator<Item = String>) -> u32 {
    lines.map(|line| calibration_value(line.as_str())).sum()
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
}
