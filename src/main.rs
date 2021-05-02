use anyhow::{Context, Result};
use std::cmp::Ordering;
use std::env;
use std::{collections::HashMap, fs::DirEntry};
use std::{path::PathBuf, usize};
use structopt::StructOpt;

/// Mapping from extension to number of files and total size.
type FileMap = HashMap<Option<String>, (usize, u64)>;

const TPIPE: &str = "├";
const LPIPE: &str = "└";
const NOEXT: &str = " N/A";

/// Method for sorting files when printing to console.
enum FileSort {
    Alphabetically,
    NumFiles,
    TotalSize,
}

impl FileSort {
    fn from_char(c: &char) -> Self {
        match c {
            'A' => Self::Alphabetically,
            'N' => Self::NumFiles,
            'S' => Self::TotalSize,
            c => panic!("Unknown file sort key {}", c),
        }
    }
}

#[derive(Debug)]
struct Files {
    filemap: FileMap,
    depth: usize,
}

impl Files {
    /// Sort instance by extension name, file number or file size.
    fn items_sorted_by(&self, sort_by: &FileSort) -> Vec<(&Option<String>, &usize, &u64)> {
        let mut item_list: Vec<(&Option<String>, &usize, &u64)> = self
            .filemap
            .iter()
            // .map(|(os, (nf, ts))| (os.as_ref().map_or(NOEXT, |s| &s[..]), nf, ts))
            .map(|(os, (nf, ts))| (os, nf, ts))
            .collect();

        // TODO: convert to Enum?
        match sort_by {
            FileSort::Alphabetically => {
                // item_list.sort_by_key(|(x, _, _)| x.to_lowercase());
                item_list.sort_by(|(a, _, _), (b, _, _)| match (a, b) {
                    (None, None) => Ordering::Equal,
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (Some(a), Some(b)) => a.to_lowercase().cmp(&b.to_lowercase()),
                })
            }
            FileSort::NumFiles => {
                item_list.sort_by_key(|(_, x, _)| *x);
                item_list.reverse();
            }
            FileSort::TotalSize => {
                item_list.sort_by_key(|(_, _, x)| *x);
                item_list.reverse();
            }
        };
        item_list
    }

    /// Draw all contained items.
    fn draw(&self, skipped: &[usize], share_dir_with_subdirs: bool, sort_by: &FileSort) {
        let total: usize = self.filemap.len();
        let mut text: String;
        let max_filenum_size = self.max_filenum_size();
        let max_extension_size = self.max_extension_size();

        for (i, (extension, num_files, total_size)) in
            self.items_sorted_by(sort_by).iter().enumerate()
        {
            // for (i, (extension, (num_files, total_size))) in self.filemap.iter().enumerate() {
            text = Self::text(
                extension
                    .as_ref()
                    .map_or(NOEXT.to_owned(), |s| format!(".{}", s)),
                num_files,
                total_size,
                &max_extension_size,
                &max_filenum_size,
            );
            print_item(
                &text,
                i + 1 == total && !share_dir_with_subdirs,
                &self.depth,
                skipped,
            );
        }
    }

    /// Check how much space (in characters) the longest file extension occupies.
    fn max_extension_size(&self) -> usize {
        self.filemap
            .keys()
            .map(|k| if let Some(s) = k { s.len() } else { 5 })
            .max()
            .expect("No maximum extension size.")
    }

    /// Check how much space (in characters) the largest number of files occupies.
    fn max_filenum_size(&self) -> usize {
        self.filemap
            .values()
            .map(|(num_files, _)| num_files)
            .max()
            .expect("No maximum number of files.")
            .to_string()
            .len()
    }

    /// Generate the text for a combination of extension, num_files and total_size
    fn text(
        extension: String,
        num_files: &usize,
        total_size: &u64,
        max_extension_size: &usize,
        max_filenum_size: &usize,
    ) -> String {
        format!(
            "{:max_extension_size$} ── {:max_filenum_size$} ── {}",
            extension,
            num_files,
            human_filesize(total_size, 2),
            max_extension_size = max_extension_size + 1,
            max_filenum_size = max_filenum_size,
        )
    }

    fn update_from_filemap(&mut self, filemap: &mut FileMap) {
        for (k, (v0, v1)) in filemap.drain() {
            if let Some((num_files, total_size)) = self.filemap.get_mut(&k) {
                *num_files += v0;
                *total_size += v1;
            } else {
                self.filemap.insert(k, (v0, v1));
            }
        }
    }
}

#[derive(Debug)]
struct Directory {
    path: PathBuf,
    subdirs: Vec<Directory>,
    files: Files,
    depth: usize, // TODO: connect the two depths by reference
}

// TODO: maximum filesize

impl Directory {
    fn new(path: PathBuf, depth: usize) -> Result<Self> {
        let mut filemap: FileMap = HashMap::new();
        let mut subdirs: Vec<PathBuf> = Vec::new();

        // QUESTION: would an iterator speed things up? Rayon?
        for entry in path
            .read_dir()
            .with_context(|| format!("Could not read directory {}.", path.to_str().unwrap()))?
        {
            if let Ok(entry) = entry {
                if let Ok(filetype) = entry.file_type() {
                    if filetype.is_dir() {
                        subdirs.push(entry.path());
                    } else if filetype.is_file() {
                        Self::update_filemap(&entry, &mut filemap)?;
                    } // TODO: handle symlinks
                }
            }
        }

        // sort directories alphabetically
        subdirs.sort_by_key(|p| {
            p.file_name()
                .expect("Could not parse dirname")
                .to_str()
                .expect("Could not convert OsStr to &str")
                .to_owned()
        });

        Ok(Self {
            path,
            subdirs: subdirs
                .into_iter()
                // QUESTION: is the unwrap here enough to propagate upwards?
                .map(|s| Self::new(s, depth + 1).unwrap())
                .collect(),
            files: Files {
                filemap,
                depth: depth + 1,
            },
            depth,
        })
    }

