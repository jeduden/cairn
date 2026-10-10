//! Line coverage per crate and per test layer, as a table, and the
//! floors it falls short of.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::lcov::Lines;
use crate::plan::{Plan, TestLayer};

/// One crate's lines and how many of them each test layer, and all of
/// them together, ran.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Row {
    pub name: String,
    pub lines: usize,
    /// Lines run, in the order of [`TestLayer::ALL`].
    pub by_layer: [usize; 3],
    pub all: usize,
}

/// Counts, per crate of `plan`, the lines `all` instruments and the
/// ones each test layer ran. A file outside every crate is passed by.
#[must_use]
pub fn tally(plan: &Plan, layers: &[(TestLayer, Lines)], all: &Lines) -> Vec<Row> {
    let mut rows: BTreeMap<&str, Row> = BTreeMap::new();
    for (file, hits) in all {
        let Some(krate) = plan.crate_of(file) else {
            continue;
        };
        let row = rows.entry(&krate.name).or_insert_with(|| Row {
            name: krate.name.clone(),
            ..Row::default()
        });
        for (line, n) in hits {
            row.lines += 1;
            row.all += usize::from(*n > 0);
            for (i, layer) in TestLayer::ALL.iter().enumerate() {
                let ran = layers.iter().filter(|(l, _)| l == layer).any(|(_, lines)| {
                    lines
                        .get(file)
                        .and_then(|f| f.get(line))
                        .is_some_and(|h| *h > 0)
                });
                row.by_layer[i] += usize::from(ran);
            }
        }
    }

    rows.into_values().collect()
}

/// `covered` of `lines` as a percentage; no lines counts as all of them.
#[must_use]
pub fn percent(covered: usize, lines: usize) -> f64 {
    if lines == 0 {
        100.0
    } else {
        ratio(covered) * 100.0 / ratio(lines)
    }
}

/// A line count as a float; counts this size convert exactly.
fn ratio(n: usize) -> f64 {
    f64::from(u32::try_from(n).unwrap_or(u32::MAX))
}

/// The rows as a Markdown table, a row for the whole workspace last.
#[must_use]
pub fn table(rows: &[Row]) -> String {
    let mut out = String::from(
        "| Crate | Lines | Unit | Integration | End-to-end | All |\n| --- | ---: | ---: | ---: | ---: | ---: |\n",
    );
    let mut total = Row {
        name: "**workspace**".into(),
        ..Row::default()
    };
    for row in rows {
        total.lines += row.lines;
        total.all += row.all;
        for (sum, n) in total.by_layer.iter_mut().zip(row.by_layer) {
            *sum += n;
        }
    }
    for row in rows.iter().chain([&total]) {
        let cell = |covered| format!("{:.1}%", percent(covered, row.lines));
        let [unit, integration, e2e] = row.by_layer.map(cell);
        let _ = writeln!(
            out,
            "| {} | {} | {unit} | {integration} | {e2e} | {} |",
            row.name,
            row.lines,
            cell(row.all)
        );
    }

    out
}

/// Every floor of `plan` a row falls short of, one line each.
#[must_use]
pub fn shortfalls(plan: &Plan, rows: &[Row]) -> Vec<String> {
    let mut out = Vec::new();
    for row in rows {
        let floors = plan.floors_of(&row.name);
        let measures = TestLayer::ALL
            .iter()
            .zip(row.by_layer)
            .map(|(layer, covered)| {
                (
                    format!("the {layer} test layer"),
                    floors.of(*layer),
                    covered,
                )
            })
            .chain([("every test layer together".to_owned(), floors.all, row.all)]);
        for (what, floor, covered) in measures {
            let Some(floor) = floor else {
                continue;
            };
            let got = percent(covered, row.lines);
            if got < floor {
                out.push(format!(
                    "coverage: {} is at {got:.1}% of its lines ({covered}/{}) in {what}, want {floor}%",
                    row.name, row.lines
                ));
            }
        }
    }

    out
}

#[cfg(test)]
mod tests;
