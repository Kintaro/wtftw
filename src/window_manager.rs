use core::Workspaces;
use config::Config;
use layout::LayoutManager;
use window_system::Rectangle;
use window_system::Window;
use window_system::WindowSystem;

pub type ScreenDetail = Rectangle;

pub struct WindowManager {
    workspaces: Workspaces
}

impl WindowManager {
    pub fn new(window_system: &WindowSystem, config: &Config) -> WindowManager {
        WindowManager {
            workspaces: Workspaces::new(String::from_str("Tall"), config.tags.clone(), window_system.get_screen_infos())
        }
    }

    pub fn is_window_managed(&self, window: Window) -> bool {
        false
    }

    pub fn manage(&mut self, window_system: &mut WindowSystem, window: Window, config: &Config) {
        let r = window_system.get_screen_infos()[0];
        window_system.show_window(window);
        self.workspaces.current.workspace.add(window);

        let screen = &self.workspaces.current;
        let workspace = &screen.workspace;
        let layout = LayoutManager::get_layout(workspace.layout.clone());
        let window_layout = layout.apply_layout(screen.screen_detail, &workspace.stack); 

        for &(win, Rectangle(x, y, w, h)) in window_layout.iter() {
            window_system.resize_window(win, w - config.border_width * 2, h - config.border_width * 2);
            window_system.move_window(win, x, y);
            window_system.set_window_border_width(win, config.border_width);
        }

        debug!("Created window \"{}\"", window_system.get_window_name(window));
    }
}