    /// Update the current file mapping with information from a given entry. Assumes the entry
    /// is a file and does not check.
    fn update_filemap(file: &DirEntry, map: &mut FileMap) -> Result<()> {
        let filepath: PathBuf = file.path();

        let filesize: u64 = filepath
            .metadata()
            .with_context(|| {
                format!(
                    "Could not fetch file metadata for {}",
                    filepath.to_str().unwrap()
                )
            })?
            .len();

        // use None to mark files with no extension
        // QUESTION: treat dotfiles separately?
        let extension: Option<String> = filepath.extension().map(|e| {
            e.to_str()
                .expect("Could not convert OsStr to String")
                .to_owned()
        });

        if let Some((num_files, total_size)) = map.get_mut(&extension) {
            *num_files += 1;
            *total_size += filesize;
        } else {
            map.insert(extension, (1, filesize));
        }
        Ok(())
    }

    /// No files are contained anywhere down the directory tree from here.
    fn is_empty(&self) -> bool {
        self.files.filemap.is_empty() && self.subdirs.iter().all(|d| d.is_empty())
    }

    fn draw(&self, is_last: bool, mut skipped: Vec<usize>, skip_empty: bool, sort_by: &FileSort) {
        // do not print empty dirs
        if skip_empty && self.is_empty() {
            return;
        }

        // if this is the last dir, add it to the list of skipped
        if is_last {
            skipped.push(self.depth)
        }

        // draw this directory
        if self.depth == 0 {
            println!(
                "{}",
                self.path
                    .to_str()
                    .expect("Could not convert dirname to str.")
                    .to_owned()
            )
        } else {
            print_item(
                self.path
                    .file_name()
                    .expect("Could not parse own dirname.")
                    .to_str()
                    .expect("Could not convert dirname to str."),
                is_last,
                &self.depth,
                &skipped,
            );
        }

        // draw all contained files
        if !self.files.filemap.is_empty() {
            self.files.draw(&skipped, !self.subdirs.is_empty(), sort_by);
        }

        // draw all contained subdirs
        for (idx, dir) in self.subdirs.iter().enumerate() {
            dir.draw(
                idx + 1 == self.subdirs.len(),
                skipped.clone(),
                skip_empty,
                sort_by,
            );
        }
    }

    /// Recursively pull files from the contained subdirectories.
    fn pull_files_from_below(&mut self, skip_empty: bool) {
        for dir in self.subdirs.iter_mut() {
            // if a subdir has further subdirectories, recurse the operation
            if !dir.subdirs.is_empty() {
                dir.pull_files_from_below(skip_empty)
            }
            self.files.update_from_filemap(&mut dir.files.filemap);
        }
        if skip_empty {
            self.subdirs.clear();
        }
    }

    /// Condense files upward, up to a certain depth.
    fn condense_to_depth(&mut self, depth: usize, skip_empty: bool) {
        for dir in self.subdirs.iter_mut() {
            if dir.depth >= depth {
                dir.pull_files_from_below(skip_empty)
            } else {
                dir.condense_to_depth(depth, skip_empty)
            }
        }
        if self.depth >= depth {
            self.pull_files_from_below(skip_empty)
        }
    }
}

// Depth zero is the depth of the items contained in the ROOT directory the program was called in.
fn vertical_bars(depth: &usize, skipped: &[usize]) -> String {
    let mut s: String = "".to_owned();
    for i in 1..*depth {
        if skipped.contains(&i) {
            s.push_str("    ")
        } else {
            s.push_str("│   ")
        }
    }
    s
}

/// Print any item text, file or directory alike.
fn print_item(text: &str, is_last: bool, depth: &usize, skipped: &[usize]) {
    println!(
        "{}{}── {}",
        vertical_bars(depth, skipped),
        if is_last { LPIPE } else { TPIPE },
        text
    )
}

// Convert bytes to easily-readable BINARY-scaled units.
fn human_filesize(bytes: &u64, decimals: usize) -> String {
    if *bytes < 2u64.pow(10) {
        format!("{} B", bytes)
    } else if *bytes < 2u64.pow(20) {
        format!("{:.1$} kiB", *bytes as f64 / 1024.0, decimals)
    } else if *bytes < 2u64.pow(40) {
        format!("{:.1$} MiB", *bytes as f64 / 1024.0f64.powi(2), decimals)
    } else {
        format!("{:.1$} GiB", *bytes as f64 / 1024.0f64.powi(3), decimals)
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "rstree",
    about = "Rust Simple Tree. Like the 'tree' command, but shows file number and file sizes."
)]
struct Opt {
    /// Root directory to scan.
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,

    /// Maximum depth to dive to.
    #[structopt(short = "d", long = "depth")]
    depth: Option<usize>,

    /// Show empty directories.
    #[structopt(short = "e", long = "show-empty")]
    show_empty: bool,

    /// Sort files: A (alphabetically), N (number of files) or S (total file size).
    #[structopt(short = "s", long = "sort-by", default_value = "S")]
    sort_by: char,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let root: PathBuf = opt
        .input
        .unwrap_or_else(|| env::current_dir().expect("Could not parse current dir."));
    let mut root_dir = Directory::new(root, 0)?;
    if let Some(d) = opt.depth {
        root_dir.condense_to_depth(d, !opt.show_empty);
    }
    root_dir.draw(
        true,
        vec![],
        !opt.show_empty,
        &FileSort::from_char(&opt.sort_by),
    );
    Ok(())
}
