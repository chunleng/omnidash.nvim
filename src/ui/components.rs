use crate::ui::nvim_primitives::buffer::{NvimBuffer, NvimBufferOption, NvimKeymap};
use nvim_oxi::{
    Result as OxiResult,
    api::{
        self,
        opts::OptionOpts,
        types::{WindowConfig, WindowRelativeTo},
    },
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum SplitWindowOption {
    Top,
    Bottom,
    Left,
    Right,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum WindowOption {
    CenteredFloat {
        height: f64,
        width: f64,
    },
    Split {
        direction: SplitWindowOption,
        ratio_wh: f64,
        edge: bool,
    },
}

#[derive(Debug, Clone)]
pub struct FixedBufferVimWindowOption {
    pub buf_type: String,
    pub buf_listed: bool,
    pub swap_file: bool,
    pub file_type: String,
    pub modifiable: bool,
    pub wrap: bool,
    pub line_break: bool,
    pub undo_levels: isize,
    pub text_width: isize,
    pub number: bool,
    pub relative_number: bool,
    pub sign_column: String,
    pub buf_keymaps: Vec<NvimKeymap>,
    pub window_option: WindowOption,
}

impl Default for FixedBufferVimWindowOption {
    fn default() -> Self {
        Self {
            buf_type: String::from("nofile"),
            buf_listed: false,
            swap_file: false,
            file_type: String::from(""),
            modifiable: true,
            wrap: true,
            line_break: true,
            undo_levels: 1000,
            text_width: 0,
            sign_column: "auto".to_string(),
            number: true,
            relative_number: true,
            buf_keymaps: vec![],
            window_option: WindowOption::CenteredFloat {
                height: 0.6,
                width: 0.6,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct FixedBufferVimWindow {
    buffer: NvimBuffer,
    window: api::Window,
}

impl FixedBufferVimWindow {
    pub fn new(option: FixedBufferVimWindowOption) -> OxiResult<Self> {
        let buffer = NvimBuffer::try_from(&option)?;

        let window = match option.window_option {
            WindowOption::CenteredFloat { height, width } => {
                let ui_width = api::get_option_value::<i64>("columns", &OptionOpts::default())?;
                let win_width = (ui_width as f64 * width) as u32;
                let ui_height = api::get_option_value::<i64>("lines", &OptionOpts::default())?;
                let win_height = (ui_height as f64 * height) as u32;

                let row = ((ui_height as f64 * (1.0 - height)) / 2.0) as u32;
                let col = ((ui_width as f64 * (1.0 - width)) / 2.0) as u32;

                let win_config = WindowConfig::builder()
                    .relative(WindowRelativeTo::Editor)
                    .width(win_width)
                    .height(win_height)
                    .row(row)
                    .col(col)
                    .build();
                api::open_win(&buffer.inner, true, &win_config)?
            }
            WindowOption::Split {
                direction,
                edge,
                ratio_wh,
            } => {
                let split_type = match (&direction, &edge) {
                    (SplitWindowOption::Top, true) | (SplitWindowOption::Left, true) => "topleft",
                    (SplitWindowOption::Bottom, true) | (SplitWindowOption::Right, true) => {
                        "botright"
                    }
                    (SplitWindowOption::Top, false) | (SplitWindowOption::Left, false) => {
                        "aboveleft"
                    }
                    (SplitWindowOption::Bottom, false) | (SplitWindowOption::Right, false) => {
                        "belowright"
                    }
                };
                let vh = match &direction {
                    SplitWindowOption::Top | SplitWindowOption::Bottom => "split",
                    SplitWindowOption::Left | SplitWindowOption::Right => "vsplit",
                };
                api::command(&format!("{} {}", split_type, vh))?;

                match &direction {
                    SplitWindowOption::Top | SplitWindowOption::Bottom => {
                        let ui_height =
                            api::get_option_value::<i64>("lines", &OptionOpts::default())?;
                        let win_height = (ui_height as f64 * ratio_wh) as u32;
                        api::command(&format!("horizontal resize {}", win_height))?;
                    }
                    SplitWindowOption::Left | SplitWindowOption::Right => {
                        let ui_width =
                            api::get_option_value::<i64>("columns", &OptionOpts::default())?;
                        let win_width = (ui_width as f64 * ratio_wh) as u32;
                        api::command(&format!("vertical resize {}", win_width))?;
                    }
                }
                let mut win = api::get_current_win();
                win.set_buf(&buffer.inner)?;
                win
            }
        };

        let win_opts = OptionOpts::builder().win(window.clone()).build();
        // Needed for this struct as we want to make sure window's buffer doesn't change
        api::set_option_value("winfixbuf", true, &win_opts)?;

        api::set_option_value("wrap", option.wrap, &win_opts)?;
        api::set_option_value("linebreak", option.line_break, &win_opts)?;
        api::set_option_value("signcolumn", option.sign_column, &win_opts)?;
        api::set_option_value("number", option.number, &win_opts)?;
        api::set_option_value("relativenumber", option.relative_number, &win_opts)?;

        Ok(Self { buffer, window })
    }

    pub fn get_buffer(&self) -> Option<api::Buffer> {
        self.buffer.get_buffer()
    }

    pub fn get_window(&self) -> Option<api::Window> {
        if self.window.is_valid() {
            Some(self.window.clone())
        } else {
            None
        }
    }
}

impl TryFrom<&FixedBufferVimWindowOption> for NvimBuffer {
    type Error = nvim_oxi::Error;

    fn try_from(value: &FixedBufferVimWindowOption) -> Result<Self, Self::Error> {
        Self::new(NvimBufferOption {
            buf_type: value.buf_type.to_string(),
            buf_listed: value.buf_listed,
            // TODO FixedBufferVimWindow actually does not have to be wiped always, but we need to
            // think of ways to ensure that we don't get leftover hidden buffers.
            buf_hidden: "wipe".to_string(),
            swap_file: value.swap_file,
            file_type: value.file_type.to_string(),
            modifiable: value.modifiable,
            undo_levels: value.undo_levels,
            text_width: value.text_width,
            buf_keymaps: value.buf_keymaps.clone(),
        })
    }
}
