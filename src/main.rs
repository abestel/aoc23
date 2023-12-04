mod day1;
mod day2;

fn main() {
    let days = [day1::run, day2::run];

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
