use nom::{
    bytes::complete::tag,
    character,
    character::complete::{
        line_ending,
        space1,
    },
    combinator::{
        all_consuming,
        map,
        opt,
    },
    multi::{
        many0,
        separated_list0,
    },
    sequence::{
        delimited,
        terminated,
        tuple,
    },
    Finish,
    IResult,
};
use rayon::prelude::*;
use std::ops::Range;

#[derive(Debug)]
struct ConversionRange {
    destination_range_start: u64,
    source_range_start: u64,
    range_length: u64,
}

impl ConversionRange {
    fn source_range_end(&self) -> u64 {
        self.source_range_start + self.range_length
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                character::complete::u64,
                space1,
                character::complete::u64,
                space1,
                character::complete::u64,
            )),
            |(destination_range_start, _, source_range_start, _, range_length)| {
                ConversionRange {
                    destination_range_start,
                    source_range_start,
                    range_length,
                }
            },
        )(input)
    }

    fn associate(
        &self,
        source_index: u64,
    ) -> Option<u64> {
        let contains = self.source_range_start <= source_index
            && source_index < (self.source_range_start + self.range_length);
        if contains {
            Some(self.destination_range_start + source_index - self.source_range_start)
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
struct ConversionMap {
    ranges: Vec<ConversionRange>,
}

impl ConversionMap {
    fn new(ranges: Vec<ConversionRange>) -> Self {
        let mut ranges = ranges;
        ranges.sort_by(|a, b| a.source_range_start.cmp(&b.source_range_start));

        Self { ranges }
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            many0(terminated(ConversionRange::parse, opt(line_ending))),
            ConversionMap::new,
        )(input)
    }

    fn associate(
        &self,
        source_index: u64,
    ) -> u64 {
        for range in &self.ranges {
            if range.source_range_start > source_index {
                break;
            }

            if let Some(destination) = range.associate(source_index) {
                return destination;
            }
        }

        source_index
    }

    fn associate_ranges(
        &self,
        ranges: Vec<Range<u64>>,
    ) -> Vec<Range<u64>> {
        ranges
            .into_iter()
            .flat_map(|r| self.associate_range(r))
            .collect()
    }

    fn associate_range(
        &self,
        range: Range<u64>,
    ) -> Vec<Range<u64>> {
        let mut res: Vec<Range<u64>> = Vec::new();
        let mut remaining = range;
        while !&remaining.is_empty() {
            let first_intersecting = self.ranges.iter().find(|r| {
                r.source_range_start.max(remaining.start)
                    < (r.source_range_start + r.range_length).min(remaining.end)
            });

            match first_intersecting {
                None => {
                    // No intersection => direct mapping, nothing else to do
                    res.push(remaining.clone());
                    remaining = Range::default();
                }
                Some(intersect) => {
                    // If some elements before the intersection, direct mapping
                    if intersect.source_range_start > remaining.start {
                        res.push(remaining.start..intersect.source_range_start);
                    }

                    // Build the mapping source -> destination for the intersection
                    let destination_start = intersect.destination_range_start
                        + remaining.start.max(intersect.source_range_start)
                        - intersect.source_range_start;
                    let length = intersect.source_range_end().min(remaining.end)
                        - remaining.start.max(intersect.source_range_start);
                    res.push(destination_start..(destination_start + length));

                    // Build the remaining range to be mapped
                    if remaining.end > intersect.source_range_end() {
                        remaining = intersect.source_range_end()..remaining.end
                    } else {
                        remaining = Range::default()
                    }
                }
            }
        }

        res
    }
}

#[derive(Debug, Default)]
struct Almanac {
    seeds: Vec<u64>,
    seed_to_soil_map: ConversionMap,
    soil_to_fertilizer_map: ConversionMap,
    fertilizer_to_water_map: ConversionMap,
    water_to_light_map: ConversionMap,
    light_to_temperature_map: ConversionMap,
    temperature_to_humidity_map: ConversionMap,
    humidity_to_location_map: ConversionMap,
}

impl Almanac {
    fn parse(input: &str) -> IResult<&str, Self> {
        let seeds = delimited(
            tuple((tag("seeds:"), space1)),
            separated_list0(
                space1,
                character::complete::u64::<&str, nom::error::Error<&str>>,
            ),
            line_ending,
        );

        let conversion_map = |name: &'static str| {
            delimited(
                tuple((tag(name), space1, tag("map:"), line_ending)),
                ConversionMap::parse,
                opt(line_ending),
            )
        };

        let almanac = map(
            tuple((
                terminated(seeds, opt(line_ending)),
                terminated(conversion_map("seed-to-soil"), opt(line_ending)),
                terminated(conversion_map("soil-to-fertilizer"), opt(line_ending)),
                terminated(conversion_map("fertilizer-to-water"), opt(line_ending)),
                terminated(conversion_map("water-to-light"), opt(line_ending)),
                terminated(conversion_map("light-to-temperature"), opt(line_ending)),
                terminated(conversion_map("temperature-to-humidity"), opt(line_ending)),
                terminated(conversion_map("humidity-to-location"), opt(line_ending)),
            )),
            |(
                seeds,
                seed_to_soil_map,
                soil_to_fertilizer_map,
                fertilizer_to_water_map,
                water_to_light_map,
                light_to_temperature_map,
                temperature_to_humidity_map,
                humidity_to_location_map,
            )| {
                Almanac {
                    seeds,
                    seed_to_soil_map,
                    soil_to_fertilizer_map,
                    fertilizer_to_water_map,
                    water_to_light_map,
                    light_to_temperature_map,
                    temperature_to_humidity_map,
                    humidity_to_location_map,
                }
            },
        );

        all_consuming(almanac)(input)
    }

    fn associate(
        &self,
        seed: u64,
    ) -> u64 {
        let soil = self.seed_to_soil_map.associate(seed);
        let fertilizer = self.soil_to_fertilizer_map.associate(soil);
        let water = self.fertilizer_to_water_map.associate(fertilizer);
        let light = self.water_to_light_map.associate(water);
        let temperature = self.light_to_temperature_map.associate(light);
        let humidity = self.temperature_to_humidity_map.associate(temperature);
        self.humidity_to_location_map.associate(humidity)
    }

    fn associate_ranges(
        &self,
        ranges: Vec<Range<u64>>,
    ) -> Vec<Range<u64>> {
        let soil = self.seed_to_soil_map.associate_ranges(ranges);
        let fertilizer = self.soil_to_fertilizer_map.associate_ranges(soil);
        let water = self.fertilizer_to_water_map.associate_ranges(fertilizer);
        let light = self.water_to_light_map.associate_ranges(water);
        let temperature = self.light_to_temperature_map.associate_ranges(light);
        let humidity = self
            .temperature_to_humidity_map
            .associate_ranges(temperature);
        self.humidity_to_location_map.associate_ranges(humidity)
    }
}

