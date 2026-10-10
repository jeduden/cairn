//! The test pyramid's shape: how many tests each test layer holds. A
//! healthy pyramid has more unit tests than integration tests, and
//! more of those than end-to-end ones.

use std::path::{Component, Path};

use crate::plan::{Plan, TestLayer};

/// Tests per test layer, in the order of [`TestLayer::ALL`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Shape {
    pub tests: [usize; 3],
    /// The bound scenarios among the end-to-end tests.
    pub scenarios: usize,
}

/// Counts the `#[test]` functions in `sources`, each file sorted into
/// its test layer by where it lives in its crate, and adds the bound
/// scenarios to the end-to-end layer.
#[must_use]
pub fn count(plan: &Plan, sources: &[(std::path::PathBuf, String)], scenarios: usize) -> Shape {
    let mut shape = Shape {
        scenarios,
        ..Shape::default()
    };
    shape.tests[TestLayer::EndToEnd as usize] += scenarios;
    for (file, text) in sources {
        let Some(layer) = plan
            .crate_of(file)
            .and_then(|c| layer_of(file.strip_prefix(&c.dir).ok()?))
        else {
            continue;
        };
        shape.tests[layer as usize] += text.lines().filter(|l| l.trim() == "#[test]").count();
    }

    shape
}

/// The test layer of a file at `rel`, a path inside its crate.
fn layer_of(rel: &Path) -> Option<TestLayer> {
    let mut parts = rel.components().filter_map(|c| match c {
        Component::Normal(p) => p.to_str(),
        _ => None,
    });
    match parts.next()? {
        "src" => Some(TestLayer::Unit),
        "tests" => {
            let target = parts.next()?;
            Some(TestLayer::of_test_target(
                target.strip_suffix(".rs").unwrap_or(target),
            ))
        }
        _ => None,
    }
}

impl Shape {
    /// A line for the report, such as
    /// `Tests per test layer: unit 300, integration 12, end-to-end 10 (6 scenarios)`.
    #[must_use]
    pub fn line(&self) -> String {
        let [unit, integration, e2e] = self.tests;
        format!(
            "Tests per test layer: unit {unit}, integration {integration}, end-to-end {e2e} ({} scenarios)",
            self.scenarios
        )
    }

    /// Why the shape is no pyramid, or `None` when it is one.
    #[must_use]
    pub fn inverted(&self) -> Option<String> {
        let [unit, integration, e2e] = self.tests;
        (unit < integration || integration < e2e).then(|| {
            format!(
                "coverage: the test pyramid is inverted: unit {unit}, integration {integration}, end-to-end {e2e}; \
                 want each test layer to hold more tests than the one above it"
            )
        })
    }
}

#[cfg(test)]
mod tests;
