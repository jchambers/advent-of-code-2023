use std::env;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let parabolic_dish = {
            let mut dish_string = String::new();
            File::open(path)?.read_to_string(&mut dish_string)?;

            ParabolicDish::from_str(dish_string.as_str())?
        };

        println!(
            "Load after tilting north: {}",
            parabolic_dish.tilt(Direction::North).load()
        );

        println!(
            "Load after 1,000,000,000 spins: {}",
            parabolic_dish.spin_cycle(1_000_000_000).load()
        );

        Ok(())
    } else {
        Err("Usage: day14 INPUT_FILE_PATH".into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ParabolicDish {
    width: usize,
    tiles: Vec<Tile>,
}

impl ParabolicDish {
    fn tilt(&self, direction: Direction) -> Self {
        let mut tilted_dish: Vec<Tile> = self
            .tiles
            .iter()
            .map(|tile| match tile {
                Tile::Empty | Tile::Round => Tile::Empty,
                Tile::Cube => Tile::Cube,
            })
            .collect();

        let mut round_indices: Vec<usize> = self
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, tile)| tile == &&Tile::Round)
            .map(|(i, _)| i)
            .collect();

        if direction == Direction::East || direction == Direction::South {
            round_indices.reverse();
        }

        // Settle the round rocks
        for round_index in round_indices {
            let mut x = round_index % self.width;
            let mut y = round_index / self.width;

            loop {
                let settled = match direction {
                    Direction::North => y == 0 || tilted_dish[self.index(x, y - 1)] != Tile::Empty,
                    Direction::South => {
                        y == self.height() - 1 || tilted_dish[self.index(x, y + 1)] != Tile::Empty
                    }
                    Direction::East => {
                        x == self.width - 1 || tilted_dish[self.index(x + 1, y)] != Tile::Empty
                    }
                    Direction::West => x == 0 || tilted_dish[self.index(x - 1, y)] != Tile::Empty,
                };

                if settled {
                    tilted_dish[self.index(x, y)] = Tile::Round;
                    break;
                }

                match direction {
                    Direction::North => {
                        y -= 1;
                    }
                    Direction::South => {
                        y += 1;
                    }
                    Direction::East => {
                        x += 1;
                    }
                    Direction::West => {
                        x -= 1;
                    }
                }
            }
        }

        Self {
            tiles: tilted_dish,
            width: self.width,
        }
    }

    fn spin(&self) -> Self {
        self.tilt(Direction::North)
            .tilt(Direction::West)
            .tilt(Direction::South)
            .tilt(Direction::East)
    }

    fn spin_cycle(&self, iterations: usize) -> Self {
        let mut previous_states: Vec<Self> = vec![self.clone()];

        for _ in 0..iterations {
            let next = previous_states.last().unwrap().spin();

            if previous_states.contains(&next) {
                let cycle_start = previous_states.iter().position(|d| d == &next).unwrap();
                let cycle_len = previous_states.len() - cycle_start;

                return previous_states[cycle_start + ((iterations - cycle_start) % cycle_len)]
                    .clone();
            }

            previous_states.push(next);
        }

        previous_states.pop().unwrap()
    }

    fn height(&self) -> usize {
        self.tiles.len() / self.width
    }

    fn index(&self, x: usize, y: usize) -> usize {
        x + (self.width * y)
    }

    fn load(&self) -> u32 {
        let height = self.height();

        self.tiles
            .iter()
            .enumerate()
            .filter(|(_, tile)| tile == &&Tile::Round)
            .map(|(i, _)| (height - (i / self.width)) as u32)
            .sum()
    }
}

impl FromStr for ParabolicDish {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(line) = string.lines().next() {
            let width = line.len();

            let tiles: Vec<Tile> = string
                .chars()
                .filter(|c| !c.is_whitespace())
                .map(Tile::try_from)
                .collect::<Result<_, _>>()?;

