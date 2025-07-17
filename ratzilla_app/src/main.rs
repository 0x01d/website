use std::{cell::RefCell, io, rc::Rc};
use web_sys::window;
use gloo::events::EventListener;

mod app;

use ratatui::{
    Terminal,
};

use ratzilla::{
    DomBackend, WebRenderer,
};

use crate::app::popstate_listener::setup_popstate_listener;

fn main() -> io::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;
    
    let window = window().expect("No window");
    let path = window.location().pathname().expect("No path"); 

    let app = Rc::new(RefCell::new(app::App::new(path)));

    let popstate_clone = Rc::clone(&app);

    let popstate_listener = EventListener::new(&window, "popstate", move |event| {
        popstate_clone.borrow_mut().handle_popstate(event.clone());
    });

    app.borrow_mut().listener = Some(popstate_listener);


    let event_state = Rc::clone(&app);
    terminal.on_key_event(move |key_event| {
        event_state.borrow_mut().handle_events(key_event);
    });

    let render_state = Rc::clone(&app);
    terminal.draw_web(move |frame| {
        render_state.borrow_mut().render(frame);
    });



    Ok(())
}
