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

use std::iter::zip;

use anyhow::Result;
use argh::FromArgs;
use atom_syndication::{Content, EntryBuilder, Feed, Person};
use chrono::{DateTime, Utc};
use log::error;

use crate::kaboom_command::KaboomCommand;
use crate::kaboom_feed::KaboomFeed;

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "add")]
/// Add entries to the feed. If *content* is supplied, its source is assumed to
/// be the same URI as *id*.
pub struct AddCommand {
    #[argh(positional)]
    /// the URI of the entry
    id: String,

    #[argh(positional)]
    /// the title of the entry
    title: String,

    #[argh(option, short = 's')]
    /// a short summary of the entry
    summary: Option<String>,

    #[argh(option, short = 'c')]
    /// the full content of the entry
    content: Option<String>,

    #[argh(option, short = 'T')]
    /// the content type of *content*; must be "text", "html", "xhtml", or a
    /// MIME type. ignored if *content* is not provided.
    content_type: Option<String>,

    #[argh(option, short = 'L')]
    /// the language of *content*, often a code like en-us. ignored if *content*
    /// is not provided
    content_language: Option<String>,

    #[argh(option, short = 'a')]
    /// name(s) of authors associated with this entry. can be specified multiple
    /// times, must have the same number of names and emails.
    author_names: Vec<String>,

    #[argh(option, short = 'A')]
    /// email address(es) of authors associated with this entry. can be specified
    /// multiple times, must have the same number of names and emails.
    author_emails: Vec<String>,

    #[argh(option, short = 'd')]
    /// the date and time, in RFC3339 format, when the entry was published
    published_at: Option<DateTime<Utc>>,

    #[argh(option, short = 'D', default = "chrono::Utc::now()")]
    /// the date, in RFC3339 format, when the entry was most recently updated
    updated_at: DateTime<Utc>,
}

impl KaboomCommand for AddCommand {
    fn run(&self, top_args: &crate::Kaboom) -> Result<()> {
        if !self.author_emails.is_empty() && self.author_names.len() != self.author_emails.len() {
            error!(
                "author-names and author-emails must be provided the same number of times each if emails are provided at all, got {} and {}",
                self.author_names.len(),
                self.author_emails.len(),
            );
        }

        let mut feed = Feed::read_from_path(&top_args.file)?;
        let mut eb = EntryBuilder::default();

        eb.id(&self.id);
        eb.title(self.title.clone());
        eb.summary(self.summary.clone().map(|s| s.into()));
        eb.published(self.published_at.map(|p| p.into()));
        eb.updated(self.updated_at);
        eb.content(self.content.clone().map(|s| Content {
            base: None,
            content_type: self.content_type.clone(),
            lang: self.content_language.clone(),
            value: Some(s),
            src: Some(self.id.clone()),
        }));

        if self.author_emails.is_empty() {
            eb.contributors(
                self.author_names
                    .iter()
                    .map(|name| Person {
                        name: name.clone(),
                        email: None,
                        uri: None,
                    })
                    .collect::<Vec<Person>>(),
            );
        } else {
            eb.contributors(
                zip(&self.author_names, &self.author_emails)
                    .map(|(name, email)| Person {
                        name: name.clone(),
                        email: Some(email.clone()),
                        uri: None,
                    })
                    .collect::<Vec<Person>>(),
            );
        }

        feed.entries.insert(0, eb.build());

        feed.write_to_path(&top_args.file)?;

        Ok(())
    }
}
