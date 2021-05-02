# Rust Extension Count

Simple command line utility to show file count and total size on a per-extension basis.

Use `rextc -h` to show the help menu.
```
rextc 0.1.0
Rust Extension Count. Like the 'tree' command, but shows file number and file sizes.

USAGE:
    rextc [FLAGS] [OPTIONS] [input]

FLAGS:
    -h, --help          Prints help information
    -e, --show-empty    Show empty directories
    -V, --version       Prints version information

OPTIONS:
    -d, --depth <depth>        Maximum depth to dive to
    -s, --sort-by <sort-by>    Sort files: A (alphabetically), N (number of files) or S (total file size) [default: S]

ARGS:
    <input>    Root directory to scan
```

## Examples
```
> rextc -d 1 ~/repositories/rust-extension-count
/Users/tuydore/repositories/rust-extension-count
├──  N/A   ── 3 ── 7.07 kiB
├── .lock  ── 1 ── 5.94 kiB
├── .md    ── 1 ── 1.82 kiB
├── .toml  ── 1 ── 504 B
├── .git
│   ├──  N/A    ── 55 ── 37.01 kiB
│   └── .sample ──  1 ── 177 B
├── .vscode
│   └── .json ── 1 ── 492 B
├── src
│   └── .rs ── 1 ── 12.12 kiB
└── target
    ├──  N/A       ── 113 ── 42.43 MiB
    ├── .dylib     ──   4 ── 37.04 MiB
    ├── .rlib      ──  20 ── 35.79 MiB
    ├── .rmeta     ──  20 ── 8.36 MiB
    ├── .bin       ──   3 ── 3.26 MiB
    ├── .o         ──  90 ── 1.43 MiB
    ├── .d         ──  31 ── 72.31 kiB
    ├── .json      ──  38 ── 14.39 kiB
    ├── .plist     ──  10 ── 6.47 kiB
    ├── .timestamp ──  37 ── 1.73 kiB
    ├── .rs        ──   1 ── 653 B
    ├── .TAG       ──   1 ── 177 B
    └── .lock      ──   1 ── 0 B
```

License: MIT.