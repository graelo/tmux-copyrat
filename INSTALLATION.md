# Installation

## A note on extending tmux functionality

Extending [tmux] functionality is easy: it boils down to adding key-bindings
which call internal commands or external programs that you provide.

The minimalistic way to add functionality is simply to add your key-bindings
directly inside `~/.tmux.conf`. I don't do this because it ends up being messy,
but it's still an option described below.

In contrast, the most flexible way to extend is via plugins. You declare your
bindings in a plugin file often located in
`~/.tmux/plugins/your-plugin-dir/plugin-file.tmux`, and optionally provide your
external programs in the same folder or elsewhere. You then simply ask tmux to
_run_ your plugin file by adding `run-shell
~/.tmux/plugins/your-plugin-dir/plugin-file.tmux` inside your `tmux.conf`. Your
key-bindings will be registered on tmux initial start.

[TPM], the tmux plugin manager, is an integrated way of doing the same. It adds
a level of indirection: when tmux first starts, it runs TPM, which asks tmux to
_run_ all the plugin files in `~/.tmux/plugins/**/` as executables.
When run, each plugin file registers their key-bindings with tmux. TPM also has
an installation mechanism for plugins.


## Minimalistic installation

As described above, a valid option is to ignore the [`copyrat.tmux`] plugin
file simply add a few key bindings to tmux. You just have to create
key-bindings which launch the `tmux-copyrat` binary with its command line
options. Notice you probably need the absolute path to the binary.

However, when creating your bindings, avoid using `run-shell` to run `tmux-copyrat`
because by design tmux launches processes without attaching them to a pty.
Take inspiration from [`copyrat.tmux`] for correct syntax.


## Standard installation (recommended)

The easiest way to install is to copy the config file [`copyrat.tmux`] into `~/.tmux/plugins/tmux-copyrat/` and tell tmux to source it either via

- sourcing it directly from your `~/.tmux.conf`: you simply add the line `source-file ~/.tmux/plugins/tmux-copyrat/copyrat.tmux`
- or, if you use [TPM], registering it with TPM in your `~/.tmux.conf`: you simply add the line


```tmux
set -g @tpm_plugins '              \
  tmux-plugins/tpm                 \
  tmux-plugins/tmux-copyrat        \  <- line added
  tmux-plugins/tmux-yank           \
  ...
```


second style of tmux integration is more declarative: you configure the tmux key bindings to pass none or very few command line arguments to `tmux-copyrat`, and ask `tmux-copyrat` to query back tmux for the rest of the configuration.



## Tmux integration

Clone the repo:

```
git clone https://github.com/graelo/tmux-copyrat ~/.tmux/plugins/tmux-copyrat
```

Compile it with [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```
cd ~/.tmux/plugins/tmux-copyrat
cargo build --release
```

Source it in your `.tmux.conf`:

```
run-shell ~/.tmux/plugins/tmux-copyrat/copyrat.tmux
```

Reload TMUX conf by running:

```
tmux source-file ~/.tmux.conf
```

## Using Tmux Plugin Manager

You can add this line to your list of [TPM](https://github.com/tmux-plugins/tpm) plugins in `.tmux.conf`:

```
set -g @plugin 'graelo/tmux-copyrat'
```

To be able to install the plugin just hit <kbd>prefix</kbd> + <kbd>I</kbd>. You should now be able to use
the plugin!

[`copyrat.tmux`]: https://raw.githubusercontent.com/graelo/tmux-copyrat/main/copyrat.tmux
[tmux]: https://tmux.github.io
[TPM]: https://github.com/tmux-plugins/tpm
