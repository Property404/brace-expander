//! This program simply expands its arguments
use brace_expander::BraceExpander;
use std::{process::ExitCode, time::Instant};

fn main() -> ExitCode {
    // Parse arguments
    let mut timer = false;
    let mut args = std::env::args().skip(1).peekable();
    while let Some(option) = args.next_if(|arg| arg.starts_with("-")) {
        if option == "--timer" {
            timer = true;
        } else if option == "--" {
            break;
        } else {
            eprintln!("Unknown option {option}");
            return ExitCode::FAILURE;
        }
    }
    let args = args.collect::<Vec<_>>().join(" ");

    // Start recording time
    let start = Instant::now();

    // Expand
    let brace_expander = BraceExpander::default().ignore_parse_failures(true);
    let expansion = brace_expander.expand(&args).expect("Infallible");
    let expansion_time = Instant::now() - start;

    // Print results
    println!("{}", expansion.join(" "));
    let total_time = Instant::now() - start;

    // Show elapsed time
    if timer {
        eprintln!("Expansion time: {expansion_time:?}");
        eprintln!("total time    : {total_time:?}");
    }

    ExitCode::SUCCESS
}
