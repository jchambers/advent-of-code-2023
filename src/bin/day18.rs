use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        {
            let dig_plan: DigPlan = BufReader::new(File::open(path)?)
                .lines()
                .map_while(Result::ok)
                .map(|line| Instruction::from_str(line.as_str()))
                .collect::<Result<_, _>>()?;

            println!("Enclosed area: {}", dig_plan.enclosed_area());
        }

        {
            let dig_plan: DigPlan = BufReader::new(File::open(path)?)
                .lines()
                .map_while(Result::ok)
                .map(|line| Instruction::from_str_color(line.as_str()))
                .collect::<Result<_, _>>()?;

            println!(
                "Enclosed area with parsed colors: {}",
                dig_plan.enclosed_area()
            );
        }

        Ok(())
    } else {
        Err("Usage: day18 INPUT_FILE_PATH".into())
    }
}

struct DigPlan {
    instructions: Vec<Instruction>,
}

impl DigPlan {
    fn enclosed_area(&self) -> u64 {
        // This is a bit of lazy cheat, but let's assume we're traveling clockwise (true in the example data and my
        // personal puzzle input). Let's also assume (less specific to the input) that the path is always the exterior
        // perimeter of the trench and there are no "pinched off" sections.
        //
        // The strategy here, then, is to get the coordinates of the vertices of the bounding polygon of the trench.
        // This is slightly complicated by off-by-one issues. If we go R4, D2, then we have:
        //
        // #####
        //     #
        //     #
        //
        // …which is the start of a 5 × 3 box (area 15). But if we just treat those directions as coordinate changes
        // (x += 4, y -= 2), then we wind up with a polygon with area 8, which is clearly incorrect. To fix that, we
        // insert a "phantom" R1 when transitioning from upward travel to downward travel, then a "phantom" L1 when
        // transitioning back. The transition between left/right gets analogous treatment with phantom U1/D1
        // instructions.
        //
        // With the bounding polygon figured out, we can use the shoelace formula to find the area of the polygon in
        // O(n).
        let mut previous_vertical_direction = Direction::Up;
        let mut previous_horizontal_direction = Direction::Right;

        let mut vertices = Vec::with_capacity(self.instructions.len());
        vertices.push((0, 0));

        for instruction in &self.instructions {
            let x_offset = match (&previous_vertical_direction, &instruction.direction) {
                (Direction::Up, Direction::Down) => 1,
                (Direction::Down, Direction::Up) => -1,
                _ => 0,
            };

            let y_offset = match (&previous_horizontal_direction, &instruction.direction) {
                (Direction::Left, Direction::Right) => 1,
                (Direction::Right, Direction::Left) => -1,
                _ => 0,
            };

            if x_offset != 0 || y_offset != 0 {
                let (x, y) = vertices.last().unwrap();
                vertices.push((x + x_offset, y + y_offset));
            }

            let (x, y) = vertices.last().unwrap();

            match instruction.direction {
                Direction::Up => vertices.push((*x, *y + instruction.distance as i32)),
                Direction::Down => vertices.push((*x, *y - instruction.distance as i32)),
                Direction::Left => vertices.push((*x - instruction.distance as i32, *y)),
                Direction::Right => vertices.push((*x + instruction.distance as i32, *y)),
            }

            if instruction.direction.is_horizontal() {
                previous_horizontal_direction = instruction.direction;
            } else {
                previous_vertical_direction = instruction.direction;
            }
        }

        // Close the polygon
        vertices.push((0, 0));

        let mut enclosed_area = 0;
        let mut windows = vertices.windows(2);

        while let Some([(x1, y1), (x2, y2)]) = windows.next() {
            enclosed_area += (*y1 as i64 + *y2 as i64) * (*x1 as i64 - *x2 as i64)
        }

        enclosed_area.unsigned_abs() / 2
    }
}

