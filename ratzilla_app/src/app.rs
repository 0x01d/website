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
//pub mod popstate_listener;

use displays::Displays;
use splash::SplashModel;
use blog::BlogModel;
use intro::IntroModel;

pub enum Msg {
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    NavigateBack,
    Select,
    SwitchTo(Displays),
    LoadSubPath(String),
    LoadHash(String),
    PushStateFromDisplay(Displays),
    UpdateBlogTags(Vec<blog::Tag>),
    UpdateBlogIndex(Vec<blog::BlogEntry>),
    ParseBlogText(String),
}

pub struct App {
    pub current: Displays,
    splash: SplashModel,
    blog: BlogModel,
    intro: IntroModel,
    window: Window,
    pub listener: Option<EventListener>,
    tx: flume::Sender<Msg>,
    rx: flume::Receiver<Msg>,
}

impl App {
    pub fn new(path: String, tx: flume::Sender<Msg>, rx: flume::Receiver<Msg>) -> Self {
        let (current, _) = Self::split_path(&path);
        let window = web_sys::window().expect("No global 'window' exists");
        Self {
            current,
            intro: IntroModel::new(),
            splash: SplashModel::new(),
            blog: BlogModel::new(tx.clone(), rx.clone()),
            window,
            listener: None,
            rx,
            tx,
        }
    }

    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::SwitchTo(s) => {
                self.current = s;
                match s {
                    Displays::Blog => self.blog.loaded_blog = None,
                    _ => {}
                }
                return
            }
            Msg::PushStateFromDisplay(s) => {
                if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
                    let _ = history.push_state_with_url(&JsValue::NULL, "", Some(s.path()));
                }
                return
            }
            Msg::UpdateBlogTags(ref tags) => {
                self.blog.tag_list = tags.to_vec();
                self.blog.tag_list_state.select(Some(0));
                return
            }
            Msg::UpdateBlogIndex(ref index) => {
                self.blog.blog_list = index.to_vec();
                if let Some(hash) = self.window.location().hash().ok() {
                    if hash.is_empty() {
                        self.blog.blog_list_filtered = index.to_vec();
                        self.blog.blog_list_state.select(Some(0));
                    } else {
                        self.update(Msg::LoadHash(hash));
                        self.blog.blog_list_state.select(Some(0));
                    }
                } else {
                    self.blog.blog_list_filtered = index.to_vec();
                    self.blog.blog_list_state.select(Some(0));
                }
                return
            }
            Msg::ParseBlogText(ref text) => {
                self.blog.parse_blog_text(text.to_string()); 
                return
            }
            _ => {}
        }
        match self.current {
            Displays::Splash => {
                for msg in self.splash.update(msg) {
                    self.update(msg);
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

    pub fn render(&mut self, frame: &mut Frame) {
        match self.current {
            Displays::Splash => self.splash.view(frame),
            Displays::Blog => self.blog.view(frame),
            _ => {}
        } 
        if let Some(msg) = self.rx.try_recv().ok() {
            self.update(msg);
        }
    }

    pub fn handle_events(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => self.update(Msg::NavigateUp),
            KeyCode::Down => self.update(Msg::NavigateDown),
            KeyCode::Left => self.update(Msg::NavigateLeft),
            KeyCode::Right => self.update(Msg::NavigateRight),
            KeyCode::Enter => self.update(Msg::Select),
            KeyCode::Backspace => self.update(Msg::NavigateBack),
            KeyCode::Esc =>  {
                self.update(Msg::SwitchTo(Displays::Splash));
                self.update(Msg::PushStateFromDisplay(Displays::Splash));
            }
            _ =>  {}
        };
    }

    //TODO: move out of app
    pub fn handle_popstate(&mut self) {
        //web_sys::console::log_1(event.as_ref());
        if let Some(path) = self.window.location().pathname().ok() {
            let (display, rest) = Self::split_path(&path);
            self.update(Msg::SwitchTo(display));
            if let Some(sub_path) = rest {
                self.update(Msg::LoadSubPath(sub_path));
            }
        }
        if let Some(hash) = self.window.location().hash().ok() {
            //web_sys::console::log_1(&hash.clone().into());
            self.update(Msg::LoadHash(hash));
        }
    }
    pub fn split_path(path: &str) -> (Displays, Option<String>) {
        //let path_chunks: Vec<&str> = path.split('/').filter(|c| !c.is_empty()).collect();
        let mut it = path.split('/').filter(|part| !part.is_empty());

        let display = Displays::from_path(it.next());
        if let Some(rest) = it.next() {
            return (display, Some(rest.to_string()))
        }

        (display, None)
    }
}
