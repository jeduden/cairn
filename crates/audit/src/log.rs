//! The log's files (OPS-02): `audit-NNNNNN.jsonl` in the log's
//! directory, numbered from 1, each `0600` and one entry per line. The
//! chain runs across files: once a file reaches [`ROTATE_AT`] bytes the
//! next entry starts the next file, chained to the last entry of the
//! one before. Appends take the directory's `lock` file, waiting for it
//! with a deadline (ENG-03), so processes appending at once each
//! chain to the entry before.

use std::fs::{self, File, TryLockError};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::time::Duration;

use record::digest::Digest;

use crate::AuditError;
use crate::entry::{Entry, Occurrence};

/// The size at which a file rotates: the entry after it starts the
/// next file.
pub const ROTATE_AT: u64 = 1 << 20;

/// What starts and ends a file's name around its number.
const PREFIX: &str = "audit-";
const SUFFIX: &str = ".jsonl";

/// The file appends lock.
const LOCK: &str = "lock";

/// How often an append tries the lock, and how long it waits between
/// tries: about 40 ms in all, inside the shortest hook budget's
/// share (NFR-02).
const LOCK_ATTEMPTS: u32 = 40;
const LOCK_PAUSE: Duration = Duration::from_millis(1);

/// What a check of the whole log found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Checked {
    pub files: usize,
    pub entries: u64,
}

/// The audit log in its directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Log {
    dir: PathBuf,
}

impl Log {
    /// The log in `dir`, which need not exist yet.
    #[must_use]
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Its directory.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The path of the log's file numbered `n`.
    #[must_use]
    pub fn file(&self, n: u32) -> PathBuf {
        self.dir.join(name(n))
    }

    /// Appends the entry reporting `occurrence` at `time`, chained to
    /// the log's last entry, making the directory `0700` and the file
    /// `0600` where they do not exist yet.
    ///
    /// # Errors
    ///
    /// Fails when the lock stays held past its deadline, when a file
    /// cannot be read or written, or when the last entry is torn or
    /// does not parse, so nothing could chain to it.
    pub fn append(&self, occurrence: &Occurrence, time: u64) -> Result<Entry, AuditError> {
        owner_only_dir()
            .create(&self.dir)
            .map_err(failed(&self.dir))?;
        let lock = self.lock()?;
        let (n, last) = self.tail()?;
        let (number, prev) = last.map_or((1, Digest::ZERO), |e| (e.number + 1, e.hash));
        let entry = Entry::next(occurrence, number, time, prev)?;
        let line = format!("{}\n", entry.line()?);
        let path = self.file(n);
        let mut options = fs::OpenOptions::new();
        options.append(true).create(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        options
            .open(&path)
            .and_then(|mut file| file.write_all(line.as_bytes()))
            .map_err(failed(&path))?;
        drop(lock);

        Ok(entry)
    }

    /// Every entry of the log, in order, each checked against the one
    /// before; none when the directory does not exist.
    ///
    /// # Errors
    ///
    /// Fails, naming where, at the first entry that breaks the chain,
    /// and when a file cannot be read.
    pub fn entries(&self) -> Result<Vec<Entry>, AuditError> {
        self.read_all().map(|(_, entries)| entries)
    }

    /// Checks the whole chain, as [`entries`](Self::entries) does.
    ///
    /// # Errors
    ///
    /// As [`entries`](Self::entries).
    pub fn verify(&self) -> Result<Checked, AuditError> {
        let (files, entries) = self.read_all()?;

        Ok(Checked {
            files,
            entries: entries.len() as u64,
        })
    }

    /// The number of files and every entry in them.
    fn read_all(&self) -> Result<(usize, Vec<Entry>), AuditError> {
        let numbers = self.numbers()?;
        let mut entries = Vec::new();
        for (i, &n) in numbers.iter().enumerate() {
            let text = self.read(n)?;
            if text.is_empty() && i + 1 < numbers.len() {
                return Err(broken(n, 0, "empty but not the last file"));
            }
            for (line, e) in parse_lines(n, &text)? {
                let (number, prev) = entries
                    .last()
                    .map_or((1, Digest::ZERO), |e: &Entry| (e.number + 1, e.hash));
                if e.number != number {
                    return Err(broken(n, line, "entry number out of order"));
                }
                if e.prev_hash != prev {
                    return Err(broken(n, line, "previous hash is not the entry before"));
                }
                entries.push(e);
            }
        }

        Ok((numbers.len(), entries))
    }

    /// The file the next entry goes to, and the entry it chains to.
    fn tail(&self) -> Result<(u32, Option<Entry>), AuditError> {
        let Some(&n) = self.numbers()?.last() else {
            return Ok((1, None));
        };
        let text = self.read(n)?;
        if text.is_empty() {
            // A file made just before a crash: the entry goes into it,
            // chained to the last one of the file before.
            let before = match n.checked_sub(1).filter(|b| *b > 0) {
                Some(b) => last_entry(b, &self.read(b)?)?,
                None => None,
            };
            return Ok((n, before));
        }
        let last = last_entry(n, &text)?;
        let next = if text.len() as u64 >= ROTATE_AT {
            n + 1
        } else {
            n
        };

        Ok((next, last))
    }

    /// The numbers of the log's files, ascending, which must run from
    /// 1 with no gap; none when the directory does not exist.
    fn numbers(&self) -> Result<Vec<u32>, AuditError> {
        let listing = match fs::read_dir(&self.dir) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            listing => listing.map_err(failed(&self.dir))?,
        };
        let mut numbers = Vec::new();
        for item in listing {
            let item = item.map_err(failed(&self.dir))?;
            numbers.extend(item.file_name().to_str().and_then(file_number));
        }
        numbers.sort_unstable();
        for (want, &n) in (1..).zip(&numbers) {
            if n != want {
                return Err(broken(want, 0, "missing"));
            }
        }

        Ok(numbers)
    }

