use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let part_sorter = {
            let mut part_sorter_string = String::new();
            File::open(path)?.read_to_string(&mut part_sorter_string)?;

            PartSorter::from_str(part_sorter_string.as_str())?
        };

        println!(
            "Sum of ratings of accepted parts: {}",
            part_sorter.accepted_part_rating_sum()
        );

        println!(
            "Distinct accepted part configurations: {}",
            part_sorter.possible_accepted_parts()
        );

        Ok(())
    } else {
        Err("Usage: day19 INPUT_FILE_PATH".into())
    }
}

struct PartSorter {
    workflows: HashMap<String, Workflow>,
    parts: Vec<Part>,
}

impl PartSorter {
    const INITIAL_RULE: &'static str = "in";

    fn accepted_part_rating_sum(&self) -> u64 {
        self.accepted_parts()
            .iter()
            .map(|part| part.rating() as u64)
            .sum()
    }

    fn accepted_parts(&self) -> Vec<Part> {
        self.parts
            .iter()
            .filter(|part| self.process_part(part, Self::INITIAL_RULE))
            .copied()
            .collect()
    }

    fn process_part(&self, part: &Part, workflow_id: &str) -> bool {
        let workflow = self
            .workflows
            .get(workflow_id)
            .expect("Referenced workflow must exist");

        match workflow.apply(part) {
            Action::Transfer(next_workflow_id) => {
                self.process_part(part, next_workflow_id.as_str())
            }
            Action::Accept => true,
            Action::Reject => false,
        }
    }

    fn possible_accepted_parts(&self) -> u64 {
        let mut stack = vec![(
            Action::Transfer(String::from(Self::INITIAL_RULE)),
            PartSpace::default(),
        )];
        let mut accepted_parts = 0;

        while let Some((action, space)) = stack.pop() {
            match action {
                Action::Transfer(workflow_id) => {
                    let workflow = self
                        .workflows
                        .get(&workflow_id)
                        .expect("Referenced workflow must exist");
                    let mut remainder = space;

                    for rule in &workflow.rules {
                        match rule.condition {
                            Condition::LessThan(component, value) => {
                                let (selected, r) = remainder.partition_less_than(component, value);
                                stack.push((rule.action.clone(), selected));

                                remainder = r;
                            }
                            Condition::GreaterThan(component, value) => {
                                let (selected, r) =
                                    remainder.partition_greater_than(component, value);
                                stack.push((rule.action.clone(), selected));

                                remainder = r;
                            }
                            Condition::MatchAll => {
                                // This should be the last entry in the list of rules; no need to worry about the
                                // remainder (but WE could set it to "empty" if we really wanted to).
                                stack.push((rule.action.clone(), remainder));
                            }
                        }
                    }
                }
                Action::Accept => accepted_parts += space.volume(),
                Action::Reject => {}
            }
        }

        accepted_parts
    }
}

impl FromStr for PartSorter {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let [workflows, parts] = string.split("\n\n").collect::<Vec<&str>>().as_slice() {
            let workflows: Vec<Workflow> = workflows
                .lines()
                .map(Workflow::from_str)
                .collect::<Result<_, _>>()?;

            let workflows: HashMap<String, Workflow> = workflows
                .into_iter()
                .map(|workflow| (workflow.id.clone(), workflow))
                .collect();

            let parts: Vec<Part> = parts
                .lines()
                .map(Part::from_str)
                .collect::<Result<_, _>>()?;

            Ok(PartSorter { workflows, parts })
        } else {
            Err("Could not separate workflows and parts".into())
        }
    }
}

struct Workflow {
    id: String,
    rules: Vec<Rule>,
}

impl Workflow {
    fn apply(&self, part: &Part) -> Action {
        self.rules
            .iter()
            .find(|rule| rule.matches(part))
            .expect("Workflows must have at least one catch-all rule")
            .action
            .clone()
    }
}

