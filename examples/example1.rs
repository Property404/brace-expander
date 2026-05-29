//! This program simply expands its arguments
use brace_expander::BraceExpander;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let brace_expander = BraceExpander::default().ignore_parse_failures(true);

    println!(
        "{}",
        brace_expander.expand(&args).expect("Infallible").join(" ")
    );
}
