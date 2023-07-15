// Copyright (C) 2023 Josh Klar aka "klardotsh" <josh@klar.sh>
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
// REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND
// FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
// INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
// LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
// OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
// PERFORMANCE OF THIS SOFTWARE.

mod kaboom_command;
mod kaboom_feed;
mod meta_command;
mod prune_command;
mod stringable_link;

use std::path::PathBuf;

use anyhow::Result;
use argh::FromArgs;
use env_logger::Env;

use kaboom_command::KaboomCommand;
use meta_command::MetaCommand;
use prune_command::PruneCommand;

pub const APP_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(FromArgs, Debug)]
/// Manage an on-disk Atom feed's entries.
pub struct Kaboom {
    #[argh(subcommand)]
    command: KaboomSubCommand,

    #[argh(option, short = 'f', default = "PathBuf::from(\"feed.xml\")")]
    /// path to Atom feed
    file: PathBuf,

    #[argh(switch, short = 'n')]
    /// do not write anything to disk, but still show what *would* change
    no_op: bool,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum KaboomSubCommand {
    Meta(MetaCommand),
    Prune(PruneCommand),
    Version(KaboomVersion),
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "version")]
/// Display version info and exit.
struct KaboomVersion {}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();

    let args: Kaboom = argh::from_env();

    match &args.command {
        KaboomSubCommand::Version(_) => {
            println!("{} {}", APP_NAME, VERSION);
            Ok(())
        }
        KaboomSubCommand::Meta(meta) => meta.run(&args),
        KaboomSubCommand::Prune(prune) => prune.run(&args),
    }
}
