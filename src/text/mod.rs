mod style;
mod styledstring;

use colored::control;
pub use style::*;
pub use styledstring::*;

#[cfg(target_family = "unix")]
pub fn init_crate_colored() {
    control::set_override(true);
}

#[cfg(target_family = "windows")]
pub fn init_crate_colored() {
    control::set_override(true);
    control::set_virtual_terminal(true).expect("set virtual terminal");
}
