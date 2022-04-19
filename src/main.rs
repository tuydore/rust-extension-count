mod file;

use anyhow::Result;
use clap::Parser;
use file::{Directory, ExtensionSortingMethod};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Root directory for extension count.
    directory: PathBuf,

    /// Sorting mode for extensions only.
    #[clap(short, long, arg_enum, default_value = "file-size")]
    sort: ExtensionSortingMethod,

    /// Depth of recursion.
    #[clap(short, long, default_value_t = 0)]
    depth: usize,

    /// Print empty directories.
    #[clap(short, long)]
    empty: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut directory = Directory::new(args.directory, 0, args.depth)?;
    directory.sort_by(args.sort);
    directory.draw(args.empty)?;
    Ok(())
}
