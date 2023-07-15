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

use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use argh::FromArgs;
use chrono::{DateTime, Utc};

use crate::kaboom_command::KaboomCommand;
use crate::Kaboom;

#[derive(Eq, Debug, PartialEq)]
pub enum PruneStrategy {
    RecentlyPublished,
    RecentlyUpdated,
    SinceDate,
}

impl Default for PruneStrategy {
    fn default() -> Self {
        Self::RecentlyPublished
    }
}

impl FromStr for PruneStrategy {
    type Err = &'static str;

    fn from_str(it: &str) -> Result<Self, Self::Err> {
        match it {
            "published" => Ok(Self::RecentlyPublished),
            "updated" => Ok(Self::RecentlyUpdated),
            _ => Err("unknown pruning strategy"),
        }
    }
}

#[derive(FromArgs, Debug)]
/// Remove entries from the Atom feed, and by default send the deleted entries
/// to a reject file for backup/archival purposes.
#[argh(subcommand, name = "prune")]
pub struct PruneCommand {
    #[argh(positional)]
    /// number of entries to keep in the feed, as sorted by *strategy*,
    /// described below
    count: usize,

    #[argh(option, short = 'R')]
    /// skip sending pruned entries to the *reject_file*, described below
    no_reject: bool,

    #[argh(option, short = 'r')]
    /// path to an Atom file (which will be created if it does not yet exist,
    /// sharing all metadata from the original feed) to store pruned entries for
    /// backup/archival purposes.
    ///
    /// by default, this will be <feed file> with any .xml extension removed, and
    /// then ".rej.xml" added
    file: Option<PathBuf>,

    #[argh(option, short = 's')]
    /// strategy used in pruning entries from the feed: published, for date
    /// of publication, updated, for date of most recent update, or since-
    /// date, which preserves only those articles authored since *since-date*,
    /// described below
    strategy: PruneStrategy,

    #[argh(option, short = 'd')]
    /// a date in YYYY-MM-DD format, used only with the since-date *strategy*,
    /// described above
    since_date: DateTime<Utc>,
}

impl KaboomCommand for PruneCommand {
    fn run(&self, top_args: &Kaboom) -> Result<()> {
        Ok(())
    }
}
