mod close;
mod open;
mod setup;
mod toggle;

pub mod action;

use nvim_oxi::{Dictionary, Object};

use crate::lua_modules::action::create_lua_action_module;

/// Build the top-level plugin module exposed to Lua.
pub fn create_lua_module() -> Dictionary {
    let mut module = Dictionary::new();
    module.insert("setup", Object::from(setup::setup_fn()));
    module.insert("open", Object::from(open::open_fn()));
    module.insert("toggle", Object::from(toggle::toggle_fn()));
    module.insert("close", Object::from(close::close_fn()));
    module.insert("action", Object::from(create_lua_action_module()));
    module
}
