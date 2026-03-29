use nvim_oxi::{
    Dictionary, Function, Object, Result as OxiResult,
    api::{notify, types::LogLevel},
};

use crate::ui::ChatWindow;

mod chat;
mod ui;

#[nvim_oxi::plugin]
fn omnidash() -> OxiResult<Dictionary> {
    let chat_window = match ChatWindow::new() {
        Ok(win) => win,
        Err(e) => {
            _ = notify(
                "error when trying to create a new chat",
                LogLevel::Error,
                &Dictionary::new(),
            );
            return Err(e);
        }
    };
    chat_window.spawn_chat_renderer()?;
    let mut chat_process = chat_window.chat_process;
    let chat_fn = Function::from_fn_mut(move |x| chat_process.send_message(x));

    Ok(Dictionary::from_iter([("chat", Object::from(chat_fn))]))
}