            if tiles.len() % width == 0 {
                Ok(ParabolicDish { width, tiles })
            } else {
                Err("Non-rectangular dish".into())
            }
        } else {
            Err("String contains no lines".into())
        }
    }
}

impl Display for ParabolicDish {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.tiles.chunks_exact(self.width).try_for_each(|row| {
            let line: String = row
                .iter()
                .map(|tile| match tile {
                    Tile::Empty => '.',
                    Tile::Round => 'O',
                    Tile::Cube => '#',
                })
                .collect();

            writeln!(f, "{}", line)
        })?;

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Tile {
    Empty,
    Round,
    Cube,
}

impl TryFrom<char> for Tile {
    type Error = Box<dyn Error>;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Tile::Empty),
            'O' => Ok(Tile::Round),
            '#' => Ok(Tile::Cube),
            _ => Err("Unrecognized tile".into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_tilt() {
        let tilted_dish = ParabolicDish::from_str(indoc! {"
            O....#....
            O.OO#....#
            .....##...
            OO.#O....O
            .O.....O#.
            O.#..O.#.#
            ..O..#O..O
            .......O..
            #....###..
            #OO..#....
        "})
        .unwrap()
        .tilt(Direction::North);

        let expected_dish = ParabolicDish::from_str(indoc! {"
            OOOO.#.O..
            OO..#....#
            OO..O##..O
            O..#.OO...
            ........#.
            ..#....#.#
            ..O..#.O.O
            ..O.......
            #....###..
            #....#....
        "})
        .unwrap();

        assert_eq!(expected_dish, tilted_dish);
    }

    #[test]
    fn test_load() {
        let tilted_dish = ParabolicDish::from_str(indoc! {"
            O....#....
            O.OO#....#
            .....##...
            OO.#O....O
            .O.....O#.
            O.#..O.#.#
            ..O..#O..O
            .......O..
            #....###..
            #OO..#....
        "})
        .unwrap()
        .tilt(Direction::North);

        assert_eq!(136, tilted_dish.load());
    }

    #[test]
    fn test_spin() {
        let spun_dish = ParabolicDish::from_str(indoc! {"
            O....#....
            O.OO#....#
            .....##...
            OO.#O....O
            .O.....O#.
            O.#..O.#.#
            ..O..#O..O
            .......O..
            #....###..
            #OO..#....
        "})
        .unwrap()
        .spin();

        let expected_dish = ParabolicDish::from_str(indoc! {"
            .....#....
            ....#...O#
            ...OO##...
            .OO#......
            .....OOO#.
            .O#...O#.#
            ....O#....
            ......OOOO
            #...O###..
            #..OO#....
        "})
        .unwrap();

        assert_eq!(expected_dish, spun_dish);
    }

    #[test]
    fn test_spin_cycle() {
        let spun_dish = ParabolicDish::from_str(indoc! {"
            O....#....
            O.OO#....#
            .....##...
            OO.#O....O
            .O.....O#.
            O.#..O.#.#
            ..O..#O..O
            .......O..
            #....###..
            #OO..#....
        "})
        .unwrap()
        .spin_cycle(3);

        let expected_dish = ParabolicDish::from_str(indoc! {"
            .....#....
            ....#...O#
            .....##...
            ..O#......
            .....OOO#.
            .O#...O#.#
            ....O#...O
            .......OOO
            #...O###.O
            #.OOO#...O
        "})
        .unwrap();

        assert_eq!(expected_dish, spun_dish);
    }

    #[test]
    fn test_spin_cycle_long() {
        let spun_dish = ParabolicDish::from_str(indoc! {"
            O....#....
            O.OO#....#
            .....##...
            OO.#O....O
            .O.....O#.
            O.#..O.#.#
            ..O..#O..O
            .......O..
            #....###..
            #OO..#....
        "})
        .unwrap()
        .spin_cycle(1_000_000_000);

        assert_eq!(64, spun_dish.load());
    }
}
