use std::collections::HashSet;
use web_sys::{
  WebGl2RenderingContext,
  WebGlProgram,
  WebGlUniformLocation,
};

fn generate_vertices(n: i32) -> Vec<(f32,f32,f32)> {
  // random in range (-0.5, 0.5)
  fn rnd() -> f32 {
    let r: f32 = rand::random();
    r - 0.5f32
  }
  // generate n points within a sphere of radius 0.5
  std::iter::from_fn(|| { Some((rnd(), rnd(), rnd())) })
    .filter(|(x,y,z)| x*x + y*y + z*z < 0.25)
    .take(n as usize)
    .collect()
}

pub struct Model {
  vertices: Vec<(f32,f32,f32)>, // 3D points
  lines: Vec<(u8,u8)>,          // vertex indices
  triangles: Vec<(u8,u8,u8)>,   // vertex indices
  distance_threshold: f32,
}

impl Model {
  const POINT_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
  const LINE_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
  const TRIANGLE_COLOR: [f32; 4] = [0.3, 0.0, 0.0, 1.0];

  pub fn new(num_vertices: i32) -> Self {
    Self {
      vertices: generate_vertices(num_vertices),
      lines: Vec::new(),
      triangles: Vec::new(),
      distance_threshold: 0.0,
    }
  }

  pub fn distance_threshold(&self) -> f32 { self.distance_threshold }

  fn distance(&self, i: usize, j: usize) -> f32 {
    let (x0, y0, z0) = self.vertices[i];
    let (x1, y1, z1) = self.vertices[j];
    let x = x1-x0;
    let y = y1-y0;
    let z = z1-z0;
    (x*x + y*y + z*z).sqrt()
  }

  // recompute the lines and triangles based on the new distance threshold
  pub fn update_distance_threshold(&mut self, distance_threshold: f32) {
    self.lines = {
      let mut result = Vec::new();
      for i in 0..self.vertices.len() {
        // (i,j) and (j,i) are the same edge, so to prevent
        // duplicates only generate the edges where i < j
        for j in i+1..self.vertices.len() {
          if self.distance(i, j) < distance_threshold {
            result.push((i as u8,j as u8));
          }
        }
      }
      result
    };
    self.triangles = {
      let lines_set: HashSet<(u8,u8)> = self.lines.iter().map(|x| *x).collect();
      // disregard vertices that aren't in any of the lines
      let vertices: HashSet<u8> = self.lines.iter().flat_map(|(a,b)| vec![*a,*b]).collect();
      let mut result: HashSet<(u8,u8,u8)> = HashSet::new();
      for &(a,b) in &self.lines {
        for &v in vertices.iter() {
          if v > b { // prevents duplicates by enforcing a < b < v
            let l1 = (a,v);
            let l2 = (b,v);
            if lines_set.contains(&l1) && lines_set.contains(&l2) {
              result.insert((a,b,v));
            }
          }
        }
      }
      result.into_iter().collect()
    };
    self.distance_threshold = distance_threshold;
  }

  pub fn load_vertex_data(
    &self,
    ctx: &WebGl2RenderingContext,
    program: &WebGlProgram,
  ) {
    let vertices: Vec<f32> = self.vertices.iter().flat_map(|(x,y,z)| vec![*x,*y,*z]).collect();
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
    let coords_location = ctx.get_attrib_location(program, "coords") as u32;
    ctx.vertex_attrib_pointer_with_i32(
      coords_location,
      3,
      WebGl2RenderingContext::FLOAT,
      false,
      0,
      0,
    );
    ctx.enable_vertex_attrib_array(coords_location);
  }

  pub fn load_index_data(
    &self,
    ctx: &WebGl2RenderingContext,
  ) {
    // flatten the lines/triangles and concatenate them into a single Vec<u8>
    let mut lines: Vec<u8> = self.lines.iter().flat_map(|(i,j)| { vec![*i, *j] }).collect();
    let mut triangles: Vec<u8> = self.triangles.iter().flat_map(|(i,j,k)| vec![*i,*j,*k]).collect();
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
  }

  fn draw_vertices(
    &self,
    ctx: &WebGl2RenderingContext,
    color_location: &Option<WebGlUniformLocation>,
  ) {
    ctx.uniform4fv_with_f32_array(
      color_location.as_ref(),
      &Self::POINT_COLOR,
    );
    ctx.draw_arrays(
      WebGl2RenderingContext::POINTS,
      0,
      self.vertices.len() as i32,
    );
  }

  fn draw_lines(
    &self,
    ctx: &WebGl2RenderingContext,
    color_location: &Option<WebGlUniformLocation>,
  ) {
    ctx.uniform4fv_with_f32_array(
      color_location.as_ref(),
      &Self::LINE_COLOR,
    );
    ctx.draw_elements_with_i32(
      WebGl2RenderingContext::LINES,
      self.lines.len() as i32 * 2,
      WebGl2RenderingContext::UNSIGNED_BYTE,
      0,
    );
  }

  fn draw_triangles(
    &self,
    ctx: &WebGl2RenderingContext,
    color_location: &Option<WebGlUniformLocation>,
  ) {
    ctx.uniform4fv_with_f32_array(
      color_location.as_ref(),
      &Self::TRIANGLE_COLOR,
    );
    ctx.draw_elements_with_i32(
      WebGl2RenderingContext::TRIANGLES,
      self.triangles.len() as i32 * 3,
      WebGl2RenderingContext::UNSIGNED_BYTE,
      // offset by the size of the line data which is at the beginning of the buffer
      self.lines.len() as i32 * 2,
    );
  }

  pub fn draw(
    &self,
    ctx: &WebGl2RenderingContext,
    program: &WebGlProgram,
    model_view_projection: &[f32],
  ) {
    let transform_location = ctx.get_uniform_location(program, "transform");
    ctx.uniform_matrix4fv_with_f32_array(
      transform_location.as_ref(),
      false,
      model_view_projection,
    );

    let color_location = ctx.get_uniform_location(program, "color");
    ctx.clear_color(0.0, 0.0, 0.0, 1.0);
    ctx.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    self.draw_triangles(&ctx, &color_location);
    self.draw_vertices(&ctx, &color_location);
    self.draw_lines(&ctx, &color_location);
  }
}
