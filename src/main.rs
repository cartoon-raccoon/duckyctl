use clap::{CommandFactory, Parser, error::ErrorKind};

mod duckypad;
mod cli;

use crate::duckypad::{DuckyPad, DuckyPadError};
use crate::cli::{Cli, CliError};

fn main() -> Result<(), DuckyPadError> {
    let args = Cli::parse();

    let dp = match DuckyPad::new() {
        Ok(dp) => dp,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1)
        }
    };

    if let Err(e) = args.run(&dp) {
        match e {
            a @ CliError::InvalidLed(_) |
            a @ CliError::InvalidColor(_) |
            a @ CliError::PgvIdxOutOfBounds(_) => {
                Cli::command().error(ErrorKind::InvalidValue, a.to_string()).exit();
            }
            a @ CliError::TooManyPgvs(_) => {
                Cli::command().error(ErrorKind::TooManyValues, a.to_string()).exit();
            }
            CliError::DPError(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
