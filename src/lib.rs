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
    //    "orange", "red", "pink", "purple", "blue", "brown", "yellow", "green", "white", "black",
    "red", "orange", "yellow", "green", "blue", "purple", "pink", "brown", "white", "black",
];

const SECTIONS: [&str; 8] = ["AB1", "AB2", "AB3", "AB4", "AB5", "AB6", "AB7", "AB8"];

const BOULDERSECTIONS: [&str; 7] = ["VRT", "GLB", "ROF", "RWV", "CAN", "LWF", "SLB"];

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
}

#[derive(Default, Serialize, Deserialize)]
struct Data {
    routes: IndexMap<RouteId, Route>,
    new_route_title: String,
    editing_route: Option<RouteId>,
    chosen_color: String,
    chosen_section: String,
    chosen_grade: String,
    modal_open: bool,
}

struct Services {
    local_storage: Storage,
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

fn before_mount(_url: Url) -> BeforeMount {
    BeforeMount::new()
        .mount_point("app")
        .mount_type(MountType::Takeover)
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
    })
}

// ------ ------
//    Update
// ------ ------

enum Msg {
    NewRouteTitleChanged(String),

    CreateNewRoute(Option<TickType>),
    RemoveRoute(RouteId),

    StartRouteEdit(RouteId),
    SaveEditingRoute,

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

        Msg::CreateNewRoute(tick_type) => {
            let id = RouteId::new_v4();

            data.routes.insert(
                id,
                Route {
                    title: mem::take(&mut data.new_route_title),
                    completed: false,
                    ticks: Vec::new(),
                    color: data.chosen_color.clone(),
                    section: data.chosen_section.clone(),
                    grade: data.chosen_grade.clone(),
                },
            );

            if let Some(tick_type) = tick_type {
                orders.send_msg(Msg::AddTickToRoute(id, tick_type));
            };

            data.routes.sort_by(|_ak, av, _bk, bv| {
                return av
                    .section
                    .cmp(&bv.section)
                    .then(av.color.cmp(&bv.color))
                    .then(av.grade.cmp(&bv.grade))
                    .then(av.title.cmp(&bv.title));
            });

            data.modal_open = false;
        }
        Msg::RemoveRoute(route_id) => {
            data.routes.shift_remove(&route_id);
        }

