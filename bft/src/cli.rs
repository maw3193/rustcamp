use std::{num::NonZeroUsize, path::PathBuf};

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    pub(crate) program: PathBuf,
    #[arg(short, long)]
    pub(crate) cells: Option<NonZeroUsize>,
    #[arg(short, long)]
    pub(crate) extensible: bool,
}
