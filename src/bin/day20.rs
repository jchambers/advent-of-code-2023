use std::collections::{HashMap, VecDeque};
use std::env;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let mut pulse_machine = {
            let mut pulse_machine_string = String::new();
            File::open(path)?.read_to_string(&mut pulse_machine_string)?;

            PulseMachine::from_str(pulse_machine_string.as_str())?
        };

        {
            let (low, high) = pulse_machine.pulses(1000);

            println!(
                "Pulse product after 1000 iterations: {} * {} = {}",
                low,
                high,
                low as u64 * high as u64
            );
        }

        Ok(())
    } else {
        Err("Usage: day20 INPUT_FILE_PATH".into())
    }
}

struct PulseMachine {
    modules: HashMap<String, Box<dyn Module>>,
}

impl PulseMachine {
    fn pulses(&mut self, button_presses: u32) -> (u32, u32) {
        let mut previous_states: Vec<String> = Vec::new();
        let mut state_cache: HashMap<String, (u32, u32)> = HashMap::new();

        let mut low_pulses = 0;
        let mut high_pulses = 0;

        for _ in 0..button_presses {
            let state = self.state();

            let pulses = self.handle_button_press();

            if let Some(loop_start_index) = &previous_states.iter().position(|s| s == &state) {
                let loop_len = previous_states.len() - loop_start_index;

                let leading_pulses = &previous_states[0..*loop_start_index]
                    .iter()
                    .map(|s| state_cache.get(s).unwrap())
                    .copied()
                    .reduce(|a, b| (a.0 + b.0, a.1 + b.1))
                    .unwrap_or((0, 0));

                let loops = (button_presses - *loop_start_index as u32) / loop_len as u32;

                let loop_pulses = &previous_states[*loop_start_index..]
                    .iter()
                    .map(|s| state_cache.get(s).unwrap())
                    .copied()
                    .reduce(|a, b| (a.0 + b.0, a.1 + b.1))
                    .map(|(low, high)| (low * loops, high * loops))
                    .unwrap_or((0, 0));

                let trailing_states =
                    button_presses - *loop_start_index as u32 - (loop_len as u32 * loops);

                let trailing_pulses = &previous_states
                    [*loop_start_index..*loop_start_index + trailing_states as usize]
                    .iter()
                    .map(|s| state_cache.get(s).unwrap())
                    .copied()
                    .reduce(|a, b| (a.0 + b.0, a.1 + b.1))
                    .unwrap_or((0, 0));

                return (
                    leading_pulses.0 + loop_pulses.0 + trailing_pulses.0,
                    leading_pulses.1 + loop_pulses.1 + trailing_pulses.1,
                );
            } else {
                previous_states.push(state.clone());
                state_cache.insert(state, pulses);
            }

            low_pulses += pulses.0;
            high_pulses += pulses.1;
        }

        (low_pulses, high_pulses)
    }

    fn handle_button_press(&mut self) -> (u32, u32) {
        let mut pulse_queue: VecDeque<(String, String, Pulse)> = VecDeque::new();
        pulse_queue.push_back((
            String::from("button"),
            String::from(Broadcaster::BROADCASTER_ID),
            Pulse::Low,
        ));

        let mut low_pulses = 0;
        let mut high_pulses = 0;

        while let Some((source, destination, pulse)) = pulse_queue.pop_front() {
            match pulse {
                Pulse::Low => low_pulses += 1,
                Pulse::High => high_pulses += 1,
            }

            // Not all outputs reference a module; some are just sinks
            if let Some(destination) = self.modules.get_mut(&destination) {
                Self::enqueue_pulses(&mut **destination, (pulse, &source), &mut pulse_queue);
            }
        }

        (low_pulses, high_pulses)
    }

    fn enqueue_pulses(
        module: &mut (impl Module + ?Sized),
        input_pulse: (Pulse, &str),
        pulse_queue: &mut VecDeque<(String, String, Pulse)>,
    ) {
        module
            .handle_pulse(input_pulse.0, input_pulse.1)
            .iter()
            .for_each(|(destination, pulse)| {
                pulse_queue.push_back((String::from(module.id()), destination.clone(), *pulse));
            });
    }

    fn state(&self) -> String {
        let mut sorted_states: Vec<String> = self
            .modules
            .values()
            .filter(|module| module.id() != Broadcaster::BROADCASTER_ID)
            .map(|module| format!("{}:{}", module.id(), module.state()))
            .collect();

        sorted_states.sort();

        sorted_states.join(";")
    }
}

impl FromStr for PulseMachine {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut modules: HashMap<String, Box<dyn Module>> = HashMap::new();
        let mut conjunctions: Vec<Conjunction> = Vec::new();

        for line in string.lines() {
            if line.starts_with(Broadcaster::BROADCASTER_ID) {
                let broadcaster = Broadcaster::from_str(line)?;
                modules.insert(String::from(broadcaster.id()), Box::new(broadcaster));
            } else if line.starts_with('%') {
                let flip_flop = FlipFlop::from_str(line)?;
                modules.insert(String::from(flip_flop.id()), Box::new(flip_flop));
            } else if line.starts_with('&') {
                let conjunction = Conjunction::from_str(line)?;
                conjunctions.push(conjunction);
            } else {
                return Err("Could not parse line".into());
            }
        }

        for conjunction in conjunctions.iter_mut() {
            let inputs: Vec<String> = modules
                .values()
                .filter(|module| {
                    module
                        .destinations()
                        .contains(&String::from(conjunction.id()))
                })
                .map(|module| String::from(module.id()))
                .collect();

            inputs.iter().for_each(|input| conjunction.add_input(input));
        }

