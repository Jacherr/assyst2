pub trait Ansi {
    fn bold(&self) -> String;
    fn italic(&self) -> String;
    fn underline(&self) -> String;
    fn strikethrough(&self) -> String;
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

impl Ansi for str {
    fn bold(&self) -> String {
        format!("\x1b[1m{}\x1b[22m", self)
    }
    fn italic(&self) -> String {
        format!("\x1b[3m{}\x1b[23m", self)
    }
    fn underline(&self) -> String {
        format!("\x1b[4m{}\x1b[24m", self)
    }
    fn strikethrough(&self) -> String {
        format!("\x1b[9m{}\x1b[29m", self)
    }
    fn fg_black(&self) -> String {
        format!("\x1b[30m{}\x1b[39m", self)
    }
    fn fg_red(&self) -> String {
        format!("\x1b[31m{}\x1b[39m", self)
    }
    fn fg_green(&self) -> String {
        format!("\x1b[32m{}\x1b[39m", self)
    }
    fn fg_yellow(&self) -> String {
        format!("\x1b[33m{}\x1b[39m", self)
    }
    fn fg_blue(&self) -> String {
        format!("\x1b[34m{}\x1b[39m", self)
    }
    fn fg_magenta(&self) -> String {
        format!("\x1b[35m{}\x1b[39m", self)
    }
    fn fg_cyan(&self) -> String {
        format!("\x1b[36m{}\x1b[39m", self)
    }
    fn fg_white(&self) -> String {
        format!("\x1b[37m{}\x1b[39m", self)
    }
    fn fg_bright_black(&self) -> String {
        format!("\x1b[90m{}\x1b[39m", self)
    }
    fn fg_bright_red(&self) -> String {
        format!("\x1b[91m{}\x1b[39m", self)
    }
    fn fg_bright_green(&self) -> String {
        format!("\x1b[92m{}\x1b[39m", self)
    }
    fn fg_bright_yellow(&self) -> String {
        format!("\x1b[93m{}\x1b[39m", self)
    }
    fn fg_bright_blue(&self) -> String {
        format!("\x1b[94m{}\x1b[39m", self)
    }
    fn fg_bright_magenta(&self) -> String {
        format!("\x1b[95m{}\x1b[39m", self)
    }
    fn fg_bright_cyan(&self) -> String {
        format!("\x1b[96m{}\x1b[39m", self)
    }
    fn fg_bright_white(&self) -> String {
        format!("\x1b[97m{}\x1b[39m", self)
    }
    fn bg_black(&self) -> String {
        format!("\x1b[40m{}\x1b[49m", self)
    }
    fn bg_red(&self) -> String {
        format!("\x1b[41m{}\x1b[49m", self)
    }
    fn bg_green(&self) -> String {
        format!("\x1b[42m{}\x1b[49m", self)
    }
    fn bg_yellow(&self) -> String {
        format!("\x1b[43m{}\x1b[49m", self)
    }
    fn bg_blue(&self) -> String {
        format!("\x1b[44m{}\x1b[49m", self)
    }
    fn bg_magenta(&self) -> String {
        format!("\x1b[45m{}\x1b[49m", self)
    }
    fn bg_cyan(&self) -> String {
        format!("\x1b[46m{}\x1b[49m", self)
    }
    fn bg_white(&self) -> String {
        format!("\x1b[47m{}\x1b[49m", self)
    }
    fn bg_bright_black(&self) -> String {
        format!("\x1b[100m{}\x1b[49m", self)
    }
    fn bg_bright_red(&self) -> String {
        format!("\x1b[101m{}\x1b[49m", self)
    }
    fn bg_bright_green(&self) -> String {
        format!("\x1b[102m{}\x1b[49m", self)
    }
    fn bg_bright_yellow(&self) -> String {
        format!("\x1b[103m{}\x1b[49m", self)
    }
    fn bg_bright_blue(&self) -> String {
        format!("\x1b[104m{}\x1b[49m", self)
    }
    fn bg_bright_magenta(&self) -> String {
        format!("\x1b[105m{}\x1b[49m", self)
    }
    fn bg_bright_cyan(&self) -> String {
        format!("\x1b[106m{}\x1b[49m", self)
    }
    fn bg_bright_white(&self) -> String {
        format!("\x1b[107m{}\x1b[49m", self)
    }
}
