use std::fmt::Display;

pub trait Ansi {
    fn a_bold(&self) -> String;
    fn a_italic(&self) -> String;
    fn a_underline(&self) -> String;
    fn a_strikethrough(&self) -> String;
    fn fg_black(&self) -> String;
    fn fg_red(&self) -> String;
    fn fg_green(&self) -> String;
    fn fg_yellow(&self) -> String;
    fn fg_blue(&self) -> String;
    fn fg_magenta(&self) -> String;
    fn fg_cyan(&self) -> String;
    fn fg_white(&self) -> String;
    fn fg_bright_black(&self) -> String;
    fn fg_bright_red(&self) -> String;
    fn fg_bright_green(&self) -> String;
    fn fg_bright_yellow(&self) -> String;
    fn fg_bright_blue(&self) -> String;
    fn fg_bright_magenta(&self) -> String;
    fn fg_bright_cyan(&self) -> String;
    fn fg_bright_white(&self) -> String;
    fn bg_black(&self) -> String;
    fn bg_red(&self) -> String;
    fn bg_green(&self) -> String;
    fn bg_yellow(&self) -> String;
    fn bg_blue(&self) -> String;
    fn bg_magenta(&self) -> String;
    fn bg_cyan(&self) -> String;
    fn bg_white(&self) -> String;
    fn bg_bright_black(&self) -> String;
    fn bg_bright_red(&self) -> String;
    fn bg_bright_green(&self) -> String;
    fn bg_bright_yellow(&self) -> String;
    fn bg_bright_blue(&self) -> String;
    fn bg_bright_magenta(&self) -> String;
    fn bg_bright_cyan(&self) -> String;
    fn bg_bright_white(&self) -> String;
}

impl<T> Ansi for T
where
    T: Display,
{
    fn a_bold(&self) -> String {
        format!("\x1b[1m{self}\x1b[22m")
    }
    fn a_italic(&self) -> String {
        format!("\x1b[3m{self}\x1b[23m")
    }
    fn a_underline(&self) -> String {
        format!("\x1b[4m{self}\x1b[24m")
    }
    fn a_strikethrough(&self) -> String {
        format!("\x1b[9m{self}\x1b[29m")
    }
    fn fg_black(&self) -> String {
        format!("\x1b[30m{self}\x1b[39m")
    }
    fn fg_red(&self) -> String {
        format!("\x1b[31m{self}\x1b[39m")
    }
    fn fg_green(&self) -> String {
        format!("\x1b[32m{self}\x1b[39m")
    }
    fn fg_yellow(&self) -> String {
        format!("\x1b[33m{self}\x1b[39m")
    }
    fn fg_blue(&self) -> String {
        format!("\x1b[34m{self}\x1b[39m")
    }
    fn fg_magenta(&self) -> String {
        format!("\x1b[35m{self}\x1b[39m")
    }
    fn fg_cyan(&self) -> String {
        format!("\x1b[36m{self}\x1b[39m")
    }
    fn fg_white(&self) -> String {
        format!("\x1b[37m{self}\x1b[39m")
    }
    fn fg_bright_black(&self) -> String {
        format!("\x1b[90m{self}\x1b[39m")
    }
    fn fg_bright_red(&self) -> String {
        format!("\x1b[91m{self}\x1b[39m")
    }
    fn fg_bright_green(&self) -> String {
        format!("\x1b[92m{self}\x1b[39m")
    }
    fn fg_bright_yellow(&self) -> String {
        format!("\x1b[93m{self}\x1b[39m")
    }
    fn fg_bright_blue(&self) -> String {
        format!("\x1b[94m{self}\x1b[39m")
    }
    fn fg_bright_magenta(&self) -> String {
        format!("\x1b[95m{self}\x1b[39m")
    }
    fn fg_bright_cyan(&self) -> String {
        format!("\x1b[96m{self}\x1b[39m")
    }
    fn fg_bright_white(&self) -> String {
        format!("\x1b[97m{self}\x1b[39m")
    }
    fn bg_black(&self) -> String {
        format!("\x1b[40m{self}\x1b[49m")
    }
    fn bg_red(&self) -> String {
        format!("\x1b[41m{self}\x1b[49m")
    }
    fn bg_green(&self) -> String {
        format!("\x1b[42m{self}\x1b[49m")
    }
    fn bg_yellow(&self) -> String {
        format!("\x1b[43m{self}\x1b[49m")
    }
    fn bg_blue(&self) -> String {
        format!("\x1b[44m{self}\x1b[49m")
    }
    fn bg_magenta(&self) -> String {
        format!("\x1b[45m{self}\x1b[49m")
    }
    fn bg_cyan(&self) -> String {
        format!("\x1b[46m{self}\x1b[49m")
    }
    fn bg_white(&self) -> String {
        format!("\x1b[47m{self}\x1b[49m")
    }
    fn bg_bright_black(&self) -> String {
        format!("\x1b[100m{self}\x1b[49m")
    }
    fn bg_bright_red(&self) -> String {
        format!("\x1b[101m{self}\x1b[49m")
    }
    fn bg_bright_green(&self) -> String {
        format!("\x1b[102m{self}\x1b[49m")
    }
    fn bg_bright_yellow(&self) -> String {
        format!("\x1b[103m{self}\x1b[49m")
    }
    fn bg_bright_blue(&self) -> String {
        format!("\x1b[104m{self}\x1b[49m")
    }
    fn bg_bright_magenta(&self) -> String {
        format!("\x1b[105m{self}\x1b[49m")
    }
    fn bg_bright_cyan(&self) -> String {
        format!("\x1b[106m{self}\x1b[49m")
    }
    fn bg_bright_white(&self) -> String {
        format!("\x1b[107m{self}\x1b[49m")
    }
}
