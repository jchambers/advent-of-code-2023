use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

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

    fn steps_to_next_ghost_exit(&self, start: &str, initial_steps: u64) -> Result<(String, u64), Box<dyn Error>> {
        let mut position = start;
        let mut steps = 0;
        let mut directions = self.directions
            .iter()
            .cycle()
            .skip(initial_steps as usize % self.directions.len());

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
                println!("{} @ {} => {} in {} steps", start, initial_steps, position, steps);
                break Ok((String::from(position), steps))
            }
        }
    }

    fn ghost_steps_to_exit(&self) -> Result<u64, Box<dyn Error>> {
        let mut positions: BinaryHeap<Position> = self.nodes.keys()
            .filter(|position| position.ends_with('A'))
            .map(|position| self.steps_to_next_ghost_exit(position, 0)
                .map(|position| Position { node: position.0, steps: position.1 }))
            .collect::<Result<_, _>>()?;

        // A map of (node, direction index) tuples to (node, steps) tuples
        let mut cache: HashMap<(String, u64), (String, u64)> = HashMap::new();

        loop {
            let min_steps = positions.peek().unwrap().steps;

            if positions.iter().all(|position| position.steps == min_steps) {
                break;
            }

            // Paths have not yet aligned; advance the traveler that's farthest behind to the next
            // potential exit.
            let earliest_position = positions.pop().unwrap();
            let direction_index = earliest_position.steps % self.directions.len() as u64;

            let (next_node, steps) = cache.entry((earliest_position.node.clone(), earliest_position.steps % self.directions.len() as u64))
                .or_insert_with(|| self.steps_to_next_ghost_exit(&earliest_position.node, direction_index).unwrap());

            positions.push(Position { node: next_node.clone(), steps: earliest_position.steps + *steps });
        }

        Ok(positions.peek().unwrap().steps)
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
}
