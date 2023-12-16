use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let contraption = {
            let mut contraption_string = String::new();
            File::open(path)?.read_to_string(&mut contraption_string)?;

            BeamContraption::from_str(contraption_string.as_str())?
        };

        println!(
            "Energized tiles: {}",
            contraption.energized_tiles(BeamHead::default())
        );

        println!("Max energized tiles: {}", contraption.max_energized_tiles());

        Ok(())
    } else {
        Err("Usage: day16 INPUT_FILE_PATH".into())
    }
}

struct BeamContraption {
    width: usize,
    tiles: Vec<Tile>,
}

impl BeamContraption {
    fn energized_tiles(&self, start: BeamHead) -> usize {
        let mut beam_heads = vec![start];
        let mut explored_tiles = vec![vec![]; self.tiles.len()];

        while let Some(beam_head) = beam_heads.pop() {
            let (x, y) = beam_head.position;

            if explored_tiles[self.index(x, y)].contains(&beam_head.heading) {
                // Avoid infinite loops!
                continue;
            }

            explored_tiles[self.index(x, y)].push(beam_head.heading);

            match self.tiles[self.index(x, y)] {
                Tile::Empty => {
                    if let Some(advanced) = self.advance_beam(&beam_head, beam_head.heading) {
                        beam_heads.push(advanced);
                    }
                }
                Tile::MirrorLeft => {
                    let advanced = match beam_head.heading {
                        Direction::Up => self.advance_beam(&beam_head, Direction::Left),
                        Direction::Down => self.advance_beam(&beam_head, Direction::Right),
                        Direction::Left => self.advance_beam(&beam_head, Direction::Up),
                        Direction::Right => self.advance_beam(&beam_head, Direction::Down),
                    };

                    if let Some(advanced) = advanced {
                        beam_heads.push(advanced);
                    }
                }
                Tile::MirrorRight => {
                    let advanced = match beam_head.heading {
                        Direction::Up => self.advance_beam(&beam_head, Direction::Right),
                        Direction::Down => self.advance_beam(&beam_head, Direction::Left),
                        Direction::Left => self.advance_beam(&beam_head, Direction::Down),
                        Direction::Right => self.advance_beam(&beam_head, Direction::Up),
                    };

                    if let Some(advanced) = advanced {
                        beam_heads.push(advanced);
                    }
                }
                Tile::SplitterHorizontal => match beam_head.heading {
                    Direction::Up | Direction::Down => {
                        if let Some(advanced) = self.advance_beam(&beam_head, Direction::Left) {
                            beam_heads.push(advanced);
                        }

                        if let Some(advanced) = self.advance_beam(&beam_head, Direction::Right) {
                            beam_heads.push(advanced);
                        }
                    }
                    Direction::Left | Direction::Right => {
                        if let Some(advanced) = self.advance_beam(&beam_head, beam_head.heading) {
                            beam_heads.push(advanced);
                        }
                    }
                },
                Tile::SplitterVertical => match beam_head.heading {
                    Direction::Up | Direction::Down => {
                        if let Some(advanced) = self.advance_beam(&beam_head, beam_head.heading) {
                            beam_heads.push(advanced);
                        }
                    }
                    Direction::Left | Direction::Right => {
                        if let Some(advanced) = self.advance_beam(&beam_head, Direction::Up) {
                            beam_heads.push(advanced);
                        }

                        if let Some(advanced) = self.advance_beam(&beam_head, Direction::Down) {
                            beam_heads.push(advanced);
                        }
                    }
                },
            }
        }

        explored_tiles
            .iter()
            .filter(|directions| !directions.is_empty())
            .count()
    }

    fn max_energized_tiles(&self) -> usize {
        let mut starting_positions = Vec::with_capacity(self.width * 2 + self.height() * 2);

        (0..self.width).for_each(|x| {
            starting_positions.push(BeamHead {
                position: (x, 0),
                heading: Direction::Down,
            });

            starting_positions.push(BeamHead {
                position: (x, self.height() - 1),
                heading: Direction::Up,
            });
        });

        (0..self.height()).for_each(|y| {
            starting_positions.push(BeamHead {
                position: (0, y),
                heading: Direction::Right,
            });

            starting_positions.push(BeamHead {
                position: (self.width - 1, y),
                heading: Direction::Left,
            });
        });

        starting_positions
            .into_iter()
            .map(|starting_position| self.energized_tiles(starting_position))
            .max()
            .unwrap_or(0)
    }

    fn advance_beam(&self, beam_head: &BeamHead, heading: Direction) -> Option<BeamHead> {
        let (x, y) = beam_head.position;

        match heading {
            Direction::Up => {
                if y > 0 {
                    Some(BeamHead {
                        position: (x, y - 1),
                        heading,
                    })
                } else {
                    None
                }
            }
            Direction::Down => {
                if y < self.height() - 1 {
                    Some(BeamHead {
                        position: (x, y + 1),
                        heading,
                    })
                } else {
                    None
                }
            }
            Direction::Left => {
                if x > 0 {
                    Some(BeamHead {
                        position: (x - 1, y),
                        heading,
                    })
                } else {
                    None
                }
            }
            Direction::Right => {
                if x < self.width - 1 {
                    Some(BeamHead {
                        position: (x + 1, y),
                        heading,
                    })
                } else {
                    None
                }
            }
        }
    }

    fn height(&self) -> usize {
        self.tiles.len() / self.width
    }

    fn index(&self, x: usize, y: usize) -> usize {
        x + (y * self.width)
    }
}

impl FromStr for BeamContraption {
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
                Ok(BeamContraption { width, tiles })
            } else {
                Err("Non-rectangular beam cave".into())
            }
        } else {
            Err("String contained no lines".into())
        }
    }
}

enum Tile {
    Empty,
    MirrorLeft,
    MirrorRight,
    SplitterHorizontal,
    SplitterVertical,
}

impl TryFrom<char> for Tile {
    type Error = Box<dyn Error>;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Tile::Empty),
            '\\' => Ok(Tile::MirrorLeft),
            '/' => Ok(Tile::MirrorRight),
            '-' => Ok(Tile::SplitterHorizontal),
            '|' => Ok(Tile::SplitterVertical),
            _ => Err("Unrecognized tile".into()),
        }
    }
}

struct BeamHead {
    position: (usize, usize),
    heading: Direction,
}

impl Default for BeamHead {
    fn default() -> Self {
        BeamHead {
            position: (0, 0),
            heading: Direction::Right,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const TEST_CONTRAPTION_STRING: &str = indoc! {r"
            .|...\....
            |.-.\.....
            .....|-...
            ........|.
            ..........
            .........\
            ..../.\\..
            .-.-/..|..
            .|....-|.\
            ..//.|....
    "};

    #[test]
    fn test_energized_tiles() {
        let contraption = BeamContraption::from_str(TEST_CONTRAPTION_STRING).unwrap();

        assert_eq!(46, contraption.energized_tiles(BeamHead::default()));
    }

    #[test]
    fn test_max_energized_tiles() {
        let contraption = BeamContraption::from_str(TEST_CONTRAPTION_STRING).unwrap();

        assert_eq!(51, contraption.max_energized_tiles());
    }
}
