use web_sys::{
  WebGl2RenderingContext,
  WebGlProgram
};

pub fn load_vertices(
  ctx: &WebGl2RenderingContext,
  program: &WebGlProgram,
  vertices: &Vec<(f32,f32,f32)>,
) {
  let vertices: Vec<f32> = vertices.iter().flat_map(|(x,y,z)| vec![*x,*y,*z]).collect();
  // create and bind a buffer for the vertex positions
  let vert_buffer = ctx.create_buffer().unwrap();
  ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vert_buffer));
  // write data into the buffer
  {
    let vert_array = unsafe { js_sys::Float32Array::view(&vertices) };
    ctx.buffer_data_with_array_buffer_view(
      WebGl2RenderingContext::ARRAY_BUFFER,
      &vert_array,
      WebGl2RenderingContext::STATIC_DRAW,
    );
  }
  // configure the buffer to be used as the 'coords' attribute in vertex shader
  let coords_location = ctx.get_attrib_location(program, "coords");
  ctx.vertex_attrib_pointer_with_i32(
    coords_location as u32,
    3,
    WebGl2RenderingContext::FLOAT,
    false,
    0,
    0,
  );
  ctx.enable_vertex_attrib_array(coords_location as u32);
}

fn rnd() -> f32 {
  let r: f32 = rand::random();
  r - 0.5f32
}

pub fn generate_vertices(n: i32) -> Vec<(f32,f32,f32)> {
  std::iter::from_fn(|| { Some((rnd(), rnd(), rnd())) })
    .filter(|(x,y,z)| x*x + y*y + z*z < 1.0)
    .take(n as usize)
    .collect()
}

