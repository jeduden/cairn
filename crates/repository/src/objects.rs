//! The object store of a clone: loose objects and packs. Alternate
//! object stores are not read.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::fsio::read_limited;
use crate::object::{MAX_OBJECT, Object};
use crate::pack::{Entry, check_header, find_offset, read_entry};
use crate::{CommitId, Error, delta, loose};

/// The most packs read. Git repacks long before this many; more is a
/// clone this crate will not open a file handle for each of.
pub(crate) const MAX_PACKS: usize = 256;

/// The longest delta chain followed. Git writes 50 by default and
/// 4095 at most; the bound stops a cycle and a hostile chain.
pub(crate) const MAX_CHAIN: usize = 1024;

/// The largest `info/alternates` read.
const MAX_ALTERNATES: u64 = 64 << 10;

/// A pack and its index, open for reading.
#[derive(Debug)]
struct Pack {
    index: File,
    pack: File,
}

/// A clone's objects, opened for reading.
#[derive(Debug)]
pub(crate) struct Objects {
    dir: PathBuf,
    packs: Vec<Pack>,
}

/// The names of the packs in `dir`, without their extension.
fn pack_names(dir: &Path) -> Result<BTreeSet<String>, Error> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(BTreeSet::new()),
        Err(e) => return Err(Error::io("a pack directory")(e)),
    };
    let mut names = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(Error::io("a pack directory"))?;
        let name = entry.file_name();
        if let Some(stem) = name.to_str().and_then(|n| n.strip_suffix(".idx")) {
            names.insert(stem.to_owned());
        }
    }
    Ok(names)
}

/// Opens `path` for reading; a file that is not there is `None`.
fn open_file(path: &Path) -> Result<Option<File>, Error> {
    match File::open(path) {
        Ok(file) => Ok(Some(file)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::io("a pack file")(e)),
    }
}

/// The pack `stem` of `dir`, unless its index or its pack is gone.
fn open_pack(dir: &Path, stem: &str) -> Result<Option<Pack>, Error> {
    let Some(pack) = open_file(&dir.join(format!("{stem}.pack")))? else {
        return Ok(None);
    };
    let Some(index) = open_file(&dir.join(format!("{stem}.idx")))? else {
        return Ok(None);
    };
    check_header(&mut &pack)?;
    Ok(Some(Pack { index, pack }))
}

impl Objects {
    /// Opens the object directory `dir`: its packs are listed and
    /// opened, in name order.
    ///
    /// # Errors
    ///
    /// [`Error::TooLarge`] for more than [`MAX_PACKS`] packs;
    /// [`Error::Corrupt`] for a pack file that is not one;
    /// [`Error::Io`] when the directory cannot be listed.
    pub(crate) fn open(dir: &Path) -> Result<Self, Error> {
        let pack_dir = dir.join("pack");
        let names = pack_names(&pack_dir)?;
        if names.len() > MAX_PACKS {
            return Err(Error::TooLarge("number of packs"));
        }
        let mut packs = Vec::with_capacity(names.len());
        for stem in &names {
            packs.extend(open_pack(&pack_dir, stem)?);
        }
        Ok(Self {
            dir: dir.to_path_buf(),
            packs,
        })
    }

    /// The object `id`, or `None` when the clone does not hold it.
    ///
    /// # Errors
    ///
    /// [`Error::Unsupported`] when the object is absent and the clone
    /// names alternate object stores, which could hold it;
    /// [`Error::Missing`] when a delta's base is absent;
    /// [`Error::TooLarge`] for a chain longer than [`MAX_CHAIN`] or an
    /// object past the bound.
    pub(crate) fn read(&self, id: &CommitId) -> Result<Option<Object>, Error> {
        if let Some(object) = loose::read_up_to(&self.dir, id, MAX_OBJECT)? {
            return Ok(Some(object));
        }
        match self.find(id)? {
            Some((pack, offset)) => self.resolve(pack, offset).map(Some),
            None => self.absent().map(|()| None),
        }
    }

    /// Whether the clone holds the object `id`.
    ///
    /// # Errors
    ///
    /// [`Error::Unsupported`] when the object is absent and the clone
    /// names alternate object stores, which could hold it.
    pub(crate) fn contains(&self, id: &CommitId) -> Result<bool, Error> {
        if loose::exists(&self.dir, id)? || self.find(id)?.is_some() {
            return Ok(true);
        }
        self.absent().map(|()| false)
    }

    /// The pack that lists `id`, and where.
    fn find(&self, id: &CommitId) -> Result<Option<(&Pack, u64)>, Error> {
        for pack in &self.packs {
            if let Some(offset) = find_offset(&mut &pack.index, id)? {
                return Ok(Some((pack, offset)));
            }
        }
        Ok(None)
    }

    /// Confirms that nothing outside the clone could hold an absent
    /// object: the clone names no alternate object store.
    fn absent(&self) -> Result<(), Error> {
        let alternates = read_limited(&self.dir.join("info/alternates"), MAX_ALTERNATES)?;
        let names_one = alternates.is_some_and(|bytes| {
            String::from_utf8_lossy(&bytes)
                .lines()
                .map(str::trim)
                .any(|line| !line.is_empty() && !line.starts_with('#'))
        });
        if names_one {
            return Err(Error::Unsupported("alternate object stores"));
        }
        Ok(())
    }

    /// The object at `offset` of `pack`: the deltas down its chain are
    /// collected, then applied to the base from the bottom up. A delta
    /// against an object by id moves on to wherever that object is, so
    /// a chain of any shape is followed without recursion.
    fn resolve(&self, pack: &Pack, offset: u64) -> Result<Object, Error> {
        let (mut pack, mut at) = (pack, offset);
        let mut chain = Chain::default();
        let mut deltas = Vec::new();
        let mut object = loop {
            chain.step()?;
            match read_entry(&mut &pack.pack, at, chain.budget)? {
                Entry::Whole(object) => break object,
                Entry::OffsetDelta { base, delta } => {
                    chain.spend(delta.len());
                    deltas.push(delta);
                    at = base;
                }
                Entry::ReferenceDelta { base, delta } => {
                    chain.spend(delta.len());
                    deltas.push(delta);
                    if let Some(object) = loose::read_up_to(&self.dir, &base, MAX_OBJECT)? {
                        break object;
                    }
                    let Some((found, offset)) = self.find(&base)? else {
                        self.absent()?;
                        return Err(Error::Missing(base));
                    };
                    (pack, at) = (found, offset);
                }
            }
        };
        for delta in deltas.iter().rev() {
            object.data = delta::apply(&object.data, delta, MAX_OBJECT)?;
        }
        Ok(object)
    }
}

/// How far a read has followed deltas: the entries visited and the
/// bytes of delta data still allowed to be held.
#[derive(Debug)]
struct Chain {
    steps: usize,
    budget: usize,
}

impl Default for Chain {
    fn default() -> Self {
        Self {
            steps: 0,
            budget: MAX_OBJECT,
        }
    }
}

impl Chain {
    /// Counts one more entry, which fails past [`MAX_CHAIN`].
    fn step(&mut self) -> Result<(), Error> {
        self.steps += 1;
        if self.steps > MAX_CHAIN {
            return Err(Error::TooLarge("delta chain"));
        }
        Ok(())
    }

    fn spend(&mut self, bytes: usize) {
        self.budget = self.budget.saturating_sub(bytes);
    }
}

#[cfg(test)]
mod tests;
