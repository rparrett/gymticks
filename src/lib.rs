#[macro_use]
extern crate indexmap;
extern crate serde_json;

use chrono::{DateTime, TimeZone, Utc};
use enclose::enc;
use indexmap::IndexMap;
use itertools::Itertools;
use seed::{prelude::*, *};
use serde::{Deserialize, Serialize};
use std::mem;
use uuid::Uuid;

mod color;
mod grade;
mod section;
mod util;

use crate::color::Color;
use crate::grade::Grade;
use crate::section::Section;

const ENTER_KEY: u32 = 13;
const STORAGE_KEY: &str = "gymticks-11";

type RouteId = Uuid;

// ------ ------
//     Model
// ------ ------

// ------ Model ------

struct Model {
    data: Data,
}

#[derive(Default, Serialize, Deserialize)]
struct Data {
    routes: IndexMap<RouteId, Route>,
    settings: Settings,
    new_route_title: String,
    editing_route: Option<RouteId>,
    chosen_color: String,
    chosen_section: String,
    chosen_grade: String,
    modal_open: bool,
}

#[derive(Default, Serialize, Deserialize)]
struct Settings {
    grades: IndexMap<String, Grade>,
    sections: IndexMap<String, Section>,
    colors: IndexMap<String, Color>,
}

// ------ Route ------

#[derive(Serialize, Deserialize, Debug)]
struct Route {
    title: String,
    completed: bool,
    color: String,
    section: String,
    grade: String,
    ticks: Vec<Tick>,
    retired: bool,
}

// ------ Tick -----
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Tick {
    typ: TickType,
    timestamp: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum TickType {
    Ascent = 0x00,
    Attempt = 0x01,
}

// ------ EditingRoute ------

#[derive(Serialize, Deserialize)]
struct EditingRoute {
    id: RouteId,
    title: String,
}

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    let mut data: Data = LocalStorage::get(STORAGE_KEY).unwrap_or_default();

    data.settings = Settings {
        colors: Color::defaults(),
        sections: Section::defaults(),
        grades: Grade::defaults(),
    };

    // TODO unwrap_or with default values instead of this nonsense?
    if data.chosen_color.is_empty() {
        data.chosen_color = data.settings.colors.iter().next().unwrap().0.to_string();
    }
    if data.chosen_section.is_empty() {
        data.chosen_section = data.settings.sections.iter().next().unwrap().0.to_string();
    }
    if data.chosen_grade.is_empty() {
        data.chosen_grade = data.settings.grades.iter().next().unwrap().0.to_string();
    }

    // TODO actually this, and the stuff above don't really need to be persisted at all.
    // we should probably keep a separate PersistantData and Data.
    data.modal_open = false;

    Model { data }
}

// ------ ------
//    Update
// ------ ------

#[derive(Clone)]
enum Msg {
    NewRouteTitleChanged(String),

    CreateNewRoute(Option<TickType>),

    StartRouteEdit(RouteId),
    SaveEditingRoute,
    RetireEditingRoute,

    AddTickToRoute(RouteId, TickType),

    ChooseColor(String),
    ChooseSection(String),
    ChooseGrade(String),

    OpenModal(),
    CloseModal(),

    ExportData(),
    StartImportData(),
    ImportData(String),

    NoOp,
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::NewRouteTitleChanged(title) => {
            model.data.new_route_title = title;
        }

        Msg::CreateNewRoute(tick_type) => {
            let id = RouteId::new_v4();

            model.data.routes.insert(
                id,
                Route {
                    title: mem::take(&mut model.data.new_route_title),
                    completed: false,
                    ticks: Vec::new(),
                    color: model.data.chosen_color.clone(),
                    section: model.data.chosen_section.clone(),
                    grade: model.data.chosen_grade.clone(),
                    retired: false,
                },
            );

            if let Some(tick_type) = tick_type {
                orders.send_msg(Msg::AddTickToRoute(id, tick_type));
            };

            let settings = &model.data.settings;

            model.data.routes.sort_by(|_ak, av, _bk, bv| {
                return settings
                    .sections
                    .get(&av.section)
                    .map_or(0i32, |s| s.sort)
                    .cmp(&settings.sections.get(&bv.section).map_or(0i32, |s| s.sort))
                    .then(
                        settings
                            .colors
                            .get(&av.color)
                            .map_or(0i32, |s| s.sort)
                            .cmp(&settings.colors.get(&bv.color).map_or(0i32, |s| s.sort)),
                    )
                    .then(
                        settings
                            .grades
                            .get(&av.grade)
                            .map_or(0i32, |s| s.sort)
                            .cmp(&settings.grades.get(&bv.grade).map_or(0i32, |s| s.sort)),
                    )
                    .then(av.title.cmp(&bv.title));
            });

            model.data.modal_open = false;
        }

