//! Convenient reading from a tab-separated text file
//!
//! This crate allows you to create a for-loop-compatible iterator over the lines of a tab-separated file.
//! Each lines is returned as a [Record](struct.Record.html) which contains the [original line](struct.Record.html#method.line)
//! and gives you access to the [individual fields](struct.Record.html#method.fields) from that line.
//!
//! See [Tabfile's](struct.Tabfile.html) documentation to see how to use this crate.

use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::ops::Range;
use std::path::Path;

type IterItem = Result<Record, Error>;

/// A read-only open handle for a tab-separated file.
///
/// To make use of this struct, put it into a for-loop:
///
/// ```
/// extern crate tabfile;
/// use tabfile::Tabfile;
///
/// # use std::fs::File;
/// # use std::io::Write;
/// # use tempfile::NamedTempFile;
/// # let mut file = NamedTempFile::new().unwrap();
/// # file.write(b"first_header\tsecond_header\nfirst_field\tsecond_field");
/// // setup for `file` ellided
///
/// let tabfile = Tabfile::open(file.path()).unwrap();
/// for line_result in tabfile {
///     match line_result {
///         Ok(line) => { // line is a tabfile::Record object
///             let fields = line.fields();
///             println!("{}", fields[0]); // print values in first column
///             let line_string = line.line(); //get the original line
///             println!("complete line={}", line_string);
///         },
///         Err(io_error) => eprintln!("{}", io_error) // std::io::Error
///     }
/// }
/// ```
///
/// `Tabfile` supports the builder pattern. You can configure the `Tabfile` before using it in a
/// for loop like so:
///
/// ```
/// # extern crate tabfile;
/// # use tabfile::Tabfile;
/// #
/// # use std::fs::File;
/// # use std::io::Write;
/// # use tempfile::NamedTempFile;
/// # let mut file = NamedTempFile::new().unwrap();
/// # file.write(b"first_header,second_header\nfirst_field,second_field");
/// // setup for `file` ellided
///
/// let tabfile = Tabfile::open(file.path())
///     .unwrap() // trust your hard disk and file system
///     .separator(',') // if you have comma-separated values
///     .comment_character('#') // if you want to ignore lines starting with #
///     .skip_lines(2); // if you know that the first 2 lines are not relevant to you
/// ```
///
/// Once you have gotten familiar with `Tabfile`, you probably want to check out
/// [Record](struct.Record.html).
///
pub struct Tabfile {
    reader: BufReader<File>,
    separator: char,
    comment_character: Option<char>,
    skip_lines: usize,
    skip_empty_lines: bool,
}

impl Tabfile {
    /// Open an existing tab file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Tabfile, Error> {
        let fd = File::open(path)?;
        Ok(Tabfile {
            reader: BufReader::new(fd),
            separator: '\t',
            comment_character: Some('#'),
            skip_lines: 0,
            skip_empty_lines: true,
        })
    }

    /// Set the separator of the tab file reader.
    ///
    /// The default is `'\t'`
    pub fn separator(mut self, sep: char) -> Self {
        self.separator = sep;
        self
    }

    /// Set the number of lines that should be skipped when reading the tab file.
    ///
    /// The default is `0`.
    pub fn skip_lines(mut self, num_lines: usize) -> Self {
        self.skip_lines = num_lines;
        self
    }

    /// Set a comment character for lines that should be ignored.
    ///
    /// All lines starting with the comment character will be ignored.
    /// By default there is no comment character. If you use this method in combination with
    /// `skip_lines` note that `skip_lines` is always checked first and the first `n` lines will be
    /// dropped regardless of whether they have a comment character at the beginning or not.
    pub fn comment_character(mut self, comment_character: char) -> Self {
        self.comment_character = Some(comment_character);
        self
    }

    /// Skip empty lines.
    ///
    /// If set to `true` (which is the default) then the iterator will only yield non-empty
    /// vectors. If you combine this with the `skip` method, then the first `n` lines will always
    /// be skipped regardless of whether or not they are empty.
    pub fn skip_empty_lines(mut self, skip: bool) -> Self {
        self.skip_empty_lines = skip;
        self
    }
}

