use nvim_oxi::{Function, api::types::LogLevel};

use crate::{get_chat_window, utils::notify};

/// Swap the output window from chat display to the detail buffer.
pub fn show_detail_fn() -> Function<(), ()> {
    Function::from_fn({
        move |()| {
            if let Ok(mut win) = get_chat_window().lock()
                && let Err(e) = win.show_detail_view()
            {
                notify(format!("{}", e), LogLevel::Error);
            }
        }
    })
}
