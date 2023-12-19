use nalgebra::{
  Isometry3,
  Matrix4,
  Perspective3,
  Point3,
  Vector3,
};
use std::{
  cell::RefCell,
  rc::Rc,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures;
use web_sys::{
  WebGl2RenderingContext,
};

// make printing a console log more convenient
#[allow(unused)]
macro_rules! log {
  ( $( $t:tt )* ) => {
    web_sys::console::log_1(&format!( $( $t )* ).into());
  }
}

mod handlers;
use handlers::*;

mod model;
use model::*;

mod shader;
use shader::*;

fn get_canvas() -> web_sys::HtmlCanvasElement {
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();
  let canvas = document
      .get_element_by_id("canvas")
      .unwrap()
      .dyn_into::<web_sys::HtmlCanvasElement>()
      .unwrap();
  canvas
}

fn get_webgl_context() -> WebGl2RenderingContext {
  let canvas = get_canvas();
  let ctx = canvas.get_context("webgl2").unwrap().unwrap()
                  .dyn_into::<WebGl2RenderingContext>().unwrap();
  ctx.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
  ctx
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
  web_sys::window().unwrap()
    .request_animation_frame(f.as_ref().unchecked_ref()).unwrap();
}

// WARNING: currently supports a maximum of 255 vertices because
// indices are represented with u8 values.
const NUM_VERTICES: i32 = 200;

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let canvas = get_canvas();
    let aspect_ratio = canvas.width() as f32 / canvas.height() as f32;

    wasm_bindgen_futures::spawn_local(async move {
      // the mouse handlers let the user click/drag to rotate the model
      let angles = set_up_mouse_handlers();

      // initialize the geometry
      let model = Rc::new(RefCell::new(Model::new(NUM_VERTICES)));
      model.borrow_mut().update_distance_threshold(0.15);

      // webgl prep
      let ctx = get_webgl_context();
      let program = make_shader_program(&ctx);
      model.borrow().load_vertex_data(&ctx, &program);
      model.borrow().load_index_data(&ctx);

      // listen for left/right arrow keys to change the distance threshold
      set_up_keypress_handler(ctx.clone(), model.clone());

      // set up the camera
      let eye = Point3::new(0.0, 0.0, 0.75);
      let target = Point3::new(0.0, 0.0, 0.0);
      let view = Isometry3::look_at_rh(&eye, &target, &Vector3::y());
      let projection = Perspective3::new(aspect_ratio, 3.14 / 2.0, 0.0, 1000.0);
      let view_projection = projection.into_inner() * view.to_homogeneous();

      // the draw function needs to be a closure that references itself. this
      // is a bit tricky to do in rust. the technique used here is to put the
      // closure in a RefCell within an Rc. the closure can reference itself
      // via the captured Rc. additionally, a copy of the Rc is made in order
      // to schedule the first draw
      let draw_fn0 = Rc::new(RefCell::new(None));
      let draw_fn1 = draw_fn0.clone();

      *draw_fn1.borrow_mut() = Some(Closure::new(move || {
        // update the transform matrix with the current rotation angles
        let (a, b) = *angles.borrow();
        let model_view_projection = {
          let model = Matrix4::from_euler_angles(b, a, 0.0);
          view_projection * model
        };
        // draw
        model.borrow().draw(&ctx, &program, model_view_projection.as_slice());
        // schedule the next draw
        request_animation_frame(draw_fn0.borrow().as_ref().unwrap());
      }));

      // schedule the first draw
      request_animation_frame(draw_fn1.borrow().as_ref().unwrap());
    });

    Ok(())
}
