# trash-cli-macos

`trash-cli-macos` is a Rust trash utility focused on macOS trash locations.
It implements a file-system-based workflow for trashing, listing, restoring,
emptying, and selectively removing trashed files without relying on Finder or
AppleScript.

For Simplified Chinese documentation, see [README_zh-CN.md](README_zh-CN.md).

## Features

- Supports the command set commonly associated with `trash-cli`:
  `put`, `list`, `restore`, `empty`, and `rm`
- Accepts both short subcommands and upstream-style aliases:
  `trash-put`, `trash-list`, `trash-restore`, `trash-empty`, `trash-rm`
- Reads native macOS Trash metadata from `.DS_Store`
- Writes new trashed entries back into the native macOS Trash structure
- Works with home trash, top-level volume trash, mounted volume trash, and
  custom trash roots
- Supports restore overwrite control and interactive multi-selection

## Status

This project intentionally does not call Finder APIs. All operations are
performed directly against native macOS Trash directories:

- trashed files are moved into the native Trash directory itself
- original-location metadata is stored in `.DS_Store`

This keeps the implementation scriptable while matching the native macOS Trash
layout instead of introducing a custom side directory.

## Supported Locations

- `~/.Trash`
- `/.Trashes/<uid>`
- `/Volumes/*/.Trashes/<uid>`
- custom `--trash-dir DIR` roots

Custom trash roots can contain either:

- native macOS Trash entries stored directly in the root directory
- native macOS metadata via `.DS_Store`

## Build

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

## Homebrew

The installed command name is `trash`.

Recommended distribution model:

1. Keep this project source in a normal repository
2. Publish a Homebrew tap repository, for example `homebrew-tap`
3. Put a formula file at `Formula/trash-cli-macos.rb`
4. Install the formula from that tap, but run the binary as `trash`

A formula template is included at:

- [Formula/trash-cli-macos.rb](Formula/trash-cli-macos.rb)

Before publishing it, replace:

- `OWNER/REPO` with your real GitHub repository path
- `sha256` with the checksum of the tagged release tarball

Typical release flow:

```sh
git tag v0.1.0
git push origin v0.1.0
shasum -a 256 trash-cli-macos-0.1.0.tar.gz
```

Typical end-user install flow after the tap is published:

```sh
brew tap YOUR_NAME/tap
brew install YOUR_NAME/tap/trash-cli-macos
trash --version
```

Notes:

- The Homebrew formula name can stay `trash-cli-macos`
- The actual executable installed by that formula is `trash`
- This is safer than naming the formula itself `trash`, which may collide
  with other formulas

## Command Overview

The binary is a single executable with subcommands:

```text
trash [restore|trash-restore] [OPTIONS] [PATH]
trash [list|trash-list] [OPTIONS] [PATH]
trash [put|trash-put] [OPTIONS] FILE...
trash [empty|trash-empty] [OPTIONS] [DAYS]
trash [rm|trash-rm] [OPTIONS] PATTERN
```

If no command is provided, the default command is `restore`.

Global options:

- `--trash-dir DIR` scan or operate on a specific trash root
- `--sort date|path|none` sort `list` and `restore` candidates
- `--overwrite` allow `restore` to replace existing destination files
- `-h`, `--help` show usage
- `--version` show version

## Command Details

### `put`

Move one or more files or directories into the trash.

```sh
trash put ./foo.txt ./build.log
trash trash-put ./dir-a ./dir-b
```

Notes:

- If a trashed name already exists inside the target trash root, the tool will
  create a unique name such as `name_1`, `name_2`, and so on
- This command updates native `.DS_Store` metadata for the files it trashes

### `list`

List trashed files.

```sh
trash list
trash trash-list ./src
trash list --sort path
```

Output format:

```text
YYYY-MM-DD HH:MM:SS /original/path
```

When a `PATH` is provided, only entries whose original location matches or is
inside that path are shown.

### `restore`

Interactively restore trashed files.

```sh
trash restore
trash restore ./src
trash trash-restore --overwrite ./src
```

Behavior:

- matching entries are displayed with zero-based indexes
- input accepts a single index, comma-separated indexes, or ranges such as
  `0,2-4`
- pressing Enter without a selection restores nothing

### `empty`

Delete trashed items permanently.

```sh
trash empty
trash trash-empty 7
```

Behavior:

- without `DAYS`, all discovered trashed items are removed
- with `DAYS`, only items trashed at least that many days ago are removed

### `rm`

Delete matching trashed items permanently.

```sh
trash rm '*.o'
trash trash-rm '/workspace/tmp/*'
```

Behavior:

- if the pattern starts with `/`, it matches the full original path
- otherwise, it matches only the basename
- supported wildcards are `*` and `?`
- quote the pattern to prevent shell expansion

## Differences From Upstream `trash-cli`

- This project is a Rust implementation for macOS trash locations
- It is a single binary with subcommands rather than separate installed
  executables
- It does not use Finder integration
- It reads and writes native macOS Trash metadata through `.DS_Store`

## Notes and Caveats

- Native macOS entries depend on `.DS_Store` metadata being readable
- If native metadata is missing for a file still present in the trash
  directory, the item may be skipped with a warning
- For native macOS entries, the displayed trash time is derived from the file
  modification time when a dedicated deletion timestamp is not available
- `restore` refuses to overwrite an existing destination unless `--overwrite`
  is set

## Example Session

```sh
trash put ./notes.txt ./tmp/output.log
trash list
trash rm '*.log'
trash restore ./notes.txt
trash empty 30
```
