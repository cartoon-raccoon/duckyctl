use clap::{Args, Parser, Subcommand};
use thiserror::Error;

use crate::duckypad::{Color, DuckyPad, DuckyPadError, LedIdx, Pgv, PgvIdx};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub fn run(&self, dp: &DuckyPad) -> Result<(), CliError> {
        match self.command {
            Command::QueryInfo => {
                let info = dp.query_info()?;
                println!("{}", info);
            }
            Command::GotoProfileNum { id } => dp.goto_profile_num(id)?,
            Command::GotoProfileName { ref name } => dp.goto_profile_name(name)?,
            Command::PrevProfile => dp.prev_profile()?,
            Command::NextProfile => dp.next_profile()?,
            Command::SetRGB { idx, ref color } => {
                let Some(idx) = LedIdx::from_repr(idx) else {
                    return Err(CliError::InvalidLed(idx))
                };
                let Some(color) = Color::from_str(color) else {
                    return Err(CliError::InvalidColor(color.to_string()))
                };

                dp.set_rgb(idx, color)?;
            }
            Command::Reset { msc } => dp.reset(msc)?,
            Command::Sleep => dp.sleep()?,
            Command::Wake => dp.wake()?,
            Command::DumpPgv { idx, next } => {
                let Some(idx) = PgvIdx::from_repr(idx) else {
                    return Err(CliError::PgvIdxOutOfBounds(idx))
                };

                let mut pgvdump = dp.dump_pgv(idx)?;

                let first = pgvdump.next().expect("pgv dump should not be empty");

                println!("{}", first);

                if let Some(next) = next {
                    for _ in 0..next {
                        if let Some(pgv) = pgvdump.next() {
                            println!("{}", pgv);
                        } else {
                            break
                        }
                    }
                }
            }
            Command::WritePgv(ref writepgv) => {
                if writepgv.size() > WritePgv::MAX_PAIRS {
                    return Err(CliError::TooManyPgvs(writepgv.size()))
                }

                dp.write_pgv(writepgv.pgvs())?;
            }
            Command::UpdateRTC => dp.update_rtc(None)?
        }

        Ok(())
    }

}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Invalid LED: expected index 19 or below, got {0}")]
    InvalidLed(u8),
    #[error("Invalid Color: could not parse '{0}' into a valid color")]
    InvalidColor(String),
    #[error("Invalid PGV: expected index 31 or below, got {0}")]
    PgvIdxOutOfBounds(u8),
    #[error("Too many PGVs: maximum 10 can be written at once, received {0}")]
    TooManyPgvs(usize),
    #[error("{0}")]
    DPError(#[from] DuckyPadError),
}

#[derive(Subcommand)]
pub enum Command {
    /// Query information from the DuckyPad.
    QueryInfo,
    /// Go to the profile by ID.
    GotoProfileNum{ id: u8, },
    /// Go to the profile by name.
    GotoProfileName{ name: String },
    /// Go to the previous profile.
    PrevProfile,
    /// Go to the next profile.
    NextProfile,
    /// Set the RGB color of an LED.
    SetRGB{
        /// The index of the LED to set.
        idx: u8,
        /// The color to set the LED to, formatted rrggbb.
        color: String,
    },
    /// Reset the duckyPad.
    Reset{
        /// Whether to boot into USB MSC mode.
        #[arg(long, short)]
        msc: bool,
    },
    /// Put the duckyPad to sleep.
    Sleep,
    /// Wake the duckyPad from sleep.
    Wake,
    /// Dump a persistent global variable.
    DumpPgv{
        idx: u8,
        /// How many subsequent variables to print.
        #[arg(long, short)]
        next: Option<u8>,
    },
    /// Write a persistent global variable.
    WritePgv(WritePgv),
    /// Update the real-time clock.
    UpdateRTC,
}

fn parse_pgv(arg: &str) -> Result<Pgv, String> {
    let (i, v) = arg
        .split_once('=')
        .ok_or_else(|| format!("expected IDX=VALUE, got `{arg}`"))?;
    let i: u8 = i.parse().map_err(|_| format!("invalid index `{i}`"))?;
    let idx = PgvIdx::from_repr(i).ok_or(format!("index {i} out of bounds, must be 31 or below"))?;
    let value: u32 = v.parse().map_err(|_| format!("invalid value `{v}`"))?;

    Ok(Pgv {idx, value})
}

#[derive(Args, Debug)]
pub struct WritePgv {
    /// PGV to write, as IDX=VALUE (repeatable)
    #[arg(
        short = 'i',
        long = "idx",
        value_name = "IDX=VALUE",
        value_parser = parse_pgv,
        required = true,
    )]
    args: Vec<Pgv>,
}

impl WritePgv {
    const MAX_PAIRS: usize = 10;

    pub fn size(&self) -> usize {
        self.args.len()
    }

    pub fn pgvs(&self) -> &[Pgv] {
        &self.args
    }
}
