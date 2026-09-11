use nvim_oxi::{Function, api::types::LogLevel};

use crate::{get_chat_window, utils::notify};

/// Swap the output window from the detail buffer back to chat display.
pub fn show_chat_fn() -> Function<(), ()> {
    Function::from_fn({
        move |()| {
            if let Ok(mut win) = get_chat_window().lock()
                && let Err(e) = win.show_chat_view()
            {
                notify(format!("{}", e), LogLevel::Error);
            }
        }
    })
}
