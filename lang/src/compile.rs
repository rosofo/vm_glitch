use std::iter::once;

use itertools::Itertools;
use vm::op::Op;

use crate::{assemble::assemble, parse::Gtch};

pub fn compile(ast: &[Gtch], bytecode_len: usize) -> Result<Vec<u8>, eyre::Report> {
    let mut ir = vec![];

    for node in ast.iter().cloned() {
        if let Gtch::RepeatGroup {
            max_iters,
            children,
        } = node
        {
            ir.extend(unroll_repeat_group(max_iters, children));
        } else {
            ir.push(node);
        }
    }

    println!("{:?}", ir);

    assemble(&ir, bytecode_len)
}

/// Unroll a repeated group statically, incrementing any arguments of the child ops
#[allow(clippy::option_map_unit_fn)]
fn unroll_repeat_group(repeats: usize, children: Vec<Gtch>) -> impl Iterator<Item = Gtch> {
    let len = children.len();
    children
        .into_iter()
        .cycle()
        .enumerate()
        .map(move |(op_idx, mut node)| {
            let group_idx = op_idx / len;

            node.copy_mut().map(|(i, j)| {
                i.idx_mut().map(|i| *i += group_idx);
                j.idx_mut().map(|j| *j += group_idx);
            });
            node.swap_mut().map(|(i, j)| {
                i.idx_mut().map(|i| *i += group_idx);
                j.idx_mut().map(|j| *j += group_idx);
            });
            node.jump_mut().map(|i| {
                i.idx_mut().map(|i| *i += group_idx);
            });
            node.sample_mut().map(|i| {
                i.idx_mut().map(|i| *i += group_idx);
            });
            node
        })
        .take(len * repeats)
}

#[cfg(test)]
mod tests {
    use crate::parse;
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn test_unrolling(ops in prop::collection::vec(prop::sample::select(&[
            "~0", "0>1", "0<>1", ".0"
        ]), 0..10).prop_map(|ops| ops.join(" "))) {
            let program = ["[0", &ops, "]"].join(" ");
            let result = parse::parse(&program).unwrap();
            compile(&result, 32).unwrap();
        }
    }
}