        conjunctions.into_iter().for_each(|conjunction| {
            modules.insert(String::from(conjunction.id()), Box::new(conjunction));
        });

        Ok(PulseMachine { modules })
    }
}

trait Module {
    fn id(&self) -> &str;
    fn destinations(&self) -> &[String];
    fn handle_pulse(&mut self, pulse: Pulse, source: &str) -> Vec<(String, Pulse)>;
    fn state(&self) -> String;
}

struct FlipFlop {
    id: String,
    destinations: Vec<String>,
    on: bool,
}

impl FlipFlop {
    fn new(id: &str, destinations: Vec<String>) -> Self {
        Self {
            id: String::from(id),
            destinations,
            on: false,
        }
    }
}

impl Module for FlipFlop {
    fn id(&self) -> &str {
        &self.id
    }

    fn destinations(&self) -> &[String] {
        &self.destinations
    }

    fn handle_pulse(&mut self, pulse: Pulse, _: &str) -> Vec<(String, Pulse)> {
        let mut pulses = Vec::with_capacity(1);

        if pulse == Pulse::Low {
            self.on = !self.on;

            let outbound_pulse = if self.on { Pulse::High } else { Pulse::Low };

            pulses.extend(
                self.destinations
                    .iter()
                    .map(|destination| (destination.clone(), outbound_pulse)),
            );
        }

        pulses
    }

    fn state(&self) -> String {
        if self.on {
            String::from("on")
        } else {
            String::from("off")
        }
    }
}

impl FromStr for FlipFlop {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(definition) = string.strip_prefix('%') {
            if let [id, destinations] = definition.split(" -> ").collect::<Vec<&str>>().as_slice() {
                let destinations: Vec<String> =
                    destinations.split(", ").map(String::from).collect();

                Ok(FlipFlop::new(id, destinations))
            } else {
                Err("Could not parse flip-flip definition".into())
            }
        } else {
            Err("String does not begin with a '%'".into())
        }
    }
}

struct Conjunction {
    id: String,
    destinations: Vec<String>,
    inputs: HashMap<String, Pulse>,
}

impl Conjunction {
    fn new(id: &str, destinations: Vec<String>) -> Self {
        Self {
            id: String::from(id),
            destinations,
            inputs: HashMap::new(),
        }
    }

    fn add_input(&mut self, input_id: &str) {
        self.inputs.insert(String::from(input_id), Pulse::Low);
    }
}

impl Module for Conjunction {
    fn id(&self) -> &str {
        &self.id
    }

    fn destinations(&self) -> &[String] {
        &self.destinations
    }

    fn handle_pulse(&mut self, pulse: Pulse, source: &str) -> Vec<(String, Pulse)> {
        self.inputs.insert(String::from(source), pulse);

        let outbound_pulse = if self.inputs.values().all(|pulse| pulse == &Pulse::High) {
            Pulse::Low
        } else {
            Pulse::High
        };

        self.destinations
            .iter()
            .map(|destination| (destination.clone(), outbound_pulse))
            .collect()
    }

    fn state(&self) -> String {
        let mut sorted_inputs: Vec<String> = self
            .inputs
            .iter()
            .map(|(id, pulse)| format!("{}={}", id, pulse))
            .collect();

        sorted_inputs.sort();

        sorted_inputs.join(",")
    }
}

impl FromStr for Conjunction {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(definition) = string.strip_prefix('&') {
            if let [id, destinations] = definition.split(" -> ").collect::<Vec<&str>>().as_slice() {
                let destinations: Vec<String> =
                    destinations.split(", ").map(String::from).collect();

                Ok(Conjunction::new(id, destinations))
            } else {
                Err("Could not parse conjunction definition".into())
            }
        } else {
            Err("String does not begin with a '&'".into())
        }
    }
}

struct Broadcaster {
    destinations: Vec<String>,
}

impl Broadcaster {
    const BROADCASTER_ID: &'static str = "broadcaster";
}

impl Module for Broadcaster {
    fn id(&self) -> &str {
        Broadcaster::BROADCASTER_ID
    }

    fn destinations(&self) -> &[String] {
        &self.destinations
    }

    fn handle_pulse(&mut self, pulse: Pulse, _: &str) -> Vec<(String, Pulse)> {
        self.destinations
            .iter()
            .map(|destination| (destination.clone(), pulse))
            .collect()
    }

    fn state(&self) -> String {
        String::new()
    }
}

impl FromStr for Broadcaster {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(destinations) = string.strip_prefix("broadcaster -> ") {
            Ok(Broadcaster {
                destinations: destinations.split(", ").map(String::from).collect(),
            })
        } else {
            Err("Could not parse broadcaster string".into())
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Pulse {
    Low,
    High,
}

impl Display for Pulse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Pulse::Low => "low",
                Pulse::High => "high",
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_pulses() {
        assert_eq!(
            (8000, 4000),
            PulseMachine::from_str(indoc! {"
            broadcaster -> a, b, c
            %a -> b
            %b -> c
            %c -> inv
            &inv -> a
        "})
            .unwrap()
            .pulses(1000)
        );

        assert_eq!(
            (4250, 2750),
            PulseMachine::from_str(indoc! {"
            broadcaster -> a
            %a -> inv, con
            &inv -> b
            %b -> con
            &con -> output
        "})
            .unwrap()
            .pulses(1000)
        );
    }
}
