//! The executable requirement matrix: every scenario under `features/`,
//! run through cucumber-rs. A scenario tagged `@pending` is declared
//! but unwritten: it is skipped and counted, never run as a pass that
//! proves nothing. Every other scenario runs with a step whose text
//! matches no definition failing, not skipped, so an unbound step
//! cannot pass.
//!
//! Each section's steps live in their own module here, named for the
//! section's feature file; the steps register themselves, so a section
//! adds a module and one `mod` line. A step text two sections both
//! define fails as ambiguous: reuse the existing text.
//!
//! `cargo test -p scenario --test bdd -- --tags @ENG-24` runs one
//! scenario by its requirement id.

mod engineering;
mod engineering_adr;
mod engineering_drift;
mod engineering_review;

use std::env;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use cucumber::World as _;
use cucumber::writer::Stats as _;
use scenario::runner;
use testkit::TempDir;

/// The state one scenario threads through its steps, with its own
/// fresh `HOME` and `CAIRN_HOME` (ENG-14): a step that starts a process
/// points it at these, never at the real home.
#[derive(cucumber::World)]
#[world(init = Self::new)]
pub struct World {
    pub home: PathBuf,
    pub cairn_home: PathBuf,
    sections: runner::Sections,
    _dir: TempDir,
}

impl fmt::Debug for World {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("World")
            .field("home", &self.home)
            .field("sections", &self.sections)
            .finish()
    }
}

impl World {
    fn new() -> io::Result<Self> {
        let dir = TempDir::new()?;
        let home = dir.path().to_path_buf();

        Ok(Self {
            cairn_home: home.join(".cairn"),
            home,
            sections: runner::Sections::default(),
            _dir: dir,
        })
    }

    /// A section's own state beside the shared world; see
    /// [`runner::Sections`].
    pub fn section<T: Default + 'static>(&mut self) -> &mut T {
        self.sections.get::<T>()
    }
}

fn main() -> ExitCode {
    let (home, marker) = (env::var_os("HOME"), env::var_os(runner::ISOLATED_HOME));
    if runner::is_isolated(home.as_deref(), marker.as_deref(), &env::temp_dir()) {
        run()
    } else {
        isolate()
    }
}

/// Runs this same executable again with `HOME` and `CAIRN_HOME` pointed
/// at a fresh temporary directory, so nothing the scenarios run can
/// reach the real home or Claude Code configuration (ENG-14).
fn isolate() -> ExitCode {
    let started = TempDir::new().and_then(|home| {
        let mut command = Command::new(env::current_exe()?);
        let status = runner::isolate(command.args(env::args_os().skip(1)), home.path()).status()?;
        Ok(runner::exit_code(status.code()))
    });

    match started {
        Ok(code) => ExitCode::from(code),
        Err(err) => {
            eprintln!("bdd: isolate HOME: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Runs the scenarios the command line selects.
fn run() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let options = match runner::parse_args(&args) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("bdd: {err}\nusage: bdd [--tags EXPR] [FILTER]...");
            return ExitCode::from(2);
        }
    };
    let features = testkit::repo_root().join("features");
    match scenario::scenarios(&features) {
        Ok(all) => {
            let pending = all.iter().filter(|s| s.pending).count();
            println!(
                "scenarios: {} bound, {pending} pending",
                all.len() - pending
            );
        }
        Err(err) => {
            eprintln!("bdd: {err}");
            return ExitCode::FAILURE;
        }
    }

    let writer = runner::block_on(
        World::cucumber()
            .fail_on_skipped()
            .with_default_cli()
            .filter_run(features, move |feature, rule, sc| {
                let tags = feature
                    .tags
                    .iter()
                    .chain(rule.into_iter().flat_map(|r| &r.tags))
                    .chain(&sc.tags);
                options.selects(&sc.name, tags)
            }),
    );

    if writer.execution_has_failed() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
