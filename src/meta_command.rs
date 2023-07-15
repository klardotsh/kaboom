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

use std::fs::File;

use anyhow::Result;
use argh::FromArgs;
use atom_syndication::{Feed, Generator as AtomGenerator};
use log::{debug, warn};

use crate::kaboom_command::KaboomCommand;
use crate::kaboom_feed::KaboomFeed;
use crate::stringable_link::StringableLink;
use crate::Kaboom;

#[derive(FromArgs, Debug)]
/// Manage the metadata of the Atom feed, for example the authors or the title.
/// Arguments provided here will set or modify the metadata. After any modifications
/// (with no flags, no modifications will be made), the new state of the feed's
/// metadata will be dumped to standard output (by default in a human-friendly
/// format, but JSON is optionally provided).
#[argh(subcommand, name = "meta")]
pub struct MetaCommand {
    #[argh(option, short = 't')]
    /// a human-readable title for the feed (this must be set the first time
    /// `kaboom meta` is called on a new file)
    title: Option<String>,

    #[argh(option, short = 'u')]
    /// a unique and permanent URI for this feed, often the URL at which it is
    /// accessed (this must be set the first time `kaboom meta` is called on a
    /// new file)
    uri: Option<String>,

    #[argh(option, short = 'r')]
    /// a web page URL related to the feed, can be provided multiple times.
    /// suffixes in the format of [rel=XXX], [type=XXX], [title=XXX], and
    /// [lang=XXX] are all supported, for example: https://www.meteo.gc.ca/
    /// rss/marine/06100_f.xml[rel=alternate][lang=fr-ca][type=application/
    /// atom+xml][title=Détroit de Haro - Météo maritime - Environnement Canada]
    rel_link: Vec<StringableLink>,
    #[argh(switch, short = 'R')]
    /// ensure that no links (except rel=self) are set in this feed's metadata.
    /// if *rel_link* are still provided, this flag will instead clear all
    /// *existing* links, and add those links as the only links in the metadata.
    remove_links: bool,

    #[argh(option, short = 'i')]
    /// an optional URL pointing to a small image providing visual
    /// identification for the feed (think like a favicon)
    icon: Option<String>,
    #[argh(switch, short = 'I')]
    /// ensure that the icon field is not set in this feed's metadata. ignored
    /// if *icon* is still provided.
    remove_icon: bool,

    #[argh(option, short = 'l')]
    /// an optional URL pointing to a larger image providing visual
    /// identification for the feed (this is distinct from *icon*
    /// above!)
    logo: Option<String>,
    #[argh(switch, short = 'L')]
    /// ensure that the logo field is not set in this feed's metadata. ignored
    /// if *logo* is still provided.
    remove_logo: bool,

    #[argh(option, short = 's')]
    /// an optional string to provide a human-readable description or subtitle
    /// for the feed
    subtitle: Option<String>,
    #[argh(switch, short = 'S')]
    /// ensure that the subtitle field is not set in this feed's metadata. ignored
    /// if *subtitle* is still provided.
    remove_subtitle: bool,

    #[argh(switch, short = 'G')]
    /// do not insert the generator block into the metadata output (which
    /// discloses within the feed that kaboom was used to generate it)
    no_generator: bool,
}

impl KaboomCommand for MetaCommand {
    fn run(&self, top_args: &Kaboom) -> Result<()> {
        let mut any_updates = false;
        let mut feed = Feed::read_from_path(&top_args.file)?;

        if let Some(title) = &self.title {
            if title != &feed.title().to_string() {
                feed.set_title(title.clone());
                any_updates = true;
            }
        }

        if let Some(uri) = &self.uri {
            if uri != &feed.id().to_string() {
                feed.set_id(uri.clone());
                any_updates = true;
            }
        }

        if self.remove_subtitle && self.subtitle.is_none() {
            feed.set_subtitle(None);
            any_updates = true;
        }

        if let Some(subtitle) = &self.subtitle {
            let text_contents = atom_syndication::Text::from(subtitle.as_str());
            let update_needed = match feed.subtitle() {
                Some(existing_st) => &text_contents != existing_st,
                None => true,
            };

            if update_needed {
                feed.set_subtitle(Some(text_contents));
            }

            any_updates = true;
        }

        if self.remove_icon && self.icon.is_none() {
            feed.set_icon(None);
            any_updates = true;
        }

        if self.icon.is_some() && self.icon != feed.icon().map(|it| it.to_string()) {
            feed.set_icon(self.icon.clone());
            any_updates = true;
        }

        if self.remove_logo && self.logo.is_none() {
            feed.set_logo(None);
            any_updates = true;
        }

        if self.logo.is_some() && self.logo != feed.logo().map(|it| it.to_string()) {
            feed.set_logo(self.logo.clone());
            any_updates = true;
        }

        if self.remove_links {
            feed.set_links(Vec::with_capacity(self.rel_link.len()));
            any_updates = true;
        }

        for rel_link in &self.rel_link {
            let rl = &rel_link.link_form;
            if let Some(existing) = feed.links.iter_mut().find(|link| link.href() == rl.href()) {
                let same = existing == rl;
                debug!(
                    "link {} aleady exists, {}",
                    &rel_link.string_form,
                    if same {
                        "seems to be equivalent, skipping!"
                    } else {
                        "modifying in place"
                    }
                );

                if !same {
                    existing.set_rel(rl.rel());
                    existing.set_hreflang(rl.hreflang.clone());
                    existing.set_mime_type(rl.mime_type.clone());
                    existing.set_title(rl.title.clone());
                    any_updates = true;
                }
            } else {
                feed.links.push(rl.clone());
                any_updates = true;
            }
        }

        if any_updates {
            feed.set_updated(chrono::Utc::now());

            if !self.no_generator {
                feed.set_generator(AtomGenerator {
                    value: crate::APP_NAME.to_string(),
                    uri: Some(crate::APP_HOMEPAGE.into()),
                    version: Some(crate::VERSION.to_string()),
                });
            }
        }

        if top_args.no_op {
            warn!("not writing results to disk because no-op was requested");
        } else {
            let temp_path = {
                let mut path = top_args.file.clone();

                if let Some(ext) = top_args.file.extension() {
                    path.set_extension(format!("{}.kaboom", ext.to_string_lossy()));
                } else {
                    path.set_extension(".xml.kaboom");
                }

                path
            };
            debug!(
                "writing results to file {}",
                temp_path.clone().into_os_string().to_string_lossy()
            );
            {
                let mut file = File::create(&temp_path)?;
                feed.write_to(&mut file)?;
                std::fs::rename(&temp_path, &top_args.file)?;
            }
        }

        println!("{}", feed.as_human_text());

        Ok(())
    }
}
