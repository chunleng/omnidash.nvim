use nvim_oxi::Function;

use crate::get_chat_window;

/// Open the chat window.
pub fn open_fn() -> Function<(), ()> {
    Function::from_fn_mut(move |()| {
        if let Ok(mut win) = get_chat_window().lock() {
            let _ = win.open();
        }
    })
}
