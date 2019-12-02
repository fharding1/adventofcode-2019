use std::fs::File;
use std::io::{BufReader, BufRead};

fn fuel_requirement(mass: i64) -> i64 {
    let mut total_fuel = 0;

    let mut cur_mass = mass;
    loop {
        cur_mass = (cur_mass / 3) - 2;
        if cur_mass <= 0 {
            break
        }
        
        total_fuel += cur_mass;
    }
    
    return total_fuel;
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