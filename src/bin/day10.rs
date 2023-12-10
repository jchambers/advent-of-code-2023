use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::ops::Neg;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let pipe_map = {
            let mut map_string = String::new();
            File::open(path)?.read_to_string(&mut map_string)?;

            PipeMap::from_str(map_string.as_str())?
        };

        println!(
            "Max distance from start: {}",
            pipe_map.max_distance_from_start()?
        );

        println!(
            "Tiles enclosed by path: {}",
            pipe_map.enclosed_tiles()?
        );

        Ok(())
    } else {
        Err("Usage: day10 INPUT_FILE_PATH".into())
    }
}

struct PipeMap {
    pipes: Vec<Option<Pipe>>,

    width: usize,
    height: usize,

    start_index: usize,
}

impl PipeMap {
    fn pipe(&self, x: isize, y: isize) -> &Option<Pipe> {
        if x < 0 || x >= self.width as isize || y < 0 || y >= self.height as isize {
            &None
        } else {
            &self.pipes[self.index(x as usize, y as usize)]
        }
    }

    fn loop_length(&self) -> Result<usize, Box<dyn Error>> {
        Ok(self.path()?.iter().filter(|cell| cell.is_some()).count())
    }

    fn enclosed_tiles(&self) -> Result<usize, Box<dyn Error>> {
        let path = self.path()?;

        // The strategy here is to use the winding number algorithm
        // (https://en.wikipedia.org/wiki/Point_in_polygon#Winding_number_algorithm), but we need to
        // be careful about bookkeeping at the corners. For consistency, assume that we're casting
        // horizontal rays along the bottom edge of each tile (the top would work, too, but we're
        // just picking one arbitrarily). That means corner tiles with an "up" exit will not produce
        // winding number changes, but tiles with a "down" exit will.
        let winding_number_changes = {
            let mut winding_number_changes = vec![0; path.len()];
            let mut position = self.start_index;
            let mut last_vertical_direction = None;

            loop {
                let next_position = match path[position].expect("Path must be contiguous") {
                    Direction::Up => {
                        last_vertical_direction = Some(Direction::Up);
                        position - self.width
                    }
                    Direction::Down => {
                        last_vertical_direction = Some(Direction::Down);
                        position + self.width
                    }
                    Direction::Left => position - 1,
                    Direction::Right => position + 1,
                };

                if self.pipes[position]
                    .as_ref()
                    .expect("Tile on path must contain pipe")
                    .exits
                    .contains(&Direction::Down)
                {
                    winding_number_changes[position] = match last_vertical_direction {
                        Some(Direction::Up) => 1,
                        Some(Direction::Down) => -1,
                        _ => panic!("Must have a last known vertical direction at corners"),
                    };
                }

                position = next_position;

                if position == self.start_index {
                    break winding_number_changes;
                }
            }
        };

        let mut enclosed_tiles = 0;

        for y in 0..self.height {
            let mut winding_number = 0;

            for x in 0..self.width {
                let index = self.index(x, y);

                winding_number += winding_number_changes[index];

                if winding_number % 2 != 0 && path[index].is_none() {
                    enclosed_tiles += 1;
                }
            }
        }

        Ok(enclosed_tiles)
    }

    fn path(&self) -> Result<Vec<Option<Direction>>, Box<dyn Error>> {
        let mut position = self.start_index;
        let mut direction = self.pipes[position]
            .as_ref()
            .ok_or("Could not find starting pipe")?
            .exits[0];

        let mut path = vec![None; self.pipes.len()];

        while path[position].is_none() {
            path[position] = Some(direction);

            let came_from = -direction;

            match direction {
                Direction::Up => position -= self.width,
                Direction::Down => position += self.width,
                Direction::Left => position -= 1,
                Direction::Right => position += 1,
            };

            direction = *self.pipes[position]
                .as_ref()
                .ok_or("Could not find connecting pipe")?
                .exits
                .iter()
                .find(|exit| exit != &&came_from)
                .ok_or("Could not find pipe exit")?;
        }

        Ok(path)
    }

    fn index(&self, x: usize, y: usize) -> usize {
        x + (y * self.width)
    }

    fn max_distance_from_start(&self) -> Result<usize, Box<dyn Error>> {
        Ok((self.loop_length()? + 1) / 2)
    }
}

impl FromStr for PipeMap {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut lines = string.lines().peekable();
        let width = lines
            .peek()
            .ok_or("Pipe map did not contain a first line")?
            .len();

        let pipes: Vec<Option<Pipe>> = lines
            .flat_map(|line| line.chars())
            .filter(|c| !c.is_whitespace())
            .map(|c| match c {
                '|' => Some(Pipe {
                    exits: [Direction::Up, Direction::Down],
                }),
                '-' => Some(Pipe {
                    exits: [Direction::Left, Direction::Right],
                }),
                'L' => Some(Pipe {
                    exits: [Direction::Up, Direction::Right],
                }),
                'J' => Some(Pipe {
                    exits: [Direction::Up, Direction::Left],
                }),
                '7' => Some(Pipe {
                    exits: [Direction::Down, Direction::Left],
                }),
                'F' => Some(Pipe {
                    exits: [Direction::Down, Direction::Right],
                }),
                // We'll treat the starting position as a special case
                'S' | '.' => None,
                _ => panic!(),
            })
            .collect();

        let height = if pipes.len() % width == 0 {
            pipes.len() / width
        } else {
            return Err("Inconsistent row width".into());
        };