        Msg::StartRouteEdit(route_id) => {
            if let Some(route) = model.data.routes.get(&route_id) {
                model.data.editing_route = Some(route_id);
                model.data.chosen_color = route.color.clone();
                model.data.chosen_section = route.section.clone();
                model.data.chosen_grade = route.grade.clone();
                model.data.new_route_title = route.title.clone();
            }

            model.data.modal_open = true;
        }
        Msg::SaveEditingRoute => {
            if let Some(editing_route) = model.data.editing_route.take() {
                if let Some(route) = model.data.routes.get_mut(&editing_route) {
                    route.title = mem::take(&mut model.data.new_route_title);
                    route.color = model.data.chosen_color.clone();
                    route.section = model.data.chosen_section.clone();
                    route.grade = model.data.chosen_grade.clone();

                    // TODO: this code is duplicated.
                    let settings = &model.data.settings;

                    model.data.routes.sort_by(|_ak, av, _bk, bv| {
                        return settings
                            .sections
                            .get(&av.section)
                            .map_or(0i32, |s| s.sort)
                            .cmp(&settings.sections.get(&bv.section).map_or(0i32, |s| s.sort))
                            .then(
                                settings
                                    .colors
                                    .get(&av.color)
                                    .map_or(0i32, |s| s.sort)
                                    .cmp(&settings.colors.get(&bv.color).map_or(0i32, |s| s.sort)),
                            )
                            .then(
                                settings
                                    .grades
                                    .get(&av.grade)
                                    .map_or(0i32, |s| s.sort)
                                    .cmp(&settings.grades.get(&bv.grade).map_or(0i32, |s| s.sort)),
                            )
                            .then(av.title.cmp(&bv.title));
                    });
                }
            }

            model.data.modal_open = false;
            model.data.editing_route = None;
        }
        Msg::RetireEditingRoute => {
            if let Some(editing_route) = model.data.editing_route.take() {
                if let Some(route) = model.data.routes.get_mut(&editing_route) {
                    route.retired = true;
                }
            }

            model.data.modal_open = false;
            model.data.editing_route = None;
        }

        Msg::AddTickToRoute(route_id, typ) => {
            if let Some(route) = model.data.routes.get_mut(&route_id) {
                let timestamp = unixTimestamp();
                route.ticks.push(Tick { typ, timestamp });
            }
        }

        Msg::ChooseColor(color) => {
            model.data.chosen_color = color;
        }

        Msg::ChooseSection(section) => {
            model.data.chosen_section = section;
        }

        Msg::ChooseGrade(grade) => {
            model.data.chosen_grade = grade;
        }

        Msg::OpenModal() => {
            model.data.editing_route = None;
            model.data.new_route_title = "".to_string();

            model.data.modal_open = true;
        }

        Msg::CloseModal() => {
            model.data.modal_open = false;
        }

        Msg::ExportData() => {
            if let Ok(json) = serde_json::to_string(&model.data) {
                exportData(json);
            }
        }

        Msg::StartImportData() => {
            startImportData();
        }

        Msg::ImportData(json) => {
            // TODO fail less silently
            if let Ok(new_data) = serde_json::from_str(&json) {
                mem::replace(&mut model.data, new_data);
            }
        }

