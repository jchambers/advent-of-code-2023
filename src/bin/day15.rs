use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let steps: Vec<String> = {
            let mut steps_string = String::new();
            File::open(path)?.read_to_string(&mut steps_string)?;

            steps_string
                .split(',')
                .map(|step| String::from(step.trim()))
                .collect()
        };

        println!(
            "Sum of hash values: {}",
            steps.iter().map(|step| hash(step) as u32).sum::<u32>()
        );

        Ok(())
    } else {
        Err("Usage: day15 INPUT_FILE_PATH".into())
    }
}

fn hash(str: &str) -> u8 {
    str.bytes()
        .fold(0, |acc, b| (((acc as u32 + b as u32) * 17) % 256) as u8)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hash() {
        assert_eq!(52, hash("HASH"));
        assert_eq!(30, hash("rn=1"));
        assert_eq!(253, hash("cm-"));
        assert_eq!(97, hash("qp=3"));
        assert_eq!(47, hash("cm=2"));
        assert_eq!(14, hash("qp-"));
        assert_eq!(180, hash("pc=4"));
        assert_eq!(9, hash("ot=9"));
        assert_eq!(197, hash("ab=5"));
        assert_eq!(48, hash("pc-"));
        assert_eq!(214, hash("pc=6"));
        assert_eq!(231, hash("ot=7"));
    }
}
