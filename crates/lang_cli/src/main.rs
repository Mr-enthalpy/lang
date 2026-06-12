use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.as_slice() {
        [_, command, path] if command == "tokens" => run_tokens(path),
        [_, command, ..] if command == "ast" || command == "diag" => {
            eprintln!("not implemented yet: {command}");
            process::exit(2);
        }
        [program, ..] => {
            eprintln!("usage: {program} tokens <path>");
            process::exit(2);
        }
        [] => unreachable!("argv always contains program name"),
    }
}

fn run_tokens(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("failed to read {path}: {error}");
            process::exit(1);
        }
    };

    let output = lang_syntax::lex(&source);
    print!("{}", lang_syntax::dump_tokens(&output.tokens));

    if !output.diagnostics.is_empty() {
        eprint!("{}", lang_syntax::dump_diagnostics(&output.diagnostics));
        process::exit(1);
    }
}
