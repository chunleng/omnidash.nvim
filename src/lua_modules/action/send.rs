use nvim_oxi::{Function, api::types::LogLevel};

use crate::{get_chat_window, utils::notify};

/// Send the current chat input.
pub fn send_fn() -> Function<(), ()> {
    Function::from_fn({
        move |()| {
            if let Ok(mut win) = get_chat_window().lock()
                && let Err(e) = win.send()
            {
                notify(format!("{}", e), LogLevel::Error);
            }
        }
    })
}
