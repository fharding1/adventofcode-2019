use std::fmt;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn input_memory() -> Vec<i64> {
        let input = "3,225,1,225,6,6,1100,1,238,225,104,0,1101,82,10,225,101,94,44,224,101,-165,224,224,4,224,1002,223,8,223,101,3,224,224,1,224,223,223,1102,35,77,225,1102,28,71,225,1102,16,36,225,102,51,196,224,101,-3468,224,224,4,224,102,8,223,223,1001,224,7,224,1,223,224,223,1001,48,21,224,101,-57,224,224,4,224,1002,223,8,223,101,6,224,224,1,223,224,223,2,188,40,224,1001,224,-5390,224,4,224,1002,223,8,223,101,2,224,224,1,224,223,223,1101,9,32,224,101,-41,224,224,4,224,1002,223,8,223,1001,224,2,224,1,223,224,223,1102,66,70,225,1002,191,28,224,101,-868,224,224,4,224,102,8,223,223,101,5,224,224,1,224,223,223,1,14,140,224,101,-80,224,224,4,224,1002,223,8,223,101,2,224,224,1,224,223,223,1102,79,70,225,1101,31,65,225,1101,11,68,225,1102,20,32,224,101,-640,224,224,4,224,1002,223,8,223,1001,224,5,224,1,224,223,223,4,223,99,0,0,0,677,0,0,0,0,0,0,0,0,0,0,0,1105,0,99999,1105,227,247,1105,1,99999,1005,227,99999,1005,0,256,1105,1,99999,1106,227,99999,1106,0,265,1105,1,99999,1006,0,99999,1006,227,274,1105,1,99999,1105,1,280,1105,1,99999,1,225,225,225,1101,294,0,0,105,1,0,1105,1,99999,1106,0,300,1105,1,99999,1,225,225,225,1101,314,0,0,106,0,0,1105,1,99999,8,226,226,224,1002,223,2,223,1006,224,329,101,1,223,223,1008,677,677,224,102,2,223,223,1006,224,344,101,1,223,223,1107,226,677,224,102,2,223,223,1005,224,359,101,1,223,223,1008,226,226,224,1002,223,2,223,1006,224,374,1001,223,1,223,1108,677,226,224,1002,223,2,223,1006,224,389,1001,223,1,223,7,677,226,224,1002,223,2,223,1006,224,404,101,1,223,223,7,226,226,224,1002,223,2,223,1005,224,419,101,1,223,223,8,226,677,224,1002,223,2,223,1006,224,434,1001,223,1,223,7,226,677,224,1002,223,2,223,1006,224,449,1001,223,1,223,107,226,677,224,1002,223,2,223,1005,224,464,1001,223,1,223,1007,677,677,224,102,2,223,223,1005,224,479,101,1,223,223,1007,226,226,224,102,2,223,223,1005,224,494,1001,223,1,223,1108,226,677,224,102,2,223,223,1005,224,509,101,1,223,223,1008,677,226,224,102,2,223,223,1005,224,524,1001,223,1,223,1007,677,226,224,102,2,223,223,1005,224,539,101,1,223,223,1108,226,226,224,1002,223,2,223,1005,224,554,101,1,223,223,108,226,226,224,102,2,223,223,1005,224,569,101,1,223,223,108,677,677,224,102,2,223,223,1005,224,584,101,1,223,223,1107,226,226,224,1002,223,2,223,1006,224,599,101,1,223,223,8,677,226,224,1002,223,2,223,1006,224,614,1001,223,1,223,108,677,226,224,102,2,223,223,1006,224,629,1001,223,1,223,1107,677,226,224,1002,223,2,223,1006,224,644,1001,223,1,223,107,677,677,224,102,2,223,223,1005,224,659,101,1,223,223,107,226,226,224,102,2,223,223,1006,224,674,1001,223,1,223,4,223,99,226";
        let memory: Vec<i64> = input
            .split(",")
            .map(|x| x.parse::<i64>().unwrap())
            .collect();
        return memory;
    }

    #[test]
    fn test_part_1() {
        let mut memory = input_memory();
        let diagnostic = evaluate(&mut memory, 1);
        assert_eq!(diagnostic.is_ok(), true);
        assert_eq!(diagnostic.unwrap(), 8332629);
    }
    
    #[test]
    fn test_part_2() {
        let mut memory = input_memory();
        let diagnostic = evaluate(&mut memory, 5);
        assert_eq!(diagnostic.is_ok(), true);
        assert_eq!(diagnostic.unwrap(), 8805067);
    }
}
