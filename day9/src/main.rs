use std::collections::HashMap;
use std::fmt;
use std::iter::FromIterator;
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver, RecvError, Sender};
use std::thread;

#[derive(Debug, Copy, Clone)]
enum Mode {
    Position,
    Immediate,
    Relative,
}

impl From<char> for Mode {
    fn from(ch: char) -> Self {
        match ch {
            '2' => Mode::Relative,
            '1' => Mode::Immediate,
            _ => Mode::Position,
        }
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
    AdjRelative(Mode),
}

impl Instruction {
    pub fn parameters(&self) -> usize {
        match *self {
            Instruction::Add(_, _, _) | Instruction::Mul(_, _, _) => 3,
            Instruction::Input(_) | Instruction::Output(_) => 1,
            Instruction::JumpTrue(_, _) | Instruction::JumpFalse(_, _) => 2,
            Instruction::LessThan(_, _, _) | Instruction::Equals(_, _, _) => 3,
            Instruction::AdjRelative(_) => 1,
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
                modes[i] = Mode::from(ch);
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
            '9' => Ok(Instruction::AdjRelative(modes[0])),
            _ => Err(IntcodeError::OpCode(Some(op_code))),
        }
    }
}

struct Intcode {
    memory: HashMap<i64, i64>,
    input: Receiver<i64>,
    output: Sender<i64>,
    pc: i64,
    relative_base: i64,
}

impl Intcode {
    pub fn new(program: &Vec<i64>, input: Receiver<i64>, output: Sender<i64>) -> Self {
        Intcode {
            memory: HashMap::from_iter(
                program
                    .iter()
                    .enumerate()
                    .map(|(index, val)| (index as i64, *val)),
            ),
            input: input,
            output: output,
            pc: 0,
            relative_base: 0,
        }
    }

    fn get_memory(&self, position: i64) -> i64 {
        *self.memory.get(&position).unwrap_or(&0)
    }

    /// Returns the "value" indicated by the offset and mode.
    fn get_value(&self, offset: i64, mode: Mode) -> i64 {
        let pos = self.pc + offset;

        match mode {
            Mode::Position => self.get_memory(self.get_memory(pos)),
            Mode::Immediate => self.get_memory(pos),
            Mode::Relative => self.get_memory(self.get_memory(pos) + self.relative_base),
        }
    }

    /// Returns the storage location indicated by the offset and mode.
    fn get_location(&self, offset: i64, mode: Mode) -> i64 {
        let pos = self.pc + offset;

        match mode {
            Mode::Position => self.get_memory(pos),
            Mode::Immediate => pos,
            Mode::Relative => self.get_memory(pos) + self.relative_base,
        }
    }

    /// Executes the current instruction and advances the program counter. If Ok
    /// None is returned, the instruction was executed successfully, and the program is
    /// not finished. If Ok Some is returned, the program has halted with a diagnostic
    /// code.
    fn step(&mut self) -> Result<Option<i64>, IntcodeError> {
        let instr = Instruction::from_str(&self.get_value(0, Mode::Immediate).to_string())?;

        match instr {
            Instruction::Add(a, b, location)
            | Instruction::Mul(a, b, location)
            | Instruction::LessThan(a, b, location)
            | Instruction::Equals(a, b, location) => {
                let a = self.get_value(1, a);
                let b = self.get_value(2, b);

                let v = match instr {
                    Instruction::Add(_, _, _) => a + b,
                    Instruction::Mul(_, _, _) => a * b,
                    Instruction::LessThan(_, _, _) => (a < b) as i64,
                    _ => (a == b) as i64,
                };

                let location = self.get_location(3, location);

                self.memory.insert(location, v);
            }
            Instruction::Input(location) => {
                let location = self.get_location(1, location);

                let v = self.input.recv()?;
                self.memory.insert(location, v);
            }
            Instruction::Output(location) => {
                let location = self.get_location(1, location);
                let out = self.get_memory(location);

                // any error means amplification is done since nobody's listening,
                // we should return the last output
                if self.output.send(out).is_err() {
                    return Ok(Some(out));
                }
            }
            Instruction::JumpTrue(a, jmp) | Instruction::JumpFalse(a, jmp) => {
                let a = self.get_value(1, a);
                let jmp = self.get_value(2, jmp);

                let cond = match instr {
                    Instruction::JumpTrue(_, _) => a != 0,
                    _ => a == 0,
                };

                if cond {
                    self.pc = jmp;
                    return Ok(None);
                }
            }
            Instruction::AdjRelative(a) => {
                let a = self.get_value(1, a);
                self.relative_base += a;
            }
        }

        let params = instr.parameters();
        self.pc += (params as i64) + 1;

        Ok(None)
    }

