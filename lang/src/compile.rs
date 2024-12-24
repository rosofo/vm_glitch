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

    assemble(&ir, 512)
}

/// Unroll a repeated group statically, incrementing any arguments of the child ops
#[allow(clippy::option_map_unit_fn)]
fn unroll_repeat_group(repeats: usize, children: Vec<Gtch>) -> impl Iterator<Item = Gtch> {
    let len = children.len();
    children
        .into_iter()
        .cycle()
        .map(|mut node| {
            node.copy_mut().map(|(i, j)| {
                i.idx_mut().map(|i| *i += 1);
                j.idx_mut().map(|j| *j += 1);
            });
            node.swap_mut().map(|(i, j)| {
                i.idx_mut().map(|i| *i += 1);
                j.idx_mut().map(|j| *j += 1);
            });
            node.jump_mut().map(|i| {
                i.idx_mut().map(|i| *i += 1);
            });
            node.sample_mut().map(|i| {
                i.idx_mut().map(|i| *i += 1);
            });
            node
        })
        .take(len * repeats)
}
