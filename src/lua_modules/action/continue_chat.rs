use nvim_oxi::{Function, api::types::LogLevel};

use crate::{get_chat_window, utils::notify};

/// Continue the chat without adding a new user message.
pub fn continue_chat_fn() -> Function<(), ()> {
    Function::from_fn({
        move |()| {
            if let Ok(mut win) = get_chat_window().lock()
                && let Err(e) = win.continue_chat()
            {
                notify(format!("{}", e), LogLevel::Error);
            }
        }
    })
}
