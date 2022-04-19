use anyhow::{anyhow, Context, Result};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

const TPIPE: &str = "├";
const LPIPE: &str = "└";
const NOEXT: &str = " N/A";

/// Applies to extensions only, directories are always sorted alphabetically.
#[derive(Debug)]
enum ExtensionSortingMethod {
    /// Sort by extension name. Files with multiple extensions (e.g. foo.tar.gz) are treated as
    /// having a single extension (tar.gz) and alphabetically ordered accordingly.
    Alphabetically,

    /// Sort by the number of files having this extension. Multiple extensions are treated as a
    /// whole (e.g. foo.tar.gz has extension tar.gz).
    FileCount,

    /// Sort by cumulative file size.
    FileSize,
}

#[derive(Debug)]
struct Extension {
    /// Extension string or None in case none exists. Symlinks are not considered.
    name: Option<String>,

    /// Number of files with the current extension.
    count: usize,

    /// Total size in bytes of files with the current extension.
    total_size_bytes: u64,
}

#[derive(Debug)]
struct Directory {
    /// Always a directory, symlinks are not considered.
    root: PathBuf,

    /// This vector is sorted mutably, prior to printing to the terminal.
    extensions: Vec<Extension>,

    /// This is always ordered alphabetically.
    subdirectories: Vec<Directory>,

    /// Recursion depth, for use in printing.
    depth: usize,
}

impl Extension {
    fn new(extension: Option<String>, size: u64) -> Self {
        Self {
            name: extension,
            count: 1,
            total_size_bytes: size,
        }
    }

    // Convert bytes to easily-readable BINARY-scaled units.
    fn total_size_bytes_human_readable(&self, decimals: usize) -> String {
        if self.total_size_bytes < 2u64.pow(10) {
            format!("{} B", self.total_size_bytes)
        } else if self.total_size_bytes < 1024u64.pow(2) {
            format!("{:.1$} kiB", self.total_size_bytes as f64 / 1024.0, decimals)
        } else if self.total_size_bytes < 1024u64.pow(3) {
            format!("{:.1$} MiB", self.total_size_bytes as f64 / 1024.0f64.powi(2), decimals)
        } else {
            format!("{:.1$} GiB", self.total_size_bytes as f64 / 1024.0f64.powi(3), decimals)
        }
    }
}

impl Directory {
    fn new(root: PathBuf, max_depth: usize) -> Self {
        Self::scan(root, 0, max_depth).unwrap_or_else(|err| panic!("{err}"))
    }

    fn scan(mut root: PathBuf, depth: usize, max_depth: usize) -> Result<Self> {
        let mut directory = Self {
            root: root.clone(),
            extensions: Vec::new(),
            subdirectories: Vec::new(),
            depth,
        };

        // When recursion limit is reached, every file below gets globbed and appended to the
        // current directory extensions.
        if depth >= max_depth {
            root.push("**");
            root.push("*");
            let pattern = root
                .to_str()
                .ok_or_else(|| anyhow!("could not convert PathBuf to &str"))?;
            for entry in glob::glob(pattern)
                .context("failed to read glob pattern")?
                .flatten()
                .filter(|entry| entry.is_file())
            {
                Self::add_file(entry.as_path(), &mut directory.extensions);
            }

        // Until recursion limit is reached, only files directly in the current directory get
        // added, while directories get parsed as subdirectories and recursively processed.
        } else {
            for entry in root.read_dir()? {
                let entry = entry?;
                let filetype = entry.file_type()?;

                if filetype.is_file() {
                    Self::add_file(entry.path().as_path(), &mut directory.extensions);
                } else if filetype.is_dir() {
                    directory
                        .subdirectories
                        .push(Self::scan(entry.path(), depth + 1, max_depth)?)
                }
            }
        }

        // Subdirectories are always sorted by name, regardless of extension sorting.
        directory.subdirectories.sort_unstable_by_key(|dir| dir.name());

        Ok(directory)
    }

    fn add_file(file: &Path, extensions: &mut Vec<Extension>) {
        let extension = file
            .extension()
            .map(|s| s.to_str().expect("extension is not valid Unicode").to_string());
        let size_bytes = file.metadata().unwrap().len();

        if let Some(previous_entry) = extensions.iter_mut().find(|e| e.name == extension) {
            previous_entry.count += 1;
            previous_entry.total_size_bytes += size_bytes;
        } else {
            extensions.push(Extension::new(extension, size_bytes));
        }
    }

