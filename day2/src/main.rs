use std::fs;
use std;

fn main() {
    let path = "input";

    let input = fs::read_to_string(path).unwrap();
    let input: Vec<i64> = input
        .split(",")
        .map(|x| x.parse::<i64>().unwrap())
        .collect();

    for noun in 0..99 {
        for verb in 0..99 {
            let mut memory = input.clone();

            memory[1] = noun;
            memory[2] = verb;
        
            let mut pc = 0;
            while memory[pc] != 99 {
                let a: i64 = memory[memory[pc+1] as usize];
                let b: i64 = memory[memory[pc+2] as usize];
                let position = memory[pc+3] as usize;
        
                match memory[pc] {
                    1 => memory[position] = a + b,
                    2 => memory[position] = a * b,
                    _ => panic!("unexpected op code"),
                }
        
                pc += 4;
            }
        
            if memory[0] == 19690720 {
                println!("{}", 100 * noun + verb);
                break;
            }
        }
    }

}
