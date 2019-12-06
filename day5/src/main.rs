use std::fmt;
use std::fs::read_to_string;
use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
enum Mode {
    Position,
    Immediate,
}

impl FromStr for Mode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(match s {
            "1" => Self::Immediate,
            _ => Self::Position,
        })
    }
}

#[derive(Debug)]
enum Instruction {
    Add(Mode, Mode, Mode),
    Mul(Mode, Mode, Mode),
    Input(Mode),
    Output(Mode),
    JumpTrue(Mode, Mode),
    JumpFalse(Mode, Mode),
    LessThan(Mode, Mode, Mode),
    Equals(Mode, Mode, Mode),
}

impl Instruction {
    pub fn parameters(&self) -> usize {
        match *self {
            Instruction::Add(_, _, _) | Instruction::Mul(_, _, _) => 3,
            Instruction::Input(_) | Instruction::Output(_) => 1,
            Instruction::JumpTrue(_, _) | Instruction::JumpFalse(_, _) => 2,
            Instruction::LessThan(_, _, _) | Instruction::Equals(_, _, _) => 3,
        }
    }
}

#[derive(Debug)]
struct OpCodeError {
    op_code: Option<char>,
}

impl fmt::Display for OpCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.op_code {
            Some(v) => write!(f, "unknown op code: {}", v),
            None => write!(f, "no op code"),
        }
    }
}

impl FromStr for Instruction {
    type Err = OpCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars = &mut s.chars().rev();

        let op_code = match chars.next() {
            Some(op_code) => op_code,
            None => return Err(OpCodeError { op_code: None }),
        };

        // skip the zero in the op code since they're two-padded and we only care about
        // the first digit
        chars.next();

        // always take three modes, default to position
        let mut modes = [Mode::Position; 3];

        for i in 0..3 {
            if let Some(ch) = chars.next() {
                if let Ok(mode) = Mode::from_str(&ch.to_string()) {
                    modes[i] = mode;
                }
            }
        }

        match op_code {
            '1' => Ok(Instruction::Add(modes[0], modes[1], modes[2])),
            '2' => Ok(Instruction::Mul(modes[0], modes[1], modes[2])),
            '3' => Ok(Instruction::Input(modes[0])),
            '4' => Ok(Instruction::Output(modes[0])),
            '5' => Ok(Instruction::JumpTrue(modes[0], modes[1])),
            '6' => Ok(Instruction::JumpFalse(modes[0], modes[1])),
            '7' => Ok(Instruction::LessThan(modes[0], modes[1], modes[2])),
            '8' => Ok(Instruction::Equals(modes[0], modes[1], modes[2])),
            _ => Err(OpCodeError {
                op_code: Some(op_code),
            }),
        }
    }
}

fn evaluate(memory: &mut Vec<i64>, input: i64) -> Result<i64, OpCodeError> {
    let mut pc = 0;
    let mut diagnostic_code = 0;

    while memory[pc] != 99 {
        let instr = Instruction::from_str(&memory[pc].to_string())?;
        let params = instr.parameters();

        match instr {
            Instruction::Add(a, b, location)
            | Instruction::Mul(a, b, location)
            | Instruction::LessThan(a, b, location)
            | Instruction::Equals(a, b, location) => {
                let a = match a {
                    Mode::Position => memory[memory[pc + 1] as usize],
                    Mode::Immediate => memory[pc + 1],
                };
                let b = match b {
                    Mode::Position => memory[memory[pc + 2] as usize],
                    Mode::Immediate => memory[pc + 2],
                };

                let v = match instr {
                    Instruction::Add(_, _, _) => a + b,
                    Instruction::Mul(_, _, _) => a * b,
                    Instruction::LessThan(_, _, _) => (a < b) as i64,
                    _ => (a == b) as i64,
                };

                let location = match location {
                    Mode::Position => memory[pc + 3] as usize,
                    Mode::Immediate => pc + 3,
                };

                memory[location as usize] = v;
            }
            Instruction::Input(location) | Instruction::Output(location) => {
                let location = match location {
                    Mode::Position => memory[pc + 1] as usize,
                    Mode::Immediate => pc + 1,
                };

                if let Instruction::Input(_) = instr {
                    memory[location as usize] = input;
                } else {
                    diagnostic_code = memory[location as usize];
                }
            }
            Instruction::JumpTrue(a, jmp) | Instruction::JumpFalse(a, jmp) => {
                let a = match a {
                    Mode::Position => memory[memory[pc + 1] as usize],
                    Mode::Immediate => memory[pc + 1],
                };
                let jmp = match jmp {
                    Mode::Position => memory[memory[pc + 2] as usize],
                    Mode::Immediate => memory[pc + 2],
                };

                let cond = match instr {
                    Instruction::JumpTrue(_, _) => a != 0,
                    _ => a == 0,
                };

                if cond {
                    pc = jmp as usize;
                    continue;
                }
            }
        }

        pc += params + 1;
    }

    Ok(diagnostic_code)
}

fn main() {
    let input = read_to_string("input").unwrap();
    let program: Vec<i64> = input
        .split(",")
        .map(|x| x.parse::<i64>().unwrap())
        .collect();

    let mut memory = program.clone();
    println!("{}", evaluate(&mut memory, 1).unwrap());

    memory = program.clone();
    println!("{}", evaluate(&mut memory, 5).unwrap());
}
