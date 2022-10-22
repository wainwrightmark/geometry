use std::{borrow::BorrowMut, ops::Add};

use geo::{prelude::*, MultiPoint, Point, Polygon};
use rand::Rng;
use sycamore::{
    prelude::*,
    rt::Event,
    web::html::{path, svg},
};
use wasm_bindgen::__rt::IntoJsResult;
use web_sys::{MouseEvent, console::log};

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    sycamore::render(|cx| {
        view! { cx, App {} }
    });
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let concavity = create_signal(cx, 0.1);
    let points = create_signal(cx, MultiPoint::<f64>::new(vec![]));

    view! {cx,
        
        SVG (points = points, concavity= concavity)
        Slider(concavity= concavity)
        ClearButton(points=points)
    }
}


#[component(inline_props)]
fn SVG<'a, G: Html>(
    cx: Scope<'a>,
    points: &'a Signal<MultiPoint<f64>>,
    concavity: &'a ReadSignal<f64>,
) -> View<G> {  
    
    return view! { cx,
        
        svg(height="500", 
        width="500", 
        style="background: lightblue;",
        xmlns="http://www.w3.org/2000/svg", 
        on:click=|ev: Event|  {points.modify().0.push(get_point(ev) )}  ) {
            (if points.get().0.len() >= 3 {
                let hull = points.get().concave_hull(*concavity.get());
                let path_string = get_path_string(&hull);
                view![cx,path(d=path_string, fill="transparent", stroke="blue" )]
            } else {
                view![cx,]
            })
            
            (View::new_fragment(
                points.get().0.iter().map(|&p| view! { cx, circle(cx=p.x(), cy=p.y(), r= 10.0 ) }).collect()
            ))
        }
    };
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
fn Slider<'a, G: Html>(cx: Scope<'a>, concavity: &'a Signal<f64>) -> View<G> {
    view! { cx,
        input(type="range", min="0", step="0.001", max="2", bind:valueAsNumber=concavity) {}
        input(type="number", min="0", step="0.001", max="2", bind:valueAsNumber=concavity) {}
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
            "Clear"
        }
    };
}
