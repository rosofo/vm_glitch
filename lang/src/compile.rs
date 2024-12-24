use std::iter::once;

use itertools::Itertools;
use vm::op::Op;

use crate::{assemble::assemble, parse::Gtch};

pub fn compile(ast: &[Gtch]) -> Result<Vec<u8>, eyre::Report> {
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

    assemble(&ir, 512)
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

    use super::*;

    #[test]
    fn test_unrolling() {
        let ast = parse::parse("[3 .0 1>2 ]").unwrap();
        let bytecode = compile(&ast).unwrap();
    }
}
