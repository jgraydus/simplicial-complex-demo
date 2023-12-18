use web_sys::{
  WebGlProgram,
  WebGl2RenderingContext,
  WebGlShader,
};

fn compile_shader(
  ctx: &WebGl2RenderingContext,
  shader_type: u32,
  src: &str
) -> WebGlShader {
  let shader = ctx.create_shader(shader_type).unwrap();
  ctx.shader_source(&shader, src);
  ctx.compile_shader(&shader);
  shader
}

fn vertex_shader(ctx: &WebGl2RenderingContext) -> WebGlShader {
  let src = "#version 300 es
    in vec4 coords;
    uniform mat4 transform;
    void main(void) {
      gl_Position = transform * coords;
      gl_PointSize = 3.0;
    }
  ";
  compile_shader(ctx, WebGl2RenderingContext::VERTEX_SHADER, src)
}

fn fragment_shader(ctx: &WebGl2RenderingContext) -> WebGlShader {
  let src = "#version 300 es
    precision mediump float;
    out vec4 out_color;
    void main(void) {
      out_color = vec4(1.0, 0.0, 0.0, 1.0);
    }    
  ";
  compile_shader(ctx, WebGl2RenderingContext::FRAGMENT_SHADER, src)
}

pub fn make_shader_program(ctx: &WebGl2RenderingContext) -> WebGlProgram {
  let program = ctx.create_program().unwrap();
  ctx.attach_shader(&program, &vertex_shader(ctx));
  ctx.attach_shader(&program, &fragment_shader(ctx));
  ctx.link_program(&program);
  let result = ctx.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
                  .as_bool().unwrap_or(false);
  if result {
    ctx.use_program(Some(&program));
  } else {
    panic!("failed to create shader program");
  }
  program
}

