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
const STORAGE_KEY: &str = "gymticks-2";

type RouteId = Uuid;

const COLORS: [&str; 10] = [
    "orange", "red", "pink", "purple", "blue", "brown", "yellow", "green", "white", "black",
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
    choosing_color: bool,
    chosen_color: String,
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
    let data = storage::load_data(&local_storage, STORAGE_KEY).unwrap_or_default();

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

    ToggleChoosingColor(),
    ChooseColor(String),

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
                },
            );
            data.routes.sort_by(|_ak, av, _bk, bv| av.title.cmp(&bv.title))
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
                    data.routes.sort_by(|_ak, av, _bk, bv| av.title.cmp(&bv.title))
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

        Msg::ToggleChoosingColor() => {
            data.choosing_color = !data.choosing_color;
        }

        Msg::ChooseColor(color) => {
            data.chosen_color = color;
            data.choosing_color = false;
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
        view_header(
            &data.new_route_title,
            &data.choosing_color,
            &data.chosen_color
        ),
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
    ]
}

// ------ header ------

fn view_header(new_route_title: &str, choosing_color: &bool, chosen_color: &String) -> Node<Msg> {
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
                   "choosing-color" => choosing_color
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
            button![
                id!("toggle-color"),
                class![chosen_color.as_str(), "toggle-color"],
                ev(Ev::Click, |_| Msg::ToggleChoosingColor()),
                "❯"
            ],
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
    let (num_sends, num_other) = route.ticks.iter().fold((0i32, 0i32), |acc, tick| {
        return match tick.typ {
            TickType::Send => (acc.0 + 1, acc.1),
            _ => (acc.0, acc.1 + 1),
        };
    });

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
                class![
                    route.color.as_ref(),
                    "color-flag"
                ],
            ],
            button![
                class!["tick-button"],
                ev(
                    Ev::Click,
                    enc!((route_id) move |_| Msg::AddTickToRoute(route_id, TickType::Send))
                ),
                "Snd"
            ],
            button![
                class!["tick-button"],
                ev(
                    Ev::Click,
                    enc!((route_id) move |_| Msg::AddTickToRoute(route_id, TickType::Attempt))
                ),
                "Att"
            ],
            label![
                ev(
                    Ev::DblClick,
                    enc!((route_id) move |_| Msg::StartRouteEdit(route_id))
                ),
                route.title
            ],
            label![format!(
                "{}",
                if num_sends > 0 { num_sends } else { num_other }
            )],
            label![route.ticks.last().map_or_else(
                || String::new(),
                |tick| {
                    format!(
                        "{}",
                        util::time_diff_in_words(Utc.timestamp(tick.timestamp.into(), 0), *time)
                    )
                }
            )],
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
