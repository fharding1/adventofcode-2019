use std::fs;
use std::collections::HashSet;
use std::iter::FromIterator;

enum Vector {
    Up(u64),
    Down(u64),
    Right(u64),
    Left(u64),
}

impl Vector {
    fn magnitude(&self) -> u64 {
        match *self {
            Vector::Up(x) | Vector::Down(x) | Vector::Right(x) | Vector::Left(x) => x,
        }
    }

    fn dir(&self) -> [i32; 2] {
        match *self {
            Vector::Up(_) => [0, 1],
            Vector::Down(_) => [0, -1],
            Vector::Right(_) => [1, 0],
            Vector::Left(_) => [-1, 0],
        }
    }
}

fn input_to_vectors(input: &str) -> [Vec<Vector>; 2] {
    let mut wires = input.lines().map(|line| {
        let path_parts = line.split(',');
        let mut path = Vec::new();

        for part in path_parts {
            let dist = part[1..].parse().unwrap();

            let vector = match part.chars().nth(0).unwrap() {
                'U' => Vector::Up(dist),
                'D' => Vector::Down(dist),
                'R' => Vector::Right(dist),
                'L' => Vector::Left(dist),
                _ => panic!("unknown vector direction"),
            };

            path.push(vector);
        }

        path
    });

    [wires.next().unwrap(), wires.next().unwrap()]
}

fn wire_to_points(wire: &Vec<Vector>)-> Vec<[i32; 2]> {
    let mut pos = [0, 0];
    let mut points = Vec::new();

    for vector in wire {
        let dir = vector.dir();
        for _ in 0..vector.magnitude() {
            pos[0] += dir[0];
            pos[1] += dir[1];
            points.push(pos);
        }
    }

    points
}

fn main() {
    let input = fs::read_to_string("input").unwrap();
    let wires = input_to_vectors(&input);

    let wire1_points = wire_to_points(&wires[0]);
    let wire1_set: HashSet<[i32; 2]> = HashSet::from_iter(wire1_points.clone().into_iter());
    let wire2_points = wire_to_points(&wires[1]);
    let wire2_set: HashSet<[i32; 2]> = HashSet::from_iter(wire2_points.clone().into_iter());

    let inter = wire1_set.intersection(&wire2_set);

    let shortest_inter = inter
        .map(|v| {
            // find the number of steps it takes to get to the intersection for both paths
            let key = wire1_points
                .iter()
                .position(|x| x[0] == v[0] && x[1] == v[1])
                .unwrap() + 1
                + wire2_points
                    .iter()
                    .position(|x| x[0] == v[0] && x[1] == v[1])
                    .unwrap() + 1;
            (key, v)
        })
        .min_by_key(|v| v.0)
        .unwrap();

    println!("{}", shortest_inter.0);
}