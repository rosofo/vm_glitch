use std::ops::Range;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{combinator, container::Seq, prelude::*};
use tracing::instrument;
use variantly::Variantly;

#[derive(Clone, Debug, Variantly)]
pub enum Atom {
    Idx(usize),
    Range(Range<usize>),
    PC,
}

#[derive(Clone, Debug, Variantly)]
pub enum Gtch {
    Copy(Atom, Atom),
    Jump(Atom),
    Sample(Atom),
    Swap(Atom, Atom),
    RepeatGroup {
        max_iters: usize,
        children: Vec<Gtch>,
    },
}

fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Gtch>, extra::Err<Rich<'a, char>>> {
    recursive(|tree| {
        let range = text::int(10)
            .then_ignore(just("-"))
            .then(text::int(10))
            .map(|(x, y): (&str, &str)| Atom::Range(x.parse().unwrap()..y.parse().unwrap()));
        let idx = text::int(10).map(|i: &str| Atom::Idx(i.parse().unwrap()));
        let pc = just("i").to(Atom::PC);

        let atom = choice((range, idx, pc));

        let copy = atom
            .clone()
            .then_ignore(just(">"))
            .then(atom.clone())
            .map(|(a1, a2)| Gtch::Copy(a1, a2));

        let jump = just(".").ignore_then(atom.clone()).map(Gtch::Jump);

        let sample = just("~").ignore_then(atom.clone()).map(Gtch::Sample);

        let swap = atom
            .clone()
            .then_ignore(just("<>"))
            .then(atom)
            .map(|(a1, a2)| Gtch::Swap(a1, a2));

        let parse_loop = text::int(10)
            .map(|d: &str| d.parse().unwrap())
            .padded()
            .then(tree.or_not().padded())
            .delimited_by(just("["), just("]"))
            .map(|(iterations, children)| Gtch::RepeatGroup {
                max_iters: iterations,
                children: children.unwrap_or(vec![]),
            });

        choice((copy, jump, sample, swap, parse_loop))
            .padded()
            .repeated()
            .collect()
    })
}

#[instrument(skip(s))]
pub fn parse(s: &str) -> Result<Vec<Gtch>, Vec<Rich<char>>> {
    let (gtch, errs) = parser().parse(s.trim()).into_output_errors();
    println!("{:#?}", gtch);
    errs.iter().for_each(|e| {
        let _ = Report::build(ReportKind::Error, e.span().into_range())
            .with_message(e.to_string())
            .with_label(
                Label::new(e.span().into_range())
                    .with_message(e.reason().to_string())
                    .with_color(Color::Red),
            )
            .finish()
            .eprint(Source::from(&s));
    });
    gtch.ok_or(errs)
}

#[cfg(test)]
mod tests {
    use crate::parse::{Atom, Gtch};

    use super::parse;
    use proptest::prelude::*;

    #[test]
    fn test_parsing() {
        parse("1>25").unwrap();
        parse(".2 50>25").unwrap();
    }

    #[test]
    fn test_parsing_ranged() {
        parse("0-200>50").unwrap();
    }

    proptest! {
        #[test]
        fn test_parsing_loop(ops in prop::collection::vec(prop::sample::select(&[
            "~0", "0>1", "0<>1", ".0"
        ]), 0..10).prop_map(|ops| ops.join(" "))) {
            let program = ["[0", &ops, "]"].join(" ");
            let result = parse(&program);
            result.unwrap();
        }
    }
}
