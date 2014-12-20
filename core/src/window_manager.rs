extern crate collections;

use self::collections::TreeSet;

use core::RationalRect;
use core::Screen;
use core::Workspace;
use core::Workspaces;
use config::GeneralConfig;
use layout::LayoutMessage;
use window_system::Rectangle;
use window_system::Window;
use window_system::WindowSystem;

pub type ScreenDetail = Rectangle;

#[deriving(Clone)]
pub struct WindowManager<'a> {
    pub running: bool,
    pub workspaces: Workspaces<'a>
}

impl<'a> WindowManager<'a> {
    /// Create a new window manager for the given window system and configuration
    pub fn new(window_system: &WindowSystem, config: &GeneralConfig<'a>) -> WindowManager<'a> {
        WindowManager {
            running: true,
            workspaces: Workspaces::new(config.layout.copy(),
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
    pub fn view(&self, window_system: &WindowSystem, index: u32,
                config: &GeneralConfig<'a>) -> WindowManager<'a> {
        if index < self.workspaces.number_workspaces() {
            debug!("switching to workspace {}", config.tags[index as uint].clone());
            self.windows(window_system, config, |w| w.view(index))
        } else {
            self.clone()
        }
    }

    pub fn move_window_to_workspace(&self, window_system: &WindowSystem,
                                    config: &GeneralConfig<'a>,
                                    index: u32) -> WindowManager<'a> {
        self.windows(window_system, config, |w| w.shift(index))
    }

    /// Rearrange the workspaces across the given screens.
    /// Needs to be called when the screen arrangement changes.
    pub fn rescreen(&self, window_system: &WindowSystem) -> WindowManager<'a> {
        let screens = window_system.get_screen_infos();
        let visible : Vec<Workspace<'a>> = (vec!(self.workspaces.current.clone())).iter()
            .chain(self.workspaces.visible.iter())
            .map(|x| x.workspace.clone())
            .collect();
        let ws : Vec<Workspace<'a>> = visible.iter()
            .chain(self.workspaces.hidden.iter())
            .map(|x| x.clone())
            .collect();

        let xs : Vec<Workspace<'a>> = ws.iter()
            .take(screens.len()).map(|x| x.clone())
            .collect();

        let sc : Vec<Screen<'a>> = xs.iter()
            .enumerate()
            .zip(screens.iter())
            .map(|((a, b), &c)| Screen::new(b.clone(), a as u32, c))
            .collect();

        self.modify_workspaces(|w: &Workspaces<'a>| {
            let mut r = w.clone();
            r.current = sc.head().unwrap().clone();
            r.visible = sc.iter().skip(1).map(|x| x.clone()).collect();
            r
        })
    }

    /// Reapply the layout to the whole workspace.
    pub fn reapply_layout(&self, window_system: &WindowSystem,
                          config: &GeneralConfig<'a>) -> WindowManager<'a> {
        for screen in self.workspaces.screens().iter() {
            let workspace = &screen.workspace;

            let Rectangle(x, y, w, h) = screen.screen_detail;
            let screen_space = Rectangle(x, y, w, h);

            let window_layout = workspace.layout.apply_layout(window_system,
                                                              screen_space,
                                                              &workspace.stack);
            let windows_only : TreeSet<Window> = window_layout.iter().map(|&(w, _)| w).collect();

            debug!("reapplying layout to {} screen", screen.screen_detail);

            for w in workspace.windows().into_iter().filter(|w| !windows_only.contains(w)) {
                window_system.hide_window(w);
            }

            // First, hide all the windows that are marked as hidden now,
            // by unmapping them from the server.
            for workspace in self.workspaces.hidden.iter() {
                match workspace.stack {
                    Some(ref s) => {
                        for win in s.integrate().into_iter() {
                            window_system.hide_window(win);
                        }
                    }
                    _ => ()
                }
            }

            // Then, show, place and resize all now visible windows.
            for &(win, Rectangle(x, y, w, h)) in window_layout.iter() {
                window_system.show_window(win);
                window_system.resize_window(win,
                                            w - config.border_width * 2,
                                            h - config.border_width * 2);
                window_system.move_window(win, x, y);
                window_system.set_window_border_width(win, config.border_width);
                window_system.set_window_border_color(win, config.border_color);
            }

            let to_restack : Vec<Window>  = window_layout.iter().map(|&(win, _)| win).chain(
                self.workspaces.floating.iter().map(|(&win, _)| win)).collect();

            window_system.restack_windows(to_restack);
        }

        if let Some(focused_window) = self.workspaces.peek() {
            window_system.set_window_border_color(focused_window, config.focus_border_color);
        }

        // Force a redraw on all windows.
        window_system.flush();

        self.clone()
    }

    pub fn unfocus_windows(&self, window_system: &WindowSystem, config: &GeneralConfig<'a>) {
        for &win in self.workspaces.visible_windows().iter() {
            window_system.set_window_border_color(win, config.border_color);
        }
    }

    /// Manage a new window that was either created just now or already present
    /// when the WM started.
    pub fn manage(&self, window_system: &WindowSystem, window: Window,
                  config: &GeneralConfig<'a>) -> WindowManager<'a> {
        // TODO: manage floating windows
        // and ensure that they stay within screen boundaries
        debug!("managing window {}", window_system.get_window_name(window));

        let size_hints = window_system.get_size_hints(window);

        let is_transient = false;
        let is_fixed_size = size_hints.min_size.is_some() && size_hints.min_size == size_hints.max_size;
        let r = RationalRect(0.0, 0.0, 200.0, 200.0);

        if is_transient || is_fixed_size {
            let i = self.workspaces.current.workspace.id;
            self.windows(window_system, config, |x| x.view(i).insert_up(window).float(window, r))
                .focus(window, window_system, config)
        } else {
            self.windows(window_system, config, |x| x.insert_up(window))
                .focus(window, window_system, config)
        }
    }

    /// Unmanage a window. This happens when a window is closed.
    pub fn unmanage(&self, window_system: &WindowSystem, window: Window,
                    config: &GeneralConfig<'a>) -> WindowManager<'a> {
        if self.workspaces.contains(window) {
            debug!("unmanaging window {}", window);
            self.windows(window_system, config, |x| x.delete(window)).reapply_layout(window_system, config)
        } else {
            self.clone()
        }
    }

    pub fn focus(&self, window: Window, window_system: &WindowSystem,
                 config: &GeneralConfig<'a>) -> WindowManager<'a> {
        let screen = self.workspaces.find_screen(window);

        if screen.screen_id == self.workspaces.current.screen_id && screen.workspace.peek() != Some(window) {
            self.windows(window_system, config, |w| w.focus_window(window))
        } else if window == window_system.get_root() {
            self.windows(window_system, config, |w| w.view(screen.workspace.id))
        } else {
            self.clone()
        }
    }

    pub fn focus_down(&self) -> WindowManager<'a> {
        self.modify_workspaces(|x| x.focus_down())
    }

    pub fn focus_up(&self) -> WindowManager<'a> {
        self.modify_workspaces(|x| x.focus_up())
    }

    pub fn modify_workspaces(&self, f: |&Workspaces<'a>| -> Workspaces<'a>) -> WindowManager<'a> {
        WindowManager { running: self.running, workspaces: f(&self.workspaces) }
    }

    pub fn reveal(&self, window_system: &WindowSystem, window: Window) -> WindowManager<'a> {
        window_system.show_window(window);
        self.clone()
    }

    pub fn windows(&self, window_system: &WindowSystem, config: &GeneralConfig<'a>,
                   f: |&Workspaces<'a>| -> Workspaces<'a>) -> WindowManager<'a> {
        let ws = f(&self.workspaces);

        let old_visible_vecs : Vec<Vec<Window>> = (vec!(self.workspaces.current.clone())).iter()
            .chain(self.workspaces.visible.iter())
            .filter_map(|x| x.workspace.stack.clone())
            .map(|x| x.integrate())
            .collect();

        let new_windows : Vec<Window> = ws.all_windows().iter()
            .filter(|&x| !self.workspaces.all_windows().contains(x))
            .map(|x| x.clone())
            .collect();

        let old_visible : Vec<Window> = old_visible_vecs.iter()
            .flat_map(|x| x.iter())
            .map(|x| x.clone())
            .collect();

        for &window in new_windows.iter() {
            window_system.set_initial_properties(window);
        }

        match self.workspaces.peek() {
            Some(win) => window_system.set_window_border_color(win, config.border_color.clone()),
            _         => ()
        }

        let result = self.modify_workspaces(f).reapply_layout(window_system, config);

        old_visible.iter().fold((),
            |_, &x| window_system.set_window_border_color(x, config.border_color.clone()));

        match result.workspaces.peek() {
            Some(focused_window) => {
                window_system.set_window_border_color(focused_window,
                                                      config.focus_border_color.clone());
                window_system.focus_window(focused_window, self);
            },
            None => window_system.focus_window(window_system.get_root(), self)
        }

        if config.focus_follows_mouse {
            window_system.remove_enter_events();
        }

        result
    }

    pub fn send_layout_message(&self, message: LayoutMessage) -> WindowManager<'a> {
        self.modify_workspaces(|w| w.send_layout_message(message))
    }

    pub fn kill_window(&self, window_system: &WindowSystem) -> WindowManager<'a> {
        self.workspaces.with_focused(|w| window_system.kill_client(w));
        self.clone()
    }
}
