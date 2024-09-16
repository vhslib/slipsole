use std::cell::Cell;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::js_sys::JsString;
use web_sys::window;
use web_sys::CanvasRenderingContext2d;
use web_sys::EventTarget;
use web_sys::HtmlCanvasElement;
use web_sys::MouseEvent;

const POINT_RADIUS: u32 = 4;

fn main() {
    console_error_panic_hook::set_once();

    let window = window().unwrap();
    let document = window.document().unwrap();

    let canvas: HtmlCanvasElement = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();

    canvas
        .set_attribute("width", &canvas.client_width().to_string())
        .unwrap();

    canvas
        .set_attribute("height", &canvas.client_height().to_string())
        .unwrap();

    let context: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();

    let state = Rc::new(State {
        curves: RefCell::new(Vec::new()),
        drag_state: Cell::new(None),
        new_curve: Cell::new(None),
        mouse: Cell::new(Point::new(0, 0)),
    });

    redraw(&canvas, &context, state.as_ref());

    add_event_listener(&canvas, "mousedown", {
        let context = context.clone();
        let canvas = canvas.clone();
        let state = state.clone();

        move |event| {
            let mouse = Point::new(event.offset_x(), event.offset_y());
            state.mouse.set(mouse);

            for (index, &curve) in state.curves.borrow().iter().enumerate() {
                if is_point_inside_circle(curve.a, POINT_RADIUS, mouse) {
                    state
                        .drag_state
                        .set(Some(DragState::new(index, PointHandle::A)));

                    return;
                }

                if is_point_inside_circle(curve.b, POINT_RADIUS, mouse) {
                    state
                        .drag_state
                        .set(Some(DragState::new(index, PointHandle::B)));

                    return;
                }

                if is_point_inside_circle(curve.c, POINT_RADIUS, mouse) {
                    state
                        .drag_state
                        .set(Some(DragState::new(index, PointHandle::C)));

                    return;
                }

                if is_point_inside_circle(curve.d, POINT_RADIUS, mouse) {
                    state
                        .drag_state
                        .set(Some(DragState::new(index, PointHandle::D)));

                    return;
                }
            }

            match state.new_curve.get() {
                None => {
                    state.new_curve.set(Some(mouse));
                }
                Some(a) => {
                    state.new_curve.set(None);

                    let ax = a.x as f64;
                    let ay = a.y as f64;
                    let dx = mouse.x as f64;
                    let dy = mouse.y as f64;

                    let adx = dx - ax;
                    let ady = dy - ay;
                    let abx = adx / 3.0;
                    let aby = ady / 3.0;
                    let bx = ax + abx;
                    let by = ay + aby;

                    let acx = adx / 3.0 * 2.0;
                    let acy = ady / 3.0 * 2.0;
                    let cx = ax + acx;
                    let cy = ay + acy;

                    let b = Point::new(bx as _, by as _);
                    let c = Point::new(cx as _, cy as _);

                    state.curves.borrow_mut().push(Curve::new(a, b, c, mouse));

                    redraw(&canvas, &context, state.as_ref());
                }
            }
        }
    });

    add_event_listener(&canvas, "mousemove", {
        let context = context.clone();
        let canvas = canvas.clone();
        let state = state.clone();

        move |event| {
            let x = event.offset_x();
            let y = event.offset_y();

            if let Some(drag_state) = state.drag_state.get() {
                let dx = x - state.mouse.get().x;
                let dy = y - state.mouse.get().y;

                {
                    let mut curves = state.curves.borrow_mut();
                    let curve = &mut curves[drag_state.curve_index];

                    let point = match drag_state.point {
                        PointHandle::A => &mut curve.a,
                        PointHandle::B => &mut curve.b,
                        PointHandle::C => &mut curve.c,
                        PointHandle::D => &mut curve.d,
                    };

                    point.x += dx;
                    point.y += dy;

                    state.mouse.set(Point::new(x, y));
                }

                state.drag_state.set(Some(drag_state));

                redraw(&canvas, &context, state.as_ref());
            }

            if state.new_curve.get().is_some() {
                state.mouse.set(Point::new(x, y));
                redraw(&canvas, &context, state.as_ref());
            }
        }
    });

    add_event_listener(&canvas, "mouseup", {
        let state = state.clone();

        move |_| {
            state.drag_state.set(None);
        }
    });
}

