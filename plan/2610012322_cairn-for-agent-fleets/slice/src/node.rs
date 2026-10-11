//! A node: the writers it holds, its key set, and what it derives.
//!
//! What a node holds is a function of the set of segments it received,
//! never of their order: an event's header is the same in every copy
//! (else the copy is a fork and refused), its content is held while
//! some copy carried it and no applied tombstone names it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::record::{Address, Event, Segment, purged_addresses};
use crate::{Error, Result, index, room};

/// One node's record and key set. Its files live under one directory.
pub struct Node {
    dir: PathBuf,
    /// The writers whose seat keys this node holds: their events are
    /// `witnessed` here, every other writer's `peer` (RCL-09).
    key_set: BTreeSet<String>,
    held: BTreeMap<String, BTreeMap<u64, Event>>,
    ranges: BTreeMap<String, BTreeSet<(u64, u64)>>,
    /// Erasure requests this node's principal applied (PEER-11). A
    /// tombstone in a writer of this node's key set is its own purge
    /// and needs none.
    erasures: BTreeSet<Address>,
}

impl Node {
    /// A node holding nothing yet, whose key set holds the seat keys of
    /// the writers named.
    pub fn new(dir: &Path, key_set: &[&str]) -> Result<Node> {
        std::fs::create_dir_all(dir)?;
        Ok(Node {
            dir: dir.to_path_buf(),
            key_set: key_set.iter().map(|s| s.to_string()).collect(),
            held: BTreeMap::new(),
            ranges: BTreeMap::new(),
            erasures: BTreeSet::new(),
        })
    }

    /// Receives a segment: verifies it, merges it with what is held,
    /// applies the tombstones that apply here, and stores the writer.
    pub fn receive(&mut self, segment: &Segment) -> Result<()> {
        segment.check()?;
        let held = self.held.entry(segment.writer.clone()).or_default();
        for e in &segment.events {
            let seq = e.header.seq;
            if let Some(h) = held.get(&seq)
                && h.hash != e.hash
            {
                return Err(Error::new(format!("{:?}: fork, refused", e.address())));
            }
            let links = [
                seq.checked_sub(1)
                    .and_then(|p| held.get(&p))
                    .map(|p| (Some(p.hash), e.header.prev_hash)),
                held.get(&(seq + 1))
                    .map(|n| (n.header.prev_hash, Some(e.hash))),
            ];
            if links.into_iter().flatten().any(|(want, got)| want != got) {
                return Err(Error::new(format!(
                    "{:?}: does not chain to what is held",
                    e.address()
                )));
            }
        }
        for e in &segment.events {
            match held.get_mut(&e.header.seq) {
                Some(h) if h.body.is_none() => h.body = e.body.clone(),
                Some(_) => {}
                None => {
                    held.insert(e.header.seq, e.clone());
                }
            }
        }
        let range = (segment.first(), segment.last());
        self.ranges
            .entry(segment.writer.clone())
            .or_default()
            .insert(range);
        let mut dirty = self.apply_tombstones();
        dirty.extend(segment.events.iter().map(Event::address));
        self.store(&dirty)
    }

    /// Applies an erasure request: this node's principal erases what
    /// the tombstone at `tombstone` names (PEER-11).
    pub fn apply_erasure(&mut self, tombstone: &Address) -> Result<()> {
        match self.event(tombstone) {
            Some(e) if e.header.kind == "tombstone" => {}
            _ => {
                return Err(Error::new(format!(
                    "{tombstone:?} is no tombstone held here"
                )));
            }
        }
        self.erasures.insert(tombstone.clone());
        let dirty = self.apply_tombstones();
        self.store(&dirty)
    }

