# Configuration

If you want to customize how is shown your tmux-copyrat hints those all available
parameters to set your perfect profile.

NOTE: for changes to take effect, you'll need to source again your `.tmux.conf` file.

- [@copyrat-key](#thumbs-key)
- [@copyrat-alphabet](#thumbs-alphabet)
- [@copyrat-reverse](#thumbs-reverse)
- [@copyrat-unique](#thumbs-unique)
- [@copyrat-position](#thumbs-position)
- [@copyrat-regexp-N](#thumbs-regexp-N)
- [@copyrat-command](#thumbs-command)
- [@copyrat-upcase-command](#thumbs-upcase-command)
- [@copyrat-bg-color](#thumbs-bg-color)
- [@copyrat-fg-color](#thumbs-fg-color)
- [@copyrat-hint-bg-color](#thumbs-hint-bg-color)
- [@copyrat-hint-fg-color](#thumbs-hint-fg-color)
- [@copyrat-select-fg-color](#thumbs-select-fg-color)
- [@copyrat-select-bg-color](#thumbs-select-bg-color)
- [@copyrat-contrast](#thumbs-contrast)

### @thumbs-key

`default: space`

Choose which key is used to enter in thumbs mode.

For example:

```
set -g @thumbs-key F
```

### @thumbs-alphabet

`default: qwerty`

Choose which set of characters is used to build hints. Review all [available alphabets](#Alphabets)

For example:

```
set -g @thumbs-alphabet dvorak-homerow
```

### @thumbs-reverse

`default: disabled`

Choose in which direction you want to assign hints. Useful to get shorter hints closer to the cursor.

For example:

```
set -g @thumbs-reverse
```

### @thumbs-unique

`default: disabled`

Choose if you want to assign the same hint for the same text spans.

For example:

```
set -g @thumbs-unique
```

### @thumbs-position

`default: left`

Choose where do you want to show the hint in the text spans. Options (left, right).

For example:

```
set -g @thumbs-position right
```

### @thumbs-regexp-N

Add extra patterns to match. This parameter can have multiple instances.

For example:

```
set -g @thumbs-regexp-1 '[a-z]+@[a-z]+.com' # Match emails
set -g @thumbs-regexp-2 '[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:' # Match MAC addresses
```

### @thumbs-command

`default: 'tmux set-buffer {}'`

Choose which command execute when you press a hint. `tmux-thumbs` will replace `{}` with the picked hint.

For example:

```
set -g @thumbs-command 'echo -n {} | pbcopy'
```

### @thumbs-upcase-command

`default: 'tmux set-buffer {} && tmux paste-buffer'`

Choose which command execute when you press a upcase hint. `tmux-thumbs` will replace `{}` with the picked hint.

For example:

```
set -g @thumbs-upcase-command 'echo -n {} | pbcopy'
```

### @thumbs-bg-color

`default: black`

Sets the background color for spans

For example:

```
set -g @thumbs-bg-color blue
```

### @thumbs-fg-color

`default: green`

Sets the foreground color for spans

For example:

```
set -g @thumbs-fg-color green
```

### @thumbs-hint-bg-color

`default: black`

Sets the background color for hints

For example:

```
set -g @thumbs-hint-bg-color blue
```

### @thumbs-hint-fg-color

`default: yellow`

Sets the foreground color for hints

For example:

```
set -g @thumbs-hint-fg-color green
```

### @thumbs-select-fg-color

`default: blue`

Sets the foreground color for selection

For example:

```
set -g @thumbs-select-fg-color red
```

### @thumbs-select-bg-color

`default: black`

Sets the background color for selection

For example:

```
set -g @thumbs-select-bg-color red
```

### @thumbs-contrast

`default: 0`

Displays hint character in square brackets for extra visibility.

For example:

```
set -g @thumbs-contrast 1
```

#### Colors

This is the list of available colors:

- black
- red
- green
- yellow
- blue
- magenta
- cyan
- white
- default

#### Alphabets

This is the list of available alphabets:

- `qwerty`: asdfqwerzxcvjklmiuopghtybn
- `qwerty-homerow`: asdfjklgh
- `qwerty-left-hand`: asdfqwerzcxv
- `qwerty-right-hand`: jkluiopmyhn
- `azerty`: qsdfazerwxcvjklmuiopghtybn
- `azerty-homerow`: qsdfjkmgh
- `azerty-left-hand`: qsdfazerwxcv
- `azerty-right-hand`: jklmuiophyn
- `qwertz`: asdfqweryxcvjkluiopmghtzbn
- `qwertz-homerow`: asdfghjkl
- `qwertz-left-hand`: asdfqweryxcv
- `qwertz-right-hand`: jkluiopmhzn
- `dvorak`: aoeuqjkxpyhtnsgcrlmwvzfidb
- `dvorak-homerow`: aoeuhtnsid
- `dvorak-left-hand`: aoeupqjkyix
- `dvorak-right-hand`: htnsgcrlmwvz
- `colemak`: arstqwfpzxcvneioluymdhgjbk
- `colemak-homerow`: arstneiodh
- `colemak-left-hand`: arstqwfpzxcv
- `colemak-right-hand`: neioluymjhk

