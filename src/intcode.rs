use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    str::FromStr,
};

use num::{PrimInt, ToPrimitive};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MemoryMode {
    Position,
    Immediate,
}

impl From<usize> for MemoryMode {
    fn from(value: usize) -> Self {
        if value == 0 {
            Self::Position
        } else {
            Self::Immediate
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct IntCode<T> {
    ins: Vec<T>,
    pc: usize,
    input: VecDeque<T>,
    output: VecDeque<T>,
}

impl<T> FromStr for IntCode<T>
where
    T: FromStr + Default,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ins = s
            .split(',')
            .map(|s| s.parse().map_err(|_| format!("parse_error: {s}")))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            ins,
            ..Self::default()
        })
    }
}

pub trait ToUsize: Sized {
    fn to(self) -> usize;
}

impl ToUsize for usize {
    fn to(self) -> usize {
        self
    }
}

impl ToUsize for i64 {
    fn to(self) -> usize {
        self.to_usize().expect("invalid to-string")
    }
}

impl<T> IntCode<T>
where
    T: PrimInt + ToUsize + std::ops::DivAssign + Display + Debug,
{
    pub fn set_values(&mut self, input: &[(T, T)]) {
        for (pos, x) in input.iter().copied() {
            self.ins[pos.to()] = x;
        }
    }

    pub fn get_value(&mut self, pos: T) -> T {
        self.ins[pos.to()]
    }

    pub fn get_output(&mut self) -> Option<T> {
        self.output.pop_front()
    }

    pub fn set_input(&mut self, input: T) {
        self.input.push_back(input)
    }

    fn write_mem(&mut self, pc: usize, mode: MemoryMode, value: T) {
        match mode {
            MemoryMode::Position => {
                let pos = self.ins[pc].to();
                self.ins[pos] = value;
            }
            MemoryMode::Immediate => self.ins[pc.to()] = value,
        }
    }

    fn read_mem(&self, pc: usize, mode: MemoryMode) -> T {
        let pos = self.ins.get(pc).copied().unwrap();
        if mode == MemoryMode::Position {
            self.ins.get(pos.to()).copied().unwrap()
        } else {
            pos
        }
    }

    fn decode_ins(&self) -> (usize, MemoryMode, MemoryMode, MemoryMode) {
        let mut ins = self.ins[self.pc].to();
        let de = ins % 100;
        ins /= 100;

        let a = ins % 10;
        ins /= 10;

        let b = ins % 10;
        ins /= 10;

        let c = ins % 10;
        ins /= 10;

        assert_eq!(ins, 0);

        (de, a.into(), b.into(), c.into())
    }

    pub fn run(&mut self) {
        loop {
            // println!("{:?}", &self.ins[self.pc..self.pc + 4]);
            let (i, mode_a, mode_b, mode_c) = self.decode_ins();
            // println!(
            //     "pc: {}, ins: {i}, mode_a: {mode_a:?}, mode_b: {mode_b:?}, mode_c: {mode_c:?}",
            //     self.pc
            // );
            match i {
                1 => {
                    // ADD
                    let a = self.read_mem(self.pc + 1, mode_a);
                    let b = self.read_mem(self.pc + 2, mode_b);
                    self.write_mem(self.pc + 3, mode_c, a + b);
                    self.pc += 4;
                }
                2 => {
                    // MUL
                    let a = self.read_mem(self.pc + 1, mode_a);
                    let b = self.read_mem(self.pc + 2, mode_b);
                    self.write_mem(self.pc + 3, mode_c, a * b);
                    self.pc += 4;
                }
                3 => {
                    // STORE INPUT
                    let v = self.input.pop_front().unwrap();
                    self.write_mem(self.pc + 1, mode_a, v);

                    // println!("writing to {} the input {}", a.to(), self.ins[a.to()]);
                    self.pc += 2;
                }
                4 => {
                    // WRITE OUTPUT
                    let a = self.read_mem(self.pc + 1, mode_a);
                    self.output.push_back(a);

                    self.pc += 2;
                }
                5 => {
                    // JMP if true
                    let a = self.read_mem(self.pc + 1, mode_a);
                    let b = self.read_mem(self.pc + 2, mode_b);
                    if a != T::zero() {
                        self.pc = b.to();
                    } else {
                        self.pc += 3;
                    }
                }
                6 => {
                    // JMP if false
                    let a = self.read_mem(self.pc + 1, mode_a);
                    let b = self.read_mem(self.pc + 2, mode_b);
                    if a == T::zero() {
                        self.pc = b.to();
                    } else {
                        self.pc += 3;
                    }
                }
                7 => {
                    // set-less-than
                    let a = self.read_mem(self.pc + 1, mode_a);
                    let b = self.read_mem(self.pc + 2, mode_b);
                    self.write_mem(
                        self.pc + 3,
                        mode_c,
                        if a < b { T::one() } else { T::zero() },
                    );
                    self.pc += 4;
                }
                8 => {
                    // set-equal
                    let a = self.read_mem(self.pc + 1, mode_a);
                    let b = self.read_mem(self.pc + 2, mode_b);
                    self.write_mem(
                        self.pc + 3,
                        mode_c,
                        if a == b { T::one() } else { T::zero() },
                    );
                    self.pc += 4;
                }
                99 => {
                    // EXIT
                    println!("{:?}", self.ins);
                    break;
                }
                x => panic!("opcode unknown: {x}"),
            }
        }
    }
}
