use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.as_slice() {
        [_, command, path] if command == "tokens" => run_tokens(path),
        [_, command, path] if command == "ast" => run_ast(path),
        [_, command, path] if command == "diag" => run_diag(path),
        [program, ..] => {
            eprintln!("usage: {program} <tokens|ast|diag> <path>");
            process::exit(2);
        }
        [] => unreachable!("argv always contains program name"),
    }
}

fn read_source(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("failed to read {path}: {error}");
            process::exit(1);
        }
    }
}

fn run_tokens(path: &str) {
    let source = read_source(path);
    let output = lang_syntax::lex(&source);
    print!("{}", lang_syntax::dump_tokens(&output.tokens));

    if !output.diagnostics.is_empty() {
        eprint!("{}", lang_syntax::dump_diagnostics(&output.diagnostics));
        process::exit(1);
    }
}

fn run_ast(path: &str) {
    let source = read_source(path);
    let output = lang_syntax::parse(&source);
    print!("{}", lang_syntax::dump_ast(&output.program));

    if !output.diagnostics.is_empty() {
        eprint!("{}", lang_syntax::dump_diagnostics(&output.diagnostics));
        process::exit(1);
    }
}

fn run_diag(path: &str) {
    let source = read_source(path);
    let output = lang_syntax::parse(&source);
    print!("{}", lang_syntax::dump_diagnostics(&output.diagnostics));

    if !output.diagnostics.is_empty() {
        process::exit(1);
    }
}
