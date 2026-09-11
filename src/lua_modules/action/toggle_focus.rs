use nvim_oxi::{Function, api::types::LogLevel};

use crate::{get_chat_window, utils::notify};

/// Toggle focus between chat input and output.
pub fn toggle_focus_fn() -> Function<(), ()> {
    Function::from_fn({
        move |()| {
            if let Ok(mut win) = get_chat_window().lock()
                && let Err(e) = win.toggle_focus()
            {
                notify(format!("{}", e), LogLevel::Error);
            }
        }
    })
}
