use std::{borrow::BorrowMut, ops::Add};

use geo::{prelude::*, MultiPoint, Point, Polygon};
use rand::Rng;
use itertools::{self, Itertools};
use sycamore::{
    prelude::*,
    rt::Event,
    web::html::{path, svg},
};
use wasm_bindgen::__rt::IntoJsResult;
use web_sys::{MouseEvent, console::log};

pub const CIRCLERADIUS: f64 = 10.0;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    sycamore::render(|cx| {
        view! { cx, App {} }
    });
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let concavity = create_signal(cx, 0.1);
    let k_nearest_points = create_signal(cx, 4.0);
    let show_concave_hull = create_signal(cx, true);
    let show_k_nearest_concave_hull = create_signal(cx, true);
    let show_convex_hull = create_signal(cx, true);
    let points = create_signal(cx, MultiPoint::<f64>::new(vec![]));

    view! {cx,
        div(style="display: flex; flex-direction: column;"){
            SVG (points = points, concavity= concavity, show_concave_hull= show_concave_hull, show_convex_hull=show_convex_hull, show_k_nearest_concave_hull=show_k_nearest_concave_hull, k_nearest_points=k_nearest_points)
            ConcaveHullSettings(concavity= concavity, show_concave_hull=show_concave_hull)
            ConvexHullSettings(show_convex_hull=show_convex_hull)
            KNearestConcaveHullSettings(show_k_nearest_concave_hull=show_k_nearest_concave_hull, points=k_nearest_points)
            ClearButton(points=points)
        }
        
    }
}


#[component(inline_props)]
fn SVG<'a, G: Html>(
    cx: Scope<'a>,
    points: &'a Signal<MultiPoint<f64>>,
    concavity: &'a ReadSignal<f64>,
    k_nearest_points: &'a ReadSignal<f64>,
    show_concave_hull: &'a ReadSignal<bool>,
    show_k_nearest_concave_hull: &'a ReadSignal<bool>,
    show_convex_hull: &'a ReadSignal<bool>,
) -> View<G> {  
    
    return view! { cx,
        
        svg(height="500", 
        width="500", 
        style="background: lightblue;",
        xmlns="http://www.w3.org/2000/svg", 
        on:click=|ev: Event| add_or_remove_point(points, get_point(ev))  ) {

            (View::new_fragment(
                points.get().0.iter().map(|&p| view! { cx, circle(cx=p.x(), cy=p.y(), r= {CIRCLERADIUS} ) }).collect()
            ))

            K_Nearest_Concave_Hull_Path(points=points, show=show_k_nearest_concave_hull, k_nearest=k_nearest_points)
            Concave_Hull_Path(points=points, concavity=concavity, show=show_concave_hull)
            Convex_Hull_Path(points=points, show=show_convex_hull)
            
        }
    };
}

#[component(inline_props)]
fn Concave_Hull_Path<'a, G: Html>(
    cx: Scope<'a>,
    points: &'a Signal<MultiPoint<f64>>,
    concavity: &'a ReadSignal<f64>,
    show: &'a ReadSignal<bool>,
) -> View<G> {
    view![cx, 
    (if *show.get() && points.get().0.len() >= 3 {
        let hull = points.get().concave_hull(*concavity.get());
        let path_string = get_path_string(&hull);
        view![cx,path(d=path_string, fill="transparent", stroke="blue",  )]
    } else {
        view![cx,]
    })]
}

#[component(inline_props)]
fn K_Nearest_Concave_Hull_Path<'a, G: Html>(
    cx: Scope<'a>,
    points: &'a Signal<MultiPoint<f64>>,
    k_nearest: &'a ReadSignal<f64>,
    show: &'a ReadSignal<bool>,
) -> View<G> {
    view![cx, 
    (if *show.get() && points.get().0.len() >= 3 {
        let hull = points.get().k_nearest_concave_hull(k_nearest.get().floor() as u32);
        let path_string = get_path_string(&hull);
        view![cx,path(d=path_string, fill="transparent", stroke="red"  )]
    } else {
        view![cx,]
    })]
}

#[component(inline_props)]
fn Convex_Hull_Path<'a, G: Html>(
    cx: Scope<'a>,
    points: &'a Signal<MultiPoint<f64>>,
    show: &'a ReadSignal<bool>,
) -> View<G> {
    view![cx, 
    (if *show.get() && points.get().0.len() >= 3 {
        let hull = points.get().convex_hull();
        let path_string = get_path_string(&hull);
        view![cx,path(d=path_string, fill="transparent", stroke="green", stroke-dasharray="4" )]
    } else {
        view![cx,]
    })]
}

fn add_or_remove_point(points: &Signal<MultiPoint>, point: Point ){

    let mut modify = points.modify();
    if let Some((index, _)) = modify.0.iter().find_position(|x| x.euclidean_distance(&point) < CIRCLERADIUS){
        modify.0.remove(index);        
    }   
    else{
        modify.0.push(point)
    }
}

fn get_point(event: Event) -> Point {
    let me: MouseEvent = event.into_js_result().unwrap().try_into().unwrap();

    Point::new(me.x() as f64, me.y() as f64)
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
fn ConcaveHullSettings<'a, G: Html>(cx: Scope<'a>, concavity: &'a Signal<f64>, show_concave_hull: &'a Signal<bool>) -> View<G> {
    view! { cx,
        div(){
            label(){"Concave Hull"}
            input(type="checkbox", bind:checked=show_concave_hull)
            input(type="range", min="0", step="0.001", max="1", bind:valueAsNumber=concavity) {}
            input(type="number", min="0", step="0.001", max="1", bind:valueAsNumber=concavity) {}
        }
        
    }
}

#[component(inline_props)]
fn ConvexHullSettings<'a, G: Html>(cx: Scope<'a>,  show_convex_hull: &'a Signal<bool>) -> View<G> {
    view! { cx,
        div(){
            label(){"Convex Hull"}
            input(type="checkbox", bind:checked=show_convex_hull)
            
        }
        
    }
}

#[component(inline_props)]
fn KNearestConcaveHullSettings<'a, G: Html>(cx: Scope<'a>, points: &'a Signal<f64>, show_k_nearest_concave_hull: &'a Signal<bool>) -> View<G> {
    view! { cx,
        div(){
            label(){"K Nearest Concave Hull"}
            input(type="checkbox", bind:checked=show_k_nearest_concave_hull)
            input(type="range", min="1", step="1", max="10", bind:valueAsNumber=points) {}
            input(type="number", min="1", step="1", max="10", bind:valueAsNumber=points) {}
        }
        
    }
}

#[component(inline_props)]
fn ClearButton<'a, G: Html>(
    cx: Scope<'a>,
    points: &'a Signal<MultiPoint<f64>>,
) -> View<G> {
    
    return view! { cx,
        button(
        on:click=|_|  {points.modify().0.clear()}  ) 
        {
            "Clear All Points"
        }
    };
}
