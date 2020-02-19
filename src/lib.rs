use chrono::{DateTime, TimeZone, Utc};
use enclose::enc;
use indexmap::IndexMap;
use seed::{
    browser::service::storage::{self, Storage},
    prelude::*,
    *,
};
use serde::{Deserialize, Serialize};
use std::mem;
use uuid::Uuid;
use web_sys::HtmlInputElement;

mod util;

const ENTER_KEY: u32 = 13;
const ESC_KEY: u32 = 27;
const STORAGE_KEY: &str = "gymticks-8";

type RouteId = Uuid;

const COLORS: [&str; 10] = [
    "orange", "red", "pink", "purple", "blue", "brown", "yellow", "green", "white", "black",
];

const SECTIONS: [&str; 8] = ["AB1", "AB2", "AB3", "AB4", "AB5", "AB6", "AB7", "AB8"];

const ROUTEGRADES: [&str; 14] = [
    "5", "6", "7", "8", "9", "10-", "10", "10+", "11-", "11", "11+", "12-", "12", "12+",
];

const BOULDERGRADES: [&str; 11] = [
    "V0-", "V0", "V0+", "V1", "V2", "V3", "V4", "V5", "V6", "V7", "?",
];

// ------ ------
//     Model
// ------ ------

// ------ Model ------

struct Model {
    data: Data,
    services: Services,
    refs: Refs,
}

#[derive(Default, Serialize, Deserialize)]
struct Data {
    routes: IndexMap<RouteId, Route>,
    new_route_title: String,
    editing_route: Option<EditingRoute>,
    chosen_color: String,
    chosen_section: String,
    chosen_grade: String,
    modal_open: bool,
}

struct Services {
    local_storage: Storage,
}

#[derive(Default)]
struct Refs {
    editing_route_input: ElRef<HtmlInputElement>,
}

// ------ Route ------

#[derive(Serialize, Deserialize)]
struct Route {
    title: String,
    completed: bool,
    color: String,
    section: String,
    grade: String,
    ticks: Vec<Tick>,
}

// ------ Tick -----
#[derive(Serialize, Deserialize)]
struct Tick {
    typ: TickType,
    timestamp: i32,
}

#[derive(Serialize, Deserialize)]
enum TickType {
    Send = 0x00,
    Attempt = 0x01,
}

// ------ EditingRoute ------

#[derive(Serialize, Deserialize)]
struct EditingRoute {
    id: RouteId,
    title: String,
}

// ------ ------
//  After Mount
// ------ ------

fn after_mount(_: Url, _: &mut impl Orders<Msg>) -> AfterMount<Model> {
    let local_storage = storage::get_storage().expect("get `LocalStorage`");
    let mut data: Data = storage::load_data(&local_storage, STORAGE_KEY).unwrap_or_default();

    // TODO unwrap_or with default values instead of this nonsense?
    if data.chosen_color.is_empty() {
        data.chosen_color = COLORS[0].to_string();
    }
    if data.chosen_section.is_empty() {
        data.chosen_section = SECTIONS[0].to_string();
    }
    if data.chosen_grade.is_empty() {
        data.chosen_grade = ROUTEGRADES[0].to_string();
    }

    // TODO actually this, and the stuff above don't really need to be persisted at all.
    // we should probably keep a separate PersistantData and Data.
    data.modal_open = false;

    AfterMount::new(Model {
        data,
        services: Services { local_storage },
        refs: Refs::default(),
    })
}

// ------ ------
//    Update
// ------ ------

enum Msg {
    NewRouteTitleChanged(String),

    CreateNewRoute,
    RemoveRoute(RouteId),

    StartRouteEdit(RouteId),
    EditingRouteTitleChanged(String),
    SaveEditingRoute,
    CancelRouteEdit,

    AddTickToRoute(RouteId, TickType),

    ChooseColor(String),
    ChooseSection(String),
    ChooseGrade(String),

    OpenModal(),
    CloseModal(),

