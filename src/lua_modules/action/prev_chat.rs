use nvim_oxi::{Function, api::types::LogLevel};

use crate::{get_chat_window, utils::notify};

/// Load the previous chat session.
pub fn prev_chat_fn() -> Function<(), ()> {
    Function::from_fn({
        move |()| {
            if let Ok(mut win) = get_chat_window().lock()
                && let Err(e) = win.load_prev_chat()
            {
                notify(format!("{}", e), LogLevel::Error);
            }
        }
    })
}