    /// Evaluate runs the program until it halts.
    pub fn evaluate(&mut self) -> Result<Option<i64>, IntcodeError> {
        while self.get_memory(self.pc) != 99 {
            let out = self.step()?;

            if out.is_some() {
                return Ok(out);
            }
        }

        Ok(None)
    }
}

fn main() {
    let input = "1102,34463338,34463338,63,1007,63,34463338,63,1005,63,53,1102,3,1,1000,109,988,209,12,9,1000,209,6,209,3,203,0,1008,1000,1,63,1005,63,65,1008,1000,2,63,1005,63,904,1008,1000,0,63,1005,63,58,4,25,104,0,99,4,0,104,0,99,4,17,104,0,99,0,0,1102,1,38,1003,1102,24,1,1008,1102,1,29,1009,1102,873,1,1026,1102,1,32,1015,1102,1,1,1021,1101,0,852,1023,1102,1,21,1006,1101,35,0,1018,1102,1,22,1019,1102,839,1,1028,1102,1,834,1029,1101,0,36,1012,1101,0,31,1011,1102,23,1,1000,1101,405,0,1024,1101,33,0,1013,1101,870,0,1027,1101,0,26,1005,1101,30,0,1004,1102,1,39,1007,1101,0,28,1017,1101,34,0,1001,1102,37,1,1014,1101,20,0,1002,1102,1,0,1020,1101,0,859,1022,1102,1,27,1016,1101,400,0,1025,1102,1,25,1010,109,-6,1207,10,29,63,1005,63,201,1001,64,1,64,1105,1,203,4,187,1002,64,2,64,109,3,2107,25,8,63,1005,63,221,4,209,1106,0,225,1001,64,1,64,1002,64,2,64,109,-4,2101,0,9,63,1008,63,18,63,1005,63,245,1106,0,251,4,231,1001,64,1,64,1002,64,2,64,109,3,2108,38,7,63,1005,63,273,4,257,1001,64,1,64,1106,0,273,1002,64,2,64,109,22,21102,40,1,0,1008,1018,40,63,1005,63,299,4,279,1001,64,1,64,1106,0,299,1002,64,2,64,109,-16,21108,41,41,10,1005,1012,321,4,305,1001,64,1,64,1105,1,321,1002,64,2,64,109,6,2102,1,-2,63,1008,63,22,63,1005,63,341,1105,1,347,4,327,1001,64,1,64,1002,64,2,64,109,21,1206,-8,359,1106,0,365,4,353,1001,64,1,64,1002,64,2,64,109,-7,21101,42,0,-6,1008,1016,44,63,1005,63,389,1001,64,1,64,1105,1,391,4,371,1002,64,2,64,109,2,2105,1,0,4,397,1106,0,409,1001,64,1,64,1002,64,2,64,109,-3,1205,0,427,4,415,1001,64,1,64,1105,1,427,1002,64,2,64,109,-13,2102,1,-1,63,1008,63,39,63,1005,63,449,4,433,1106,0,453,1001,64,1,64,1002,64,2,64,109,-10,1202,4,1,63,1008,63,20,63,1005,63,479,4,459,1001,64,1,64,1106,0,479,1002,64,2,64,109,7,2108,37,-2,63,1005,63,495,1105,1,501,4,485,1001,64,1,64,1002,64,2,64,109,4,21101,43,0,1,1008,1010,43,63,1005,63,523,4,507,1106,0,527,1001,64,1,64,1002,64,2,64,109,-4,1208,-5,23,63,1005,63,549,4,533,1001,64,1,64,1106,0,549,1002,64,2,64,109,-4,1208,7,27,63,1005,63,565,1106,0,571,4,555,1001,64,1,64,1002,64,2,64,109,15,1205,4,587,1001,64,1,64,1106,0,589,4,577,1002,64,2,64,109,-7,1202,-7,1,63,1008,63,18,63,1005,63,613,1001,64,1,64,1106,0,615,4,595,1002,64,2,64,109,5,21107,44,43,1,1005,1015,635,1001,64,1,64,1105,1,637,4,621,1002,64,2,64,109,-2,21102,45,1,6,1008,1018,44,63,1005,63,661,1001,64,1,64,1105,1,663,4,643,1002,64,2,64,109,-18,1207,6,24,63,1005,63,685,4,669,1001,64,1,64,1105,1,685,1002,64,2,64,109,4,2101,0,8,63,1008,63,21,63,1005,63,707,4,691,1105,1,711,1001,64,1,64,1002,64,2,64,109,17,1206,5,725,4,717,1105,1,729,1001,64,1,64,1002,64,2,64,109,9,21107,46,47,-9,1005,1015,751,4,735,1001,64,1,64,1106,0,751,1002,64,2,64,109,-9,1201,-6,0,63,1008,63,26,63,1005,63,775,1001,64,1,64,1106,0,777,4,757,1002,64,2,64,109,-15,1201,0,0,63,1008,63,23,63,1005,63,803,4,783,1001,64,1,64,1105,1,803,1002,64,2,64,109,-1,2107,30,10,63,1005,63,819,1106,0,825,4,809,1001,64,1,64,1002,64,2,64,109,24,2106,0,5,4,831,1105,1,843,1001,64,1,64,1002,64,2,64,109,-5,2105,1,5,1001,64,1,64,1105,1,861,4,849,1002,64,2,64,109,14,2106,0,-5,1105,1,879,4,867,1001,64,1,64,1002,64,2,64,109,-17,21108,47,44,4,1005,1019,899,1001,64,1,64,1105,1,901,4,885,4,64,99,21101,0,27,1,21102,915,1,0,1106,0,922,21201,1,58969,1,204,1,99,109,3,1207,-2,3,63,1005,63,964,21201,-2,-1,1,21101,0,942,0,1105,1,922,22102,1,1,-1,21201,-2,-3,1,21101,957,0,0,1106,0,922,22201,1,-1,-2,1106,0,968,21201,-2,0,-2,109,-3,2105,1,0";
    let program: Vec<i64> = input
        .split(",")
        .map(|x| x.parse::<i64>().unwrap())
        .collect();

    let (send_in, recv_in) = channel();
    send_in.send(2).unwrap();

    let (send_out, recv_out) = channel();

    let mut computer = Intcode::new(&program, recv_in, send_out);

    thread::spawn(move || {
        computer.evaluate().unwrap();
    });

    for out in recv_out.iter() {
        println!("{}", out);
    }
}
