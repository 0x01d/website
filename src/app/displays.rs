#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Displays {
    Splash,
    Blog,
    Tools,
    Acknowledgements,
    Contact,
    About,
    Intro,
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
        }
    }
}
