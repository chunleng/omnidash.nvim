use nvim_oxi::{Function, api::types::LogLevel};

use crate::{get_chat_window, utils::notify};

/// Start a new chat session.
pub fn new_chat_fn() -> Function<(), ()> {
    Function::from_fn({
        move |()| {
            if let Ok(mut win) = get_chat_window().lock()
                && let Err(e) = win.new_chat()
            {
                notify(format!("{}", e), LogLevel::Error);
            }
        }
    })
}