impl FromStr for Workflow {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let [id, rules] = string.split('{').collect::<Vec<&str>>().as_slice() {
            if let Some(rules) = rules.strip_suffix('}') {
                let rules: Vec<Rule> = rules
                    .split(',')
                    .map(Rule::from_str)
                    .collect::<Result<_, _>>()?;

                Ok(Workflow {
                    id: String::from(*id),
                    rules,
                })
            } else {
                Err("No trailing bracket on rules".into())
            }
        } else {
            Err("Could not parse workflow string".into())
        }
    }
}

struct Rule {
    condition: Condition,
    action: Action,
}

impl Rule {
    fn matches(&self, part: &Part) -> bool {
        match &self.condition {
            Condition::LessThan(component, value) => part[*component] < *value,
            Condition::GreaterThan(component, value) => part[*component] > *value,
            Condition::MatchAll => true,
        }
    }
}

impl FromStr for Rule {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let [condition, action] = string.split(':').collect::<Vec<&str>>().as_slice() {
            let condition = if let [component, value] =
                condition.split('<').collect::<Vec<&str>>().as_slice()
            {
                Condition::LessThan(Component::from_str(component)?, value.parse()?)
            } else if let [component, value] =
                condition.split('>').collect::<Vec<&str>>().as_slice()
            {
                Condition::GreaterThan(Component::from_str(component)?, value.parse()?)
            } else {
                return Err("Could not parse rule string".into());
            };

            Ok(Rule {
                condition,
                action: Action::from_str(action)?,
            })
        } else {
            Ok(Rule {
                condition: Condition::MatchAll,
                action: Action::from_str(string)?,
            })
        }
    }
}

#[derive(Eq, PartialEq)]
enum Condition {
    LessThan(Component, u32),
    GreaterThan(Component, u32),
    MatchAll,
}

#[derive(Clone)]
enum Action {
    Transfer(String),
    Accept,
    Reject,
}

impl FromStr for Action {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(match string {
            "A" => Action::Accept,
            "R" => Action::Reject,
            _ => Action::Transfer(String::from(string)),
        })
    }
}

#[derive(Copy, Clone, Default)]
struct Part {
    x: u32,
    m: u32,
    a: u32,
    s: u32,
}

impl Part {
    fn rating(&self) -> u32 {
        self.x + self.m + self.a + self.s
    }
}

impl FromStr for Part {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(components) = string.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
            let mut part = Part::default();

            for component in components.split(',') {
                if let [component, value] = component.split('=').collect::<Vec<&str>>().as_slice() {
                    let component = Component::from_str(component)?;
                    let value = value.parse()?;

                    part[component] = value;
                }
            }

            Ok(part)
        } else {
            Err("Could not parse rule string".into())
        }
    }
}

impl Index<Component> for Part {
    type Output = u32;

    fn index(&self, component: Component) -> &Self::Output {
        match component {
            Component::X => &self.x,
            Component::M => &self.m,
            Component::A => &self.a,
            Component::S => &self.s,
        }
    }
}

