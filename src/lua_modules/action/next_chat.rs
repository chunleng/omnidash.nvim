use nvim_oxi::{Function, api::types::LogLevel};

use crate::{get_chat_window, utils::notify};

/// Load the next chat session.
pub fn next_chat_fn() -> Function<(), ()> {
    Function::from_fn({
        move |()| {
            if let Ok(mut win) = get_chat_window().lock()
                && let Err(e) = win.load_next_chat()
            {
                notify(format!("{}", e), LogLevel::Error);
            }
        }
    })
}