    /// Erases the content and commitment key of every event an
    /// applicable tombstone names; returns the addresses it erased.
    fn apply_tombstones(&mut self) -> BTreeSet<Address> {
        let named: Vec<Address> = self
            .events()
            .into_iter()
            .filter(|t| t.header.kind == "tombstone")
            .filter(|t| {
                self.key_set.contains(&t.header.writer) || self.erasures.contains(&t.address())
            })
            .filter_map(|t| t.body.as_ref())
            .flat_map(|b| purged_addresses(&b.content))
            .collect();
        let mut erased = BTreeSet::new();
        for a in named {
            if let Some(e) = self.held.get_mut(&a.writer).and_then(|w| w.get_mut(&a.seq))
                && e.body.take().is_some()
            {
                erased.insert(a);
            }
        }
        erased
    }

    pub fn event(&self, a: &Address) -> Option<&Event> {
        self.held.get(&a.writer)?.get(&a.seq)
    }

    /// Every held event, in (writer, seq) order.
    pub fn events(&self) -> Vec<&Event> {
        self.held.values().flat_map(|w| w.values()).collect()
    }

    /// How the writer's events reached this node (RCL-09).
    pub fn origin(&self, writer: &str) -> &'static str {
        if self.key_set.contains(writer) {
            "witnessed"
        } else {
            "peer"
        }
    }

    /// The deployment mode the writer recorded in its first event.
    fn deployment_mode(&self, writer: &str) -> Option<&str> {
        let first = self.held.get(writer)?.get(&1)?;
        if first.header.kind != "run_start" {
            return None;
        }
        first
            .body
            .as_ref()?
            .content
            .get("deployment_mode")?
            .as_str()
    }

    /// The default trust policy (PRV-02), for the classes the slice
    /// records: on the recording node only, structural, `operator` and
    /// `harness_meta` events, and `user` events while the recorded
    /// deployment mode is `interactive`. Everything else is untrusted.
    pub fn trust(&self, e: &Event) -> &'static str {
        let w = &e.header.writer;
        let trusted = self.origin(w) == "witnessed"
            && match e.header.provenance.as_str() {
                "structural" | "operator" | "harness_meta" => true,
                "user" => self.deployment_mode(w) == Some("interactive"),
                _ => false,
            };
        if trusted { "trusted" } else { "untrusted" }
    }

    /// Writes each held segment holding a changed address, as the node
    /// now holds it, under `segments/<writer>/<first>-<last>.seg`.
    fn store(&self, changed: &BTreeSet<Address>) -> Result<()> {
        for (writer, ranges) in &self.ranges {
            let dir = self.dir.join("segments").join(writer);
            std::fs::create_dir_all(&dir)?;
            for &(first, last) in ranges {
                let lo = Address {
                    writer: writer.clone(),
                    seq: first,
                };
                let hi = Address {
                    writer: writer.clone(),
                    seq: last,
                };
                if changed.range(lo..=hi).next().is_none() {
                    continue;
                }
                let events: Vec<Event> = self.held[writer]
                    .range(first..=last)
                    .map(|(_, e)| e.clone())
                    .collect();
                let s = Segment {
                    writer: writer.clone(),
                    events,
                };
                write_atomic(
                    &dir.join(format!("{first:012}-{last:012}.seg")),
                    &s.encode(),
                )?;
            }
        }
        Ok(())
    }

    /// Rebuilds the search index and room state from the held writers
    /// and the key set alone (ADM-08) into a new file that replaces the
    /// old one; returns its path.
    pub fn rebuild(&self) -> Result<PathBuf> {
        let path = self.dir.join("index.db");
        let next = self.dir.join("index.db.next");
        let _ = std::fs::remove_file(&next);
        index::build(&next, self, &room::derive(self))?;
        std::fs::rename(&next, &path)?;
        Ok(path)
    }

    /// Every file the node stores.
    pub fn files(&self) -> Result<Vec<PathBuf>> {
        let mut out = Vec::new();
        let mut dirs = vec![self.dir.clone()];
        while let Some(d) = dirs.pop() {
            for entry in std::fs::read_dir(&d)? {
                let p = entry?.path();
                if p.is_dir() {
                    dirs.push(p)
                } else {
                    out.push(p)
                }
            }
        }
        out.sort();
        Ok(out)
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
