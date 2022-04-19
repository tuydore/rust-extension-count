# Rust Extension Count

Simple command line utility to show file count and total size on a per-extension basis.

Use `rextc -h` to show the help menu.
```
rextc 2.0.0
tuydore <tuydore+github@protonmail.com>
Like the 'tree' command, but recursively shows file number and file sizes for on a per-extension
basis.

USAGE:
    rextc [OPTIONS] <DIRECTORY>

ARGS:
    <DIRECTORY>    Root directory for extension count

OPTIONS:
    -d, --depth <DEPTH>    Depth of recursion [default: 0]
    -e, --empty            Print empty directories
    -h, --help             Print help information
    -s, --sort <SORT>      Sorting mode for extensions only [default: file-size] [possible values:
                           alphabetically, file-count, file-size]
    -V, --version          Print version information
```

## Examples
```
> rextc -d 1 rust-extension-count
rust-extension-count
├── N/A  ── 3 ──   7.07 kiB
├── lock ── 1 ──   6.50 kiB
├── md   ── 1 ──   1.98 kiB
├── toml ── 2 ──    541 B  
├── .git
│   ├── N/A    ── 113 ──  94.36 kiB
│   └── sample ──   1 ──    177 B  
├── .vscode
│   └── json ── 1 ──    492 B  
├── src
│   └── rs ── 2 ──  12.91 kiB
├── target
│   ├── json      ──  65 ──  28.41 kiB
│   ├── TAG       ──   1 ──    177 B  
│   ├── N/A       ── 160 ──  27.83 MiB
│   ├── timestamp ──  65 ──   3.05 kiB
│   ├── rs        ──   1 ──    653 B  
│   ├── o         ── 646 ──  17.40 MiB
│   ├── d         ──  56 ── 127.66 kiB
│   ├── ll        ──   1 ──    242 B  
│   ├── rlib      ──  22 ──  44.36 MiB
│   ├── rmeta     ──  41 ──  18.58 MiB
│   ├── dylib     ──   2 ──  11.62 MiB
│   ├── bin       ──  23 ──  18.01 MiB
│   └── lock      ──   9 ──      0 B  
└── tests
    ├── baz ── 1 ──     10 B  
    ├── foo ── 2 ──     20 B  
    ├── bar ── 1 ──      5 B  
    └── N/A ── 1 ──     20 B  
```

License: MIT.