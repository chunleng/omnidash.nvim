use nvim_oxi::{Function, api::types::LogLevel};

use crate::{get_chat_window, utils::notify};

/// Stop the current streaming response.
pub fn stop_streaming_fn() -> Function<(), ()> {
    Function::from_fn({
        move |()| {
            if let Ok(mut win) = get_chat_window().lock()
                && let Err(e) = win.stop_streaming()
            {
                notify(format!("{}", e), LogLevel::Error);
            }
        }
    })
}