    /// The text of file `n`.
    fn read(&self, n: u32) -> Result<String, AuditError> {
        let path = self.file(n);
        fs::read_to_string(&path).map_err(failed(&path))
    }

    /// Takes the lock, trying until its deadline.
    fn lock(&self) -> Result<File, AuditError> {
        let path = self.dir.join(LOCK);
        let mut options = fs::OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        let file = options.open(&path).map_err(failed(&path))?;
        for _ in 0..LOCK_ATTEMPTS {
            match file.try_lock() {
                Ok(()) => return Ok(file),
                Err(TryLockError::WouldBlock) => std::thread::sleep(LOCK_PAUSE),
                Err(TryLockError::Error(source)) => return Err(AuditError::Io { path, source }),
            }
        }

        Err(AuditError::Busy { path })
    }
}

/// The name of file `n`.
fn name(n: u32) -> String {
    format!("{PREFIX}{n:06}{SUFFIX}")
}

/// The number a log file's name gives it: six digits, from 1.
fn file_number(name: &str) -> Option<u32> {
    let digits = name.strip_prefix(PREFIX)?.strip_suffix(SUFFIX)?;
    let six = digits.len() == 6 && digits.bytes().all(|b| b.is_ascii_digit());

    six.then(|| digits.parse().ok())
        .flatten()
        .filter(|n| *n > 0)
}

/// Each line of file `n`'s `text` with its number, from 1, and the
/// entry it holds; a last line with no line break is torn.
fn parse_lines(n: u32, text: &str) -> Result<Vec<(usize, Entry)>, AuditError> {
    (1..)
        .zip(text.split_inclusive('\n'))
        .map(|(line, piece)| {
            let whole = piece
                .strip_suffix('\n')
                .ok_or_else(|| broken(n, line, "torn entry"))?;
            Entry::parse(whole)
                .map(|e| (line, e))
                .map_err(|why| broken(n, line, why))
        })
        .collect()
}

/// The last entry of file `n`'s `text`.
fn last_entry(n: u32, text: &str) -> Result<Option<Entry>, AuditError> {
    let lines = text.split_inclusive('\n').count();
    let Some(piece) = text.split_inclusive('\n').next_back() else {
        return Ok(None);
    };
    let whole = piece
        .strip_suffix('\n')
        .ok_or_else(|| broken(n, lines, "torn entry"))?;

    Entry::parse(whole)
        .map(Some)
        .map_err(|why| broken(n, lines, why))
}

fn broken(n: u32, line: usize, why: &'static str) -> AuditError {
    AuditError::Broken {
        file: name(n),
        line,
        why,
    }
}

fn failed(path: &Path) -> impl FnOnce(io::Error) -> AuditError + '_ {
    move |source| AuditError::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn owner_only_dir() -> fs::DirBuilder {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);

    builder
}

#[cfg(test)]
mod tests;
