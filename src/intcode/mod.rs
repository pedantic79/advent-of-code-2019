use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct IntCode<T> {
    ins: Vec<T>,
    pc: usize,
}

impl<T> FromStr for IntCode<T>
where
    T: FromStr + Default,
{
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ins = s
            .split(',')
            .map(|s| s.parse())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { pc: 0, ins })
    }
}

impl IntCode<usize> {
    pub fn set_values(&mut self, input: &[(usize, usize)]) {
        for (pos, x) in input.iter().copied() {
            self.ins[pos] = x;
        }
    }

    pub fn get_value(&mut self, pos: usize) -> usize {
        self.ins[pos]
    }

    pub fn run(&mut self) {
        loop {
            match self.ins[self.pc] {
                1 => {
                    let a = self.ins[self.pc + 1];
                    let b = self.ins[self.pc + 2];
                    let c = self.ins[self.pc + 3];

                    self.ins[c] = self.ins[a] + self.ins[b];
                }
                2 => {
                    let a = self.ins[self.pc + 1];
                    let b = self.ins[self.pc + 2];
                    let c = self.ins[self.pc + 3];

                    self.ins[c] = self.ins[a] * self.ins[b];
                }
                99 => break,
                x => panic!("opcode unknown: {x}"),
            }
            self.pc += 4;
        }
    }
}
