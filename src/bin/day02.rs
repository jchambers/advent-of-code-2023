use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let games: Vec<Game> = BufReader::new(File::open(path)?)
            .lines()
            .map_while(Result::ok)
            .map(|line| Game::from_str(line.as_str()))
            .collect::<Result<Vec<_>, _>>()?;

        println!(
            "Sum of 'possible' game IDs: {}",
            games
                .iter()
                .filter(|game| game.is_possible(12, 13, 14))
                .map(|game| game.id)
                .sum::<u32>()
        );

        println!(
            "Sum of powers of minimal cube sets: {}",
            games
                .iter()
                .map(|game| game.minimum_cubes_power())
                .sum::<u32>()
        );

        Ok(())
    } else {
        Err("Usage: day02 INPUT_FILE_PATH".into())
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Game {
    id: u32,
    samples: Vec<Sample>,
}

impl Game {
    fn is_possible(&self, red: u32, green: u32, blue: u32) -> bool {
        !self
            .samples
            .iter()
            .any(|sample| sample.red > red || sample.green > green || sample.blue > blue)
    }

    fn minimum_cubes_power(&self) -> u32 {
        let red = self
            .samples
            .iter()
            .map(|sample| sample.red)
            .max()
            .unwrap_or(0);

        let green = self
            .samples
            .iter()
            .map(|sample| sample.green)
            .max()
            .unwrap_or(0);

        let blue = self
            .samples
            .iter()
            .map(|sample| sample.blue)
            .max()
            .unwrap_or(0);

        red * green * blue
    }
}

impl FromStr for Game {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let [game_id, samples] = string.split(": ").collect::<Vec<&str>>().as_slice() {
            let id = if let ["Game", id] = game_id.split(' ').collect::<Vec<&str>>().as_slice() {
                id.parse()?
            } else {
                return Err("Unparseable game ID".into());
            };

            let samples = samples
                .split("; ")
                .map(Sample::from_str)
                .collect::<Result<Vec<Sample>, _>>()?;

            Ok(Game { id, samples })
        } else {
            Err("Unparseable game string".into())
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Sample {
    red: u32,
    green: u32,
    blue: u32,
}

impl FromStr for Sample {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut red = 0;
        let mut green = 0;
        let mut blue = 0;

        for count in string.split(", ") {
            if let [n, color] = count.split(' ').collect::<Vec<&str>>().as_slice() {
                let n = n.parse()?;

                match *color {
                    "red" => {
                        red = n;
                    }

                    "green" => {
                        green = n;
                    }

                    "blue" => {
                        blue = n;
                    }

                    _ => {
                        return Err("Unexpected color".into());
                    }
                }
            } else {
                return Err("Unprocessable cube count".into());
            }
        }

        Ok(Sample { red, green, blue })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_game_from_string() {
        assert_eq!(
            Game {
                id: 1,
                samples: vec![
                    Sample {
                        red: 4,
                        green: 0,
                        blue: 3
                    },
                    Sample {
                        red: 1,
                        green: 2,
                        blue: 6
                    },
                    Sample {
                        red: 0,
                        green: 2,
                        blue: 0
                    }
                ]
            },
            Game::from_str("Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green").unwrap()
        );
    }

    #[test]
    fn test_game_is_possible() {
        for (game_string, expect_possible) in [
            (
                "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green",
                true,
            ),
            (
                "Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue",
                true,
            ),
            (
                "Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red",
                false,
            ),
            (
                "Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red",
                false,
            ),
            (
                "Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green",
                true,
            ),
        ] {
            assert_eq!(
                expect_possible,
                Game::from_str(game_string).unwrap().is_possible(12, 13, 14)
            );
        }
    }

    #[test]
    fn test_minimum_cubes_power() {
        for (game_string, expected_power) in [
            (
                "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green",
                48
            ),
            (
                "Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue",
                12,
            ),
            (
                "Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red",
                1560,
            ),
            (
                "Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red",
                630,
            ),
            (
                "Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green",
                36
            ),
        ] {
            assert_eq!(
                expected_power,
                Game::from_str(game_string).unwrap().minimum_cubes_power()
            );
        }
    }
}
