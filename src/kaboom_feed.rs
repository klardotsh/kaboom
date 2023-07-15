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
use std::io::BufReader;
use std::path::Path;

use anyhow::Result;
use atom_syndication::Feed;

use crate::stringable_link::StringableLink;

pub trait KaboomFeed {
    fn as_human_text(&self) -> String;
    fn links_as_human_text(&self) -> Option<String>;
    fn read_from_path(path: &Path) -> Result<Feed>;
}

impl KaboomFeed for Feed {
    // Clippy incorrectly(?) believes that, for example, **self.title() can
    // take the place of self.title().to_string(), despite that **str doesn't
    // have a compile-time-known size.
    #[allow(clippy::to_string_in_format_args)]
    fn as_human_text(&self) -> String {
        format!(
            "title={}{}\nuri={}\nupdated_at={}{}{}{}",
            self.title().to_string(),
            self.subtitle()
                .map_or("".into(), |st| format!("\nsubtitle={}", st.to_string())),
            self.id(),
            self.updated(),
            self.icon().map_or("".into(), |st| format!("\nicon={}", st)),
            self.logo().map_or("".into(), |st| format!("\nlogo={}", st)),
            self.links_as_human_text()
                .map_or("".into(), |joined| format!("\n{}", joined)),
        )
    }

    fn links_as_human_text(&self) -> Option<String> {
        let links = self.links();

        if links.is_empty() {
            return None;
        }

        links
            .iter()
            .map(|it| format!("link={}", StringableLink::from(it)))
            .collect::<Vec<String>>()
            .join("\n")
            .into()
    }

    fn read_from_path(path: &Path) -> Result<Feed> {
        let file = File::open(path)?;
        Ok(Feed::read_from(BufReader::new(file))?)
    }
}
