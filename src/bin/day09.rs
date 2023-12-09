use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let sequences: Vec<Sequence> = BufReader::new(File::open(path)?)
            .lines()
            .map_while(Result::ok)
            .map(|line| Sequence::from_str(line.as_str()))
            .collect::<Result<Vec<_>, _>>()?;

        println!(
            "Sum of next values: {}",
            sequences
                .iter()
                .map(|sequence| sequence.next())
                .sum::<Result<i32, _>>()?
        );

        println!(
            "Sum of previous values: {}",
            sequences
                .iter()
                .map(|sequence| sequence.previous())
                .sum::<Result<i32, _>>()?
        );

        Ok(())
    } else {
        Err("Usage: day09 INPUT_FILE_PATH".into())
    }
}

struct Sequence {
    values: Vec<i32>,
}

impl Sequence {
    fn next(&self) -> Result<i32, Box<dyn Error>> {
        Self::derive_next(&self.values)
    }

    fn previous(&self) -> Result<i32, Box<dyn Error>> {
        Self::derive_previous(&self.values)
    }

    fn derive(values: &[i32]) -> Result<Vec<i32>, Box<dyn Error>> {
        let derivative: Vec<i32> = values
            .windows(2)
            .filter_map(|pair| if let [a, b] = pair { Some(b - a) } else { None })
            .collect();

        if derivative.is_empty() {
            Err("Empty derived sequence".into())
        } else {
            Ok(derivative)
        }
    }

    fn derive_next(values: &[i32]) -> Result<i32, Box<dyn Error>> {
        if values.iter().all(|&v| v == 0) {
            return Ok(0);
        }

        let derivative = Self::derive(values)?;

        Ok(values.last().unwrap() + Self::derive_next(&derivative)?)
    }

    fn derive_previous(values: &[i32]) -> Result<i32, Box<dyn Error>> {
        if values.iter().all(|&v| v == 0) {
            return Ok(0);
        }

        let derivative = Self::derive(values)?;

        Ok(values.first().unwrap() - Self::derive_previous(&derivative)?)
    }
}

impl FromStr for Sequence {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(Sequence {
            values: string
                .split(' ')
                .map(|s| s.parse())
                .collect::<Result<_, _>>()?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_next() {
        assert_eq!(
            18,
            Sequence::from_str("0 3 6 9 12 15").unwrap().next().unwrap()
        );
        assert_eq!(
            28,
            Sequence::from_str("1 3 6 10 15 21")
                .unwrap()
                .next()
                .unwrap()
        );
        assert_eq!(
            68,
            Sequence::from_str("10 13 16 21 30 45")
                .unwrap()
                .next()
                .unwrap()
        );
    }

    #[test]
    fn test_previous() {
        assert_eq!(
            5,
            Sequence::from_str("10 13 16 21 30 45")
                .unwrap()
                .previous()
                .unwrap()
        );
    }
}
