#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub fg: Option<Color>,
    pub bold: bool,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    Blue,
    BrightBlack,
    Cyan,
    Green,
    Magenta,
    Purple,
    Red,
    Yellow,
}

#[allow(dead_code)]
impl Style {
    pub fn new() -> Style {
        let fg = None;
        let bold = false;
        Style { fg, bold }
    }

    pub fn blue(mut self) -> Style {
        self.fg = Some(Color::Blue);
        self
    }

    pub fn bright_black(mut self) -> Style {
        self.fg = Some(Color::BrightBlack);
        self
    }

    pub fn cyan(mut self) -> Style {
        self.fg = Some(Color::Cyan);
        self
    }

    pub fn green(mut self) -> Style {
        self.fg = Some(Color::Green);
        self
    }

    pub fn magenta(mut self) -> Style {
        self.fg = Some(Color::Magenta);
        self
    }

    pub fn purple(mut self) -> Style {
        self.fg = Some(Color::Purple);
        self
    }

    pub fn red(mut self) -> Style {
        self.fg = Some(Color::Red);
        self
    }

    pub fn yellow(mut self) -> Style {
        self.fg = Some(Color::Yellow);
        self
    }

    pub fn bold(mut self) -> Style {
        self.bold = true;
        self
    }
}
