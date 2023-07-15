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

use std::fmt::Display;
use std::str::FromStr;

use atom_syndication::Link as AtomLink;
use log::debug;

#[derive(Clone, Debug, PartialEq)]
pub struct StringableLink {
    pub link_form: AtomLink,
    pub string_form: String,
}

impl Display for StringableLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string_form)
    }
}

impl From<&AtomLink> for StringableLink {
    fn from(it: &AtomLink) -> Self {
        Self {
            link_form: it.clone(),
            string_form: link_to_string(&it),
        }
    }
}

impl From<&str> for StringableLink {
    fn from(it: &str) -> Self {
        Self {
            link_form: string_to_link(&it),
            string_form: String::from(it),
        }
    }
}

impl FromStr for StringableLink {
    type Err = &'static str;

    fn from_str(it: &str) -> Result<Self, Self::Err> {
        Ok(StringableLink::from(it))
    }
}

impl Into<AtomLink> for StringableLink {
    fn into(self) -> AtomLink {
        self.link_form
    }
}

impl Into<String> for StringableLink {
    fn into(self) -> String {
        self.string_form
    }
}

fn link_to_string(it: &AtomLink) -> String {
    format!(
        "{}{}{}{}{}",
        it.href(),
        {
            let rel = it.rel();
            if rel == "" {
                "".into()
            } else {
                format!("[rel={}]", rel)
            }
        },
        it.mime_type()
            .map_or("".to_string(), |mime| format!("[type={}]", mime)),
        it.hreflang()
            .map_or("".to_string(), |hl| format!("[lang={}]", hl)),
        it.title()
            .map_or("".to_string(), |title| format!("[title={}]", title)),
    )
}

#[test]
fn link_to_string_behavior() {
    let link1 = AtomLink {
        href: "https://example.com/feed.xml".into(),
        rel: "self".into(),
        hreflang: Some("en-us".into()),
        mime_type: Some("application/atom+xml".into()),
        title: Some("An example feed".into()),
        length: None,
    };

    assert_eq!(
        "https://example.com/feed.xml[rel=self][type=application/atom+xml][lang=en-us][title=An example feed]",
        link_to_string(&link1),
    );

    let link2 = AtomLink {
        href: "https://example.com/feed.xml".into(),
        rel: "".into(),
        hreflang: Some("en-us".into()),
        mime_type: Some("application/atom+xml".into()),
        title: Some("An example feed".into()),
        length: None,
    };

    assert_eq!(
        "https://example.com/feed.xml[type=application/atom+xml][lang=en-us][title=An example feed]",
        link_to_string(&link2),
    );

    let link3 = AtomLink {
        href: "https://example.com/feed.xml".into(),
        rel: "self".into(),
        hreflang: None,
        mime_type: Some("application/atom+xml".into()),
        title: Some("An example feed".into()),
        length: None,
    };

    assert_eq!(
        "https://example.com/feed.xml[rel=self][type=application/atom+xml][title=An example feed]",
        link_to_string(&link3),
    );

    let link4 = AtomLink {
        href: "https://example.com/feed.xml".into(),
        rel: "self".into(),
        hreflang: None,
        mime_type: None,
        title: Some("An example feed".into()),
        length: None,
    };

    assert_eq!(
        "https://example.com/feed.xml[rel=self][title=An example feed]",
        link_to_string(&link4),
    );

    let link5 = AtomLink {
        href: "https://example.com/feed.xml".into(),
        rel: "self".into(),
        hreflang: None,
        mime_type: None,
        title: None,
        length: None,
    };

    assert_eq!(
        "https://example.com/feed.xml[rel=self]",
        link_to_string(&link5),
    );
}

