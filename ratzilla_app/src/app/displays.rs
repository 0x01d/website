#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Displays {
    Splash,
    Blog,
    Tools,
    Acknowledgements,
    Contact,
    About,
    Intro,
    E404,
}

impl Displays {
    pub fn all_visible() -> Vec<Displays> {
        [
            Displays::Blog,
            Displays::Tools,
            Displays::Acknowledgements,
            Displays::Contact,
            Displays::About,
        ]
        .into()
    }

    pub fn label(self) -> &'static str {
        match self {
            Displays::Splash => unreachable!("Splash is hidden"),
            Displays::Blog => "Blog",
            Displays::Tools => "Tools",
            Displays::Acknowledgements => "Acknowledgements",
            Displays::Contact => "Contact",
            Displays::About => "About",
            Displays::Intro => unreachable!("Intro is hidden"),
            Displays::E404 => unreachable!("404 is hidden"),
        }
    }
    /// Convert from URL path to Displays enum
    pub fn from_path(path: &str) -> Displays {
        match path {
            "/" | "/splash" => Displays::Splash,
            "/intro" => Displays::Intro,
            "/blog" => Displays::Blog,
            "/tools" => Displays::Tools,
            "/acknowledgements" => Displays::Acknowledgements,
            "/contact" => Displays::Contact,
            "/about" => Displays::About,
            _ => Displays::E404,
        }
    }

    /// Get URL path for this display
    pub fn path(self) -> &'static str {
        match self {
            Displays::Splash => "/splash",
            Displays::Intro => "/intro",
            Displays::Blog => "/blog",
            Displays::Tools => "/tools",
            Displays::Acknowledgements => "/acknowledgements",
            Displays::Contact => "/contact",
            Displays::About => "/about",
            Displays::E404 => "/404",
        }
    }
}
