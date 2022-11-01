use geo::{prelude::*, MultiPoint, Point, Polygon};
use itertools::{self, Itertools};

use sycamore::{prelude::*, rt::Event};
use wasm_bindgen::__rt::IntoJsResult;
use web_sys::{Element, MouseEvent};

pub const CIRCLERADIUS: f64 = 10.0;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    sycamore::render(|cx| {
        view! { cx, App {} }
    });
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    provide_context_ref(cx, create_signal(cx, MultiPoint::<f64>::new(vec![])));

    provide_context_ref(cx, create_signal(cx, ConvexHullSettings { show: true }));
    provide_context_ref(
        cx,
        create_signal(
            cx,
            ConcaveHullSettings {
                show: true,
                concavity: 0.1,
            },
        ),
    );
    provide_context_ref(
        cx,
        create_signal(
            cx,
            KNearestConcaveHullSettings {
                show: true,
                points: 4,
            },
        ),
    );

    view! {cx,
        main(class="container-fluid"){
            SVG ()
        }
        
        footer(){
            div(class="container")
            {
                ConvexHullControl{}
                ConcaveHullControl()
                KNearestConcaveHullControl{}
                ClearButton{}
            }
        }

    }
}

#[component()]
fn SVG<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let points = use_context::<Signal<MultiPoint<f64>>>(cx);
    let node_ref = create_node_ref(cx);

    return view! { cx,

        svg(
            ref=node_ref,
            height="500",
        width="100%",
        style="background: lightblue;",
        xmlns="http://www.w3.org/2000/svg",
        on:click=|ev: Event| add_or_remove_point(points, get_point(ev, node_ref))  ) {

            (View::new_fragment(
                points.get().0.iter().map(|&p| view! { cx, circle(cx=p.x(), cy=p.y(), color="black", r= {CIRCLERADIUS} ) }).collect()
            ))

            K_Nearest_Concave_Hull_Path{}
            Concave_Hull_Path()
            Convex_Hull_Path{}

        }
    };
}

#[component()]
fn Concave_Hull_Path<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let settings = use_context::<Signal<ConcaveHullSettings>>(cx);
    let points = use_context::<Signal<MultiPoint<f64>>>(cx);
    view![
        cx,
        (if settings.get().show && points.get().0.len() >= 3 {
            let hull = points.get().concave_hull(settings.get().concavity);
            let path_string = get_path_string(&hull);
            view![
                cx,
                path(d = path_string, fill = "transparent", stroke = "blue",)
            ]
        } else {
            view![cx,]
        })
    ]
}

#[component]
fn K_Nearest_Concave_Hull_Path<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    log::info!("K Nearest Path");
    let settings = use_context::<Signal<KNearestConcaveHullSettings>>(cx);
    let points = use_context::<Signal<MultiPoint<f64>>>(cx);
    settings.track();
    view![
        cx,
        (if settings.get().show && points.get().0.len() >= 3 {
            let hull = points.get().k_nearest_concave_hull(settings.get().points);
            let path_string = get_path_string(&hull);
            view![
                cx,
                path(d = path_string, fill = "transparent", stroke = "red")
            ]
        } else {
            view![cx,]
        })
    ]
}

#[component()]
fn Convex_Hull_Path<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let points = use_context::<Signal<MultiPoint<f64>>>(cx);

    let settings = use_context::<Signal<ConvexHullSettings>>(cx);

    view![
        cx,
        (if settings.get().show && points.get().0.len() >= 3 {
            let hull = points.get().convex_hull();
            let path_string = get_path_string(&hull);
            view![
                cx,
                path(
                    d = path_string,
                    fill = "transparent",
                    stroke = "green",
                    stroke - dasharray = "4"
                )
            ]
        } else {
            view![cx,]
        })
    ]
}

fn add_or_remove_point(points: &Signal<MultiPoint>, point: Point) {
    let mut modify = points.modify();
    if let Some((index, _)) = modify
        .0
        .iter()
        .find_position(|x| x.euclidean_distance(&point) < CIRCLERADIUS)
    {
        modify.0.remove(index);
    } else {
        modify.0.push(point)
    }
}

