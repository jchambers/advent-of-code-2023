use std::collections::HashMap;
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
            "Steps between AAA and ZZZ: {}",
            network_map.steps_between("AAA", "ZZZ").unwrap()
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
    fn steps_between(&self, start: &str, end: &str) -> Option<u32> {
        let mut position = start;
        let mut steps = 0;
        let mut directions = self.directions.iter().cycle();

        while position != end {
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

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_steps_between() {
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

            assert_eq!(Some(2), node_map.steps_between("AAA", "ZZZ"));
        }

        {
            let node_map = NetworkMap::from_str(indoc! {"
                LLR

                AAA = (BBB, BBB)
                BBB = (AAA, ZZZ)
                ZZZ = (ZZZ, ZZZ)
            "})
            .unwrap();

            assert_eq!(Some(6), node_map.steps_between("AAA", "ZZZ"));
        }
    }
}
