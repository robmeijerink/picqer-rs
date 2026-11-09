//! Checks the crate against the snapshots of the documentation in `tests/docs`.
//!
//! `cargo xtask docs` rewrites the snapshots from the live documentation. `coverage.json` says,
//! for every documented endpoint, whether this crate covers it, so a new or removed endpoint
//! fails here until someone decides what it means for the crate.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/docs")
}

fn read(name: &str) -> Value {
    let path = directory().join(name);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn strings(value: &Value) -> BTreeSet<String> {
    value
        .as_array()
        .expect("an array")
        .iter()
        .map(|item| item.as_str().expect("a string").to_owned())
        .collect()
}

fn documented() -> BTreeMap<String, BTreeSet<String>> {
    let mut pages = BTreeMap::new();
    for entry in fs::read_dir(directory()).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        if !name.ends_with(".json") || name == "coverage.json" || name == "changelog.json" {
            continue;
        }
        let endpoints: BTreeSet<String> = read(&name)["endpoints"]
            .as_array()
            .expect("endpoints")
            .iter()
            .filter(|endpoint| !endpoint["path"].as_str().unwrap_or("").is_empty())
            .map(|endpoint| {
                format!(
                    "{} {}",
                    endpoint["method"].as_str().unwrap(),
                    endpoint["path"].as_str().unwrap()
                )
            })
            .collect();
        if !endpoints.is_empty() {
            pages.insert(name.trim_end_matches(".json").to_owned(), endpoints);
        }
    }
    pages
}

fn coverage() -> Value {
    read("coverage.json")
}

#[test]
fn every_documented_endpoint_is_either_covered_or_listed_as_not_covered() {
    let coverage = coverage();
    let pages = &coverage["pages"];
    let mut problems = Vec::new();
    for (page, endpoints) in documented() {
        let entry = &pages[&page];
        if entry.is_null() {
            problems.push(format!("{page}: no entry in coverage.json"));
            continue;
        }
        let mut accounted = strings(&entry["covered"]);
        accounted.extend(strings(&entry["not_covered"]));
        for endpoint in endpoints.difference(&accounted) {
            problems.push(format!(
                "{page}: documented but not accounted for: {endpoint}"
            ));
        }
        for endpoint in accounted.difference(&endpoints) {
            problems.push(format!(
                "{page}: accounted for but no longer documented: {endpoint}"
            ));
        }
    }
    let known: BTreeSet<_> = documented().into_keys().collect();
    for page in pages.as_object().unwrap().keys() {
        if !known.contains(page) {
            problems.push(format!("{page}: in coverage.json but not documented"));
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

#[test]
fn nothing_is_both_covered_and_not_covered() {
    let coverage = coverage();
    for (page, entry) in coverage["pages"].as_object().unwrap() {
        let covered = strings(&entry["covered"]);
        let not_covered = strings(&entry["not_covered"]);
        let both: Vec<_> = covered.intersection(&not_covered).collect();
        assert!(both.is_empty(), "{page}: {both:?}");
    }
}

#[test]
fn endpoints_outside_the_snapshot_really_are_not_in_it() {
    let coverage = coverage();
    let all: BTreeSet<String> = documented().into_values().flatten().collect();
    for endpoint in strings(&coverage["not_in_snapshot"]) {
        assert!(
            !all.contains(&endpoint),
            "{endpoint} is documented now, so move it to the page it belongs to"
        );
    }
}

fn source() -> String {
    fn collect(directory: PathBuf, text: &mut String) {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(path, text);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                text.push_str(&fs::read_to_string(path).unwrap());
            }
        }
    }
    let mut text = String::new();
    collect(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut text,
    );
    text
}

#[test]
fn covered_endpoints_use_path_segments_that_exist_in_the_source() {
    let coverage = coverage();
    let source = source();
    let mut endpoints: BTreeSet<String> = strings(&coverage["not_in_snapshot"]);
    for entry in coverage["pages"].as_object().unwrap().values() {
        endpoints.extend(strings(&entry["covered"]));
    }
    let mut problems = Vec::new();
    for endpoint in endpoints {
        let path = endpoint.split_once(' ').unwrap().1;
        let path = path.split('?').next().unwrap();
        for segment in path
            .split('/')
            .filter(|s| !s.is_empty() && !s.starts_with('{'))
        {
            if !source.contains(&format!("\"{segment}\"")) {
                problems.push(format!("{endpoint}: `{segment}` is not used by the crate"));
            }
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

#[test]
fn the_changelog_snapshot_has_entries() {
    let changelog = read("changelog.json");
    assert!(!changelog["entries"].as_array().unwrap().is_empty());
}
