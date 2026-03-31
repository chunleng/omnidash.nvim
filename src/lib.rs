use std::sync::{Arc, Mutex};

use nvim_oxi::{Dictionary, Function, Object, Result as OxiResult};

use crate::ui::ChatWindow;

mod chat;
mod ui;

#[nvim_oxi::plugin]
fn omnidash() -> OxiResult<Dictionary> {
    let chat_window = Arc::new(Mutex::new(ChatWindow::new()));

    let chat_fn = Function::from_fn_mut({
        let win_clone = chat_window.clone();
        move |x| {
            if let Ok(mut win) = win_clone.lock() {
                let _ = win.open();
                win.chat_process.send_message(x)
            }
        }
    });

    let open_fn = Function::from_fn_mut({
        let win_clone = chat_window.clone();
        move |()| {
            if let Ok(mut win) = win_clone.lock() {
                let _ = win.open();
            }
        }
    });

    let mut module = Dictionary::new();
    module.insert("chat", Object::from(chat_fn));
    module.insert("open", Object::from(open_fn));
    Ok(module)
}