        Msg::NoOp => (),
    }

    // Save data into LocalStorage. It should be optimized in a real-world application.
    LocalStorage::insert(STORAGE_KEY, &model.data).expect("save data to LocalStorage");
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> Vec<Node<Msg>> {
    let data = &model.data;
    nodes![
        header![
            C!["navbar"],
            section![
                C!["navbar-section"],
                a![attrs! {
                    At::Href => "#"
                }],
            ],
            section![C!["navbar-center"], "gymticks"],
            section![
                C!["navbar-section"],
                button![
                    C!["btn btn-primary"],
                    ev(Ev::Click, move |_| Msg::OpenModal()),
                    i![C!["icon", "icon-plus"]]
                ]
            ]
        ],
        if data.routes.is_empty() {
            vec![]
        } else {
            vec![div![
                C!["container grid-sm"],
                view_main(&data.routes),
                view_aggregate(&data.routes),
            ]]
        },
        view_footer(),
        view_modal(
            &data.modal_open,
            &data.new_route_title,
            &data.editing_route,
            &data.chosen_color,
            &data.chosen_section,
            &data.chosen_grade,
            &data.settings.colors,
            &data.settings.sections,
            &data.settings.grades,
        ),
    ]
}

// ------ header ------

fn view_modal(
    modal_open: &bool,
    new_route_title: &str,
    editing_route: &Option<RouteId>,
    chosen_color: &String,
    chosen_section: &String,
    chosen_grade: &String,
    colors: &IndexMap<String, Color>,
    sections: &IndexMap<String, Section>,
    grades: &IndexMap<String, Grade>,
) -> Node<Msg> {
    div![
        C!["modal", IF!(*modal_open => "active")],
        a![
            C!["modal-overlay"],
            ev(Ev::Click, move |_| Msg::CloseModal())
        ],
        div![
            C!["modal-container"],
            div![
                C!["modal-body"],
                div![
                    C!["content"],
                    div![div![
                        C!["description-and-flag"],
                        div![
                            C![chosen_color.as_str(), "color-flag"],
                            div![chosen_section],
                            div![chosen_grade],
                        ],
                        input![
                            C!["form-input"],
                            attrs! {
                                At::Placeholder => "Description of route";
                                At::AutoFocus => true.as_at_value();
                                At::Value => new_route_title;
                            },
                            keyboard_ev(Ev::KeyDown, |keyboard_event| {
                                if keyboard_event.key_code() == ENTER_KEY {
                                    Msg::CreateNewRoute(None)
                                } else {
                                    Msg::NoOp
                                }
                            }),
                            input_ev(Ev::Input, Msg::NewRouteTitleChanged),
                        ],
                    ],],
                    div![
                        C!["color-chooser",],
                        colors
                            .iter()
                            .map(|(key, _color)| {
                                div![
                                    C![
                                       key.as_str(),
                                       IF!(chosen_color == key => "active"),
                                    ],
                                    ev(
                                        Ev::Click,
                                        enc!((key) move |_| Msg::ChooseColor(key.to_string()))
                                    )
                                ]
                            })
                            .collect::<Vec<Node<Msg>>>()
                    ],
                    div![
                        C!["section-chooser",],
                        sections
                            .iter()
                            .group_by(|(_k, v)| v.group.to_owned())
                            .into_iter()
                            .map(|(_key, group)| {
                                div![
                                    C!["section-chooser-row"],
                                    group
                                        .map(|(key, _section)| {
                                            div![
                                                C![
                                                   key.as_str(),
                                                   "section-chooser-item",
                                                   IF!(chosen_section == key => "active"),
                                                ],
                                                ev(
                                                    Ev::Click,
                                                    enc!((key) move |_| Msg::ChooseSection(key.to_string()))
                                                ),
                                                key
                                            ]
                                        })
                                        .collect::<Vec<Node<Msg>>>()
                                ]
                            })
                            .collect::<Vec<Node<Msg>>>(),
                    ],
                    div![
                        C!["section-chooser",],
                        grades
                            .iter()
                            .group_by(|(_k, v)| v.group.to_owned())
                            .into_iter()
                            .map(|(_key, group)| {
                                div![
                                    C!["section-chooser-row"],
                                    group
                                        .map(|(key, _grade)| {
                                            div![
                                                C![
                                                   key.as_str(),
                                                   "section-chooser-item",
                                                   IF!(chosen_grade == key => "active"),
                                                ],
                                                ev(
                                                    Ev::Click,
                                                    enc!((key) move |_| Msg::ChooseGrade(key.to_string()))
                                                ),
                                                key
                                            ]
                                        })
                                        .collect::<Vec<Node<Msg>>>()
                                ]
                            })
                            .collect::<Vec<Node<Msg>>>()
                    ],
                ],
                if editing_route.is_some() {
                    div![
                        C!["modal-buttons"],
                        button![
                            C!["btn btn-primary new-route-button"],
                            ev(Ev::Click, |_| Msg::SaveEditingRoute),
                            "Save Changes"
                        ],
                        button![
                            C!["btn btn-error new-route-button"],
                            ev(Ev::Click, |_| Msg::RetireEditingRoute),
                            "Retire Route"
                        ],
                    ]
                } else {
                    div![
                        C!["modal-buttons"],
                        button![
                            C!["btn btn-primary new-route-button"],
                            ev(Ev::Click, move |_| Msg::CreateNewRoute(Some(
                                TickType::Ascent
                            ))),
                            "SND"
                        ],
                        button![
                            C!["btn new-route-button"],
                            ev(Ev::Click, move |_| Msg::CreateNewRoute(Some(
                                TickType::Attempt
                            ))),
                            "ATT"
                        ],
                        button![
                            C!["btn btn-secondary new-route-button"],
                            ev(Ev::Click, move |_| Msg::CreateNewRoute(None)),
                            "Add Without Tick"
                        ],
                    ]
                }
            ],
        ]
    ]
}

