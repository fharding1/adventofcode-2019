use std::collections::{HashMap};
use std::iter::successors;
use std::fs;

fn main() {
    let input = fs::read_to_string("input").unwrap();

    let graph: HashMap<&str, &str> = input
        .split("\n")
        .map(|line| {
            let mut parts = line.split(")");

            let parent = parts.next().unwrap();
            let child = parts.next().unwrap();

            (child, parent)
        })
        .collect();

    let count = graph.values()
        .fold(0, |acc, n| acc + successors(graph.get(*n), |n| graph.get(*n)).count()+1);

    println!("{}", count);

    let your_ancestors: Vec<&&str> = successors(graph.get("YOU"), |n| graph.get(*n)).collect();
    let santas_ancestors: Vec<&&str> = successors(graph.get("SAN"), |n| graph.get(*n)).collect();

    for (i, v) in your_ancestors.iter().enumerate() {
        for (j, w) in santas_ancestors.iter().enumerate() {
            if v == w {
                println!("{}", i+j);
                return
            }
        }
    }
}
