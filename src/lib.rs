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
macro_rules! log {
  ( $( $t:tt )* ) => {
    web_sys::console::log_1(&format!( $( $t )* ).into());
  }
}

mod geometry;
use geometry::*;

mod handlers;
use handlers::*;

mod shader;
use shader::*;

const SIZE: u32 = 600;

fn get_canvas() -> web_sys::HtmlCanvasElement {
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();
  let canvas = document
      .get_element_by_id("canvas")
      .unwrap()
      .dyn_into::<web_sys::HtmlCanvasElement>()
      .unwrap();
  canvas.set_height(SIZE);
  canvas.set_width(SIZE);
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

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    wasm_bindgen_futures::spawn_local(async move {
      let angles = set_up_mouse_handlers();
      set_up_keypress_handler();

      let ctx = get_webgl_context();
      let program = make_shader_program(&ctx);

      let num_vertices: i32 = 100;
      log!("generating {} random vertices", num_vertices);
      let vertices = generate_vertices(num_vertices);
      log!("computing distances");
      let distances = Distances::new(&vertices);
      log!("loading vertex data");
      load_vertices(&ctx, &program, &vertices);

      let distance_threshold: f32 = 0.25;

      let lines = distances.lines(distance_threshold);
      let num_lines = lines.len() as i32;
      let mut lines: Vec<u8> = lines.iter().flat_map(|(i,j)| { vec![*i, *j] }).collect();

      let triangles = distances.triangles(distance_threshold);
      let num_triangles = triangles.len() as i32;
      let mut triangles: Vec<u8> = triangles.iter().flat_map(|(i,j,k)| vec![*i,*j,*k]).collect();

      let mut geometry = Vec::new();
      geometry.append(&mut lines);
      geometry.append(&mut triangles);

      let index_buffer = ctx.create_buffer().unwrap();
      ctx.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
      ctx.buffer_data_with_u8_array(
        WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
        &geometry,
        WebGl2RenderingContext::STATIC_DRAW,
      );

      let eye = Point3::new(0.0, 0.0, 1.0);
      let target = Point3::new(0.0, 0.0, 0.0);
      let view = Isometry3::look_at_rh(&eye, &target, &Vector3::y());
      let projection = Perspective3::new(1.0, 3.14 / 2.0, 0.0, 1000.0);
      let view_projection = projection.into_inner() * view.to_homogeneous();
      let transform_location = ctx.get_uniform_location(&program, "transform");

      let color_location = ctx.get_uniform_location(&program, "color");
      let color1: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
      let color2: [f32; 4] = [0.4, 0.0, 0.0, 1.0];

      let draw_fn0= Rc::new(RefCell::new(None));
      let draw_fn1 = draw_fn0.clone();

      *draw_fn1.borrow_mut() = Some(Closure::new(move || {
        let (a, b) = *angles.borrow();

        let model = Matrix4::from_euler_angles(b, a, 0.0);
        let model_view_projection = view_projection * model;

        ctx.uniform_matrix4fv_with_f32_array(
          transform_location.as_ref(),
          false,
          model_view_projection.as_slice()
        );

        ctx.clear_color(0.0, 0.0, 0.0, 1.0);
        ctx.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        ctx.uniform4fv_with_f32_array(
          color_location.as_ref(),
          &color2,
        );

        ctx.draw_elements_with_i32(
          WebGl2RenderingContext::TRIANGLES,
          num_triangles*3,
          WebGl2RenderingContext::UNSIGNED_BYTE,
          num_lines*2,
        );

        ctx.uniform4fv_with_f32_array(
          color_location.as_ref(),
          &color1,
        );

        ctx.draw_arrays(
          WebGl2RenderingContext::POINTS,
          0,
          num_vertices,
        );

        ctx.draw_elements_with_i32(
          WebGl2RenderingContext::LINES,
          num_lines*2,
          WebGl2RenderingContext::UNSIGNED_BYTE,
          0,
        );

        request_animation_frame(draw_fn0.borrow().as_ref().unwrap());
      }));

      request_animation_frame(draw_fn1.borrow().as_ref().unwrap());
    });

    Ok(())
}
