use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let dig_plan: DigPlan = BufReader::new(File::open(path)?)
            .lines()
            .map_while(Result::ok)
            .map(|line| Instruction::from_str(line.as_str()))
            .collect::<Result<_, _>>()?;

        println!("Enclosed area: {}", dig_plan.enclosed_area());

        Ok(())
    } else {
        Err("Usage: day18 INPUT_FILE_PATH".into())
    }
}

struct DigPlan {
    instructions: Vec<Instruction>,
}

impl DigPlan {
    fn line_segments(&self) -> Vec<LineSegment> {
        let mut segments = Vec::with_capacity(self.instructions.len());
        let mut last_position = (0, 0);

        for instruction in &self.instructions {
            let distance = instruction.distance as i32;

            let mut next_position = match instruction.direction {
                Direction::Up => (last_position.0, last_position.1 + distance),
                Direction::Down => (last_position.0, last_position.1 - distance),
                Direction::Left => (last_position.0 - distance, last_position.1),
                Direction::Right => (last_position.0 + distance, last_position.1),
            };

            segments.push(LineSegment {
                start: last_position,
                end: next_position,
            });

            last_position = next_position;
        }

        // Close the perimeter if it's not closed already
        if last_position != (0, 0) {
            segments.push(LineSegment {
                start: last_position,
                end: (0, 0),
            });
        }

        segments
    }

    fn perimeter(&self) -> u32 {
        self.instructions
            .iter()
            .map(|instruction| instruction.distance as u32)
            .sum::<u32>()
    }

    fn enclosed_area(&self) -> u32 {
        let segments = self.line_segments();

        let min_x = segments
            .iter()
            .map(|segment| segment.start.0.min(segment.end.0))
            .min()
            .unwrap();

        let max_x = segments
            .iter()
            .map(|segment| segment.start.0.max(segment.end.0))
            .max()
            .unwrap();

        let min_y = segments
            .iter()
            .map(|segment| segment.start.1.min(segment.end.1))
            .min()
            .unwrap();

        let max_y = segments
            .iter()
            .map(|segment| segment.start.1.max(segment.end.1))
            .max()
            .unwrap();

        let width = (max_x - min_x) + 1;
        let height = (max_y - min_y) + 1;

        let mut excavated_tiles = vec![false; (width * height) as usize];

        // Fill perimeter tiles; we only need to pay attention to horizontal segments because we'll
        // get the vertical segments on area fill.
        segments
            .iter()
            .filter(|segment| segment.is_horizontal())
            .map(|segment| segment.translate(-min_x, -min_y))
            .for_each(|segment| {
                let start_index = segment.start.0 + (width * segment.start.1);
                let end_index = segment.end.0 + (width * segment.end.1);

                for i in start_index.min(end_index) as usize..=start_index.max(end_index) as usize {
                    excavated_tiles[i] = true;
                }
            });

        // Fill interior tiles
        let vertical_segments: Vec<&LineSegment> = segments
            .iter()
            .filter(|segment| segment.is_vertical())
            .collect();

        for y in 0..height {
            let mut intersecting_vertical_segments: Vec<LineSegment> = vertical_segments
                .iter()
                .map(|segment| segment.translate(-min_x, -min_y))
                .filter(|segment| segment.intersects_horizontal_line(y))
                .filter(|segment| segment.start.1.min(segment.end.1) < y)
                .collect();

            intersecting_vertical_segments.sort_by(|a, b| a.start.0.cmp(&b.start.0));

            for pair in intersecting_vertical_segments.chunks_exact(2) {
                if let [left, right] = pair {
                    let start_index = left.start.0 + (width * y);
                    let end_index = right.start.0 + (width * y);

                    for i in start_index as usize..=end_index as usize {
                        excavated_tiles[i] = true;
                    }
                }
            }
        }

        excavated_tiles.iter().filter(|&&t| t).count() as u32
    }
}

impl FromIterator<Instruction> for DigPlan {
    fn from_iter<T: IntoIterator<Item = Instruction>>(iter: T) -> Self {
        DigPlan {
            instructions: iter.into_iter().collect(),
        }
    }
}

struct Instruction {
    direction: Direction,
    distance: usize,
    _color: String,
}

impl FromStr for Instruction {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let [direction, distance, color] = string.split(' ').collect::<Vec<&str>>().as_slice() {
            let direction = Direction::from_str(direction)?;
            let distance = distance.parse()?;
            let _color = String::from(*color);

            Ok(Instruction {
                direction,
                distance,
                _color,
            })
        } else {
            Err("Could not parse instructiion".into())
        }
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl FromStr for Direction {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "U" => Ok(Direction::Up),
            "D" => Ok(Direction::Down),
            "L" => Ok(Direction::Left),
            "R" => Ok(Direction::Right),
            _ => Err("Unrecognized direction".into()),
        }
    }
}

struct LineSegment {
    start: (i32, i32),
    end: (i32, i32),
}

impl LineSegment {
    fn is_vertical(&self) -> bool {
        self.start.0 == self.end.0 && self.start.1 != self.end.1
    }

    fn is_horizontal(&self) -> bool {
        self.start.1 == self.end.1 && self.start.0 != self.end.0
    }

    fn intersects_horizontal_line(&self, y: i32) -> bool {
        self.start.1.min(self.end.1) <= y && self.start.1.max(self.end.1) >= y
    }

    fn translate(&self, x: i32, y: i32) -> Self {
        LineSegment {
            start: (self.start.0 + x, self.start.1 + y),
            end: (self.end.0 + x, self.end.1 + y),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const TEST_INSTRUTIONS: &str = indoc! {"
        R 6 (#70c710)
        D 5 (#0dc571)
        L 2 (#5713f0)
        D 2 (#d2c081)
        R 2 (#59c680)
        D 2 (#411b91)
        L 5 (#8ceee2)
        U 2 (#caa173)
        L 1 (#1b58a2)
        U 2 (#caa171)
        R 2 (#7807d2)
        U 3 (#a77fa3)
        L 2 (#015232)
        U 2 (#7a21e3)
    "};

    #[test]
    fn test_perimeter() {
        let dig_plan: DigPlan = TEST_INSTRUTIONS
            .lines()
            .map(Instruction::from_str)
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(38, dig_plan.perimeter());
    }

    #[test]
    fn test_enclosed_area() {
        let dig_plan: DigPlan = TEST_INSTRUTIONS
            .lines()
            .map(Instruction::from_str)
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(62, dig_plan.enclosed_area());
    }
}
