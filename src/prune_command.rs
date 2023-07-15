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
use atom_syndication::{Entry as AtomEntry, Feed};
use chrono::{DateTime, Utc};
use log::warn;

use crate::kaboom_command::KaboomCommand;
use crate::kaboom_feed::KaboomFeed;
use crate::Kaboom;

type AtomEntries = Vec<AtomEntry>;

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
            "since-date" => Ok(Self::SinceDate),
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
    reject_file: Option<PathBuf>,

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
        let mut any_updates = false;
        let mut feed = Feed::read_from_path(&top_args.file)?;

        if self.count <= feed.entries().len() {
            warn!("not pruning anything because feed already includes <= target count");
        } else {
            let rejected = self.truncate_returning_rejects(&mut feed.entries);

            if !self.no_reject {
                let mut rej_feed = feed.clone();
                rej_feed.set_entries(rejected);
            }
        }

        Ok(())
    }
}

impl PruneCommand {
    /// Sort the entries based on the desired strategy, and retain only as many
    /// in *entries* as necessary to fulfil criteria (modifying the input Vec
    /// in-place). Return the remainder as a new Vec.
    fn truncate_returning_rejects(&self, entries: &mut AtomEntries) -> AtomEntries {
        match self.strategy {
            PruneStrategy::RecentlyPublished => {
                entries.sort_by_key(|it| it.published);
                entries.split_off(self.count)
            }
            PruneStrategy::RecentlyUpdated => {
                entries.sort_by_key(|it| it.updated);
                entries.split_off(self.count)
            }
            PruneStrategy::SinceDate => {
                entries.sort_by_key(|it| it.published);
                let ppoint = entries.partition_point(|e| {
                    e.published().map_or(false, |pubd| pubd >= &self.since_date)
                });
                if ppoint > self.count {
                    entries.split_off(self.count)
                } else {
                    entries.split_off(ppoint)
                }
            }
        }
    }
}
