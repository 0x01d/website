use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, Event};
use std::rc::Rc;
use std::cell::RefCell;
use crate::app::{App, Msg, Displays};

pub fn setup_popstate_listener(app: Rc<RefCell<App>>) {
    let closure = Closure::<dyn FnMut(_)>::new(move |_event: Event| {
        if let Some(path) = window().expect("No window").location().pathname().ok() {
            let display = Displays::from_path(&path); 
            app.borrow_mut().update(Msg::SwitchTo(display));
        }
    });

    let win = window().expect("no window");
    win.add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
        .expect("could not add popstate listener");

}

/*
   fn current_pathname() -> Option<String> {
   let location = window()?.location();
   let origin = location.origin().ok();         // e.g., "http://localhost:8080"
   let pathname = location.pathname().ok();     // e.g., "/blog/post-1"
   let search = location.search().ok();         // e.g., "?id=42"
   let hash = location.hash().ok();             // e.g., "#section"
   }
   */
