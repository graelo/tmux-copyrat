extern crate clap;
extern crate termion;

mod alphabets;
mod colors;
mod state;
mod view;

use self::clap::{App, Arg};
use clap::crate_version;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{self, Read};

// TODO: position as an enum ::Leading ::Trailing

fn app_args<'a>() -> clap::ArgMatches<'a> {
  App::new("thumbs")
    .version(crate_version!())
    .about("A lightning fast version copy/pasting like vimium/vimperator")
    .arg(
      Arg::with_name("alphabet")
        .help("Sets the alphabet")
        .long("alphabet")
        .short("a")
        .default_value("qwerty"),
    )
    .arg(
      Arg::with_name("format")
        .help("Specifies the out format for the picked hint. (%U: Upcase, %H: Hint)")
        .long("format")
        .short("f")
        .default_value("%H"),
    )
    .arg(
      Arg::with_name("foreground_color")
        .help("Sets the foregroud color for matches")
        .long("fg-color")
        .default_value("green"),
    )
    .arg(
      Arg::with_name("background_color")
        .help("Sets the background color for matches")
        .long("bg-color")
        .default_value("black"),
    )
    .arg(
      Arg::with_name("hint_foreground_color")
        .help("Sets the foregroud color for hints")
        .long("hint-fg-color")
        .default_value("yellow"),
    )
    .arg(
      Arg::with_name("hint_background_color")
        .help("Sets the background color for hints")
        .long("hint-bg-color")
        .default_value("black"),
    )
    .arg(
      Arg::with_name("select_foreground_color")
        .help("Sets the foreground color for selection")
        .long("select-fg-color")
        .default_value("blue"),
    )
    .arg(
      Arg::with_name("select_background_color")
        .help("Sets the background color for selection")
        .long("select-bg-color")
        .default_value("black"),
    )
    .arg(
      Arg::with_name("multi")
        .help("Enable multi-selection")
        .long("multi")
        .short("m"),
    )
    .arg(
      Arg::with_name("reverse")
        .help("Reverse the order for assigned hints")
        .long("reverse")
        .short("r"),
    )
    .arg(
      Arg::with_name("unique")
        .help("Don't show duplicated hints for the same match")
        .long("unique")
        .short("u"),
    )
    .arg(
      Arg::with_name("position")
        .help("Hint position")
        .long("position")
        .default_value("left")
        .short("p"),
    )
    .arg(
      Arg::with_name("regexp")
        .help("Use this regexp as extra pattern to match")
        .long("regexp")
        .short("x")
        .takes_value(true)
        .multiple(true),
    )
    .arg(
      Arg::with_name("contrast")
        .help("Put square brackets around hint for visibility")
        .long("contrast")
        .short("c"),
    )
    .arg(
      Arg::with_name("target")
        .help("Stores the hint in the specified path")
        .long("target")
        .short("t")
        .takes_value(true),
    )
    .get_matches()
}

fn main() {
  let args = app_args();
  let format = args.value_of("format").unwrap();
  let alphabet = args.value_of("alphabet").unwrap();
  let position = args.value_of("position").unwrap();
  let target = args.value_of("target");
  let multi = args.is_present("multi");
  let reverse = args.is_present("reverse");
  let unique = args.is_present("unique");
  let contrast = args.is_present("contrast");
  let regexp = if let Some(items) = args.values_of("regexp") {
    items.collect::<Vec<_>>()
  } else {
    [].to_vec()
  };

  let foreground_color = colors::get_color(args.value_of("foreground_color").unwrap());
  let background_color = colors::get_color(args.value_of("background_color").unwrap());
  let hint_foreground_color = colors::get_color(args.value_of("hint_foreground_color").unwrap());
  let hint_background_color = colors::get_color(args.value_of("hint_background_color").unwrap());
  let select_foreground_color = colors::get_color(args.value_of("select_foreground_color").unwrap());
  let select_background_color = colors::get_color(args.value_of("select_background_color").unwrap());

  // Copy the pane contents (piped in via stdin) into a buffer, and split lines.
  let mut buffer = String::new();
  let stdin = io::stdin();
  let mut handle = stdin.lock();

  handle.read_to_string(&mut buffer).unwrap();

  let lines: Vec<&str> = buffer.split('\n').collect();

  let mut state = state::State::new(&lines, alphabet, &regexp);

  let rendering_edge = if position == "left" {
    view::RenderingEdge::Leading
  } else {
    view::RenderingEdge::Trailing
  };

  let rendering_colors = colors::RenderingColors {
    focus_fg_color: select_foreground_color,
    focus_bg_color: select_background_color,
    normal_fg_color: foreground_color,
    normal_bg_color: background_color,
    hint_fg_color: hint_foreground_color,
    hint_bg_color: hint_background_color,
  };

  let contrast_style = if contrast {
    Some(view::ContrastStyle::Surrounded('[', ']'))
  } else {
    None
  };

  let selections = {
    let mut viewbox = view::View::new(
      &mut state,
      multi,
      reverse,
      unique,
      rendering_edge,
      &rendering_colors,
      contrast_style,
    );

    viewbox.present()
  };

  // Early exit, signaling tmux we had no selections.
  if selections.is_empty() {
    ::std::process::exit(1);
  }

  let output = selections
    .iter()
    .map(|(text, upcase)| {
      let upcase_value = if *upcase { "true" } else { "false" };

      let mut output = format.to_string();

      output = str::replace(&output, "%U", upcase_value);
      output = str::replace(&output, "%H", text.as_str());
      output
    })
    .collect::<Vec<_>>()
    .join("\n");

  match target {
    None => println!("{}", output),
    Some(target) => {
      let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(target)
        .expect("Unable to open the target file");

      file.write(output.as_bytes()).unwrap();
    }
  }
}
