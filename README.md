# kaboom(1): Atom feed management for the casual blogger.

> ... or whatever-er.

`kaboom` provides the absolute basics of authoring an
[Atom feed](https://en.wikipedia.org/wiki/Atom_(web_standard)) as an XML file on
disk, targeted towards DIY bloggers using static site generators (or just plain
old HTML files) that don't already generate Atom feeds. However, it may be
generally useful for other Atom feed purposes (for example, I've tested some of
the basic utilities against
[Environment Canada's marine weather alert feeds](https://weather.gc.ca/marine/forecast_e.html?mapID=02&siteID=06100)).

## Installation

> Void Linux users, `xbps-install -S kaboom` is all you need to do. I maintain
> that package, so it should reasonably-always-ish be up to date.

- Make sure you have a working `cargo` (which comes with Rust via your
  distribution or `rustup` or whatever)
- Clone the repo: `git clone https://git.sr.ht/~klardotsh/kaboom`
- Build and install it: `cargo install --path .`

## Usage

For the most part, the output of `kaboom --help` (and then the `--help` of each
of the subcommands) should get you going. If not, send me an email (or ping me
on Mastodon, or whatever) and let me know what troubles you had so I can try to
make the built-in help/documentation better. My contact info can be found [on my
website](https://klar.sh/contact.html).

Here's a snapshot of that output, though, in case you're just driving by:

<details>
<summary>kaboom --help</summary>
<pre>
Usage: kaboom [-f <file>] [-n] <command> [<args>]

Manage an on-disk Atom feed's entries.

Options:
  -f, --file        path to Atom feed
  -n, --no-op       do not write anything to disk, but still show what *would*
                    change
  --help            display usage information

Commands:
  add               Add entries to the feed. If *content* is supplied, its
                    source is assumed to be the same URI as *id*.
  meta              Manage the metadata of the Atom feed, for example the
                    authors or the title. Arguments provided here will set or
                    modify the metadata. After any modifications (with no flags,
                    no modifications will be made), the new state of the feed's
                    metadata will be dumped to standard output (by default in a
                    human-friendly format, but JSON is planned later).
  prune             Remove entries from the Atom feed, and by default send the
                    deleted entries to a reject file for backup/archival
                    purposes.
  version           Display version info and exit.
</pre>
</details>

<details>
<summary>kaboom add --help</summary>
<pre>
Usage: kaboom add <id> <title> [-s <summary>] [-c <content>] [-T <content-type>] [-L <content-language>] [-a <author-names...>] [-A <author-emails...>] [-d <published-at>] [-D <updated-at>]

Add entries to the feed. If *content* is supplied, its source is assumed to be the same URI as *id*.

Positional Arguments:
  id                the URI of the entry
  title             the title of the entry

Options:
  -s, --summary     a short summary of the entry
  -c, --content     the full content of the entry
  -T, --content-type
                    the content type of *content*; must be "text", "html",
                    "xhtml", or a MIME type. ignored if *content* is not
                    provided.
  -L, --content-language
                    the language of *content*, often a code like en-us. ignored
                    if *content* is not provided
  -a, --author-names
                    name(s) of authors associated with this entry. can be
                    specified multiple times, must have the same number of names
                    and emails.
  -A, --author-emails
                    email address(es) of authors associated with this entry. can
                    be specified multiple times, must have the same number of
                    names and emails.
  -d, --published-at
                    the date and time, in RFC3339 format, when the entry was
                    published
  -D, --updated-at  the date, in RFC3339 format, when the entry was most
                    recently updated
  --help            display usage information
</pre>
</details>

<details>
<summary>kaboom meta --help</summary>
<pre>
Usage: kaboom meta [-t <title>] [-u <uri>] [-r <rel-link...>] [-R] [-i <icon>] [-I] [-l <logo>] [-L] [-s <subtitle>] [-S] [-G]

Manage the metadata of the Atom feed, for example the authors or the title. Arguments provided here will set or modify the metadata. After any modifications (with no flags, no modifications will be made), the new state of the feed's metadata will be dumped to standard output (by default in a human-friendly format, but JSON is planned later).

Options:
  -t, --title       a human-readable title for the feed (this must be set the
                    first time `kaboom meta` is called on a new file)
  -u, --uri         a unique and permanent URI for this feed, often the URL at
                    which it is accessed (this must be set the first time
                    `kaboom meta` is called on a new file)
  -r, --rel-link    a web page URL related to the feed, can be provided multiple
                    times. suffixes in the format of [rel=XXX], [type=XXX],
                    [title=XXX], and [lang=XXX] are all supported, for example:
                    https://www.meteo.gc.ca/
                    rss/marine/06100_f.xml[rel=alternate][lang=fr-ca][type=application/
                    atom+xml][title=Détroit de Haro - Météo maritime -
                    Environnement Canada]
  -R, --remove-links
                    ensure that no links (except rel=self) are set in this
                    feed's metadata. if *rel_link* are still provided, this flag
                    will instead clear all *existing* links, and add those links
                    as the only links in the metadata.
  -i, --icon        an optional URL pointing to a small image providing visual
                    identification for the feed (think like a favicon)
  -I, --remove-icon ensure that the icon field is not set in this feed's
                    metadata. ignored if *icon* is still provided.
  -l, --logo        an optional URL pointing to a larger image providing visual
                    identification for the feed (this is distinct from *icon*
                    above!)
  -L, --remove-logo ensure that the logo field is not set in this feed's
                    metadata. ignored if *logo* is still provided.
  -s, --subtitle    an optional string to provide a human-readable description
                    or subtitle for the feed
  -S, --remove-subtitle
                    ensure that the subtitle field is not set in this feed's
                    metadata. ignored if *subtitle* is still provided.
  -G, --no-generator
                    do not insert the generator block into the metadata output
                    (which discloses within the feed that kaboom was used to
                    generate it)
  --help            display usage information
</pre>
</details>

<details>
<summary>kaboom prune --help</summary>
<pre>
Usage: kaboom prune <count> [-R <no-reject>] [-r <reject-file>] [-s <strategy>] [-d <since>]

Remove entries from the Atom feed, and by default send the deleted entries to a reject file for backup/archival purposes.

Positional Arguments:
  count             number of entries to keep in the feed, as sorted by
                    *strategy*, described below

Options:
  -R, --no-reject   skip sending pruned entries to the *reject_file*, described
                    below
  -r, --reject-file path to an Atom file (which will be created if it does not
                    yet exist, sharing all metadata from the original feed) to
                    store pruned entries for backup/archival purposes. by
                    default, this will be <feed file> with any .xml extension
                    removed, and then ".rej.xml" added
  -s, --strategy    strategy used in pruning entries from the feed: published,
                    for date of publication, updated, for date of most recent
                    update, or since- date, which preserves only those articles
                    authored since *since-date*, described below
  -d, --since       a date and time, in RFC3339 format, used only with the
                    since-date *strategy*, described above
  --help            display usage information
</pre>
</details>

## An example

Let's say I wanted to create a whole new Atom feed for my brand-spankin'-new
blog, from scratch. The blog has just one post, so far, but I really want my
friends to know when the next one's coming. Cool!

```
# kaboom meta -t "klardotsh's super awesome blog" -u "https://example.com/feed.xml" -r "https://example.com[rel=related]" -i 'https://example.com/favicon.ico'   
title=klardotsh's super awesome blog
uri=https://example.com/feed.xml
updated_at=2023-07-16 01:23:52.168573852 +00:00
icon=https://example.com/favicon.ico
link=https://example.com[rel=related]

# kaboom add https://example.com/001-foobar.html "001: Foobar" -s "It's like a normal bar, but instead of serving ciders, they serve foo. Dave Grohl then comes in and fights said foo. And then everybody clapped." -a 'klardotsh' -d 2023-07-15T18:30:00-07:00 -A 'klardotsh@example.com'
## There's no output here because the command succeeded, which we can assert with `echo $?`
## Now, what's in that XML feed?

# xml_pp feed.xml
<?xml version="1.0"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>klardotsh&apos;s super awesome blog</title>
  <id>https://example.com/feed.xml</id>
  <updated>2023-07-16T01:23:52.168573852+00:00</updated>
  <generator uri="https://sr.ht/~klardotsh/kaboom" version="0.1.0">kaboom</generator>
  <icon>https://example.com/favicon.ico</icon>
  <link href="https://example.com" rel="related"/>
  <entry>
    <title>001: Foobar</title>
    <id>https://example.com/001-foobar.html</id>
    <updated>2023-07-16T01:29:58.501136355+00:00</updated>
    <contributor>
      <name>klardotsh</name>
      <email>klardotsh@example.com</email>
    </contributor>
    <published>2023-07-16T01:30:00+00:00</published>
    <summary>It&apos;s like a normal bar, but instead of serving ciders, they serve foo. Dave Grohl then comes in and fights said foo. And then everybody clapped.</summary>
  </entry>
</feed>
```

Well would ya look at that. Hand that feed URL to all your buddies and
they'll be entertained whenever you publish the next tall tale of running into
celebrities in punny situations.

### What if I mess up? How do I remove things?

Not implemented yet. You'll have to go hand-remove from the XML for now, though
this is high on my TODO list.

## Legal Bullshit

Look dude, this entire project, less comments and blank lines, is well under
a thousand lines of Rust at time of writing this paragraph, I really don't
care what you do with it, I just hope you have fun with it if you happen to
use it. See [COPYING](/COPYING) for the
[Zero-Clause BSD License](https://www.tldrlegal.com/license/bsd-0-clause-license),
which is maximally-permissive and as close as I can get to a public domain
dedication without pissing the EU and Fedora and a bunch of other entities off.