impl IndexMut<Component> for Part {
    fn index_mut(&mut self, component: Component) -> &mut Self::Output {
        match component {
            Component::X => &mut self.x,
            Component::M => &mut self.m,
            Component::A => &mut self.a,
            Component::S => &mut self.s,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Component {
    X,
    M,
    A,
    S,
}

impl FromStr for Component {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "x" => Ok(Component::X),
            "m" => Ok(Component::M),
            "a" => Ok(Component::A),
            "s" => Ok(Component::S),
            _ => Err("Unrecognized component".into()),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct PartSpace {
    x_range: (u32, u32),
    m_range: (u32, u32),
    a_range: (u32, u32),
    s_range: (u32, u32),
}

impl PartSpace {
    fn volume(&self) -> u64 {
        [self.x_range, self.m_range, self.a_range, self.s_range]
            .iter()
            .map(|(start, end)| ((end - start) + 1) as u64)
            .product()
    }

    fn partition_less_than(&self, component: Component, value: u32) -> (Self, Self) {
        let mut selected = *self;
        let mut remainder = *self;

        let original_range = self[component];

        selected[component] = (original_range.0, value - 1);
        remainder[component] = (value, original_range.1);

        (selected, remainder)
    }

    fn partition_greater_than(&self, component: Component, value: u32) -> (Self, Self) {
        let mut selected = *self;
        let mut remainder = *self;

        let original_range = self[component];

        selected[component] = (value + 1, original_range.1);
        remainder[component] = (original_range.0, value);

        (selected, remainder)
    }
}

impl Default for PartSpace {
    fn default() -> Self {
        PartSpace {
            x_range: (1, 4000),
            m_range: (1, 4000),
            a_range: (1, 4000),
            s_range: (1, 4000),
        }
    }
}

impl Index<Component> for PartSpace {
    type Output = (u32, u32);

    fn index(&self, component: Component) -> &Self::Output {
        match component {
            Component::X => &self.x_range,
            Component::M => &self.m_range,
            Component::A => &self.a_range,
            Component::S => &self.s_range,
        }
    }
}

impl IndexMut<Component> for PartSpace {
    fn index_mut(&mut self, component: Component) -> &mut Self::Output {
        match component {
            Component::X => &mut self.x_range,
            Component::M => &mut self.m_range,
            Component::A => &mut self.a_range,
            Component::S => &mut self.s_range,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const TEST_SORTER_STRING: &str = indoc! {"
        px{a<2006:qkq,m>2090:A,rfg}
        pv{a>1716:R,A}
        lnx{m>1548:A,A}
        rfg{s<537:gd,x>2440:R,A}
        qs{s>3448:A,lnx}
        qkq{x<1416:A,crn}
        crn{x>2662:A,R}
        in{s<1351:px,qqz}
        qqz{s>2770:qs,m<1801:hdj,R}
        gd{a>3333:R,R}
        hdj{m>838:A,pv}

        {x=787,m=2655,a=1222,s=2876}
        {x=1679,m=44,a=2067,s=496}
        {x=2036,m=264,a=79,s=2244}
        {x=2461,m=1339,a=466,s=291}
        {x=2127,m=1623,a=2188,s=1013}
    "};

    #[test]
    fn test_accepted_part_rating_sum() {
        let part_sorter = PartSorter::from_str(TEST_SORTER_STRING).unwrap();

        assert_eq!(19114, part_sorter.accepted_part_rating_sum());
    }

    #[test]
    fn test_possible_accepted_parts() {
        let part_sorter = PartSorter::from_str(TEST_SORTER_STRING).unwrap();

        assert_eq!(167_409_079_868_000, part_sorter.possible_accepted_parts());
    }

    #[test]
    fn test_part_space_volume() {
        assert_eq!(4000 * 4000 * 4000 * 4000, PartSpace::default().volume());
    }

    #[test]
    fn test_part_space_partition_less_than() {
        let expected_selected = PartSpace {
            x_range: (1, 4000),
            m_range: (1, 4000),
            a_range: (1, 999),
            s_range: (1, 4000),
        };

        let expected_remainder = PartSpace {
            x_range: (1, 4000),
            m_range: (1, 4000),
            a_range: (1000, 4000),
            s_range: (1, 4000),
        };

        assert_eq!(
            (expected_selected, expected_remainder),
            PartSpace::default().partition_less_than(Component::A, 1000)
        );

        assert_eq!(
            PartSpace::default().volume(),
            expected_selected.volume() + expected_remainder.volume()
        );
    }

    #[test]
    fn test_part_space_partition_greater_than() {
        let expected_selected = PartSpace {
            x_range: (1, 4000),
            m_range: (1001, 4000),
            a_range: (1, 4000),
            s_range: (1, 4000),
        };

        let expected_remainder = PartSpace {
            x_range: (1, 4000),
            m_range: (1, 1000),
            a_range: (1, 4000),
            s_range: (1, 4000),
        };

        assert_eq!(
            (expected_selected, expected_remainder),
            PartSpace::default().partition_greater_than(Component::M, 1000)
        );

        assert_eq!(
            PartSpace::default().volume(),
            expected_selected.volume() + expected_remainder.volume()
        );
    }
}
