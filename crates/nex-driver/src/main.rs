//! `nex` — the command line driver for the Nex programming language.

mod diag;

use clap::{Parser, Subcommand};
use diag::Diagnostic;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "nex",
    version,
    about = "The Nex programming language",
    long_about = None,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile a program to a native executable.
    Build {
        /// The entry-point source file.
        file: PathBuf,
        /// Where to write the executable.
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// Type-check and run a program.
    Run {
        /// The entry-point source file.
        file: PathBuf,
    },
    /// Type-check a program without running it.
    Check {
        /// The entry-point source file.
        file: PathBuf,
    },
    /// Reformat source files in place.
    Fmt {
        /// Files to format. Defaults to the whole project.
        files: Vec<PathBuf>,
        /// Report which files would change instead of writing them.
        #[arg(long)]
        check: bool,
    },
    /// Run the tests in a program.
    Test {
        /// Only run tests whose name contains this string.
        filter: Option<String>,
    },
    /// Print the token stream for a source file. Development aid.
    Lex {
        /// The source file to scan.
        file: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("nex: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Lex { file } => cmd_lex(&file),
        Command::Build { .. } => Err(unimplemented("build", "Phase 8 (LLVM backend)")),
        Command::Run { .. } => Err(unimplemented("run", "Phase 5 (interpreter)")),
        Command::Check { .. } => Err(unimplemented("check", "Phase 6 (type checker)")),
        Command::Fmt { .. } => Err(unimplemented("fmt", "Phase 15 (formatter)")),
        Command::Test { .. } => Err(unimplemented("test", "Phase 10 (stdlib + test runner)")),
    }
}

fn unimplemented(name: &str, phase: &str) -> String {
    format!("`{name}` is not implemented yet; it arrives in {phase}")
}

fn cmd_lex(path: &Path) -> Result<(), String> {
    let src = read_source(path)?;
    let display = path.display().to_string();
    let (tokens, errors) = nex_lexer::tokenize(&src);

    for token in &tokens {
        println!("{:?}@{:?}", token.kind, token.span);
    }

    if errors.is_empty() {
        return Ok(());
    }

    let diagnostics: Vec<Diagnostic> = errors
        .iter()
        .map(|error| Diagnostic {
            message: error.kind.to_string(),
            span: error.span,
            help: error.help().map(str::to_string),
        })
        .collect();

    eprint!("{}", diag::render(&display, &src, &diagnostics));
    Err(format!("{} lexical error(s) in {display}", errors.len()))
}

fn read_source(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|error| format!("cannot read `{}`: {error}", path.display()))
}
