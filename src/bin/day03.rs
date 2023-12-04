use crate::Cell::{Digit, Empty, Gear, OtherSymbol};
use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let schematic = {
            let mut board_string = String::new();
            File::open(path)?.read_to_string(&mut board_string)?;

            EngineSchematic::from_str(board_string.as_str())?
        };

        println!(
            "Sum of part numbers adjacent to symbols: {}",
            schematic.part_number_sum()
        );

        println!("Sum of gear ratios: {}", schematic.gear_ratio_sum());

        Ok(())
    } else {
        Err("Usage: day03 INPUT_FILE_PATH".into())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Cell {
    Empty,
    Digit(u8),
    Gear,
    OtherSymbol,
}

struct EngineSchematic {
    cells: Vec<Cell>,

    width: usize,
    height: usize,
}

impl EngineSchematic {
    fn cell(&self, x: isize, y: isize) -> Cell {
        if x < 0 || x >= self.width as isize || y < 0 || y >= self.height as isize {
            Empty
        } else {
            *self
                .cells
                .get(y as usize * self.width + x as usize)
                .unwrap()
        }
    }

    fn has_adjacent_symbol(&self, x: isize, y: isize) -> bool {
        for i in x - 1..=x + 1 {
            for j in y - 1..=y + 1 {
                match self.cell(i, j) {
                    Gear | OtherSymbol => return true,
                    _ => continue,
                }
            }
        }

        false
    }

    fn part_number_sum(&self) -> u32 {
        let mut part_number_sum = 0;

        let mut current_part_number = 0;
        let mut found_adjacent_symbol = false;

        for y in 0..self.height {
            for x in 0..self.width {
                match self.cell(x as isize, y as isize) {
                    Digit(n) => {
                        if !found_adjacent_symbol {
                            found_adjacent_symbol =
                                self.has_adjacent_symbol(x as isize, y as isize);
                        }

                        current_part_number *= 10;
                        current_part_number += n as u32;
                    }
                    _ => {
                        if found_adjacent_symbol {
                            part_number_sum += current_part_number;
                        }

                        current_part_number = 0;
                        found_adjacent_symbol = false;
                    }
                }
            }

            if found_adjacent_symbol {
                part_number_sum += current_part_number;
            }

            current_part_number = 0;
            found_adjacent_symbol = false;
        }

        part_number_sum
    }

    fn adjacent_gear_indices(&self, x: isize, y: isize) -> HashSet<usize> {
        let mut adjacent_gear_indices = HashSet::new();

        for i in x - 1..=x + 1 {
            for j in y - 1..=y + 1 {
                if self.cell(i, j) == Gear {
                    adjacent_gear_indices.insert(i as usize * self.width + j as usize);
                }
            }
        }

        adjacent_gear_indices
    }

    fn gear_ratio_sum(&self) -> u32 {
        let mut part_numbers_by_adjacent_gear_indices = HashMap::new();

        let mut current_part_number = 0;

        for y in 0..self.height {
            let mut adjacent_gear_indices = HashSet::new();

            for x in 0..self.width {
                match self.cell(x as isize, y as isize) {
                    Digit(n) => {
                        adjacent_gear_indices
                            .extend(self.adjacent_gear_indices(x as isize, y as isize));

                        current_part_number *= 10;
                        current_part_number += n as u32;
                    }
                    _ => {
                        for gear_index in &adjacent_gear_indices {
                            part_numbers_by_adjacent_gear_indices
                                .entry(*gear_index)
                                .or_insert(Vec::new())
                                .push(current_part_number)
                        }

                        current_part_number = 0;
                        adjacent_gear_indices.clear();
                    }
                }
            }

            for gear_index in &adjacent_gear_indices {
                part_numbers_by_adjacent_gear_indices
                    .entry(*gear_index)
                    .or_insert(Vec::new())
                    .push(current_part_number)
            }

            current_part_number = 0;
            adjacent_gear_indices.clear();
        }

        part_numbers_by_adjacent_gear_indices
            .iter()
            .filter_map(|(_, adjacent_part_numbers)| {
                if adjacent_part_numbers.len() == 2 {
                    Some(adjacent_part_numbers.iter().product::<u32>())
                } else {
                    None
                }
            })
            .sum()
    }
}

impl FromStr for EngineSchematic {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let width = string.find('\n').unwrap_or(string.len());

        let cells: Vec<Cell> = string
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| match c {
                '.' => Empty,
                '0'..='9' => Digit(c.to_digit(10).unwrap() as u8),
                '*' => Gear,
                _ => OtherSymbol,
            })
            .collect();

        if cells.len() % width != 0 {
            Err("Non-rectangular engine schematic".into())
        } else {
            let height = cells.len() / width;

            Ok(EngineSchematic {
                cells,
                width,
                height,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const TEST_SCHEMATIC: &str = indoc! {"
        467..114..
        ...*......
        ..35..633.
        ......#...
        617*......
        .....+.58.
        ..592.....
        ......755.
        ...$.*....
        .664.598..
    "};

    #[test]
    fn test_part_number_sum() {
        assert_eq!(
            4361,
            EngineSchematic::from_str(TEST_SCHEMATIC)
                .unwrap()
                .part_number_sum()
        );
    }

    #[test]
    fn test_gear_ratio_sum() {
        assert_eq!(
            467835,
            EngineSchematic::from_str(TEST_SCHEMATIC)
                .unwrap()
                .gear_ratio_sum()
        );
    }
}
