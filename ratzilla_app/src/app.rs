use ratzilla::{
    event::{KeyCode, KeyEvent},
};
use ratatui::Frame;
use web_sys::{Window, PopStateEvent, Event};
use wasm_bindgen::JsValue;
use gloo::events::EventListener;

mod intro;
mod displays;
mod splash;
mod blog;
mod tui_helpers;
pub mod popstate_listener;

use displays::Displays;
use splash::SplashModel;
use blog::BlogModel;
use intro::IntroModel;

pub enum Msg {
    NavigateUp,
    NavigateDown,
    Select,
    SwitchTo(Displays),
}

pub struct App {
    pub current: Displays,
    splash: SplashModel,
    blog: BlogModel,
    intro: IntroModel,
    window: Window,
    pub listener: Option<EventListener>
}

impl App {
    pub fn new(path: String) -> Self {
        let current = Displays::from_path(&path);
        let window = web_sys::window().expect("No global 'window' exists");
        Self {
            current,
            intro: IntroModel::new(),
            splash: SplashModel::new(),
            blog: BlogModel::new(),
            window,
            listener: None,
        }
    }

    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::SwitchTo(s) => {
                self.current = s;
                if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
                    let _ = history.push_state_with_url(&JsValue::NULL, "", Some(s.path()));
                }
                match s {
                    Displays::Blog => {
                        self.blog.fetch_tags_and_index();
                    },
                    _ => {}
                }
            }
            _ => {}
        }
        match self.current {
            Displays::Splash => {
                if let Some(m) = self.splash.update(msg) {
                    self.update(m);
                }
            }
            Displays::Blog => {
                match msg {
                    _ => self.blog.update(msg),
                }
            }
            _ => {}
        }
    }
    /*
       pub fn view<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
       match self.current {
       AppState::Splash => self.splash.view(f, area),
       AppState::Blog => self.blog.view(f, area),
       }
       }
       */
    pub fn render(&mut self, frame: &mut Frame) {
        match self.current {
            Displays::Splash => self.splash.view(frame),
            Displays::Blog => self.blog.view(frame),
            _ => {}
        } 
    }

    pub fn handle_events(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => self.update(Msg::NavigateUp),
            KeyCode::Down => self.update(Msg::NavigateDown),
            KeyCode::Enter => self.update(Msg::Select),
            KeyCode::Esc => self.update(Msg::SwitchTo(Displays::Splash)),
            _ =>  {}
        };
    }

    pub fn handle_popstate(&mut self, event: Event) {
        web_sys::console::log_1(event.as_ref());
        if let Some(path) = self.window.location().pathname().ok() {
            let current = Displays::from_path(&path);
            self.update(Msg::SwitchTo(current));
        }
        //window.
    }
}
