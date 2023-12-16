use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::{env, iter};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let instructions: Vec<String> = {
            let mut instructions_string = String::new();
            File::open(path)?.read_to_string(&mut instructions_string)?;

            instructions_string
                .split(',')
                .map(|step| String::from(step.trim()))
                .collect()
        };

        println!(
            "Sum of hash values: {}",
            instructions
                .iter()
                .map(|step| LightBoxHashMap::hash(step) as u32)
                .sum::<u32>()
        );

        let mut hash_map = LightBoxHashMap::default();

        instructions
            .iter()
            .try_for_each(|instruction| hash_map.apply_instruction(instruction))?;

        println!("Focusing power: {}", hash_map.focusing_power());

        Ok(())
    } else {
        Err("Usage: day15 INPUT_FILE_PATH".into())
    }
}

struct LightBoxHashMap {
    boxes: [Vec<Lens>; 256],
}

impl LightBoxHashMap {
    fn apply_instruction(&mut self, instruction: &str) -> Result<(), Box<dyn Error>> {
        if let Some(label) = instruction.strip_suffix('-') {
            let hash = Self::hash(label);

            if let Some(position) = self.boxes[hash].iter().position(|lens| lens.label == label) {
                self.boxes[hash].remove(position);
            }

            Ok(())
        } else if let [label, focal_length] =
            instruction.split('=').collect::<Vec<&str>>().as_slice()
        {
            let hash = Self::hash(label);

            if let Some(position) = self.boxes[hash]
                .iter()
                .position(|lens| lens.label.as_str() == *label)
            {
                self.boxes[hash][position].focal_length = focal_length.parse()?;
            } else {
                self.boxes[hash].push(Lens {
                    label: String::from(*label),
                    focal_length: focal_length.parse()?,
                });
            }

            Ok(())
        } else {
            Err("Unrecognized instruction".into())
        }
    }

    fn hash(str: &str) -> usize {
        str.bytes()
            .fold(0, |acc, b| ((acc + b as usize) * 17) % 256)
    }

    fn focusing_power(&self) -> u32 {
        self.boxes
            .iter()
            .enumerate()
            .flat_map(|(b, lenses)| {
                lenses
                    .iter()
                    .enumerate()
                    .map(move |(l, lens)| (b as u32 + 1) * (l as u32 + 1) * lens.focal_length)
            })
            .sum()
    }
}

impl Default for LightBoxHashMap {
    fn default() -> Self {
        LightBoxHashMap {
            boxes: iter::repeat_with(Vec::new)
                .take(256)
                .collect::<Vec<Vec<Lens>>>()
                .try_into()
                .unwrap(),
        }
    }
}

#[derive(Debug)]
struct Lens {
    label: String,
    focal_length: u32,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hash() {
        assert_eq!(52, LightBoxHashMap::hash("HASH"));
        assert_eq!(30, LightBoxHashMap::hash("rn=1"));
        assert_eq!(253, LightBoxHashMap::hash("cm-"));
        assert_eq!(97, LightBoxHashMap::hash("qp=3"));
        assert_eq!(47, LightBoxHashMap::hash("cm=2"));
        assert_eq!(14, LightBoxHashMap::hash("qp-"));
        assert_eq!(180, LightBoxHashMap::hash("pc=4"));
        assert_eq!(9, LightBoxHashMap::hash("ot=9"));
        assert_eq!(197, LightBoxHashMap::hash("ab=5"));
        assert_eq!(48, LightBoxHashMap::hash("pc-"));
        assert_eq!(214, LightBoxHashMap::hash("pc=6"));
        assert_eq!(231, LightBoxHashMap::hash("ot=7"));
    }

    #[test]
    fn test_focusing_power() {
        let mut hash_map = LightBoxHashMap::default();

        "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7"
            .split(',')
            .try_for_each(|instruction| hash_map.apply_instruction(instruction))
            .unwrap();

        assert_eq!(145, hash_map.focusing_power());
    }
}
