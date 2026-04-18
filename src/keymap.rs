use std::sync::{Arc, Mutex};

use nvim_oxi::{Dictionary, Function, Object, api::types::LogLevel};

use crate::{ui::ChatWindow, utils::notify};

pub fn create_lua_keymap_module(chat_window: Arc<Mutex<ChatWindow>>) -> Dictionary {
    let mut keymap_dict = Dictionary::new();
    keymap_dict.insert("send", Object::from(send_fn(chat_window.clone())));
    keymap_dict.insert("close", Object::from(close_fn(chat_window.clone())));
    keymap_dict.insert("next_chat", Object::from(next_chat_fn(chat_window.clone())));
    keymap_dict.insert("prev_chat", Object::from(prev_chat_fn(chat_window.clone())));
    keymap_dict.insert("new_chat", Object::from(new_chat_fn(chat_window.clone())));
    keymap_dict.insert("dismiss_chat", Object::from(dismiss_chat_fn(chat_window)));

    keymap_dict
}

fn send_fn(chat_window: Arc<Mutex<ChatWindow>>) -> Function<(), ()> {
    Function::from_fn({
        move |()| {
            if let Ok(mut win) = chat_window.lock() {
                if let Err(e) = win.send() {
                    notify(format!("{}", e), LogLevel::Error);
                }
            }
        }
    })
}

fn close_fn(chat_window: Arc<Mutex<ChatWindow>>) -> Function<(), ()> {
    Function::from_fn({
        move |()| {
            if let Ok(mut win) = chat_window.lock() {
                if let Err(e) = win.close() {
                    notify(format!("{}", e), LogLevel::Error);
                }
            }
        }
    })
}

fn next_chat_fn(chat_window: Arc<Mutex<ChatWindow>>) -> Function<(), ()> {
    Function::from_fn({
        move |()| {
            if let Ok(mut win) = chat_window.lock() {
                if let Err(e) = win.load_next_chat() {
                    notify(format!("{}", e), LogLevel::Error);
                }
            }
        }
    })
}

fn prev_chat_fn(chat_window: Arc<Mutex<ChatWindow>>) -> Function<(), ()> {
    Function::from_fn({
        move |()| {
            if let Ok(mut win) = chat_window.lock() {
                if let Err(e) = win.load_prev_chat() {
                    notify(format!("{}", e), LogLevel::Error);
                }
            }
        }
    })
}

fn new_chat_fn(chat_window: Arc<Mutex<ChatWindow>>) -> Function<(), ()> {
    Function::from_fn({
        move |()| {
            if let Ok(mut win) = chat_window.lock() {
                if let Err(e) = win.new_chat() {
                    notify(format!("{}", e), LogLevel::Error);
                }
            }
        }
    })
}

fn dismiss_chat_fn(chat_window: Arc<Mutex<ChatWindow>>) -> Function<(), ()> {
    Function::from_fn({
        move |()| {
            if let Ok(mut win) = chat_window.lock() {
                if let Err(e) = win.dismiss_chat() {
                    notify(format!("{}", e), LogLevel::Error);
                }
            }
        }
    })
}