impl IntoIterator for Tabfile {
    type Item = IterItem;
    type IntoIter = RowIterator;
    fn into_iter(self) -> Self::IntoIter {
        RowIterator::new(self)
    }
}

/// Iterator over the lines of a tab file.
///
/// ```
/// extern crate tabfile;
/// use tabfile::Tabfile;
///
/// # use std::fs::File;
/// # use std::io::Write;
/// # use tempfile::NamedTempFile;
/// # let mut file = NamedTempFile::new().unwrap();
/// # file.write(b"first_header\tsecond_header\nfirst_field\tsecond_field");
/// // setup for `file` ellided
///
/// let tsv_file = Tabfile::open(file.path()).unwrap();
/// let iter = tsv_file.into_iter();
/// // or alternatively
/// # let tsv_file = Tabfile::open(file.path()).unwrap();
/// for record_result in tsv_file { // this creates a RowIterator
///     // do stuff
/// }
/// ```
pub struct RowIterator {
    tabfile: Tabfile,
}

impl RowIterator {
    fn new(tabfile: Tabfile) -> RowIterator {
        RowIterator { tabfile }
    }
}

impl Iterator for RowIterator {
    type Item = IterItem;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut line = String::new();
            match self.tabfile.reader.read_line(&mut line) {
                Ok(line_length) => {
                    if self.tabfile.skip_lines > 0 {
                        self.tabfile.skip_lines -= 1;
                        continue;
                    }
                    if line_length == 0 {
                        return None; // iterator exhausted
                    } else {
                        if let Some(comment_char) = self.tabfile.comment_character {
                            if line.starts_with(comment_char) {
                                continue; // fetch next line
                            }
                        }
                        if line.trim() == "" && self.tabfile.skip_empty_lines {
                            continue;
                        }
                        return Some(Ok(Record::new(line, self.tabfile.separator)));
                    }
                }
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

/// One line from a tab-separated file.
///
/// A `Record` gives you access to the original line as well as the individual fields of the
/// line. You can keep ownership of the `Record` even if you continue looping over the
/// [Tabfile](struct.Tabfile.html).
pub struct Record {
    line: String,
    ranges: Vec<Range<usize>>,
}

impl Record {
    fn new(line: String, separator: char) -> Record {
        let mut slice_start = 0;
        let mut slice_end = 0;
        let mut seen_newline = false;
        let mut ranges = Vec::new();
        for c in line.chars() {
            if c == separator {
                ranges.push(slice_start..slice_end);
                slice_start = slice_end + c.len_utf8();
            } else if c == '\n' || c == '\r' {
                seen_newline = true;
                ranges.push(slice_start..slice_end);
                break; // no tolerance for multiline strings
            }
            slice_end += c.len_utf8();
        }
        if !seen_newline {
            ranges.push(slice_start..line.len())
        }
        Record { line, ranges }
    }

    /// Get the individual (tab-)separated fields of a line
    ///
    /// This method generates string slices on the fly from precomputed positions.
    pub fn fields(&self) -> Vec<&str> {
        let mut result = Vec::new();
        for range in &self.ranges {
            result.push(&self.line[range.clone()])
        }
        result
    }

    /// Get the original line unchanged
    pub fn line(&self) -> &str {
        &self.line
    }

    /// Get the number of fields
    pub fn len(&self) -> usize {
        self.ranges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    use tempfile::{tempdir, TempDir};

    const FOUR_COLUMN: &[u8] = b"line noise\n\nfoo\tbar\tbaz\tquux\nalpha\tbeta\tgamma\tdelta\n\nLeonardo\tMichelangelo\tDonatello\tRaphael\n#please ignore me\nred\tyellow\tgreen";
    const UNICODE: &[u8] =
        "ä line with Ünicöde symböls\tmøre wørds tø ræd\néverything îs strànge\t💣ℝ is it?\n"
            .as_bytes();
    const EMPTY: &[u8] = b"\t\t\tleft\t\t\tright\t\t\t";

    fn setup(test_file_conents: &[u8]) -> (TempDir, PathBuf) {
        let test_dir = tempdir().unwrap();
        let test_file_path = test_dir.path().join("four_column.tsv");
        let mut test_file = File::create(test_file_path.clone()).unwrap();
        test_file.write(test_file_conents).unwrap();
        (test_dir, test_file_path) // return test_dir so that it is not destroyed
    }

    #[test]
    fn four_column() {
        let (_test_dir, test_file_path) = setup(FOUR_COLUMN);
        let tabfile = Tabfile::open(test_file_path)
            .unwrap()
            .comment_character('#')
            .skip_lines(2)
            .separator('\t');

        let mut iterations = 0;
        for (i, line) in tabfile.into_iter().enumerate() {
            iterations += 1;
            let record = line.unwrap();
            let fields = record.fields();
            match i {
                0 => {
                    assert_eq!(fields[0], "foo");
                    assert_eq!(fields[1], "bar");
                    assert_eq!(fields[2], "baz");
                    assert_eq!(fields[3], "quux");
                    assert_eq!(record.line(), "foo\tbar\tbaz\tquux\n");
                    assert_eq!(record.len(), 4);
                }
                1 => {
                    assert_eq!(fields[0], "alpha");
                    assert_eq!(fields[1], "beta");
                    assert_eq!(fields[2], "gamma");
                    assert_eq!(fields[3], "delta");
                    assert_eq!(record.len(), 4);
                }
                2 => {
                    assert_eq!(fields[0], "Leonardo");
                    assert_eq!(fields[1], "Michelangelo");
                    assert_eq!(fields[2], "Donatello");
                    assert_eq!(fields[3], "Raphael");
                    assert_eq!(record.len(), 4);
                }
                3 => {
                    assert_eq!(fields[0], "red");
                    assert_eq!(fields[1], "yellow");
                    assert_eq!(fields[2], "green");
                    assert_eq!(record.line(), "red\tyellow\tgreen"); // no newline
                    assert_eq!(record.len(), 3);
                }
                _ => assert!(false),
            }
        }
        assert_eq!(iterations, 4);
    }

    #[test]
    fn unicode() {
        let (_test_dir, test_file_path) = setup(UNICODE);
        let tabfile = Tabfile::open(test_file_path).unwrap();
        let mut iterations = 0;
        for (i, line) in tabfile.into_iter().enumerate() {
            iterations += 1;
            let record = line.unwrap();
            let fields = record.fields();
            match i {
                0 => {
                    assert_eq!(fields[0], "ä line with Ünicöde symböls");
                    assert_eq!(fields[1], "møre wørds tø ræd");
                    assert_eq!(record.len(), 2);
                }
                1 => {
                    assert_eq!(fields[0], "éverything îs strànge");
                    assert_eq!(fields[1], "💣ℝ is it?");
                    assert_eq!(record.len(), 2);
                }
                _ => assert!(false),
            }
        }
        assert_eq!(iterations, 2);
    }

    #[test]
    fn empty() {
        let (_test_dir, test_file_path) = setup(EMPTY);
        let tabfile = Tabfile::open(test_file_path)
            .unwrap()
            .skip_empty_lines(false);
        let mut iterations = 0;
        for (i, line) in tabfile.into_iter().enumerate() {
            iterations += 1;
            let record = line.unwrap();
            let fields = record.fields();
            match i {
                0 => {
                    assert_eq!(fields[0], "");
                    assert_eq!(fields[1], "");
                    assert_eq!(fields[2], "");
                    assert_eq!(fields[3], "left");
                    assert_eq!(fields[4], "");
                    assert_eq!(fields[5], "");
                    assert_eq!(fields[6], "right");
                    assert_eq!(fields[7], "");
                    assert_eq!(fields[8], "");
                    assert_eq!(fields[9], "");
                    assert_eq!(record.len(), 10);
                }
                _ => assert!(false),
            }
        }
        assert_eq!(iterations, 1);
    }
}
