use ariadne::*;
use itertools::Itertools;
use vm::op::{Op, Opcode};

use crate::parse::Gtch;

pub fn assemble<'a>(gtch: impl IntoIterator<Item = &'a Gtch>, bytecode_size: usize) -> Vec<u8> {
    gtch.into_iter()
        .flat_map(|gtch| match gtch {
            Gtch::Copy(range, j) => range
                .clone()
                .flat_map(|i| vec![Opcode::Copy as u8, i as u8, *j as u8])
                .collect_vec(),
            Gtch::Jump(i) => {
                vec![Opcode::Jump as u8, *i as u8]
            }
            Gtch::Sample(i) => {
                vec![Opcode::Sample as u8, *i as u8]
            }
        })
        .pad_using(bytecode_size, |_| 0)
        .collect_vec()
}