fn redraw(canvas: &HtmlCanvasElement, context: &CanvasRenderingContext2d, state: &State) {
    context.clear_rect(
        0.0,
        0.0,
        canvas.client_width() as _,
        canvas.client_height() as _,
    );

    for &curve in state.curves.borrow().iter() {
        draw_curve(context, curve);
    }

    if let Some(a) = state.new_curve.get() {
        context.set_stroke_style(&JsString::from("black"));
        context.begin_path();
        context.move_to(a.x as _, a.y as _);
        context.line_to(state.mouse.get().x as _, state.mouse.get().y as _);
        context.stroke();
    }
}

struct State {
    curves: RefCell<Vec<Curve>>,
    drag_state: Cell<Option<DragState>>,
    new_curve: Cell<Option<Point>>,
    mouse: Cell<Point>,
}

#[derive(Clone, Copy)]
struct DragState {
    curve_index: usize,
    point: PointHandle,
}

impl DragState {
    fn new(curve_index: usize, point: PointHandle) -> Self {
        Self { curve_index, point }
    }
}

#[derive(Clone, Copy)]
enum PointHandle {
    A,
    B,
    C,
    D,
}

fn is_point_inside_circle(center: Point, radius: u32, point: Point) -> bool {
    let cx = center.x as f64;
    let cy = center.y as f64;
    let x = point.x as f64;
    let y = point.y as f64;
    let r = radius as f64;

    (cx - x).powi(2) + (cy - y).powi(2) <= r.powi(2)
}

fn draw_curve(context: &CanvasRenderingContext2d, curve: Curve) {
    let ax = curve.a.x as f64;
    let ay = curve.a.y as f64;
    let bx = curve.b.x as f64;
    let by = curve.b.y as f64;
    let cx = curve.c.x as f64;
    let cy = curve.c.y as f64;
    let dx = curve.d.x as f64;
    let dy = curve.d.y as f64;

    context.set_stroke_style(&JsString::from("black"));
    context.begin_path();
    context.move_to(ax, ay);
    context.bezier_curve_to(bx, by, cx, cy, dx, dy);
    context.stroke();

    context.begin_path();
    context.move_to(ax, ay);
    context.line_to(bx, by);
    context.stroke();

    context.begin_path();
    context.move_to(dx, dy);
    context.line_to(cx, cy);
    context.stroke();

    context.set_fill_style(&JsString::from("blue"));
    context.begin_path();
    context
        .arc(ax, ay, POINT_RADIUS as _, 0.0, 2.0 * PI)
        .unwrap();
    context
        .arc(dx, dy, POINT_RADIUS as _, 0.0, 2.0 * PI)
        .unwrap();
    context.fill();

    context.set_fill_style(&JsString::from("red"));
    context.begin_path();
    context
        .arc(bx, by, POINT_RADIUS as _, 0.0, 2.0 * PI)
        .unwrap();
    context
        .arc(cx, cy, POINT_RADIUS as _, 0.0, 2.0 * PI)
        .unwrap();
    context.fill();
}

#[derive(Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy)]
struct Curve {
    a: Point,
    b: Point,
    c: Point,
    d: Point,
}

impl Curve {
    fn new(a: Point, b: Point, c: Point, d: Point) -> Self {
        Self { a, b, c, d }
    }
}

fn add_event_listener(
    target: &EventTarget,
    kind: &str,
    listener: impl FnMut(MouseEvent) + 'static,
) {
    let closure = Closure::<dyn FnMut(_)>::new(listener);

    target
        .add_event_listener_with_callback(kind, closure.as_ref().unchecked_ref())
        .unwrap();

    closure.forget();
}