        let start_index = string
            .chars()
            .filter(|c| !c.is_whitespace())
            .enumerate()
            .find(|&(_, c)| c == 'S')
            .map(|(index, _)| index)
            .ok_or("Could not find start position")?;

        let start_x = (start_index % width) as isize;
        let start_y = (start_index / width) as isize;

        let mut pipe_map = PipeMap {
            pipes,

            width,
            height,

            start_index,
        };

        let start_has_left_exit = pipe_map
            .pipe(start_x - 1, start_y)
            .as_ref()
            .map(|pipe| pipe.exits.contains(&Direction::Right))
            .unwrap_or(false);

        let start_has_right_exit = pipe_map
            .pipe(start_x + 1, start_y)
            .as_ref()
            .map(|pipe| pipe.exits.contains(&Direction::Left))
            .unwrap_or(false);

        let start_has_up_exit = pipe_map
            .pipe(start_x, start_y - 1)
            .as_ref()
            .map(|pipe| pipe.exits.contains(&Direction::Down))
            .unwrap_or(false);

        let start_has_down_exit = pipe_map
            .pipe(start_x, start_y + 1)
            .as_ref()
            .map(|pipe| pipe.exits.contains(&Direction::Up))
            .unwrap_or(false);

        let start_exits = match (
            start_has_left_exit,
            start_has_right_exit,
            start_has_up_exit,
            start_has_down_exit,
        ) {
            (false, false, true, true) => [Direction::Up, Direction::Down],
            (true, true, false, false) => [Direction::Left, Direction::Right],
            (false, true, true, false) => [Direction::Up, Direction::Right],
            (true, false, true, false) => [Direction::Up, Direction::Left],
            (true, false, false, true) => [Direction::Down, Direction::Left],
            (false, true, false, true) => [Direction::Down, Direction::Right],
            _ => return Err("Could not assign directions to starting pipe".into()),
        };

        pipe_map.pipes[start_index] = Some(Pipe { exits: start_exits });

        Ok(pipe_map)
    }
}

struct Pipe {
    exits: [Direction; 2],
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Neg for Direction {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_loop_length() {
        {
            let pipe_map = PipeMap::from_str(indoc! {"
                .....
                .S-7.
                .|.|.
                .L-J.
                .....
            "})
            .unwrap();

            assert_eq!(8, pipe_map.loop_length().unwrap());
        }

        {
            let pipe_map = PipeMap::from_str(indoc! {"
                ..F7.
                .FJ|.
                SJ.L7
                |F--J
                LJ...
            "})
            .unwrap();

            assert_eq!(16, pipe_map.loop_length().unwrap());
        }
    }

    #[test]
    fn test_max_distance_from_start() {
        {
            let pipe_map = PipeMap::from_str(indoc! {"
                .....
                .S-7.
                .|.|.
                .L-J.
                .....
            "})
            .unwrap();

            assert_eq!(4, pipe_map.max_distance_from_start().unwrap());
        }

        {
            let pipe_map = PipeMap::from_str(indoc! {"
                ..F7.
                .FJ|.
                SJ.L7
                |F--J
                LJ...
            "})
            .unwrap();

            assert_eq!(8, pipe_map.max_distance_from_start().unwrap());
        }
    }

    #[test]
    fn test_enclosed_tiles() {
        {
            let pipe_map = PipeMap::from_str(indoc! {"
                ...........
                .S-------7.
                .|F-----7|.
                .||.....||.
                .||.....||.
                .|L-7.F-J|.
                .|..|.|..|.
                .L--J.L--J.
                ...........
            "})
            .unwrap();

            assert_eq!(4, pipe_map.enclosed_tiles().unwrap());
        }

        {
            let pipe_map = PipeMap::from_str(indoc! {"
                ..........
                .S------7.
                .|F----7|.
                .||....||.
                .||....||.
                .|L-7F-J|.
                .|..||..|.
                .L--JL--J.
                ..........
            "})
            .unwrap();

            assert_eq!(4, pipe_map.enclosed_tiles().unwrap());
        }

        {
            let pipe_map = PipeMap::from_str(indoc! {"
                .F----7F7F7F7F-7....
                .|F--7||||||||FJ....
                .||.FJ||||||||L7....
                FJL7L7LJLJ||LJ.L-7..
                L--J.L7...LJS7F-7L7.
                ....F-J..F7FJ|L7L7L7
                ....L7.F7||L7|.L7L7|
                .....|FJLJ|FJ|F7|.LJ
                ....FJL-7.||.||||...
                ....L---J.LJ.LJLJ...
            "})
            .unwrap();

            assert_eq!(8, pipe_map.enclosed_tiles().unwrap());
        }

        {
            let pipe_map = PipeMap::from_str(indoc! {"
                FF7FSF7F7F7F7F7F---7
                L|LJ||||||||||||F--J
                FL-7LJLJ||||||LJL-77
                F--JF--7||LJLJ7F7FJ-
                L---JF-JLJ.||-FJLJJ7
                |F|F-JF---7F7-L7L|7|
                |FFJF7L7F-JF7|JL---7
                7-L-JL7||F7|L7F-7F7|
                L.L7LFJ|||||FJL7||LJ
                L7JLJL-JLJLJL--JLJ.L
            "})
            .unwrap();

            assert_eq!(10, pipe_map.enclosed_tiles().unwrap());
        }
    }
}
