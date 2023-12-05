mod day1;
mod day2;
mod day3;
mod day4;
mod day5;

fn main() {
    let days = [day1::run, day2::run, day3::run, day4::run, day5::run];

    days.iter().enumerate().for_each(|(index, day_fn)| {
        if index != 0 {
            println!("\n\n");
        }

        let day = index + 1;
        println!("==== Day {} ====", day);
        day_fn();
        println!("==== Day {} ====", day);
    })
}