impl FromIterator<Instruction> for DigPlan {
    fn from_iter<T: IntoIterator<Item = Instruction>>(iter: T) -> Self {
        DigPlan {
            instructions: iter.into_iter().collect(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Instruction {
    direction: Direction,
    distance: u32,
}

impl Instruction {
    fn from_str_color(string: &str) -> Result<Self, Box<dyn Error>> {
        if let [_, _, color] = string.split(' ').collect::<Vec<&str>>().as_slice() {
            if let Some(color) = color
                .strip_prefix("(#")
                .and_then(|color| color.strip_suffix(')'))
            {
                if color.len() == 6 {
                    let distance = u32::from_str_radix(&color[0..5], 16)?;

                    let direction = match &color[5..] {
                        "0" => Direction::Right,
                        "1" => Direction::Down,
                        "2" => Direction::Left,
                        "3" => Direction::Up,
                        _ => {
                            return Err("Unrecognized direction".into());
                        }
                    };

                    Ok(Instruction {
                        direction,
                        distance,
                    })
                } else {
                    Err("Unexpected color length".into())
                }
            } else {
                Err("Could not find color component of string".into())
            }
        } else {
            Err("Could not parse instruction".into())
        }
    }
}

impl FromStr for Instruction {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let [direction, distance, _color] = string.split(' ').collect::<Vec<&str>>().as_slice() {
            let direction = Direction::from_str(direction)?;
            let distance = distance.parse()?;

            Ok(Instruction {
                direction,
                distance,
            })
        } else {
            Err("Could not parse instruction".into())
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn is_horizontal(&self) -> bool {
        match self {
            Direction::Left | Direction::Right => true,
            Direction::Up | Direction::Down => false,
        }
    }
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
    fn test_enclosed_area() {
        let dig_plan: DigPlan = TEST_INSTRUTIONS
            .lines()
            .map(Instruction::from_str)
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(62, dig_plan.enclosed_area());
    }

    #[test]
    fn test_instruction_from_str_color() {
        for (string, expected_instruction) in [
            (
                "R 6 (#70c710)",
                Instruction {
                    distance: 461937,
                    direction: Direction::Right,
                },
            ),
            (
                "D 5 (#0dc571)",
                Instruction {
                    distance: 56407,
                    direction: Direction::Down,
                },
            ),
            (
                "L 2 (#5713f0)",
                Instruction {
                    distance: 356671,
                    direction: Direction::Right,
                },
            ),
            (
                "D 2 (#d2c081)",
                Instruction {
                    distance: 863240,
                    direction: Direction::Down,
                },
            ),
            (
                "R 2 (#59c680)",
                Instruction {
                    distance: 367720,
                    direction: Direction::Right,
                },
            ),
            (
                "D 2 (#411b91)",
                Instruction {
                    distance: 266681,
                    direction: Direction::Down,
                },
            ),
            (
                "L 5 (#8ceee2)",
                Instruction {
                    distance: 577262,
                    direction: Direction::Left,
                },
            ),
            (
                "U 2 (#caa173)",
                Instruction {
                    distance: 829975,
                    direction: Direction::Up,
                },
            ),
            (
                "L 1 (#1b58a2)",
                Instruction {
                    distance: 112010,
                    direction: Direction::Left,
                },
            ),
            (
                "U 2 (#caa171)",
                Instruction {
                    distance: 829975,
                    direction: Direction::Down,
                },
            ),
            (
                "R 2 (#7807d2)",
                Instruction {
                    distance: 491645,
                    direction: Direction::Left,
                },
            ),
            (
                "U 3 (#a77fa3)",
                Instruction {
                    distance: 686074,
                    direction: Direction::Up,
                },
            ),
            (
                "L 2 (#015232)",
                Instruction {
                    distance: 5411,
                    direction: Direction::Left,
                },
            ),
            (
                "U 2 (#7a21e3)",
                Instruction {
                    distance: 500254,
                    direction: Direction::Up,
                },
            ),
        ] {
            assert_eq!(
                expected_instruction,
                Instruction::from_str_color(string).unwrap()
            );
        }
    }

    #[test]
    fn test_enclosed_area_color() {
        let dig_plan: DigPlan = TEST_INSTRUTIONS
            .lines()
            .map(Instruction::from_str_color)
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(952_408_144_115, dig_plan.enclosed_area());
    }
}
