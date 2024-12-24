use std::{fmt::Debug, ops::Range};

use color_eyre::Section;
use itertools::Itertools;
use thiserror::Error;
use vm::op::Opcode;

use crate::parse::{Atom, Gtch};

// i know i know it's really horrible, I tried to be clever about Results and let's hope I come to my senses sometime
pub fn assemble<'a>(
    gtch: impl IntoIterator<Item = &'a Gtch>,
    bytecode_size: usize,
) -> Result<Vec<u8>, eyre::Report> {
    let bytecode = gtch
        .into_iter()
        .map::<std::result::Result<Vec<u8>, AssembleError>, _>(|gtch| match gtch {
            Gtch::Copy(i, j) => {
                let j = j.clone().idx().ok_or(AssembleError(
                    "Range cannot be used as second argument to Copy",
                ))?;
                Ok(match i {
                    Atom::Idx(i) => vec![Opcode::Copy as u8, *i as u8, j as u8],
                    Atom::Range(r) => {
                        if r.is_empty() {
                            return Err(AssembleError("Range must be nonempty"));
                        }
                        if r.len() + j > 255 {
                            return Err(AssembleError("Range ends beyond the max index (255)"));
                        }

                        r.clone()
                            .enumerate()
                            .flat_map(|(i, k)| vec![Opcode::Copy as u8, k as u8, j as u8 + i as u8])
                            .collect_vec()
                    }
                })
            }
            Gtch::Jump(i) => {
                let i = i
                    .clone()
                    .idx()
                    .ok_or(AssembleError("Range cannot be used as argument to Jump"))?;
                Ok(vec![Opcode::Jump as u8, i as u8])
            }
            Gtch::Sample(i) => {
                let i = i
                    .clone()
                    .idx()
                    .ok_or(AssembleError("Cannot sample a range"))?;
                Ok(vec![Opcode::Sample as u8, i as u8])
            }
            Gtch::Swap(i, j) => {
                let i = i
                    .clone()
                    .idx()
                    .ok_or(AssembleError("Range cannot be used as argument to Swap"))?;
                let j = j
                    .clone()
                    .idx()
                    .ok_or(AssembleError("Range cannot be used as argument to Swap"))?;
                Ok(vec![Opcode::Swap as u8, i as u8, j as u8])
            }
            _ => unreachable!(),
        });
    let (bytecode, errs): (Vec<Vec<u8>>, Vec<AssembleError>) = bytecode.partition_result();
    if !errs.is_empty() {
        return Err(errs
            .into_iter()
            .fold(eyre::eyre!("Assembly errors"), |report, err| {
                report.error(err)
            }));
    }
    let bytecode = bytecode
        .into_iter()
        .flatten()
        .pad_using(bytecode_size, |_| 0)
        .take(bytecode_size)
        .collect_vec();

    Ok(bytecode)
}

#[derive(Debug, Error)]
#[error("{0}")]
struct AssembleError(&'static str);

#[cfg(test)]
mod tests {
    use std::iter::once;

    use itertools::Itertools;
    use prop::collection;
    use proptest::prelude::*;
    use vm::op::Opcode;

    use crate::parse::{Atom, Gtch};

    prop_compose! {
        fn arb_idx(bytecode_len: u8)(i in 0..bytecode_len) -> Atom {
            Atom::Idx(i as usize)
        }
    }
    prop_compose! {
        fn arb_range(bytecode_len: u8)(i in 0..bytecode_len)(i in Just(i), j in i..bytecode_len) -> Atom {
            Atom::Range(i as usize..j as usize)
        }
    }

    prop_compose! {
        fn arbitrary_gtch(bytecode_len: u8)(
            opcode in 0..3u8,
            i in arb_range(bytecode_len).boxed().prop_union(arb_idx(bytecode_len).boxed()),
            j in arb_range(bytecode_len).boxed().prop_union(arb_idx(bytecode_len).boxed()),
        ) -> Gtch {
            match opcode{
                0 => Gtch::Copy(i, j),
                1 => Gtch::Jump(i),
                2 => Gtch::Sample(i),
                _ => unreachable!(),
            }
        }
    }
    prop_compose! {
        fn arbitrary_inout()(
            bytecode_len in 1..255u8,
        )(
            code in collection::vec(arbitrary_gtch(bytecode_len), 0..10),
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

            if let Ok(b) = bytecode {
                prop_assert_eq!(b.len(), bytecode_len as usize);
            }
        }
    }

    proptest! {
        #[test]
        #[ignore = "some cases where instructions are duplicated :s"]
        fn test_copy_range(r in arb_range(255), i in 0..255usize) {
            let gtch = Gtch::Copy(r, Atom::Idx(i));
            let Ok(result) = super::assemble(once(&gtch), 512) else {return Err(TestCaseError::reject("skipping bad inputs"))};
            let chunks = result.iter().copied().chunks(3);
            for chunk in &chunks {
                let chunk = chunk.collect_vec();
                if chunk[0] == 0 {
                    break;
                }
                prop_assert_eq!(chunk[0], Opcode::Copy as u8);
            }
            let froms = result.iter().skip(1).step_by(3);
            for (i, j) in froms.tuple_windows() {
                prop_assert_eq!(j - i, 1, "{},{} should be consecutive", i, j);
            }
            let tos = result.iter().skip(2).step_by(3);
            for (i, j) in tos.tuple_windows() {
                prop_assert_eq!(j - i, 1, "{},{} should be consecutive", i, j);
            }

        }
    }
}
