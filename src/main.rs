use std::path::PathBuf;
use std::str::FromStr;

use argh::FromArgs;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;

const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

static GENERATOR: Lazy<atom_syndication::Generator> = Lazy::new(|| atom_syndication::Generator {
    value: APP_NAME.to_string(),
    uri: Some(env!("CARGO_PKG_HOMEPAGE").to_string()),
    version: Some(VERSION.to_string()),
});

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
    Meta(Meta),
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
struct Meta {
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
    /// a web page URL related to the feed, can be provided multiple times
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

    match args.command {
        KaboomSubCommand::Version(_) => {
            println!("{} {}", APP_NAME, VERSION);
            Ok(())
        }
        _ => Err("Unimplemented command".into()),
    }
}
