//! The ENG-12 steps: the end-to-end suite runs each component confined
//! to its boundary and fails on a socket outside it, naming the
//! component and the caller. Only the core's row can run today; the
//! outline stays pending until M7 to M9 build the room-view component,
//! the launcher and the peer, publish and bridge components, and the
//! testkit's confinement gains their rows. The probe that stands in for
//! a component is this runner itself, started with the probe argument.

use std::env;
use std::process::Output;

use cucumber::{given, then, when};
use testkit::boundary::{self, Component, Confinement};

use crate::World;

/// What the ENG-12 steps track: the confinement the scenario named,
/// and how the probe's confined start ended.
#[derive(Debug, Default)]
pub struct Boundary {
    confinement: Option<Confinement>,
    ended: Option<Result<Output, boundary::Error>>,
}

impl Boundary {
    fn confinement(&self) -> Result<&Confinement, String> {
        self.confinement
            .as_ref()
            .ok_or_else(|| "no step confined a component".to_owned())
    }
}

fn boundary(w: &mut World) -> &mut Boundary {
    w.section::<Boundary>()
}

#[given(regex = r#"^the end-to-end suite runs "([^"]+)" confined to its boundary, which (.+)$"#)]
fn confined(w: &mut World, name: String, which: String) -> Result<(), String> {
    let component = Component::named(&name).map_err(|e| e.to_string())?;
    if component.confinement() != which {
        return Err(format!(
            "the suite's confinement of the {name} {}, not {which}",
            component.confinement()
        ));
    }
    boundary(w).confinement = Some(Confinement::of(component).map_err(|e| e.to_string())?);
    Ok(())
}

#[when(regex = r#"^"([^"]+)" opens a socket outside the boundary the register assigns it$"#)]
fn opens_a_socket(w: &mut World, name: String) -> Result<(), String> {
    let home = w.home.clone();
    let b = boundary(w);
    let confinement = b.confinement()?;
    if confinement.component().name() != name {
        return Err(format!(
            "the scenario confined the {}, not the {name}",
            confinement.component().name()
        ));
    }
    let probe = env::current_exe().map_err(|e| format!("find the probe: {e}"))?;
    b.ended = Some(confinement.run(&probe, &[boundary::PROBE], &home));
    Ok(())
}

#[then(regex = r"^the suite fails naming the component and the caller$")]
fn fails_naming(w: &mut World) -> Result<(), String> {
    let b = boundary(w);
    let name = b.confinement()?.component().name();
    match b.ended.take() {
        None => Err("no step started the probe".to_owned()),
        Some(Ok(_)) => Err(format!("the suite let the {name}'s socket through")),
        Some(Err(err @ boundary::Error::Socket { .. })) => {
            let msg = err.to_string();
            let named =
                msg.contains(&format!("the {name} opened a socket")) && msg.contains(file!());
            if named {
                Ok(())
            } else {
                Err(format!(
                    "the failure does not name the {name} and this step: {msg}"
                ))
            }
        }
        Some(Err(other)) => Err(other.to_string()),
    }
}
