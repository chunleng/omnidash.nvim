use serde::Deserialize;

use nvim_oxi::{Function, Object, api::types::LogLevel, serde::Deserializer};

use crate::{
    CONFIG,
    config::{TenonConfig, user::TenonUserConfig},
    utils::notify,
};

/// Initialize the plugin configuration. Ignores repeated calls.
pub fn setup_fn() -> Function<Object, ()> {
    Function::from_fn_mut(|conf: Object| {
        if CONFIG.get().is_some() {
            notify(
                "[tenon.nvim] setup() called after config already initialized; ignoring",
                LogLevel::Warn,
            );
            return;
        }
        CONFIG.get_or_init(|| {
            match TenonUserConfig::deserialize(Deserializer::new(conf))
                .map_err(|e| e.into())
                .and_then(TenonConfig::try_from)
            {
                Ok(res) => res,
                Err(e) => {
                    notify(
                        format!("[tenon.nvim] error reading config: {}", e),
                        LogLevel::Error,
                    );
                    notify("[tenon.nvim] using default config", LogLevel::Warn);
                    TenonConfig::default()
                }
            }
        });
    })
}
