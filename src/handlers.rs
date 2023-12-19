use std::{
  cell::RefCell,
  rc::Rc,
};
use wasm_bindgen::prelude::*;
use web_sys::{
  KeyboardEvent,
  MouseEvent,
};

// creates the behavior that allows click+drag to change the
// rotation angles. the function returns a shared reference
// to the two angle values
pub fn set_up_mouse_handlers() -> Rc<RefCell<(f32, f32)>> {
  let angles = Rc::new(RefCell::new((0f32, 0f32)));
  let mouse_down_state = Rc::new(RefCell::new(None));

  let handle_mousedown = {
    let angles = angles.clone();
    let mouse_down_state = mouse_down_state.clone();
    Closure::<dyn FnMut(MouseEvent)>::new(move |evt: MouseEvent| {
      // when left mouse button is pressed, remember the current state
      if evt.button() == 0 {
        let (a, b) = *angles.borrow();
        let (x, y) = (evt.offset_x(), evt.offset_y());
        *mouse_down_state.borrow_mut() = Some((a, b, x, y));
      }
    })
  };
  let handle_mouseup = {
    let mouse_down_state = mouse_down_state.clone();
    Closure::<dyn FnMut(MouseEvent)>::new(move |evt: MouseEvent| {
      // when left mouse button is released, forget the saved state
      if evt.button() == 0 { // left click
        *mouse_down_state.borrow_mut() = None;
      }
    })
  };
  let handle_mousemove = {
    let angles = angles.clone();
    let mouse_down_state = mouse_down_state.clone();
    Closure::<dyn FnMut(MouseEvent)>::new(move |evt: MouseEvent| {
      let x1 = evt.offset_x();
      let y1 = evt.offset_y();
      // if the left mouse button is being held down, update the angles
      if let Some((a, b, x0, y0)) = *mouse_down_state.borrow() {
        let a = a + ((x1 - x0) as f32) / 100.0;
        let b = b + ((y1 - y0) as f32) / 100.0;
        *angles.borrow_mut() = (a, b);
      }
    })
  };
  let window = web_sys::window().unwrap();
  window.set_onmousedown(Some(handle_mousedown.as_ref().unchecked_ref()));
  handle_mousedown.forget();
  window.set_onmouseup(Some(handle_mouseup.as_ref().unchecked_ref()));
  handle_mouseup.forget();
  window.set_onmousemove(Some(handle_mousemove.as_ref().unchecked_ref()));
  handle_mousemove.forget();

  angles
}

pub fn set_up_keypress_handler() {
  let handle_keypress = {
    Closure::<dyn FnMut(KeyboardEvent)>::new(move |evt: KeyboardEvent| {
      match evt.key().as_ref() {
        "ArrowLeft" => {
          log!("ArrowLeft");
        },
        "ArrowRight" => {
          log!("ArrowRight");
        },
        _ => {},
      }
    })
  };
  let window = web_sys::window().unwrap();
  window.set_onkeydown(Some(handle_keypress.as_ref().unchecked_ref()));
  handle_keypress.forget();
}
