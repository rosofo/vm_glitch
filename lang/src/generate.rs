use itertools::Itertools;
use rand::prelude::*;
use tracing::instrument;

pub fn generate() -> String {
    gen_children(true)
}

#[instrument]
fn gen_children(recurse: bool) -> String {
    let mut rng = thread_rng();
    let mut choices: Vec<Box<dyn Fn() -> String>> = vec![
        Box::new(|| format!(".{}", thread_rng().gen_range(0..256))),
        Box::new(|| format!("~{}", thread_rng().gen_range(0..256))),
        Box::new(|| {
            format!(
                "{}>{}",
                thread_rng().gen_range(0..256),
                thread_rng().gen_range(0..256)
            )
        }),
        Box::new(|| {
            format!(
                "{}<>{}",
                thread_rng().gen_range(0..256),
                thread_rng().gen_range(0..256)
            )
        }),
        Box::new(|| {
            if recurse {
                format!(
                    "[{} {}]",
                    thread_rng().gen_range(1..256),
                    gen_children(false)
                )
            } else {
                "".to_string()
            }
        }),
    ];
    (1..rng.gen_range(2..10))
        .map(|i| {
            let choice = choices.choose_mut(&mut rng).unwrap();
            choice()
        })
        .join(" ")
}
