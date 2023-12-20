use nom::{
    branch::alt,
    bytes::complete::{
        is_not,
        tag,
    },
    character::complete::{
        alpha1,
        char,
        line_ending,
        space1,
    },
    combinator::{
        all_consuming,
        map,
        value,
    },
    multi::{
        many1,
        separated_list1,
    },
    sequence::{
        separated_pair,
        terminated,
        tuple,
    },
    Finish,
    IResult,
};
use std::{
    cell::RefCell,
    collections::HashMap,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Pulse {
    High,
    Low,
}

#[derive(Clone, Copy, Debug)]
enum FlipFlopState {
    On,
    Off,
}

impl FlipFlopState {
    fn toggle(&self) -> Self {
        match self {
            FlipFlopState::On => FlipFlopState::Off,
            FlipFlopState::Off => FlipFlopState::On,
        }
    }
}

#[derive(Debug)]
struct BaseComponent<'a> {
    parents: Vec<&'a str>,
    children: Vec<&'a str>,
}

impl<'a> BaseComponent<'a> {
    fn new(
        parents: Vec<&'a str>,
        children: Vec<&'a str>,
    ) -> Self {
        BaseComponent {
            parents,
            children,
        }
    }

    fn propagate_to_children(
        &self,
        pulse: Pulse,
    ) -> Vec<(&'a str, Pulse)> {
        self.children.iter().map(|label| (*label, pulse)).collect()
    }
}

#[derive(Debug)]
enum Component<'a> {
    Broadcaster(BaseComponent<'a>),
    FlipFlop {
        base: BaseComponent<'a>,
        state: RefCell<FlipFlopState>,
    },
    Conjunction {
        base: BaseComponent<'a>,
        states: RefCell<HashMap<&'a str, Pulse>>,
    },
    Output(BaseComponent<'a>),
}

impl<'a> Component<'a> {
    fn output(
        parents: Vec<&'a str>,
    ) -> Self {
        Component::Output(BaseComponent::new(parents, vec![]))
    }

    fn broadcaster(
        children: Vec<&'a str>,
    ) -> Self {
        Component::Broadcaster(BaseComponent::new(vec![], children))
    }

    fn flip_flop(
        parents: Vec<&'a str>,
        children: Vec<&'a str>,
    ) -> Self {
        Component::FlipFlop {
            base: BaseComponent::new(parents, children),
            state: RefCell::new(FlipFlopState::Off),
        }
    }

    fn conjunction(
        parents: Vec<&'a str>,
        children: Vec<&'a str>,
    ) -> Self {
        let initial_state = parents
            .iter()
            .map(|&parent_label| (parent_label, Pulse::Low))
            .collect();

        Component::Conjunction {
            base: BaseComponent::new(parents, children),
            states: RefCell::new(initial_state),
        }
    }

    fn base(&self) -> &BaseComponent {
        match self {
            Component::Broadcaster(base) => base,
            Component::FlipFlop { base, .. } => base,
            Component::Conjunction { base, .. } => base,
            Component::Output(base) => base,
        }
    }

    fn receive(
        &mut self,
        from: &'a str,
        pulse: Pulse,
    ) -> Vec<(&'a str, Pulse)> {
        match self {
            Component::Broadcaster(base) => base.propagate_to_children(pulse),
            Component::FlipFlop { base, state, .. } => {
                match pulse {
                    // If a flip-flop module receives a high pulse, it is ignored and nothing happens.
                    Pulse::High => vec![],

                    // However, if a flip-flop module receives a low pulse, it flips between on and off.
                    Pulse::Low => {
                        base.propagate_to_children(
                            match state.replace_with(|state| state.toggle()) {
                                // If it was on, it turns off and sends a low pulse.
                                FlipFlopState::On => Pulse::Low,
                                // If it was off, it turns on and sends a high pulse.
                                FlipFlopState::Off => Pulse::High,
                            },
                        )
                    }
                }
            }

            Component::Conjunction { base, states } => {
                // When a pulse is received, the conjunction module first updates its memory for that input.
                states.borrow_mut().insert(from, pulse);

                // Then, if it remembers high pulses for all inputs, it sends a low pulse; otherwise, it sends a high pulse.
                base.propagate_to_children(
                    if states.borrow().values().all(|pulse| pulse == &Pulse::High) {
                        Pulse::Low
                    } else {
                        Pulse::High
                    },
                )
            }
            Component::Output(_) => vec![],
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ComponentType {
    Broadcaster,
    FlipFlop,
    Conjunction,
}

#[derive(Debug)]
struct RawComponent<'a> {
    component_type: ComponentType,
    label: &'a str,
    children: Vec<&'a str>,
}

impl<'a> RawComponent<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let component_type_and_label = alt((
            map(tag("broadcaster"), |x| (ComponentType::Broadcaster, x)),
            tuple((
                alt((
                    value(ComponentType::FlipFlop, char('%')),
                    value(ComponentType::Conjunction, char('&')),
                )),
                alpha1,
            )),
        ));

        map(
            separated_pair(
                component_type_and_label,
                tuple((space1, tag("->"), space1)),
                separated_list1(tuple((char(','), space1)), is_not(",\n")),
            ),
            |((component_type, label), children)| {
                RawComponent {
                    component_type,
                    label,
                    children,
                }
            },
        )(input)
    }
}