pub fn first(
    name: &str,
    data: &str,
) {
    let (_, almanac) = Almanac::parse(data).finish().unwrap();

    let min_location = &almanac
        .seeds
        .iter()
        .map(|seed| almanac.associate(*seed))
        .min()
        .unwrap_or_default();

    println!("[{}] Min location is {:?}", name, min_location);
}

pub fn second(
    name: &str,
    data: &str,
) {
    let (_, almanac) = Almanac::parse(data).finish().unwrap();

    let min_location = almanac
        .seeds
        .chunks_exact(2)
        .collect::<Vec<_>>()
        .par_iter()
        .map(|chunk| {
            let start = chunk[0];
            let size = chunk[1];
            let range = start..(start + size);

            range
                .into_par_iter()
                .map(|seed| almanac.associate(seed))
                .min()
                .unwrap_or_default()
        })
        .min()
        .unwrap_or_default();

    println!("[{}] Min location is {:?}", name, min_location);
}

pub fn second_v2(
    name: &str,
    data: &str,
) {
    let (_, almanac) = Almanac::parse(data).finish().unwrap();

    let ranges = almanac
        .seeds
        .chunks_exact(2)
        .map(|chunk| {
            let start = chunk[0];
            let size = chunk[1];
            start..(start + size)
        })
        .collect::<Vec<_>>();

    // Map the ranges
    let destination_ranges = almanac.associate_ranges(ranges);

    // Min destination is the min start of the destination ranges
    let min_location = destination_ranges
        .iter()
        .map(|r| r.start)
        .min()
        .unwrap_or_default();

    println!("[{}] Min location is {:?}", name, min_location);
}

pub fn run() {
    first("First example", include_str!("data/day5/ex1")); // 35
    first("First", include_str!("data/day5/input")); // 227653707
    second("Second example", include_str!("data/day5/ex1")); // 46
    second("Second", include_str!("data/day5/input")); // 78775051
    second_v2("Second example V2", include_str!("data/day5/ex1")); // 46
    second_v2("Second V2", include_str!("data/day5/input")); // 46
}
