use std::collections::HashMap;

use sdl2::keyboard::Keycode;
use vecm::vec::*;



pub struct InputHandler {
    pressed: HashMap<Control, bool>,
    pub left_click: bool,
    pub right_click: bool,
    pub mouse_pos: Vec2u,
    mouse_delta: Vec2i,
    mouse_wheel_delta: i32,
}


macro_rules! key_mappings {
    ($($control_match:tt => $keycode:tt),*press: $($press_control_match:tt => $press_keycode:tt),*) => {

        #[derive(PartialEq, Eq, Hash)]
        pub enum Control {
            $($control_match),*,
            $($press_control_match),*
        }

        impl Control {
            #[allow(dead_code)]
            pub fn keycode(&self) -> Keycode {
                match self {
                    $(Control::$control_match => Keycode::$keycode),*,
                    $(Control::$press_control_match => Keycode::$press_keycode),*,
//                    _ => panic!("Control not mapped!")
                }
            }
    
            pub fn from_keycode(keycode: Keycode) -> Option<Control> {
                match keycode {
                    $(Keycode::$keycode => Some(Control::$control_match)),*,
                    $(Keycode::$press_keycode => Some(Control::$press_control_match)),*,
                    _ => None
                }
            }


            pub fn press_controls() -> Vec<Control> {
                vec![
                    $(Control::$press_control_match),*,
                ]
            }
        }
        
    };
}

key_mappings! {
    Up => W,
    Down => S,
    Left => A,
    Right => D,
    Escape => Escape
  press:
    ZoomIn => Plus,
    ZoomOut => Minus
}


impl InputHandler {

    pub fn new() -> InputHandler {
        InputHandler { pressed: HashMap::new(), left_click: false, right_click: false,mouse_pos: Vec2u::zero(), mouse_delta: Vec2i::zero(), mouse_wheel_delta: 0 }
    }

    pub fn add_mouse_delta(&mut self, delta: Vec2i) {
        self.mouse_delta += delta;
    }

    pub fn add_mouse_wheel_delta(&mut self, delta: i32) {
        self.mouse_wheel_delta += delta;
    }

    pub fn mouse_delta(&self) -> Vec2i { self.mouse_delta }
    pub fn mouse_wheel_delta(&self) -> i32 { self.mouse_wheel_delta }
 
    pub fn set_key(&mut self, key: Keycode, down: bool) {
        match Control::from_keycode(key) {
            Some(control) => { self.pressed.insert(control, down); },
            _ => ()
        }
    }

    pub fn mouse_down(&mut self, button: sdl2::mouse::MouseButton) {
        use sdl2::mouse::MouseButton::*;
        match button {
            Left => self.left_click = true,
            Right => self.right_click = true,
            _ => ()
        }
    }
    pub fn mouse_up(&mut self, button: sdl2::mouse::MouseButton) {
        use sdl2::mouse::MouseButton::*;
        match button {
            Left => self.left_click = false,
            Right => self.right_click = false,
            _ => ()
        }
    }

    pub fn pressed(&self, control: Control) -> bool {
        if self.pressed.contains_key(&control) {
            self.pressed[&control]
        } else {
            false
        }
    }

    pub fn frame_reset(&mut self) {
        for control in Control::press_controls() {
            self.pressed.insert(control, false);
        }
        self.mouse_delta = Vec2i::zero();
        self.mouse_wheel_delta = 0;
    } 
}