fn parse(input: &str) -> IResult<&str, HashMap<&str, Component>> {
    all_consuming(map(
        many1(terminated(RawComponent::parse, line_ending)),
        |raw_components| {
            // Easy access to components
            let label_to_children: HashMap<_, _> = raw_components
                .iter()
                .map(|raw_component| (raw_component.label, &raw_component.children))
                .collect();

            let label_to_parents = raw_components
                .iter()
                .flat_map(|raw_component| {
                    raw_component
                        .children
                        .iter()
                        .map(|child_label| (*child_label, raw_component.label))
                })
                .fold(HashMap::new(), |mut map, (child_label, parent_label)| {
                    map.entry(child_label)
                        .and_modify(|parent_labels: &mut Vec<&str>| {
                            parent_labels.push(parent_label)
                        })
                        .or_insert_with(|| vec![parent_label]);
                    map
                });

            // Build actual components
            let mut components = HashMap::<&str, Component>::new();

            // Find the component without children
            raw_components
                .iter()
                .filter_map(|raw_component| {
                    raw_component
                        .children
                        .iter()
                        .find(|label| !label_to_children.contains_key(*label))
                })
                .for_each(|output_label| {
                    components.insert(
                        output_label,
                        Component::output(
                            label_to_parents
                                .get(output_label)
                                .cloned()
                                .unwrap_or_default(),
                        ),
                    );
                });

            for RawComponent {
                component_type,
                label,
                children,
            } in &raw_components
            {
                let parents = label_to_parents
                    .get(label)
                    .cloned()
                    .unwrap_or_default();

                match component_type {
                    ComponentType::Broadcaster => {
                        components.insert(label, Component::broadcaster( children.clone()))
                    }
                    ComponentType::FlipFlop => {
                        components.insert(label, Component::flip_flop(parents, children.clone()))
                    }
                    ComponentType::Conjunction => {
                        components.insert(label, Component::conjunction(parents, children.clone()))
                    }
                };
            }

            components
        },
    ))(input)
}

fn push_button(
    components: &mut HashMap<&str, Component>,
    mut on_pulse: impl FnMut(&str, &str, Pulse),
) {
    let mut children_pulses = vec![("button", vec![("broadcaster", Pulse::Low)])];

    while !children_pulses.is_empty() {
        let mut next_children_pulses = Vec::new();

        for (parent_label, pulses) in children_pulses {
            for (child_label, pulse) in pulses {
                on_pulse(parent_label, child_label, pulse);

                next_children_pulses.push((
                    child_label,
                    components
                        .get_mut(child_label)
                        .map(|child| child.receive(parent_label, pulse))
                        .unwrap_or_default(),
                ))
            }
        }

        children_pulses = next_children_pulses;
    }
}

fn first(
    name: &str,
    data: &str,
) {
    let (_, mut components) = parse(data).finish().unwrap();

    let mut high = 0_u32;
    let mut low = 0_u32;
    for _ in 0..1000 {
        push_button(&mut components, |_, _, pulse| {
            match pulse {
                Pulse::High => high += 1,
                Pulse::Low => low += 1,
            }
        });
    }

    println!("[{}] H{} | L{} | P{}", name, high, low, high * low);
}

fn second(
    name: &str,
    data: &str,
) {
    let (_, mut components) = parse(data).finish().unwrap();

    // The input has rx as the output, that has a single conjunction parent
    // This is unsafe
    let rx_parent = String::from(components["rx"].base().parents[0]);
    let rx_parent_parents_size = components[rx_parent.as_str()].base().parents.len();

    // Now we want to get all the parents to send a high pulse and see when this happens
    let mut high_pulse_at: HashMap<String, usize> = HashMap::new();
    let mut index = 0_usize;
    loop {
        index += 1;

        push_button(&mut components, |parent_label, child_label, pulse| {
            match pulse {
                Pulse::High => {
                    if child_label == rx_parent {
                        let parent_label = String::from(parent_label);
                        high_pulse_at.entry(parent_label).or_insert(index);
                    }
                }
                Pulse::Low => {}
            }
        });

        if rx_parent_parents_size == high_pulse_at.len() {
            break;
        }
    }

    // The iteration as which rx receives a low pulse is the LCM of the indexes
    let lcm = high_pulse_at
        .values()
        .fold(1, |lcm, index| num::integer::lcm(lcm, *index));
    println!("[{}] Low pulse at {:?}", name, lcm);
}

pub fn run() {
    first("First example 1", include_str!("data/day20/ex1")); // H4000 | L8000 | P32000000
    first("First example 2", include_str!("data/day20/ex2")); // H2750 | L4250 | P11687500
    first("First", include_str!("data/day20/input")); // H48760 | L18124 | P883726240
    second("Second", include_str!("data/day20/input")); // 211 712 400 442 661
}