    NoOp,
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    let data = &mut model.data;
    match msg {
        Msg::NewRouteTitleChanged(title) => {
            data.new_route_title = title;
        }

        Msg::CreateNewRoute => {
            data.routes.insert(
                RouteId::new_v4(),
                Route {
                    title: mem::take(&mut data.new_route_title),
                    completed: false,
                    ticks: Vec::new(),
                    color: data.chosen_color.clone(),
                    section: data.chosen_section.clone(),
                    grade: data.chosen_grade.clone(),
                },
            );
            data.routes.sort_by(|_ak, av, _bk, bv| {
                // TODO this concatenation seems inefficient, but I have no
                // idea how to sort by multiple criteria
                let a = av.section.clone() + &av.grade + &av.title;
                let b = bv.section.clone() + &bv.grade + &bv.title;

                return a.cmp(&b);
            });
            data.modal_open = false;
        }
        Msg::RemoveRoute(route_id) => {
            data.routes.shift_remove(&route_id);
        }

        Msg::StartRouteEdit(route_id) => {
            if let Some(route) = data.routes.get(&route_id) {
                data.editing_route = Some({
                    EditingRoute {
                        id: route_id,
                        title: route.title.clone(),
                    }
                });
            }

            let input = model.refs.editing_route_input.clone();
            orders.after_next_render(move |_| {
                input.get().expect("get `editing_route_input`").select();
                Msg::NoOp
            });
        }
        Msg::EditingRouteTitleChanged(title) => {
            if let Some(ref mut editing_route) = data.editing_route {
                editing_route.title = title
            }
        }
        Msg::SaveEditingRoute => {
            if let Some(editing_route) = data.editing_route.take() {
                if let Some(route) = data.routes.get_mut(&editing_route.id) {
                    route.title = editing_route.title;

                    // TODO: this code is duplicated. can we just implement some
                    // trait for a Route and use .sort?
                    data.routes.sort_by(|_ak, av, _bk, bv| {
                        // TODO this concatenation seems inefficient, but I have no
                        // idea how to sort by multiple criteria
                        let a = av.section.clone() + &av.grade + &av.title;
                        let b = bv.section.clone() + &bv.grade + &bv.title;

                        return a.cmp(&b);
                    })
                }
            }
        }
        Msg::CancelRouteEdit => {
            data.editing_route = None;
        }

        Msg::AddTickToRoute(route_id, typ) => {
            if let Some(route) = data.routes.get_mut(&route_id) {
                let timestamp = unixTimestamp();
                route.ticks.push(Tick { typ, timestamp });
            }
        }

        Msg::ChooseColor(color) => {
            data.chosen_color = color;
        }

        Msg::ChooseSection(section) => {
            data.chosen_section = section;
        }

        Msg::ChooseGrade(grade) => {
            data.chosen_grade = grade;
        }

        Msg::OpenModal() => {
            data.modal_open = true;
        }

        Msg::CloseModal() => {
            data.modal_open = false;
        }

        Msg::NoOp => (),
    }
    // Save data into LocalStorage. It should be optimized in a real-world application.
    storage::store_data(&model.services.local_storage, STORAGE_KEY, &data);
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> impl View<Msg> {
    let data = &model.data;
    nodes![
        div![
            class!["topbar"],
            div![
                "gymticks"
            ],
            div![
                button![
                    ev(Ev::Click, move |_| Msg::OpenModal()),
                    "+"
                ]
            ]
        ],
        div![
            class!["modal-container", "open" => &data.modal_open],
            div![
                class!["modal"],
                view_header(
                    &data.new_route_title,
                    &data.chosen_color,
                    &data.chosen_section,
                    &data.chosen_grade
                ),
            ]
        ],
        if data.routes.is_empty() {
            vec![]
        } else {
            vec![
                view_main(
                    &data.routes,
                    &data.editing_route,
                    &model.refs.editing_route_input,
                ),
                view_footer(),
            ]
        },
        div![
            class!["modal-bg", "open" => &data.modal_open],
            ev(Ev::Click, move |_| Msg::CloseModal())
        ]
    ]
}

// ------ header ------

fn view_header(
    new_route_title: &str,
    chosen_color: &String,
    chosen_section: &String,
    chosen_grade: &String,
) -> Node<Msg> {
    header![
        class!["header"],
        h1!["gymticks"],
        div![
            input![
                class!["new-route"],
                attrs! {
                    At::Placeholder => "Description of route";
                    At::AutoFocus => true.as_at_value();
                    At::Value => new_route_title;
                },
                keyboard_ev(Ev::KeyDown, |keyboard_event| {
                    if keyboard_event.key_code() == ENTER_KEY {
                        Msg::CreateNewRoute
                    } else {
                        Msg::NoOp
                    }
                }),
                input_ev(Ev::Input, Msg::NewRouteTitleChanged),
            ],
            div![
                class![
                   "color-chooser",
                ],
                COLORS.iter().filter_map(|hex| {
                    Some(div![
                        class![
                           hex.as_ref(),
                           "active" => chosen_color == hex
                        ],
                        ev(Ev::Click, move |_| Msg::ChooseColor(hex.to_string()))
                    ])
                })
            ],
            div![
                class![
                   "section-chooser",
                ],
                SECTIONS.iter().filter_map(|abbrev| {
                    Some(div![
                        class![
                           abbrev.as_ref(),
                           "active" => chosen_section == abbrev
                        ],
                        ev(Ev::Click, move |_| Msg::ChooseSection(abbrev.to_string())),
                        abbrev
                    ])
                })
            ],
            div![
                class![
                   "grade-chooser",
                ],
                ROUTEGRADES.iter().filter_map(|grade| {
                    Some(div![
                        class![
                           grade.as_ref(),
                           "active" => chosen_grade == grade
                        ],
                        ev(Ev::Click, move |_| Msg::ChooseGrade(grade.to_string())),
                        grade
                    ])
                })
            ],
            div![
                class![
                   "grade-chooser",
                ],
                BOULDERGRADES.iter().filter_map(|grade| {
                    Some(div![
                        class![
                           grade.as_ref(),
                           "active" => chosen_grade == grade
                        ],
                        ev(Ev::Click, move |_| Msg::ChooseGrade(grade.to_string())),
                        grade
                    ])
                })
            ],
            button![
                id!("toggle-color"),
                class![chosen_color.as_str(), "toggle-color"],
                div![chosen_section.as_str()],
                div![chosen_grade.as_str()]
            ],
        ],
        button![
            class!["tick-button new-route-button"],
            ev(Ev::Click, move |_| Msg::CreateNewRoute),
            "Add Route"
        ]
    ]
}

// ------ main ------

fn view_main(
    routes: &IndexMap<RouteId, Route>,
    editing_route: &Option<EditingRoute>,
    editing_route_input: &ElRef<HtmlInputElement>,
) -> Node<Msg> {
    section![
        class!["main"],
        view_routes(routes, editing_route, editing_route_input)
    ]
}

fn view_routes(
    routes: &IndexMap<RouteId, Route>,
    editing_route: &Option<EditingRoute>,
    editing_route_input: &ElRef<HtmlInputElement>,
) -> Node<Msg> {
    let time = Utc.timestamp(unixTimestamp().into(), 0);

    ul![
        class!["route-list"],
        routes.iter().filter_map(|(route_id, route)| {
            Some(view_route(
                route_id,
                route,
                editing_route,
                editing_route_input,
                &time,
            ))
        })
    ]
}

fn view_route(
    route_id: &RouteId,
    route: &Route,
    editing_route: &Option<EditingRoute>,
    editing_route_input: &ElRef<HtmlInputElement>,
    time: &DateTime<Utc>,
) -> Node<Msg> {
    let mut num_sends = 0;
    let mut num_attempts = 0;
    let mut attempts_to_send = 0;
    let mut attempts_since_send = 0;

    for tick in &route.ticks {
        match tick.typ {
            TickType::Send => {
                num_sends = num_sends + 1;
                attempts_since_send = 0;
            }
            TickType::Attempt => {
                num_attempts = num_attempts + 1;
                if num_sends > 0 {
                    attempts_since_send = attempts_since_send + 1;
                } else {
                    attempts_to_send = attempts_to_send + 1;
                }
            }
        }
    }

    li![
        class![
           "completed" => num_sends > 0,
           "editing" => match editing_route {
               Some(editing_route) if &editing_route.id == route_id => true,
               _ => false
           }
        ],
        div![
            class!["view"],
            div![
                class![route.color.as_ref(), "color-flag"],
                div![route.section],
                div![route.grade],
            ],
            button![
                class!["tick-button"],
                ev(
                    Ev::Click,
                    enc!((route_id) move |_| Msg::AddTickToRoute(route_id, TickType::Send))
                ),
                label!["SND"]
            ],
            button![
                class!["tick-button"],
                ev(
                    Ev::Click,
                    enc!((route_id) move |_| Msg::AddTickToRoute(route_id, TickType::Attempt))
                ),
                label!["ATT"]
            ],
            label![
                ev(
                    Ev::DblClick,
                    enc!((route_id) move |_| Msg::StartRouteEdit(route_id))
                ),
                route.title
            ],
            div![
                class!["stats"],
                div![
                    class!["stats-sends"],
                    div![num_sends.to_string()],
                    div![route.ticks.last().map_or_else(
                        || String::new(),
                        |tick| {
                            format!(
                                "{}",
                                util::time_diff_in_words(
                                    Utc.timestamp(tick.timestamp.into(), 0),
                                    *time
                                )
                            )
                        }
                    )]
                ],
                div![
                    class!["stats-attempts"],
                    div![attempts_to_send.to_string()],
                    div![attempts_since_send.to_string()]
                ],
            ],
            button![
                class!["destroy"],
                ev(
                    Ev::Click,
                    enc!((route_id) move |_| Msg::RemoveRoute(route_id))
                )
            ]
        ],
        match editing_route {
            Some(editing_route) if &editing_route.id == route_id => {
                input![
                    el_ref(editing_route_input),
                    class!["edit"],
                    attrs! {At::Value => editing_route.title},
                    ev(Ev::Blur, |_| Msg::SaveEditingRoute),
                    input_ev(Ev::Input, Msg::EditingRouteTitleChanged),
                    keyboard_ev(Ev::KeyDown, |keyboard_event| {
                        // @TODO rafactor to `match` once it can accept constants
                        let code = keyboard_event.key_code();
                        if code == ENTER_KEY {
                            Msg::SaveEditingRoute
                        } else if code == ESC_KEY {
                            Msg::CancelRouteEdit
                        } else {
                            Msg::NoOp
                        }
                    }),
                ]
            }
            _ => empty![],
        }
    ]
}

// ------ footer ------

fn view_footer() -> Node<Msg> {
    footer![class!["footer"],]
}

#[wasm_bindgen]
extern "C" {
    fn unixTimestamp() -> i32;
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn render() {
    App::builder(update, view)
        .after_mount(after_mount)
        .build_and_start();
}
