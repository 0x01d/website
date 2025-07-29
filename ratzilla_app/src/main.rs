use std::{cell::RefCell, io, rc::Rc};
use web_sys::window;
use gloo::events::EventListener;

mod app;

use crate::app::Msg;

use ratatui::{
    Terminal,
};

use ratzilla::{
    CanvasBackend, WebRenderer, event::MouseButton, event::MouseEventKind
};


fn main() -> io::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let backend = CanvasBackend::new()?;
    let mut terminal = Terminal::new(backend)?;
    
    let window = window().expect("No window");
    let path = window.location().pathname().expect("No path"); 

    let (tx, rx) = flume::unbounded::<Msg>();

    let app = Rc::new(RefCell::new(app::App::new(path, tx.clone(), rx.clone())));

    let popstate_clone = Rc::clone(&app);

    let popstate_listener = EventListener::new(&window, "popstate", move |event| {
        popstate_clone.borrow_mut().handle_popstate();
    });

    app.borrow_mut().listener = Some(popstate_listener);
    
    // Send a popstate to load app on correct page
    Rc::clone(&app).borrow_mut().handle_popstate();


    let mouse_app_clone = Rc::clone(&app);
    terminal.on_mouse_event( move |mouse_event| {
        mouse_app_clone.borrow_mut().handle_mouse_events(mouse_event)
    });

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
