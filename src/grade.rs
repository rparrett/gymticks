use serde::{Deserialize, Serialize};
use indexmap::IndexMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Grade {
    pub group: String,
    pub label: String,
    pub sort: i32
}

impl Grade {
    pub fn defaults() -> IndexMap<String, Grade> {
        return indexmap!{
            "5".into() => Grade { group: "A".into(), label: "5".into(), sort: 1 }, 
            "6".into() => Grade { group: "A".into(), label: "6".into(), sort: 2 }, 
            "7".into() => Grade { group: "A".into(), label: "7".into(), sort: 3 }, 
            "8".into() => Grade { group: "A".into(), label: "8".into(), sort: 4 }, 
            "9".into() => Grade { group: "A".into(), label: "9".into(), sort: 5 }, 
            "10-".into() => Grade { group: "A".into(), label: "10-".into(), sort: 6 }, 
            "10".into() => Grade { group: "A".into(), label: "10".into(), sort: 7 }, 
            "10+".into() => Grade { group: "A".into(), label: "10+".into(), sort: 8 }, 
            "11-".into() => Grade { group: "A".into(), label: "11-".into(), sort: 9 }, 
            "11".into() => Grade { group: "A".into(), label: "11".into(), sort: 10 }, 
            "11+".into() => Grade { group: "A".into(), label: "11+".into(), sort: 11 }, 
            "12-".into() => Grade { group: "A".into(), label: "12-".into(), sort: 12 }, 
            "12".into() => Grade { group: "A".into(), label: "12".into(), sort: 13 }, 
            "12+".into() => Grade { group: "A".into(), label: "12+".into(), sort: 14 },

            "V0-".into() => Grade { group: "B".into(), label: "V0-".into(), sort: 15 }, 
            "V0".into() => Grade { group: "B".into(), label: "V0".into(), sort: 16 }, 
            "V0+".into() => Grade { group: "B".into(), label: "V0+".into(), sort: 17 }, 
            "V1".into() => Grade { group: "B".into(), label: "V1".into(), sort: 18 }, 
            "V2".into() => Grade { group: "B".into(), label: "V2".into(), sort: 19 }, 
            "V3".into() => Grade { group: "B".into(), label: "V3".into(), sort: 20 }, 
            "V4".into() => Grade { group: "B".into(), label: "V4".into(), sort: 21 }, 
            "V5".into() => Grade { group: "B".into(), label: "V5".into(), sort: 22 }, 
            "V6".into() => Grade { group: "B".into(), label: "V6".into(), sort: 23 }, 
            "V7".into() => Grade { group: "B".into(), label: "V7".into(), sort: 24 }, 
        }
    }
}
