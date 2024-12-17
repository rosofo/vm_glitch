use std::ops::Range;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{combinator::SeparatedBy, prelude::*};

#[derive(Clone, Debug)]
pub enum Gtch {
    Copy(Range<usize>, usize),
    Jump(usize),
    Sample(usize),
}

fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Gtch>, extra::Err<Rich<'a, char>>> {
    let range = text::int(10).then_ignore(just(",")).then(text::int(10));
    let copy = range.then_ignore(just(">")).then(text::int(10)).map(
        |((a1, a2), b): ((&str, &str), &str)| {
            Gtch::Copy(a1.parse().unwrap()..a2.parse().unwrap(), b.parse().unwrap())
        },
    );

    let jump = just(".")
        .ignore_then(text::int(10))
        .map(|i: &str| Gtch::Jump(i.parse().unwrap()));

    let sample = just("~")
        .ignore_then(text::int(10))
        .map(|i: &str| Gtch::Sample(i.parse().unwrap()));

    choice((copy, jump, sample)).padded().repeated().collect()
}

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
    use super::parse;

    #[test]
    fn test_parsing() {
        parse("1,50>25");
        parse(".2 1,50>25");
    }
}
