use nvim_oxi::{Function, api::types::LogLevel};

use crate::{get_chat_window, utils::notify};

/// Close the chat window.
pub fn close_fn() -> Function<(), ()> {
    Function::from_fn(move |()| {
        if let Ok(mut win) = get_chat_window().lock()
            && let Err(e) = win.close()
        {
            notify(format!("{}", e), LogLevel::Error);
        }
    })
}
