use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Section {
    pub group: String,
    pub label: String,
    pub sort: i32,
}

impl Section {
    pub fn defaults() -> IndexMap<String, Section> {
        return indexmap! {
            "AB1".into() => Section { group: "A".into(), label: "AB1".into(), sort: 1 },
            "AB2".into() => Section { group: "A".into(), label: "MAP".into(), sort: 2 },
            "AB3".into() => Section { group: "A".into(), label: "AB3".into(), sort: 3 },
            "AB4".into() => Section { group: "A".into(), label: "AB4".into(), sort: 4 },
            "AB5".into() => Section { group: "A".into(), label: "AB5".into(), sort: 5 },
            "AB6".into() => Section { group: "A".into(), label: "AB6".into(), sort: 6 },
            "AB7".into() => Section { group: "A".into(), label: "AB7".into(), sort: 7 },
            "AB8".into() => Section { group: "A".into(), label: "AB8".into(), sort: 8 },
            "SLB".into() => Section { group: "B".into(), label: "SLB".into(), sort: 9 },
            "LWV".into() => Section { group: "B".into(), label: "LWV".into(), sort: 10 },
            "CAN".into() => Section { group: "B".into(), label: "CAN".into(), sort: 11 },
            "RWV".into() => Section { group: "B".into(), label: "RWV".into(), sort: 12 },
            "ROF".into() => Section { group: "B".into(), label: "ROF".into(), sort: 13 },
            "GLB".into() => Section { group: "B".into(), label: "GLB".into(), sort: 14 },
            "VRT".into() => Section { group: "B".into(), label: "VRT".into(), sort: 15 }
        };
    }
}
