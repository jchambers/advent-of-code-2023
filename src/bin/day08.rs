use std::cmp::Ordering;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use std::{cmp, env};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let network_map = {
            let mut map_string = String::new();
            File::open(path)?.read_to_string(&mut map_string)?;

            NetworkMap::from_str(map_string.as_str())?
        };

        println!(
            "Human steps between AAA and ZZZ: {}",
            network_map.human_steps_to_exit().unwrap()
        );

        println!(
            "Ghost steps between AAA and ZZZ: {}",
            network_map.ghost_steps_to_exit().unwrap()
        );

        Ok(())
    } else {
        Err("Usage: day08 INPUT_FILE_PATH".into())
    }
}

struct NetworkMap {
    directions: Vec<Direction>,
    nodes: HashMap<String, (String, String)>,
}

impl NetworkMap {
    fn human_steps_to_exit(&self) -> Option<u32> {
        let mut position = "AAA";
        let mut steps = 0;
        let mut directions = self.directions.iter().cycle();

        while position != "ZZZ" {
            if let Some(destinations) = self.nodes.get(position) {
                position = match directions.next().unwrap() {
                    Direction::Left => &destinations.0,
                    Direction::Right => &destinations.1,
                };

                steps += 1;
            } else {
                return None;
            }
        }

        Some(steps)
    }

    fn steps_to_first_ghost_exit(&self, start: &str) -> Result<u64, Box<dyn Error>> {
        let mut position = start;
        let mut steps = 0;
        let mut directions = self.directions.iter().cycle();

        loop {
            if let Some(destinations) = self.nodes.get(position) {
                position = match directions.next().unwrap() {
                    Direction::Left => &destinations.0,
                    Direction::Right => &destinations.1,
                };

                steps += 1;
            } else {
                return Err("Destination node not found".into());
            }

            if position.ends_with('Z') {
                break Ok(steps);
            }
        }
    }

    fn ghost_steps_to_exit(&self) -> Result<u64, Box<dyn Error>> {
        // Weeeell this is frustrating. This problem's solution appears to depend on noticing that
        // the inputs have been specially crafted such that each "ghost" travels in a long cycle,
        // and each each contains exactly one exit (i.e. there's no bouncing between exits). That
        // means the exit time is the LCM of all of the cycle lengths.
        //
        // To make THAT easier, it turns out that all of the cycle lengths all have exactly two
        // prime factors, and one of those prime factors is common to all of the cycle lengths.
        let cycle_lengths: Vec<u64> = self
            .nodes
            .keys()
            .filter(|position| position.ends_with('A'))
            .map(|position| self.steps_to_first_ghost_exit(position))
            .collect::<Result<_, _>>()?;

        cycle_lengths
            .iter()
            .copied()
            .reduce(least_common_multiple)
            .ok_or("Could not calculate cycle lengths".into())
    }
}

impl FromStr for NetworkMap {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let [directions, nodes] = string.split("\n\n").collect::<Vec<&str>>().as_slice() {
            let directions = directions
                .chars()
                .map(Direction::try_from)
                .collect::<Result<_, _>>()?;

            let nodes = nodes
                .lines()
                .map(|line| {
                    (
                        String::from(&line[0..3]),
                        (String::from(&line[7..10]), String::from(&line[12..15])),
                    )
                })
                .collect();

            Ok(NetworkMap { directions, nodes })
        } else {
            Err("Could not parse directions and node map".into())
        }
    }
}

enum Direction {
    Left,
    Right,
}

impl TryFrom<char> for Direction {
    type Error = Box<dyn Error>;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'L' => Ok(Direction::Left),
            'R' => Ok(Direction::Right),
            _ => Err("Unexpected direction".into()),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Position {
    node: String,
    steps: u64,
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering so we can build a min-heap
        other.steps.cmp(&self.steps)
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Use the Euclidean Algorithm to find the GCD
fn greatest_common_divisor(a: u64, b: u64) -> u64 {
    if a == 0 {
        b
    } else if b == 0 {
        a
    } else {
        let max = cmp::max(a, b);
        let min = cmp::min(a, b);

        greatest_common_divisor(min, max % min)
    }
}

fn least_common_multiple(a: u64, b: u64) -> u64 {
    (a * b) / greatest_common_divisor(a, b)
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_human_steps_to_exit() {
        {
            let node_map = NetworkMap::from_str(indoc! {"
                RL

                AAA = (BBB, CCC)
                BBB = (DDD, EEE)
                CCC = (ZZZ, GGG)
                DDD = (DDD, DDD)
                EEE = (EEE, EEE)
                GGG = (GGG, GGG)
                ZZZ = (ZZZ, ZZZ)
            "})
            .unwrap();

            assert_eq!(Some(2), node_map.human_steps_to_exit());
        }

        {
            let node_map = NetworkMap::from_str(indoc! {"
                LLR

                AAA = (BBB, BBB)
                BBB = (AAA, ZZZ)
                ZZZ = (ZZZ, ZZZ)
            "})
            .unwrap();

            assert_eq!(Some(6), node_map.human_steps_to_exit());
        }
    }

    #[test]
    fn test_ghost_steps_to_exit() {
        let node_map = NetworkMap::from_str(indoc! {"
                LR

                11A = (11B, XXX)
                11B = (XXX, 11Z)
                11Z = (11B, XXX)
                22A = (22B, XXX)
                22B = (22C, 22C)
                22C = (22Z, 22Z)
                22Z = (22B, 22B)
                XXX = (XXX, XXX)
            "})
        .unwrap();

        assert_eq!(6, node_map.ghost_steps_to_exit().unwrap());
    }

    #[test]
    fn test_greatest_common_divisor() {
        assert_eq!(6, greatest_common_divisor(270, 192));
    }

    #[test]
    fn test_least_common_multiple() {
        assert_eq!(15, least_common_multiple(3, 5));
        assert_eq!(12, least_common_multiple(4, 6));
    }
}
