/// Common configuration options for the window manager.
pub struct Config {
    /// Whether focus follows mouse movements or
    /// only click events and keyboard movements.
    pub focus_follows_mouse: bool,
    /// Border color for focused windows.
    pub focus_border_color: uint,
    /// Border color for unfocused windows.
    pub border_color: uint,
    /// Border width. This is the same for both, focused and unfocused.
    pub border_width: uint,
    /// Default spacing between windows
    pub spacing: uint,
    /// Default terminal to start
    pub terminal: String,
    /// Default tags for workspaces
    pub tags: Vec<String>
}

impl Config {
    /// Create the default configuration.
    pub fn default() -> Config {
        Config {
            focus_follows_mouse: true,
            focus_border_color:  0x00FFB6B0,
            border_color:        0x00FFFFFF,
            border_width:        2,
            spacing:             10,
            terminal:            String::from_str("xterm"),
            tags:                vec!(
                                     String::from_str("1"), 
                                     String::from_str("2"))

        }
    }
}
