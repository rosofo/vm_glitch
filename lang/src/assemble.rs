use itertools::Itertools;
use vm::op::Opcode;

use crate::parse::Gtch;

pub fn assemble<'a>(gtch: impl IntoIterator<Item = &'a Gtch>, bytecode_size: usize) -> Vec<u8> {
    gtch.into_iter()
        .flat_map(|gtch| match gtch {
            Gtch::Copy(i, j) => {
                vec![Opcode::Copy as u8, *i as u8, *j as u8]
            }
            Gtch::Jump(i) => {
                vec![Opcode::Jump as u8, *i as u8]
            }
            Gtch::Sample(i) => {
                vec![Opcode::Sample as u8, *i as u8]
            }
        })
        .pad_using(bytecode_size, |_| 0)
        .take(bytecode_size)
        .collect_vec()
}

#[cfg(test)]
mod tests {
    use prop::collection;
    use proptest::prelude::*;

    use crate::parse::Gtch;

    prop_compose! {
        fn arbitrary_gtch(bytecode_len: u8)(
            opcode in 0..3u8,
            i in 0..bytecode_len,
            j in 0..bytecode_len,
        ) -> Gtch {
            match opcode{
                0 => Gtch::Copy(i as usize, j as usize),
                1 => Gtch::Jump(i as usize),
                2 => Gtch::Sample(i as usize),
                _ => unreachable!(),
            }
        }
    }
    prop_compose! {
        fn arbitrary_inout()(
            bytecode_len in 1..255u8,
        )(
            code in collection::vec(arbitrary_gtch(bytecode_len), 0..512),
            bytecode_len in Just(bytecode_len),
        ) -> (u8, Vec<Gtch>) {
            (bytecode_len, code)
        }
    }

    proptest! {
        #[test]
        fn test_assembler_output_always_of_given_length(
            (bytecode_len, code) in arbitrary_inout()
        ) {
            let bytecode = super::assemble(&code, bytecode_len as usize);
            prop_assert_eq!(bytecode.len(), bytecode_len as usize);
        }
    }
}
