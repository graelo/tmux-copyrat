# tmux-copyrat

[![crate](https://img.shields.io/crates/v/tmux-copyrat.svg)](https://crates.io/crates/tmux-copyrat)
[![documentation](https://docs.rs/tmux-copyrat/badge.svg)](https://docs.rs/tmux-copyrat)
[![minimum rustc 1.8](https://img.shields.io/badge/rustc-1.56+-blue.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![edition 2021](https://img.shields.io/badge/edition-2021-blue.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
[![tmux 3.x](https://img.shields.io/badge/tmux-3.0+-blue.svg)](https://tmux.github.io)
[![build status](https://github.com/graelo/tmux-copyrat/workflows/main/badge.svg)](https://github.com/graelo/tmux-copyrat/actions)

A tmux-plugin for copy-pasting spans of text from the [tmux] pane's history
into a clipboard.

**Use case**: you're in tmux and press the key binding to highlight, say dates.
This makes `tmux-copyrat` search within tmux's current pane history and
highlight all spans of text which correspond to a date. All spans are displayed
with a one or two key _hint_, which you can then press to copy-paste the span
into the tmux clipboard or the system clipboard. Check out the demo below.

The name is a tribute to [tmux-copyrat], which I used for many years.


## Demo

[![demo](https://asciinema.org/a/232775.png?ts=1)](https://asciinema.org/a/232775?autoplay=1)


## Usage

Restart tmux after the plugin is installed and configured (see both
[INSTALLATION.md] and [CONFIGURATION.md] pages). Press one of the pre-defined
tmux key-bindings (see table below) in order to highlight spans of text
matching a specific pattern. To yank some text span in the tmux buffer, press
the corresponding _hint_, or press <kbd>Esc</kbd> to cancel and exit.

If instead you want to yank the text span into the system clipboard, either
press the caps version of the key hint (for instance <kbd>E</kbd> instead of
<kbd>e</kbd>), or first toggle the destination buffer with the <kbd>space</kbd>
key and press the hint with no caps.

You can also use the <kbd>n</kbd> and <kbd>p</kbd> (or <kbd>Up</kbd> and
<kbd>Down</kbd>) keys to move focus across the highlighted spans. Press
<kbd>y</kbd> to yank the focused span into the tmux buffer, or press
<kbd>Y</kbd> to yank it into the system clipboard.

By default, span highlighting starts from the bottom of the terminal, but you
can reverse that behavior with the `--reverse` option (more on that in the
[Configuration.md] page). The `--focus wrap-around` option makes navigation
go back to the first span.


### Matched patterns and default key-bindings

tmux-copyrat can match one or more pre-defined (named) patterns, but you can
add your own too.

The default configuration provided in the [`copyrat.tmux`](copyrat.tmux) plugin
file provides the following key-bindings. Because they all start with
<kbd>prefix</kbd> + <kbd>t</kbd>, the table below only lists the keyboard key
that comes after. For instance, for URLs, the key is <kbd>u</kbd>, but you
should type <kbd>prefix</kbd> + <kbd>t</kbd> + <kbd>u</kbd>.

| key binding      | searches for                           | pattern name      |
| ---              | ---                                    | ---               |
| <kbd>c</kbd>     | Hex color codes                        | `hexcolor`        |
| <kbd>d</kbd>     | Dates or datetimes                     | `datetime`        |
| <kbd>D</kbd>     | Docker/Podman IDs                      | `docker`          |
| <kbd>e</kbd>     | Emails                                 | `email`           |
| <kbd>G</kbd>     | String of 4+ digits                    | `digits`          |
| <kbd>h</kbd>     | SHA-1/-2 short & long                  | `sha`             |
| <kbd>m</kbd>     | Markdown URLs `[..](matched-url)`      | `markdown-url`    |
| <kbd>p</kbd>     | Abs. and rel. filepaths                | `path`            |
| <kbd>P</kbd>     | Hex numbers and pointer addresses      | `pointer-address` |
|                  | strings inside single quotes           | `quoted-single`   |
|                  | strings inside double quotes           | `quoted-double`   |
|                  | strings inside backticks               | `quoted-tick`     |
| <kbd>q</kbd>     | strings inside single/double/backticks |                   |
| <kbd>u</kbd>     | URLs                                   | `url`             |
| <kbd>U</kbd>     | UUIDs                                  | `uuid`            |
| <kbd>v</kbd>     | version numbers                        | `version`         |
| <kbd>4</kbd>     | IPv4 addresses                         | `4`               |
| <kbd>6</kbd>     | IPv6 addresses                         | `6`               |
| <kbd>space</kbd> | All patterns                           |                   |

If you want additional patterns, you can provide them via the
`--custom-pattern` command line option (short option: `-X`), see
[CONFIGURATION.md].


## The `copyrat` companion executable

The central binary of this crate is `tmux-copyrat`, however there is also the
`copyrat` executable. It simply provides the same functionality, without any
tmux dependency or integration.

You can use `copyrat` to search a span of text that you provide to stdin.

For instance here is a bunch of text, with dates and git hashes which you can
search with copyrat.

```console
$ echo -n '* e006b06 - (12 days ago = 2021-03-04T12:23:34) e006b06 e006b06 swapper: Make quotes\n/usr/local/bin/git\n\nlorem\n/usr/local/bin\nlorem\n/usr/local/bin/git\n* e006b06 - (12 days ago = 2021-03-04T12:23:34) e006b06 e006b06 swapper: Make quotes' \
    | ./target/release/copyrat -r --unique-hint -s bold -X '(loca)' -x sha datetime
```

You will see the following in your terminal

![[copyrat-output.png](images/copyrat-output.png)](images/copyrat-output.png)

You may have noticed that all identical spans share the same _hint_, this is
due to the `-unique-hint` option (`-u`). The hints are in bold text, due to the `--hint-style bold` option (`-s`).


## Tmux compatibility

This is the known list of versions of `tmux` compatible with `tmux-thumbs`:

| Version | Compatible |
|:-------:|:----------:|
|   3.0+  |     ✅     |
|   2.9a  |     ✅     |
|   2.8   |      ❓    |
|   2.7   |      ❓    |
|   2.6   |     ✅     |
|   2.5   |      ❓    |
|   2.4   |      ❓    |
|   2.3   |      ❓    |
|   1.8   |      ❓    |
|   1.7   |      ❓    |

Please report incompatibilities as you find them, I'll add them to the list.


## Standalone `thumbs`

This project started as a `tmux` plugin but after reviewing it with some
friends we decided to explore all the possibilities of decoupling thumbs from
`tmux`. You can install it with a simple command:

```
cargo install thumbs
```

And those are all available options:

```
thumbs 0.4.1
A lightning fast version copy/pasting like vimium/vimperator

USAGE:
    thumbs [FLAGS] [OPTIONS]

FLAGS:
    -c, --contrast    Put square brackets around hint for visibility
    -h, --help        Prints help information
    -m, --multi       Enable multi-selection
    -r, --reverse     Reverse the order for assigned hints
    -u, --unique      Don't show duplicated hints for the same span
    -V, --version     Prints version information

OPTIONS:
    -a, --alphabet <alphabet>                          Sets the alphabet [default: qwerty]
        --bg-color <background_color>                  Sets the background color for spans [default: black]
        --fg-color <foreground_color>                  Sets the foregroud color for spans [default: green]
    -f, --format <format>
            Specifies the out format for the picked hint. (%U: Upcase, %H: Hint) [default: %H]

        --hint-bg-color <hint_background_color>        Sets the background color for hints [default: black]
        --hint-fg-color <hint_foreground_color>        Sets the foregroud color for hints [default: yellow]
    -p, --position <position>                          Hint position [default: left]
    -x, --regexp <regexp>...                           Use this regexp as extra pattern to match
        --select-bg-color <select_background_color>    Sets the background color for selection [default: black]
        --select-fg-color <select_foreground_color>    Sets the foreground color for selection [default: blue]
    -t, --target <target>                              Stores the hint in the specified path
```


If you want to enjoy terminal hints, you can do things like this without `tmux`:

```
> alias pick='thumbs -u -r | xsel --clipboard -i'
> git log | pick
```

Or multi selection:

```
> git log | thumbs -m
1df9fa69c8831ac042c6466af81e65402ee2a007
4897dc4ecbd2ac90b17de95e00e9e75bb540e37f
```

Standalone `thumbs` has some similarities to [FZF](https://github.com/junegunn/fzf).

## Background

As I said, this project is based in [tmux-fingers](https://github.com/Morantron/tmux-fingers). Morantron did an extraordinary job, building all necessary pieces in Bash to achieve the text picker behaviour. He only deserves my gratitude for all the time I have been using [tmux-fingers](https://github.com/Morantron/tmux-fingers).

During a [Fosdem](https://fosdem.org/) conf, we had the idea to rewrite it to another language. He had these thoughts many times ago but it was hard to start from scratch. So, we decided to start playing with Node.js and [react-blessed](https://github.com/Yomguithereal/react-blessed), but we detected some unacceptable latency when the program booted. We didn't investigate much about this latency.

During those days another alternative appeared, called [tmux-picker](https://github.com/RTBHOUSE/tmux-picker), implemented in python and reusing many parts from [tmux-fingers](https://github.com/Morantron/tmux-fingers). It was nice, because it was fast and added original terminal color support.

I was curious to know if this was possible to be written in [Rust](https://www.rust-lang.org/), and soon I realized that was something doable. The ability to implement tests for all critic parts of the application give you a great confidence about it. On the other hand, Rust has an awesome community that lets you achieve this kind of project in a short period of time.


## Run code-coverage

Install the llvm-tools-preview component and grcov

```sh
rustup component add llvm-tools-preview
cargo install grcov
```

Install nightly

```sh
rustup toolchain install nightly
```

The following make invocation will switch to nigthly run the tests using
Cargo, and output coverage HTML report in `./coverage/`

```sh
make coverage
```

The coverage report is located in `./coverage/index.html`



## License

Licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[tmux]: https://tmux.github.io
[tmux-copyrat]: https://github.com/tmux-plugins/tmux-copycat
[CONFIGURATION.md]: CONFIGURATION.md
[INSTALLATION.md]: INSTALLATION.md
