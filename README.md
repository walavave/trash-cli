# trash-cli

Only Works on macOS (Already tested on macOS 26) 

`walavave/trash-cli` is a Rust trash utility focused on macOS trash locations.
It implements a file-system-based workflow for trashing, listing, restoring,
emptying, and selectively removing trashed files without relying on Finder or
AppleScript.

## Features

- Supports the command:
  `put`, `list`, `restore`, `empty`, and `rm`
- Reads native macOS Trash metadata from `.DS_Store`
- Writes new trashed entries back into the native macOS Trash structure
- Works with home trash, top-level volume trash, mounted volume trash, and
  custom trash roots
- Supports restore overwrite control and interactive multi-selection

## Install

**Recommend**: Get the binary release, untar it, and move it somewhere on your $PATH (only support Mac with arm chip)

```sh
tar -xzf "trash-cli-${VERSION}-aarch64-apple-darwin.tar.gz"
sudo install -m 755 "trash-cli-${VERSION}-aarch64-apple-darwin/trash" /usr/local/bin/trash
```
When you run the command for the first time, such as `trash --version`, a system warning will pop up. Go to System Settings > Privacy & Security, then scroll to the bottom and click `Allow Anyway`.

or build it with cargo:

```sh
cargo install --path . --locked
```

macOS users can install it with Homebrew:

```sh
brew tap walavave/tap
brew install --formula walavave/tap/trash-cli
```

## Usage

### Command Overview

```text
trash list [--sort date|path|none] [--trash-dir DIR] [PATH]
trash restore [--sort date|path|none] [--trash-dir DIR] [--overwrite] [PATH]
trash put [--trash-dir DIR] FILE...
trash empty [--trash-dir DIR] [DAYS]
trash rm [--trash-dir DIR] PATTERN
```

If no command is provided, the tool shows help.

Global options:

- `-h`, `--help` show usage
- `--version` show version

`rm PATTERN`:

- If `PATTERN` starts with `/`, it matches the full original path
- Otherwise, it matches only the basename
- Supported wildcards are `*` and `?`
- Quote the pattern to prevent shell expansion

### Command Details

#### `put`

Move one or more files or directories into the trash.

```sh
trash put ./foo.txt ./build.log
trash put ./dir-a ./dir-b
```

- If a trashed name already exists inside the target trash root, the tool will
  create a unique name such as `name_1`, `name_2`, and so on
- This command updates native `.DS_Store` metadata for the files it trashes

#### `list`

List trashed files.

```sh
trash list
trash list ./src
trash list --sort path
```

Output format:

```text
YYYY-MM-DD HH:MM:SS /original/path
```

- When a `PATH` is provided, only entries whose original location matches or is
inside that path are shown.
- **Date and time shown are the last modification time of files, NOT the deletion time**.


#### `restore`

Interactively restore trashed files.

```sh
trash restore
trash restore ./src
trash restore --overwrite ./src
```

- matching entries are displayed with zero-based indexes
- input accepts a single index, comma-separated indexes, or ranges such as
  `0,2-4`
- pressing Enter without a selection restores nothing

#### `empty`

Delete trashed items permanently.

```sh
trash empty
trash empty 7
```

- without `DAYS`, all discovered trashed items are removed
- with `DAYS`, only items trashed at least that many days ago are removed

#### `rm`

Delete matching trashed items permanently.

```sh
trash rm *.o
trash rm /workspace/tmp/*
```

## How to build

```sh
cargo build
```

Build the installable binary named `trash`:

```sh
cargo build --release
./target/release/trash --version
```

Run the test suite:

```sh
cargo test
```

## Related

If you are using Linux, you should try [trash-cli](https://github.com/andreafrancia/trash-cli)