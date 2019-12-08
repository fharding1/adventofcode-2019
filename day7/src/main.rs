use itertools::Itertools;
use std::fmt;
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver, RecvError, Sender};
use std::thread;

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
enum IntcodeError {
    OpCode(Option<char>),
    Input(RecvError),
}

impl From<RecvError> for IntcodeError {
    fn from(error: RecvError) -> Self {
        IntcodeError::Input(error)
    }
}

impl fmt::Display for IntcodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            IntcodeError::OpCode(code) => match code {
                Some(code) => write!(f, "unknown op code: {}", code),
                None => write!(f, "empty op code"),
            },
            IntcodeError::Input(recv_err) => write!(f, "unable to get input: {}", recv_err),
        }
    }
}

impl FromStr for Instruction {
    type Err = IntcodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars = &mut s.chars().rev();

        let op_code = match chars.next() {
            Some(op_code) => op_code,
            None => return Err(IntcodeError::OpCode(None)),
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
            _ => Err(IntcodeError::OpCode(Some(op_code))),
        }
    }
}

fn evaluate(
    memory: &mut Vec<i64>,
    input: Receiver<i64>,
    output: Sender<i64>,
) -> Result<Option<i64>, IntcodeError> {
    let mut pc = 0;

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
                    let v = input.recv()?;
                    memory[location as usize] = v;
                } else {
                    let out = memory[location as usize];

                    // any error means amplification is done since nobody's listening,
                    // we should return the last output
                    if output.send(out).is_err() {
                        return Ok(Some(out));
                    }
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

    Ok(None)
}

fn main() {
    let input = "3,8,1001,8,10,8,105,1,0,0,21,42,67,84,109,122,203,284,365,446,99999,3,9,1002,9,3,9,1001,9,5,9,102,4,9,9,1001,9,3,9,4,9,99,3,9,1001,9,5,9,1002,9,3,9,1001,9,4,9,102,3,9,9,101,3,9,9,4,9,99,3,9,101,5,9,9,1002,9,3,9,101,5,9,9,4,9,99,3,9,102,5,9,9,101,5,9,9,102,3,9,9,101,3,9,9,102,2,9,9,4,9,99,3,9,101,2,9,9,1002,9,3,9,4,9,99,3,9,101,2,9,9,4,9,3,9,101,1,9,9,4,9,3,9,101,1,9,9,4,9,3,9,1001,9,1,9,4,9,3,9,101,1,9,9,4,9,3,9,1002,9,2,9,4,9,3,9,1002,9,2,9,4,9,3,9,1001,9,2,9,4,9,3,9,101,1,9,9,4,9,3,9,1002,9,2,9,4,9,99,3,9,1001,9,1,9,4,9,3,9,101,2,9,9,4,9,3,9,102,2,9,9,4,9,3,9,101,1,9,9,4,9,3,9,102,2,9,9,4,9,3,9,1001,9,1,9,4,9,3,9,101,1,9,9,4,9,3,9,1002,9,2,9,4,9,3,9,101,2,9,9,4,9,3,9,1002,9,2,9,4,9,99,3,9,101,2,9,9,4,9,3,9,101,2,9,9,4,9,3,9,101,2,9,9,4,9,3,9,101,1,9,9,4,9,3,9,101,1,9,9,4,9,3,9,102,2,9,9,4,9,3,9,1002,9,2,9,4,9,3,9,1002,9,2,9,4,9,3,9,101,2,9,9,4,9,3,9,1001,9,1,9,4,9,99,3,9,1001,9,1,9,4,9,3,9,101,1,9,9,4,9,3,9,102,2,9,9,4,9,3,9,1002,9,2,9,4,9,3,9,1001,9,2,9,4,9,3,9,1001,9,1,9,4,9,3,9,1001,9,2,9,4,9,3,9,1002,9,2,9,4,9,3,9,1002,9,2,9,4,9,3,9,102,2,9,9,4,9,99,3,9,102,2,9,9,4,9,3,9,1002,9,2,9,4,9,3,9,101,2,9,9,4,9,3,9,101,2,9,9,4,9,3,9,101,1,9,9,4,9,3,9,1002,9,2,9,4,9,3,9,101,1,9,9,4,9,3,9,1001,9,2,9,4,9,3,9,102,2,9,9,4,9,3,9,101,1,9,9,4,9,99";
    let memory: Vec<i64> = input
        .split(",")
        .map(|x| x.parse::<i64>().unwrap())
        .collect();

    let mut max_thruster_signal = 0;

    for settings in [5, 6, 7, 8, 9].iter().permutations(5) {
        // below code is very verbose but honestly I don't care at the moment

        let (a_send, a_rec) = channel::<i64>();
        let (b_send, b_rec) = channel::<i64>();
        let (c_send, c_rec) = channel::<i64>();
        let (d_send, d_rec) = channel::<i64>();
        let (e_send, e_rec) = channel::<i64>();

        a_send.send(*settings[0]).unwrap();
        b_send.send(*settings[1]).unwrap();
        c_send.send(*settings[2]).unwrap();
        d_send.send(*settings[3]).unwrap();
        e_send.send(*settings[4]).unwrap();

        a_send.send(0).unwrap();

        let mut a_mem = memory.clone();
        let mut b_mem = memory.clone();
        let mut c_mem = memory.clone();
        let mut d_mem = memory.clone();
        let mut e_mem = memory.clone();

        thread::spawn(move || {
            evaluate(&mut a_mem, a_rec, b_send).unwrap();
        });
        thread::spawn(move || {
            evaluate(&mut b_mem, b_rec, c_send).unwrap();
        });
        thread::spawn(move || {
            evaluate(&mut c_mem, c_rec, d_send).unwrap();
        });
        thread::spawn(move || {
            evaluate(&mut d_mem, d_rec, e_send).unwrap();
        });

        let diagnostic_code = evaluate(&mut e_mem, e_rec, a_send).unwrap().unwrap();

        if diagnostic_code > max_thruster_signal {
            max_thruster_signal = diagnostic_code;
        }
    }

    println!("{}", max_thruster_signal);
}