fn string_to_link(it: &str) -> AtomLink {
    let mut link = AtomLink::default();
    let mut rem_input = String::from(it);

    // Some feeds never set a rel, and the default of the atom_syndication
    // crate is "alternate", which I don't entirely agree with: what I see in the wild
    // (admittedly, limited sample set) seems to be that a link with no rel points to
    // the site that the feed describes: which is link=related.
    link.set_rel("related");

    loop {
        // If the string repr doesn't end with a bracket, we're assuming the
        // remainder is a verbatim URL, and don't care what it contains.
        if !rem_input.ends_with(']') {
            debug!("no rbracket: {}", &rem_input);
            link.set_href(rem_input);
            break;
        }

        if let Some(lidx) = rem_input.rfind('[') {
            let eidx = match rem_input.rfind('=') {
                Some(candidate) => {
                    if candidate < lidx {
                        // While we found an opening bracket, it came *after*
                        // the last equals sign, and so this [] pair definitely
                        // doesn't refer to an instruction we can parse. Bye!
                        debug!("= before [: {}", &rem_input);
                        link.set_href(rem_input);
                        break;
                    }
                    candidate
                }
                None => {
                    // No equals sign found, so this [] pair is part of the URL,
                    // not an instruction. Adios!
                    debug!("no equals: {}", &rem_input);
                    link.set_href(rem_input);
                    break;
                }
            };

            // We now have *at least* a [=] blob, which isn't a valid
            // instruction, but is at least something we can start validity
            // checking.
            let rem_len = rem_input.len();
            match rem_input[lidx + 1..rem_len - 1].split_at(eidx - lidx) {
                ("rel=", rel) => link.set_rel(rel),
                ("type=", mime) => link.set_mime_type(mime.to_string()),
                ("title=", title) => link.set_title(title.to_string()),
                ("lang=", lang) => link.set_hreflang(lang.to_string()),
                (key, val) => {
                    // Tag not recognized as anything we can parse, so assume
                    // it's a trailing part of the URL instead.
                    debug!("unparseable instruction: key={} val={}", key, val);
                    link.set_href(rem_input);
                    break;
                }
            }

            // We're completely done with this instruction, remove it from our
            // ever-destructing string, and proceed to the next loop iteration.
            rem_input.truncate(lidx);
        } else {
            // Hm, we have a trailing bracket, but it was never opened.
            // Presumably this is a part of the URL, too. See ya!
            link.set_href(rem_input);
            break;
        }
    }

    link
}

#[test]
fn string_to_link_behavior() {
    let link1 = AtomLink {
        href: "https://example.com/feed.xml".into(),
        rel: "self".into(),
        hreflang: Some("en-us".into()),
        mime_type: Some("application/atom+xml".into()),
        title: Some("An example feed".into()),
        length: None,
    };

    assert_eq!(
        link1,
        string_to_link("https://example.com/feed.xml[rel=self][type=application/atom+xml][lang=en-us][title=An example feed]"),
    );

    let link2 = AtomLink {
        href: "https://example.com/feed.xml".into(),
        rel: "related".into(),
        hreflang: Some("en-us".into()),
        mime_type: Some("application/atom+xml".into()),
        title: Some("An example feed".into()),
        length: None,
    };

    assert_eq!(
        link2,
        string_to_link("https://example.com/feed.xml[type=application/atom+xml][lang=en-us][title=An example feed]"),
    );

    let link3 = AtomLink {
        href: "https://example.com/feed.xml".into(),
        rel: "self".into(),
        hreflang: None,
        mime_type: Some("application/atom+xml".into()),
        title: Some("An example feed".into()),
        length: None,
    };

    assert_eq!(
        link3,
        string_to_link("https://example.com/feed.xml[rel=self][type=application/atom+xml][title=An example feed]"),
    );

    let link4 = AtomLink {
        href: "https://example.com/feed.xml".into(),
        rel: "self".into(),
        hreflang: None,
        mime_type: None,
        title: Some("An example feed".into()),
        length: None,
    };

    assert_eq!(
        link4,
        string_to_link("https://example.com/feed.xml[rel=self][title=An example feed]"),
    );

    let link5 = AtomLink {
        href: "https://example.com/feed.xml".into(),
        rel: "self".into(),
        hreflang: None,
        mime_type: None,
        title: None,
        length: None,
    };

    assert_eq!(
        link5,
        string_to_link("https://example.com/feed.xml[rel=self]"),
    );

    let link6 = AtomLink {
        href: "https://example.com/feed.xml[=]".into(),
        rel: "related".into(),
        hreflang: None,
        mime_type: None,
        title: None,
        length: None,
    };

    assert_eq!(link6, string_to_link("https://example.com/feed.xml[=]"),);

    let link7 = AtomLink {
        href: "https://example.com/feed.xml]".into(),
        rel: "related".into(),
        hreflang: None,
        mime_type: None,
        title: None,
        length: None,
    };

    assert_eq!(link7, string_to_link("https://example.com/feed.xml]"),);

    let link8 = AtomLink {
        href: "https://example.com/feed.xml[]".into(),
        rel: "related".into(),
        hreflang: None,
        mime_type: None,
        title: None,
        length: None,
    };

    assert_eq!(link8, string_to_link("https://example.com/feed.xml[]"),);
}