// ------ main ------

fn view_main(routes: &IndexMap<RouteId, Route>) -> Node<Msg> {
    section![routes
        .iter()
        .filter(|(_k, v)| !v.retired)
        .group_by(|(_k, v)| v.section.to_owned())
        .into_iter()
        .map(|(_k, group)| {
            let route_ids = group.into_iter().map(|(k, _v)| k.clone()).collect();

            div![C!["main card"], div![view_routes(routes, route_ids)]]
        })
        .collect::<Vec<Node<Msg>>>()]
}

fn view_routes(routes: &IndexMap<RouteId, Route>, route_ids: Vec<RouteId>) -> Node<Msg> {
    let time = Utc.timestamp(unixTimestamp().into(), 0);

    ul![
        C!["route-list"],
        route_ids
            .iter()
            .filter_map(|route_id| {
                if let Some(route) = routes.get(route_id) {
                    Some(view_route(route_id, route, &time))
                } else {
                    None
                }
            })
            .collect::<Vec<Node<Msg>>>()
    ]
}

fn view_route(route_id: &RouteId, route: &Route, time: &DateTime<Utc>) -> Node<Msg> {
    let mut num_ascents = 0;
    let mut num_attempts = 0;
    let mut attempts_to_ascent = 0;
    let mut attempts_since_ascent = 0;
    let mut last_ascent = 0;
    let mut last_attempt = 0;
    let mut _ascent_streak = 0;

    // TODO: can we iterate our way out of this mess?

    for tick in &route.ticks {
        match tick.typ {
            TickType::Ascent => {
                last_ascent = tick.timestamp;
                num_ascents += 1;
                attempts_since_ascent = 0;
                _ascent_streak += 1;
            }
            TickType::Attempt => {
                _ascent_streak = 0;
                last_attempt = tick.timestamp;
                num_attempts += 1;
                if num_ascents > 0 {
                    attempts_since_ascent += 1;
                } else {
                    attempts_to_ascent += 1;
                }
            }
        }
    }

    let ascent_text = if num_ascents == 0 {
        String::from("unsent")
    } else if attempts_to_ascent == 0 {
        format!("{} snd (flsh)", num_ascents)
    } else if attempts_to_ascent > 0 {
        format!("{} snd ({} att)", num_ascents, attempts_to_ascent)
    } else {
        // unreachable?
        String::new()
    };

    let att_text = if num_attempts == 0 && num_ascents == 0 {
        String::from("unattempted")
    } else if num_ascents == 0 {
        format!(
            "{} att (att {})",
            num_attempts,
            util::time_diff_in_words(Utc.timestamp(last_attempt.into(), 0), *time)
        )
    } else if last_ascent >= last_attempt {
        format!(
            "{} att (snd {})",
            attempts_since_ascent,
            util::time_diff_in_words(Utc.timestamp(last_ascent.into(), 0), *time)
        )
    } else {
        format!(
            "{} att (att {})",
            attempts_since_ascent,
            util::time_diff_in_words(Utc.timestamp(last_attempt.into(), 0), *time)
        )
    };

    li![
        C![IF!(num_ascents > 0 => "completed")],
        div![
            C!["view"],
            div![
                C![route.color.as_str(), "color-flag"],
                div![route.section.as_str()],
                div![route.grade.as_str()],
                ev(
                    Ev::Click,
                    enc!((route_id) move |_| Msg::StartRouteEdit(route_id))
                ),
            ],
            button![
                C!["tick-button btn btn-primary"],
                ev(
                    Ev::Click,
                    enc!((route_id) move |_| Msg::AddTickToRoute(route_id, TickType::Ascent))
                ),
                "SND"
            ],
            button![
                C!["tick-button btn"],
                ev(
                    Ev::Click,
                    enc!((route_id) move |_| Msg::AddTickToRoute(route_id, TickType::Attempt))
                ),
                "ATT"
            ],
            label![
                ev(
                    Ev::DblClick,
                    enc!((route_id) move |_| Msg::StartRouteEdit(route_id))
                ),
                route.title.as_str()
            ],
            div![
                C!["stats"],
                div![C!["stats-ascents"], ascent_text,],
                div![C!["stats-attempts"], att_text,],
            ]
        ],
    ]
}