    fn sort_by(&mut self, method: ExtensionSortingMethod) {
        match method {
            ExtensionSortingMethod::Alphabetically => {
                // TODO: remove this clone
                self.extensions.sort_unstable_by_key(|e| e.name.clone())
            }
            ExtensionSortingMethod::FileCount => self.extensions.sort_unstable_by_key(|e| e.count),
            ExtensionSortingMethod::FileSize => self.extensions.sort_unstable_by_key(|e| e.total_size_bytes),
        }
    }

    fn count(&self, extension: Option<&str>) -> usize {
        self.extensions
            .iter()
            .find(|e| e.name.as_deref() == extension)
            .map_or(0, |e| e.count)
    }

    fn size(&self, extension: Option<&str>) -> Option<u64> {
        self.extensions
            .iter()
            .find(|e| e.name.as_deref() == extension)
            .map(|e| e.total_size_bytes)
    }

    fn name(&self) -> String {
        self.root
            .file_name()
            .expect("directory cannot be an ellipsis")
            .to_str()
            .expect("could not convert directory name to string")
            .to_string()
    }

    fn draw_aux(&self, last: bool, mut skipped: Vec<usize>) {
        if last {
            skipped.push(self.depth);
        }

        // Draw the current directory iteself.
        if self.depth == 0 {
            println!("{}", self.name());
        } else {
            print_item(&self.name(), last, self.depth, &skipped);
        }

        // Draw the contained extensions.
        for (idx, extension) in self.extensions.iter().enumerate() {
            print_item(
                &extension.to_string(),
                !self.subdirectories.is_empty() || idx + 1 == self.extensions.len(),
                self.depth + 1,
                &skipped,
            )
        }

        // Draw the subdirectories.
        for (idx, subdirectory) in self.subdirectories.iter().enumerate() {
            subdirectory.draw_aux(idx + 1 == self.subdirectories.len(), skipped.clone())
        }
    }
}

// Depth zero is the depth of the items contained in the root directory the program was called in.
fn vertical_bars(depth: usize, skipped: &[usize]) -> String {
    let mut s: String = "".to_owned();
    for i in 1..depth {
        if skipped.contains(&i) {
            s.push_str("    ")
        } else {
            s.push_str("│   ")
        }
    }
    s
}

/// Print any item text, file or directory alike.
fn print_item(text: &str, last: bool, depth: usize, skipped: &[usize]) {
    println!(
        "{}{}── {}",
        vertical_bars(depth, skipped),
        if last { LPIPE } else { TPIPE },
        text
    )
}

impl Display for Extension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:>5} ── {:>3} ── {}",
            self.name.as_ref().unwrap_or(&NOEXT.to_string()),
            self.count,
            self.total_size_bytes_human_readable(2),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TESTS_DIR: &str = env!("CARGO_MANIFEST_DIR");

    fn tests_dir() -> PathBuf {
        PathBuf::from(TESTS_DIR).join("tests")
    }

    mod directory {
        use super::*;

        #[test]
        fn test_new() {
            let root = tests_dir();
            let directory = Directory::new(root.clone(), 0);
            assert_eq!(directory.depth, 0);
            assert_eq!(directory.root, root);
            assert_eq!(directory.extensions.len(), 3);
            assert_eq!(directory.subdirectories.len(), 0);
            assert_eq!(directory.name(), "tests");
        }

        #[test]
        fn test_count() {
            let directory = Directory::new(tests_dir(), 0);
            assert_eq!(directory.count(Some("foo")), 2);
            assert_eq!(directory.count(Some("bar")), 1);
            assert_eq!(directory.count(Some("non-existent")), 0);
            assert_eq!(directory.count(None), 1);
        }

        #[test]
        fn test_size() {
            let directory = Directory::new(tests_dir(), 0);
            assert_eq!(directory.size(Some("foo")), Some(20));
            assert_eq!(directory.size(Some("bar")), Some(5));
            assert_eq!(directory.size(Some("non-existent")), None);
            assert_eq!(directory.size(None), Some(20));
        }

        #[test]
        fn test_recursion() {
            let directory = Directory::new(tests_dir(), 1);
            let subdirectory = directory.subdirectories.first().expect("no subdirectories found");

            assert_eq!(subdirectory.name(), "dirA");
            assert_eq!(subdirectory.count(Some("bar")), 1);
            assert_eq!(subdirectory.size(Some("bar")), Some(5));
        }

        #[test]
        #[ignore = "visual check"]
        fn test_draw() {
            let directory = Directory::new(tests_dir(), 1);
            directory.draw(true, Vec::new());
        }
    }
}
