use nvim_oxi::{Function, api::types::LogLevel};

use crate::{get_chat_window, utils::notify};

/// Dismiss the current chat session.
pub fn dismiss_chat_fn() -> Function<(), ()> {
    Function::from_fn({
        move |()| {
            if let Ok(mut win) = get_chat_window().lock()
                && let Err(e) = win.dismiss_chat()
            {
                notify(format!("{}", e), LogLevel::Error);
            }
        }
    })
}
