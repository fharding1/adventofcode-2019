use std::fs::File;
use std::io::{BufReader, BufRead};

fn fuel_requirement(mass: i64) -> i64 {
    let fuel = (mass / 3) - 2;
    if fuel <= 0 {
        return 0;
    }

    return fuel + fuel_requirement(fuel);
}

fn main() {
    let path = "input";

    let input = File::open(path).unwrap();
    let buffered = BufReader::new(input);

    let mut total_fuel = 0;

    for line in buffered.lines() {
        let mass: i64 = line.unwrap().parse().unwrap();
        total_fuel += fuel_requirement(mass);
    }
    
    println!("Total Fuel: {}", total_fuel);
}