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

mod stringable_link;

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;

use argh::FromArgs;
use atom_syndication::{Feed, Generator as AtomGenerator};
use chrono::{DateTime, Utc};

use stringable_link::StringableLink;

const APP_HOMEPAGE: &'static str = env!("CARGO_PKG_HOMEPAGE");
const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn get_generator_info() -> AtomGenerator {
    AtomGenerator {
        value: APP_NAME.to_string(),
        uri: Some(APP_HOMEPAGE.into()),
        version: Some(VERSION.to_string()),
    }
}

#[derive(FromArgs, Debug)]
/// Manage an on-disk Atom feed's entries.
struct Kaboom {
    #[argh(subcommand)]
    command: KaboomSubCommand,

    #[argh(option, short = 'f', default = "PathBuf::from(\"feed.xml\")")]
    /// path to Atom feed
    file: PathBuf,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum KaboomSubCommand {
    Meta(MetaCommand),
    Prune(Prune),
    Version(KaboomVersion),
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "version")]
/// Display version info and exit.
struct KaboomVersion {}

#[derive(FromArgs, Debug)]
/// Manage the metadata of the Atom feed, for example the authors or the title.
/// Arguments provided here will set or modify the metadata. After any modifications
/// (with no flags, no modifications will be made), the new state of the feed's
/// metadata will be dumped to standard output (by default in a human-friendly
/// format, but JSON is optionally provided).
#[argh(subcommand, name = "meta")]
struct MetaCommand {
    #[argh(option, short = 't')]
    /// a human-readable title for the feed (this must be set the first time
    /// `kaboom meta` is called on a new file)
    title: Option<String>,

    #[argh(option, short = 'u')]
    /// a unique and permanent URI for this feed, usually the URL at which it is
    /// accessed (this must be set the first time `kaboom meta` is called on a
    /// new file)
    url: Option<String>,

    #[argh(option, short = 'r')]
    /// a web page URL related to the feed, can be provided multiple times.
    /// suffixes in the format of [rel=XXX], [type=XXX], [title=XXX], and
    /// [lang=XXX] are all supported, for example: https://www.meteo.gc.ca/
    /// rss/marine/06100_f.xml[rel=alternate][lang=fr-ca][type=application/
    /// atom+xml][title=Détroit de Haro - Météo maritime - Environnement Canada]
    rel_link: Vec<String>,
    #[argh(switch, short = 'R')]
    /// ensure that no links (except rel=self) are set in this feed's metadata
    remove_links: bool,

    #[argh(option, short = 'i')]
    /// an optional URL pointing to a small image providing visual
    /// identification for the feed (think like a favicon)
    icon: Option<String>,
    #[argh(switch, short = 'I')]
    /// ensure that the icon field is not set in this feed's metadata
    remove_icon: bool,

    #[argh(option, short = 'l')]
    /// an optional URL pointing to a larger image providing visual
    /// identification for the feed (this is distinct from *icon*
    /// above!)
    logo: Option<String>,
    #[argh(switch, short = 'L')]
    /// ensure that the logo field is not set in this feed's metadata
    remove_logo: bool,

    #[argh(option, short = 's')]
    /// an optional string to provide a human-readable description or subtitle
    /// for the feed
    subtitle: Option<String>,
    #[argh(switch, short = 'S')]
    /// ensure that the subtitle field is not set in this feed's metadata
    remove_subtitle: bool,
}

#[derive(FromArgs, Debug)]
/// Remove entries from the Atom feed, and by default send the deleted entries
/// to a reject file for backup/archival purposes.
#[argh(subcommand, name = "prune")]
struct Prune {
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

#[derive(Eq, Debug, PartialEq)]
enum PruneStrategy {
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

fn main() -> Result<(), String> {
    let args: Kaboom = argh::from_env();

    match &args.command {
        KaboomSubCommand::Version(_) => {
            println!("{} {}", APP_NAME, VERSION);
            Ok(())
        }

        KaboomSubCommand::Meta(meta) => do_meta(&args, meta),

        _ => Err("Unimplemented command".into()),
    }
}

fn do_meta(top_args: &Kaboom, args: &MetaCommand) -> Result<(), String> {
    let file = File::open(&top_args.file).unwrap();
    let feed = Feed::read_from(BufReader::new(file)).unwrap();

    println!("{}", prettify_feed_meta(&feed));

    Ok(())
}

fn prettify_feed_meta(feed: &Feed) -> String {
    format!(
        "title={}\nurl={}{}",
        feed.title.to_string(),
        feed.id,
        prettify_links_if_present(&feed).map_or("".into(), |joined| format!("\n{}", joined)),
    )
}

fn prettify_links_if_present(feed: &Feed) -> Option<String> {
    let links = feed.links();

    if links.len() == 0 {
        return None;
    }

    links
        .iter()
        .map(|it| format!("link={}", StringableLink::from_link(it).string_form))
        .collect::<Vec<String>>()
        .join("\n")
        .into()
}
