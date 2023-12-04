use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let cards: Vec<Card> = BufReader::new(File::open(path)?)
            .lines()
            .map_while(Result::ok)
            .map(|line| Card::from_str(line.as_str()))
            .collect::<Result<_, _>>()?;

        println!(
            "Sum of card values: {}",
            cards.iter().map(Card::score).sum::<u32>()
        );

        Ok(())
    } else {
        Err("Usage: day04 INPUT_FILE_PATH".into())
    }
}

struct Card {
    _id: u32,

    winning_numbers: Vec<u32>,
    drawn_numbers: Vec<u32>,
}

impl Card {
    fn score(&self) -> u32 {
        let matching_numbers = self
            .drawn_numbers
            .iter()
            .filter(|n| self.winning_numbers.contains(n))
            .count();

        (1 << matching_numbers) >> 1
    }
}

impl FromStr for Card {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let [card_id, numbers] = string.split(':').collect::<Vec<&str>>().as_slice() {
            let _id = if let ["Card", id] = card_id
                .split(' ')
                .filter(|s| !s.is_empty())
                .collect::<Vec<&str>>()
                .as_slice()
            {
                id.parse()?
            } else {
                return Err("Could not parse card ID".into());
            };

            let (winning_numbers, drawn_numbers) = if let [winning_numbers, drawn_numbers] =
                numbers.split('|').collect::<Vec<&str>>().as_slice()
            {
                let winning_numbers = winning_numbers
                    .split(' ')
                    .filter_map(|s| s.parse().ok())
                    .collect();

                let drawn_numbers = drawn_numbers
                    .split(' ')
                    .filter_map(|s| s.parse().ok())
                    .collect();

                (winning_numbers, drawn_numbers)
            } else {
                return Err("Could not parse numbers".into());
            };

            Ok(Card {
                _id,
                winning_numbers,
                drawn_numbers,
            })
        } else {
            Err("Could not parse card definition".into())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_card_score() {
        for (card, expected_score) in [
            ("Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53", 8),
            ("Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19", 2),
            ("Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1", 2),
            ("Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83", 1),
            ("Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36", 0),
            ("Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11", 0),
        ] {
            assert_eq!(expected_score, Card::from_str(card).unwrap().score());
        }
    }
}
