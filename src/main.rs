use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    let Some((program, command_args)) = args.split_first() else {
        eprintln!("buas: command is required");
        return ExitCode::FAILURE;
    };

    let status = match Command::new(program).args(command_args).status() {
        Ok(status) => status,
        Err(error) => {
            eprintln!("buas: failed to execute {program:?}: {error}");
            return ExitCode::FAILURE;
        }
    };

    match status.code() {
        Some(code) => ExitCode::from(code as u8),
        None => ExitCode::FAILURE,
    }
}
