use color::Color;
use colored::Colorize;

pub trait Decorized: std::fmt::Display {
    type Color: Color;
    fn decorized(&self) -> colored::ColoredString {
        self.to_string().color(Self::Color::color())
    }
    fn decorized_with_prefix(&self) -> colored::ColoredString {
        self.decorized()
    }
}

impl Decorized for crate::version::Version {
    type Color = color::Cyan;
    fn decorized_with_prefix(&self) -> colored::ColoredString {
        let with_prefix = format!("PHP {}", self.to_string());
        with_prefix.color(Self::Color::color())
    }
}
impl Decorized for crate::alias::Alias {
    type Color = color::Cyan;
}
impl Decorized for std::path::Display<'_> {
    type Color = color::Yellow;
}

pub mod color {
    pub trait Color {
        fn color() -> colored::Color;
    }

    pub struct Red {}
    impl Color for Red {
        fn color() -> colored::Color {
            colored::Color::Red
        }
    }
    pub struct Green {}
    impl Color for Green {
        fn color() -> colored::Color {
            colored::Color::Green
        }
    }
    pub struct Yellow {}
    impl Color for Yellow {
        fn color() -> colored::Color {
            colored::Color::Yellow
        }
    }
    pub struct Cyan {}
    impl Color for Cyan {
        fn color() -> colored::Color {
            colored::Color::Cyan
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use derive_more::Display;

    #[derive(Debug, Display)]
    struct Phantom(String);
    impl Decorized for Phantom {
        type Color = color::Red;
    }

    #[test]
    fn test() {
        let phantom = Phantom("phantom data".to_owned());
        println!("normal: {}", phantom);
        println!("decorized: {}", phantom.decorized())
    }

    #[test]
    fn path() {
        let path = std::env::current_dir().unwrap();
        println!("{}", path.display().decorized());
    }
}
