use std::{cell::RefCell, io, rc::Rc};
mod app;

use ratatui::{
    layout::Alignment,
    style::{Color, Stylize},
    widgets::{Block, BorderType, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

use ratzilla::{
    event::{KeyCode, KeyEvent},
    DomBackend, WebRenderer,
};

fn main() -> io::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;

    let app = Rc::new(RefCell::new(app::App::new()));

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
