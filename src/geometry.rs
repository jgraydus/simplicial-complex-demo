use nalgebra::{
  DMatrix,
};
use std::collections::HashSet;
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

pub struct Distances {
  inner: Vec<(f32, (u8, u8))>
}

fn distance(points: &Vec<(f32,f32,f32)>, i: usize, j: usize) -> f32 {
  let (x0, y0, z0) = points[i];
  let (x1, y1, z1) = points[j];
  let x = x1-x0;
  let y = y1-y0;
  let z = z1-z0;
  (x*x + y*y + z*z).sqrt()
}

impl Distances {
  pub fn new(points: &Vec<(f32,f32,f32)>) -> Self {
    let mut inner = Vec::new();
    for i in 0..points.len() {
      for j in i+1..points.len() {
        let d = distance(points, i, j);
        inner.push((d, (i as u8, j as u8)));
      }
    }
    inner.sort_by(|(d1, _), (d2, _)| d1.partial_cmp(d2).unwrap());
    Distances { inner }
  }

  pub fn lines(&self, d: f32) -> Vec<(u8,u8)> {
    self.inner.iter()
      .map_while(|(d0,p)| if *d0 < d { Some(*p) } else { None })
      .collect()
  }

  pub fn triangles(&self, d: f32) -> Vec<(u8,u8,u8)> {
    let lines = self.lines(d);
    let lines_set: HashSet<(u8,u8)> = lines.iter().map(|x| *x).collect();
    let vertices: HashSet<u8> = lines.iter().flat_map(|(a,b)| vec![*a,*b]).collect();
    let mut result: HashSet<(u8,u8,u8)> = HashSet::new();
    for (a,b) in lines {
      for &v in vertices.iter() {
        if v > b {
          let l1 = (a,v);
          let l2 = (b,v);
          if lines_set.contains(&l1) && lines_set.contains(&l2) {
            result.insert((a,b,v));
          }
        }
      }
    }
    result.into_iter().collect()
  }

}

