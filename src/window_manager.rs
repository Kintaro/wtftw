use core::Screen;
use core::Workspace;
use core::Workspaces;
use config::Config;
use layout::LayoutManager;
use window_system::Rectangle;
use window_system::Window;
use window_system::WindowSystem;

pub type ScreenDetail = Rectangle;

#[deriving(Clone)]
pub struct WindowManager {
    workspaces: Workspaces
}

impl WindowManager {
    /// Create a new window manager for the given window system and configuration
    pub fn new(window_system: &WindowSystem, config: &Config) -> WindowManager {
        WindowManager {
            workspaces: Workspaces::new(String::from_str("Tall"),
                                        config.tags.clone(),
                                        window_system.get_screen_infos())
        }
    }

    /// Checks if the given window is already managed by the WindowManager
    pub fn is_window_managed(&self, window: Window) -> bool {
        self.workspaces.contains(window)
    }

    /// Switch to the workspace given by index. If index is out of bounds, 
    /// just do nothing and return.
    /// Then, reapply the layout to show the changes.
    pub fn view(&self, window_system: &WindowSystem, index: u32, config: &Config) -> WindowManager {
        if index < self.workspaces.number_workspaces() {
            debug!("switching to workspace {}", config.tags[index as uint].clone());
            self.windows(window_system, config, |w| w.view(index)).reapply_layout(window_system, config)
        } else {
            self.clone()
        }
    }

    pub fn move_window_to_workspace(&self, window_system: &WindowSystem, 
                                    config: &Config, index: u32) -> WindowManager {
        self.windows(window_system, config, |w| w.shift(index))
            .reapply_layout(window_system, config)
    }

    /// Rearrange the workspaces across the given screens.
    /// Needs to be called when the screen arrangement changes.
    pub fn rescreen(&mut self, window_system: &WindowSystem) {
        let screens = window_system.get_screen_infos();
        let visible : Vec<Workspace> = (vec!(self.workspaces.current.clone())).iter()
            .chain(self.workspaces.visible.iter())
            .map(|x| x.workspace.clone())
            .collect();
        let ws : Vec<Workspace> = visible.iter()
            .chain(self.workspaces.hidden.iter())
            .map(|x| x.clone())
            .collect();

        let xs : Vec<Workspace> = ws.iter().take(screens.len()).map(|x| x.clone()).collect();
        let ys : Vec<Workspace> = ws.iter().skip(screens.len()).map(|x| x.clone()).collect();

        let sc : Vec<Screen> = xs.iter()
            .enumerate()
            .zip(screens.iter())
            .map(|((a, b), &c)| Screen::new(b.clone(), a as u32, c))
            .collect();

        self.workspaces.current = sc.head().unwrap().clone();
        self.workspaces.visible = sc.iter().skip(1).map(|x| x.clone()).collect();
    }

    /// Reapply the layout to the whole workspace.
    pub fn reapply_layout(&self, window_system: &WindowSystem, config: &Config) -> WindowManager {
        let screen = &self.workspaces.current;
        let workspace = &screen.workspace;
        let layout = LayoutManager::get_layout(workspace.layout.clone());

        let Rectangle(x, y, w, h) = screen.screen_detail;
        let screen_space = Rectangle(x, y + 20, w, h - 20);

        let window_layout = layout.apply_layout(screen_space, &workspace.stack); 

        debug!("reapplying layout to {} screen", screen.screen_detail);
        
        // First, hide all the windows that are marked as hidden now,
        // by unmapping them from the server.
        for workspace in self.workspaces.hidden.iter() {
            match workspace.stack {
                Some(ref s) => {
                    for &win in s.integrate().iter() {
                        window_system.hide_window(win);
                    }
                }
                _ => ()
            }
        }

        // Then, show, place and resize all now visible windows.
        for &(win, Rectangle(x, y, w, h)) in window_layout.iter() {
            debug!("Show window {} ({})", win, window_system.get_window_name(win));
            window_system.show_window(win);
            window_system.resize_window(win, w - config.border_width * 2, h - config.border_width * 2);
            window_system.move_window(win, x, y);
            window_system.set_window_border_width(win, config.border_width);
        }
        
        // Force a redraw on all windows.
        window_system.flush();

        self.clone()
    }

    /// Manage a new window that was either created just now or already present
    /// when the WM started.
    pub fn manage(&mut self, window_system: &WindowSystem, window: Window, config: &Config) {
        self.workspaces.current.workspace.add(window);
        self.reapply_layout(window_system, config);   
        self.windows(window_system, config, |x| x.clone());
        debug!("managing window \"{}\" ({})", window_system.get_window_name(window), window);
    }

    /// Unmanage a window. This happens when a window is closed.
    pub fn unmanage(&mut self, window_system: &WindowSystem, window: Window, config: &Config) {
        if self.workspaces.contains(window) {
            debug!("unmanaging window {}", window);
            self.workspaces = self.workspaces.delete(window);
            self.reapply_layout(window_system, config);
            self.windows(window_system, config, |x| x.clone());
        }
    }

    pub fn focus_down(&self) -> WindowManager {
        let mut w = self.clone();
        w.workspaces = self.workspaces.focus_down();
        w
    }

    pub fn focus_up(&self) -> WindowManager {
        let mut w = self.clone();
        w.workspaces = self.workspaces.focus_up();
        w
    }

    pub fn modify_workspaces(&self, f: |&Workspaces| -> Workspaces) -> WindowManager {
        let mut w = self.clone();
        w.workspaces = f(&self.workspaces);
        w
    }

    pub fn windows(&self, window_system: &WindowSystem, config: &Config, 
                   f: |&Workspaces| -> Workspaces) -> WindowManager {
        let old_visible_vecs : Vec<Vec<Window>> = (vec!(self.workspaces.current.clone())).iter()
            .chain(self.workspaces.visible.iter())
            .filter_map(|x| x.workspace.stack.clone())
            .map(|x| x.integrate())
            .collect();
        let old_visible : Vec<Window> = old_visible_vecs.iter()
            .flat_map(|x| x.iter())
            .map(|x| x.clone())
            .collect();

        let result = self.modify_workspaces(f); 

        old_visible.iter().fold((), 
            |_, &x| window_system.set_window_border_color(x, config.border_color.clone()));

        match result.workspaces.peek() {
            Some(focused_window) => {
                window_system.set_window_border_color(focused_window, config.focus_border_color.clone());
                window_system.focus_window(focused_window);
            },
            None => ()
        }

        result
    }
}
