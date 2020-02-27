use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Color {
    pub group: String,
    pub label: String,
    pub sort: i32,
}

impl Color {
    pub fn defaults() -> IndexMap<String, Color> {
        return indexmap! {
            "red".into() => Color { group: "A".into(), label: "red".into(), sort: 1 },
            "orange".into() => Color { group: "A".into(), label: "orange".into(), sort: 2 },
            "yellow".into() => Color { group: "A".into(), label: "yellow".into(), sort: 3 },
            "green".into() => Color { group: "A".into(), label: "green".into(), sort: 4 },
            "blue".into() => Color { group: "A".into(), label: "blue".into(), sort: 5 },
            "purple".into() => Color { group: "A".into(), label: "purple".into(), sort: 6 },
            "pink".into() => Color { group: "A".into(), label: "pink".into(), sort: 7 },
            "brown".into() => Color { group: "A".into(), label: "brown".into(), sort: 8 },
            "white".into() => Color { group: "A".into(), label: "white".into(), sort: 9 },
            "black".into() => Color { group: "A".into(), label: "black".into(), sort: 10 },
        };
    }
}
