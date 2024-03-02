# fmark

## Description

This Rust program uses dmenu, bemenu, rofi, or fzf to manage bookmarks.
The bookmarks are stored in a plain text file, sorted by category and then by title and
the fields are padded into even columns and whitespace is trimmed.

## Features

- View bookmarks
- Go to bookmarks
- Create new bookmarks
- Modify existing bookmarks
- Delete bookmarks

## Build

To build fmark from source installing `rust` and `cargo` are required, then follow these steps:

```shell
git clone --depth 1 https://github.com/vannrr/fmark
cd ./bookmarks
cargo build --release

```

## Usage

```
Usage: fmark [OPTIONS]

This program can search and modify a formatted plain text list of websites.

format:
  {T}{Project's Github} {C}{Development} {U}{https://github.com/vannrr/fmark}


Options:
  -m, --menu             Menu program to use.
                         Supported programs are 'bemenu', 'dmenu', 'rofi', 'fzf'.
                         Default: (bemenu)
  -b, --browser          Browser command URLs will be passed to.
                         Default: (firefox)
  -p, --path             Path to the bookmark file.
                         Default: ($HOME/.bookmarks)
  -r, --rows             Number of rows to show in the menu.
                         Default: (20)
  -h, --help             Show this help message and exit.

Environment Variables:
FMARK_DEFAULT_OPTS       Default options
                         (e.g. '--menu bemenu --rows 20')
```

## License

This software is distributed under the
[MIT license](https://opensource.org/licenses/MIT).