fn get_point<G: Html>(event: Event, node_ref: &NodeRef<G>) -> Point {
    let me: MouseEvent = event.into_js_result().unwrap().try_into().unwrap();
    let node = node_ref.get::<DomNode>();
    let element: Element = node.unchecked_into();

    let rect = element.get_bounding_client_rect();

    Point::new(me.x() as f64 - rect.x(), me.y() as f64 - rect.y())
}

fn get_path_string(polygon: &Polygon) -> String {
    let mut iter = polygon.coords_iter();
    let first = iter.next().unwrap();
    let mut s = format!("M {} {} ", first.x, first.y);
    while let Some(p) = iter.next() {
        s.push_str(format!("L {} {} ", p.x, p.y).as_str());
    }
    s.push_str("Z");

    s
}

#[component(inline_props)]
fn LabelledCheckbox<'a, G: Html>(cx: Scope<'a>, signal: &'a Signal<bool>, str: &'a str) -> View<G> {
    view! { cx,
        label(for="checkbox"){
            (str.to_string())
            input(type="checkbox", name="checkbox", bind:checked=signal)
        }
    }
}

#[component(inline_props)]
fn NumberInput<'a, G: Html>(
    cx: Scope<'a>,
    signal: &'a Signal<f64>,
    str: &'a str,
    min: f64,
    max: f64,
    step: f64,
) -> View<G> {
    view! { cx,
        label(for="slider"){
            (format!("{}: {}", str, signal.get() ) )
            input(type="range", name="slider",  min={min}, step={step}, max={max}, bind:valueAsNumber=signal) {}
            // input(type="number", min={min}, step={step}, max={max}, bind:valueAsNumber=signal) {}
        }
    }
}

#[component()]
fn ConvexHullControl<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let settings = use_context::<Signal<ConvexHullSettings>>(cx);

    let show_signal = create_signal(cx, settings.get().show);
    create_effect(cx, || settings.modify().show = *show_signal.get());

    view! { cx,
        LabelledCheckbox(signal= show_signal, str= "Convex Hull ")
    }
}

#[component()]
fn ConcaveHullControl<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let settings = use_context::<Signal<ConcaveHullSettings>>(cx);

    let show_signal = create_signal(cx, settings.get().show);
    create_effect(cx, || {
        settings.modify().show = *show_signal.get();
    });

    let concavity_signal = create_signal(cx, settings.get().concavity);
    create_effect(cx, || settings.modify().concavity = *concavity_signal.get());

    view! { cx,
        LabelledCheckbox(signal= show_signal, str= "Concave Hull ")
        NumberInput(signal=concavity_signal, str= "Concavity", min = 0.0, max = 1.0, step= 0.001)

    }
}

#[component()]
fn KNearestConcaveHullControl<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let settings = use_context::<Signal<KNearestConcaveHullSettings>>(cx);

    let show_signal = create_signal(cx, settings.get().show);
    create_effect(cx, || {
        settings.modify().show = *show_signal.get();
    });

    let points_signal = create_signal(cx, settings.get().points.into());
    create_effect(cx, || {
        settings.modify().points = *points_signal.get() as u32
    });

    view! { cx,
        LabelledCheckbox(signal= show_signal, str= "K Nearest Concave Hull ")
        NumberInput(signal=points_signal, str= "Points", min = 1.0, max = 10.0, step= 1.0)

    }
}

#[component()]
fn ClearButton<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let points = use_context::<Signal<MultiPoint<f64>>>(cx);

    return view! { cx,
        button(
        on:click=|_|  {points.modify().0.clear()}  )
        {
            "Clear All Points"
        }
    };
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConvexHullSettings {
    pub show: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KNearestConcaveHullSettings {
    pub show: bool,
    pub points: u32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ConcaveHullSettings {
    pub show: bool,
    pub concavity: f64,
}
