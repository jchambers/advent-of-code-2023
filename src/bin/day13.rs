use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::ops::Not;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let mirror_fields: Vec<MirrorField> = {
            let mut fields_string = String::new();
            File::open(path)?.read_to_string(&mut fields_string)?;

            fields_string
                .split("\n\n")
                .map(MirrorField::from_str)
                .collect::<Result<_, _>>()?
        };

        println!(
            "Sum of scores: {}",
            mirror_fields
                .iter()
                .map(|mirror_field| mirror_field.reflection().unwrap().score())
                .sum::<u32>()
        );

        println!(
            "Sum of scores with smudged mirrors: {}",
            mirror_fields
                .iter()
                .inspect(|_| println!("Field"))
                .map(|mirror_field| mirror_field.smudged_reflection().unwrap().score())
                .sum::<u32>()
        );

        Ok(())
    } else {
        Err("Usage: day13 INPUT_FILE_PATH".into())
    }
}

struct MirrorField {
    width: usize,
    tiles: Vec<Tile>,
}

impl MirrorField {
    fn reflection(&self) -> Option<Reflection> {
        self.find_partition_row()
            .map(Reflection::Horizontal)
            .or_else(|| {
                self.transpose()
                    .find_partition_row()
                    .map(Reflection::Vertical)
            })
    }

    fn smudged_reflection(&self) -> Option<Reflection> {
        let original_reflection = self.reflection();

        for smudged_tile in 0..self.tiles.len() {
            let mut smudged_tiles = self.tiles.clone();
            smudged_tiles[smudged_tile] = !self.tiles[smudged_tile];

            let smudged_reflection = MirrorField {
                width: self.width,
                tiles: smudged_tiles,
            }.reflection();

            if smudged_reflection.is_some() && smudged_reflection != original_reflection {
                return smudged_reflection;
            }
        }

        None
    }

    fn find_partition_row(&self) -> Option<usize> {
        let height = self.tiles.len() / self.width;

        for row in 1..height {
            let mut top = row - 1;
            let mut bottom = row;

            loop {
                if self.tiles[top * self.width..(top + 1) * self.width]
                    == self.tiles[bottom * self.width..(bottom + 1) * self.width]
                {
                    if top == 0 || bottom == height - 1 {
                        return Some(row);
                    }

                    top -= 1;
                    bottom += 1;
                } else {
                    break;
                }
            }
        }

        None
    }

    fn height(&self) -> usize {
        self.tiles.len() / self.width
    }

    fn transpose(&self) -> Self {
        let mut transposed_tiles = Vec::with_capacity(self.tiles.len());

        for col in 0..self.width {
            for row in 0..self.height() {
                transposed_tiles.push(self.tiles[col + (row * self.width)]);
            }
        }

        Self {
            width: self.height(),
            tiles: transposed_tiles,
        }
    }
}

impl FromStr for MirrorField {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(line) = string.lines().next() {
            let width = line.len();

            let tiles: Vec<Tile> = string
                .chars()
                .filter(|c| !c.is_whitespace())
                .map(Tile::try_from)
                .collect::<Result<_, _>>()?;

            if tiles.len() % width != 0 {
                return Err("Non-rectangular field shape".into());
            }

            Ok(MirrorField { width, tiles })
        } else {
            Err("String contained no lines".into())
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Reflection {
    Horizontal(usize),
    Vertical(usize),
}

impl Reflection {
    fn score(&self) -> u32 {
        match self {
            Reflection::Horizontal(row) => *row as u32 * 100,
            Reflection::Vertical(column) => *column as u32,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Tile {
    Ash,
    Rock,
}

impl TryFrom<char> for Tile {
    type Error = Box<dyn Error>;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Tile::Ash),
            '#' => Ok(Tile::Rock),
            _ => Err("Unrecognized tile".into()),
        }
    }
}

impl Not for Tile {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Tile::Ash => Tile::Rock,
            Tile::Rock => Tile::Ash,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_reflection() {
        assert_eq!(
            Some(Reflection::Vertical(5)),
            MirrorField::from_str(indoc! {"
                #.##..##.
                ..#.##.#.
                ##......#
                ##......#
                ..#.##.#.
                ..##..##.
                #.#.##.#.
            "})
            .unwrap()
            .reflection()
        );

        assert_eq!(
            Some(Reflection::Horizontal(4)),
            MirrorField::from_str(indoc! {"
                #...##..#
                #....#..#
                ..##..###
                #####.##.
                #####.##.
                ..##..###
                #....#..#
            "})
            .unwrap()
            .reflection()
        );
    }

    #[test]
    fn test_smudged_reflection() {
        assert_eq!(
            Some(Reflection::Horizontal(3)),
            MirrorField::from_str(indoc! {"
                #.##..##.
                ..#.##.#.
                ##......#
                ##......#
                ..#.##.#.
                ..##..##.
                #.#.##.#.
            "})
                .unwrap()
                .smudged_reflection()
        );

        assert_eq!(
            Some(Reflection::Horizontal(1)),
            MirrorField::from_str(indoc! {"
                #...##..#
                #....#..#
                ..##..###
                #####.##.
                #####.##.
                ..##..###
                #....#..#
            "})
                .unwrap()
                .smudged_reflection()
        );
    }

    #[test]
    fn test_reflection_score() {
        assert_eq!(400, Reflection::Horizontal(4).score());
        assert_eq!(5, Reflection::Vertical(5).score());
    }
}