fn view_aggregate(routes: &IndexMap<RouteId, Route>) -> Node<Msg> {
    let midnight = midnight();

    let mut today = 0;
    let mut total = 0;

    for tick in routes.iter().flat_map(|route| &route.1.ticks) {
        match tick.typ {
            TickType::Ascent if tick.timestamp > midnight => {
                today += 1;
                total += 1;
            }
            TickType::Ascent => {
                total += 1;
            }
            _ => {}
        }
    }

    div![
        C!["aggregate", "card"],
        div![C!["card-header"], div![C!["h5", "card-title"], "Stats"]],
        div![
            C!["card-body"],
            table![
                tr![td!["Sends Today"], td![format!("{}", today)]],
                tr![td!["Sends Total"], td![format!("{}", total)]]
            ],
        ]
    ]
}

// ------ footer ------

fn view_footer() -> Node<Msg> {
    footer![
        C!["footer", "grid-sm", "info"],
        p![
            "created by ",
            a![
                attrs! {
                    At::Href => "https://github.com/rparrett/"
                },
                "rob parrett"
            ],
            " with ",
            a![
                attrs! {
                    At::Href => "https://github.com/seed-rs/"
                },
                "seed-rs"
            ],
        ],
        p![
            a![
                attrs! {
                    At::Href => "#"
                },
                "export data",
                ev(Ev::Click, move |_| Msg::ExportData()),
            ],
            "\u{00A0}\u{00B7}\u{00A0}",
            a![
                attrs! {
                    At::Href => "#"
                },
                "import data",
                ev(Ev::Click, move |_| Msg::StartImportData()),
            ]
        ]
    ]
}

#[wasm_bindgen]
extern "C" {
    fn unixTimestamp() -> i32;
    fn midnight() -> i32;
    fn exportData(data: String);
    fn startImportData();
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen]
pub fn start() -> Box<[JsValue]> {
    let app = App::start("app", init, update, view);

    create_closures_for_js(&app)
}

fn create_closures_for_js(app: &App<Msg, Model, Vec<Node<Msg>>>) -> Box<[JsValue]> {
    let import_data = wrap_in_permanent_closure(enc!((app) move |data| {
        app.update(Msg::ImportData(data))
    }));

    vec![import_data].into_boxed_slice()
}

fn wrap_in_permanent_closure<T>(f: impl FnMut(T) + 'static) -> JsValue
where
    T: wasm_bindgen::convert::FromWasmAbi + 'static,
{
    // `Closure::new` isn't in `stable` Rust (yet) - it's a custom implementation from Seed.
    // If you need more flexibility, use `Closure::wrap`.
    let closure = Closure::new(f);
    let closure_as_js_value = closure.as_ref().clone();
    // `forget` leaks `Closure` - we should use it only when
    // we want to call given `Closure` more than once.
    closure.forget();
    closure_as_js_value
}
