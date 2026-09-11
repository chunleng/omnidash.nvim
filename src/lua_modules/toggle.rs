use nvim_oxi::{Function, api::types::LogLevel};

use crate::{get_chat_window, utils::notify};

/// Toggle the chat window.
pub fn toggle_fn() -> Function<(), ()> {
    Function::from_fn(move |()| {
        if let Ok(mut win) = get_chat_window().lock()
            && let Err(e) = win.toggle()
        {
            notify(format!("{}", e), LogLevel::Error);
        }
    })
}