        Msg::StartRouteEdit(route_id) => {
            if let Some(route) = data.routes.get(&route_id) {
                data.editing_route = Some(route_id);
                data.chosen_color = route.color.clone();
                data.chosen_section = route.section.clone();
                data.chosen_grade = route.grade.clone();
                data.new_route_title = route.title.clone();
            }

            data.modal_open = true;
        }
        Msg::SaveEditingRoute => {
            if let Some(editing_route) = data.editing_route.take() {
                if let Some(route) = data.routes.get_mut(&editing_route) {
                    route.title = mem::take(&mut data.new_route_title);
                    route.color = data.chosen_color.clone();
                    route.section = data.chosen_section.clone();
                    route.grade = data.chosen_grade.clone();

                    // TODO: this code is duplicated. can we just implement some
                    // trait for a Route and use .sort? We might want different
                    // sort options though.
                    data.routes.sort_by(|_ak, av, _bk, bv| {
                        return av
                            .section
                            .cmp(&bv.section)
                            .then(av.color.cmp(&bv.color))
                            .then(av.grade.cmp(&bv.grade))
                            .then(av.title.cmp(&bv.title));
                    })
                }
            }

            data.modal_open = false;
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
        header![
            class!["navbar"],
            section![
                class!["navbar-section"],
                a![attrs! {
                    At::Href => "#"
                }],
            ],
            section![class!["navbar-center"], "gymticks"],
            section![
                class!["navbar-section"],
                button![
                    class!["btn btn-primary"],
                    ev(Ev::Click, move |_| Msg::OpenModal()),
                    i![class!["icon", "icon-plus"]]
                ]
            ]
        ],
        if data.routes.is_empty() {
            vec![]
        } else {
            vec![div![
                class!["container grid-sm"],
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
            &data.chosen_grade
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
) -> Node<Msg> {
    div![
        class!["modal", "active" => modal_open],
        a![
            class!["modal-overlay"],
            ev(Ev::Click, move |_| Msg::CloseModal())
        ],
        div![
            class!["modal-container"],
            div![
                class!["modal-body"],
                div![
                    class!["content"],
                    div![div![
                        class!["description-and-flag"],
                        div![
                            class![chosen_color.as_ref(), "color-flag"],
                            div![chosen_section],
                            div![chosen_grade],
                        ],
                        input![
                            class!["form-input"],
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
                        class!["color-chooser",],
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
                        class!["section-chooser",],
                        div![
                            class!["section-chooser-row"],
                            SECTIONS.iter().filter_map(|abbrev| {
                                Some(div![
                                    class![
                                       abbrev.as_ref(),
                                       "active" => chosen_section == abbrev,
                                       "section-chooser-item"
                                    ],
                                    ev(Ev::Click, move |_| Msg::ChooseSection(abbrev.to_string())),
                                    abbrev
                                ])
                            })
                        ],
                        div![
                            class!["section-chooser-row"],
                            BOULDERSECTIONS.iter().filter_map(|abbrev| {
                                Some(div![
                                    class![
                                       abbrev.as_ref(),
                                       "active" => chosen_section == abbrev,
                                       "section-chooser-item"
                                    ],
                                    ev(Ev::Click, move |_| Msg::ChooseSection(abbrev.to_string())),
                                    abbrev
                                ])
                            })
                        ]
                    ],
                    div![
                        class!["section-chooser",],
                        div![
                            class!["section-chooser-row"],
                            ROUTEGRADES.iter().filter_map(|grade| {
                                Some(div![
                                    class![
                                       grade.as_ref(),
                                       "active" => chosen_grade == grade,
                                       "section-chooser-item"
                                    ],
                                    ev(Ev::Click, move |_| Msg::ChooseGrade(grade.to_string())),
                                    grade
                                ])
                            })
                        ],
                        div![
                            class!["section-chooser-row"],
                            BOULDERGRADES.iter().filter_map(|grade| {
                                Some(div![
                                    class![
                                       grade.as_ref(),
                                       "active" => chosen_grade == grade,
                                       "section-chooser-item"
                                    ],
                                    ev(Ev::Click, move |_| Msg::ChooseGrade(grade.to_string())),
                                    grade
                                ])
                            })
                        ]
                    ],
                ],
                if editing_route.is_some() {
                    div![
                        class!["modal-buttons"],
                        button![
                            class!["btn btn-primary new-route-button"],
                            ev(Ev::Click, move |_| Msg::SaveEditingRoute),
                            "Save Changes"
                        ]
                    ]
                } else {
                    div![
                        class!["modal-buttons"],
                        button![
                            class!["btn btn-primary new-route-button"],
                            ev(Ev::Click, move |_| Msg::CreateNewRoute(Some(
                                TickType::Send
                            ))),
                            "SND"
                        ],
                        button![
                            class!["btn new-route-button"],
                            ev(Ev::Click, move |_| Msg::CreateNewRoute(Some(
                                TickType::Attempt
                            ))),
                            "ATT"
                        ],
                        button![
                            class!["btn btn-secondary new-route-button"],
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
    section![class!["main card"], div![view_routes(routes)]]
}

fn view_routes(routes: &IndexMap<RouteId, Route>) -> Node<Msg> {
    let time = Utc.timestamp(unixTimestamp().into(), 0);

    ul![
        class!["route-list"],
        routes
            .iter()
            .filter_map(|(route_id, route)| { Some(view_route(route_id, route, &time,)) })
    ]
}

fn view_route(route_id: &RouteId, route: &Route, time: &DateTime<Utc>) -> Node<Msg> {
    let mut num_sends = 0;
    let mut num_attempts = 0;
    let mut attempts_to_send = 0;
    let mut attempts_since_send = 0;
    let mut last_send = 0;
    let mut last_attempt = 0;
    let mut _send_streak = 0;

    // TODO: can we iterate our way out of this mess?

    for tick in &route.ticks {
        match tick.typ {
            TickType::Send => {
                last_send = tick.timestamp;
                num_sends += 1;
                attempts_since_send = 0;
                _send_streak += 1;
            }
            TickType::Attempt => {
                _send_streak = 0;
                last_attempt = tick.timestamp;
                num_attempts += 1;
                if num_sends > 0 {
                    attempts_since_send += 1;
                } else {
                    attempts_to_send += 1;
                }
            }
        }
    }

    let send_text = if num_sends == 0 {
        String::from("unsent")
    } else if attempts_to_send == 0 {
        format!("{} snd (flsh)", num_sends)
    } else if attempts_to_send > 0 {
        format!("{} snd ({} att)", num_sends, attempts_to_send)
    } else {
        // unreachable?
        String::new()
    };

    let att_text = if num_attempts == 0 && num_sends == 0 {
        String::from("unattempted")
    } else if last_send > last_attempt {
    } else if num_sends == 0 {
        format!(
            "{} att (att {})",
            num_attempts,
            util::time_diff_in_words(Utc.timestamp(last_attempt.into(), 0), *time)
        )
        format!(
            "{} att (snd {})",
            attempts_since_send,
            util::time_diff_in_words(Utc.timestamp(last_send.into(), 0), *time)
        )
    } else { 
        format!(
            "{} att (att {})",
            attempts_since_send,
            util::time_diff_in_words(Utc.timestamp(last_attempt.into(), 0), *time)
        )
    };

    li![
        class![
           "completed" => num_sends > 0
        ],
        div![
            class!["view"],
            div![
                class![route.color.as_ref(), "color-flag"],
                div![route.section],
                div![route.grade],
                ev(
                    Ev::Click,
                    enc!((route_id) move |_| Msg::StartRouteEdit(route_id))
                ),
            ],
            button![
                class!["tick-button btn btn-primary"],
                ev(
                    Ev::Click,
                    enc!((route_id) move |_| Msg::AddTickToRoute(route_id, TickType::Send))
                ),
                "SND"
            ],
            button![
                class!["tick-button btn"],
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
                route.title
            ],
            div![
                class!["stats"],
                div![class!["stats-sends"], send_text,],
                div![class!["stats-attempts"], att_text,],
            ],
            button![
                class!["btn btn-error"],
                i![class!["icon icon-cross"]],
                ev(
                    Ev::Click,
                    enc!((route_id) move |_| Msg::RemoveRoute(route_id))
                )
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
            TickType::Send if tick.timestamp > midnight => {
                today += 1;
                total += 1;
            },
            TickType::Send => {
                total += 1;
            }
            _ => {}
        }
    }

    div![
        class!["aggregate", "card"],
        div![class!["card-header"], div![class!["h5", "card-title"], "Stats"]],
        div![
            class!["card-body"],
            table![
                tr![
                    td!["Sends Today"],
                    td![format!("{}", today)]
                ],
                tr![
                    td!["Sends Total"],
                    td![format!("{}", total)]
                ]
            ],
        ]
    ]
}

// ------ footer ------

fn view_footer() -> Node<Msg> {
    footer![
        class!["footer", "grid-sm", "info"],
        p![
            "Created by ",
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
            ]
        ]
    ]
}

#[wasm_bindgen]
extern "C" {
    fn unixTimestamp() -> i32;
    fn midnight() -> i32;
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn render() {
    App::builder(update, view)
        .before_mount(before_mount)
        .after_mount(after_mount)
        .build_and_start();
}
