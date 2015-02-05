extern crate collections;

use core::RationalRect;
use core::Screen;
use core::Workspace;
use core::Workspaces;
use config::GeneralConfig;
use layout::LayoutMessage;
use window_system::Rectangle;
use window_system::Window;
use window_system::WindowSystem;

use std::rc::Rc;
use std::collections::BTreeMap;

pub type ScreenDetail = Rectangle;
pub type MouseDrag<'a> = Box<Fn(u32, u32, WindowManager<'a>, &WindowSystem) -> WindowManager<'a> + 'a>;

#[derive(Clone)]
pub struct WindowManager<'a> {
    pub running: bool,
    pub dragging: Option<Rc<MouseDrag<'a>>>,
    pub workspaces: Workspaces<'a>,
    pub waiting_unmap: BTreeMap<Window, Window>
}

impl<'a> WindowManager<'a> {
    /// Create a new window manager for the given window system and configuration
    pub fn new(window_system: &WindowSystem, config: &GeneralConfig<'a>) -> WindowManager<'a> {
        WindowManager {
            running: true,
            dragging: None,
            workspaces: Workspaces::new(config.layout.copy(),
                                        config.tags.clone(),
                                        window_system.get_screen_infos()),
            waiting_unmap: BTreeMap::new()
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
            debug!("switching to workspace {}", config.tags[index as usize].clone());
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
            r.current = sc.first().unwrap().clone();
            r.visible = sc.iter().skip(1).map(|x| x.clone()).collect();
            r
        })
    }

    /// Reapply the layout to the whole workspace.
    pub fn reapply_layout(&self, window_system: &WindowSystem,
                          _: &GeneralConfig<'a>) -> Vec<(Window, Rectangle)> {
        self.workspaces.screens().into_iter().map(|s| {
            let mut ms = s.clone();
            let vs : Vec<(Window, Rectangle)> = self.workspaces.view(s.workspace.id)
                .with(Vec::new(), |x| x.integrate()).into_iter()
                .filter(|x| self.workspaces.floating.contains_key(x))
                .map(|x| (x, WindowManager::scale_rational_rect(s.screen_detail, self.workspaces.floating[x])))
                .chain(ms.workspace.layout.apply_layout(window_system, ms.screen_detail,
                    &self.workspaces.view(ms.workspace.id).current.workspace.stack
                        .map_or(None, |x| x.filter(|w| !self.workspaces.floating.contains_key(w)))).into_iter())
                .collect();

            window_system.restack_windows(vs.iter().map(|x| x.0).collect());

            vs
        }).flat_map(|x| x.into_iter()).collect()
    }

    pub fn update_layouts(&self, window_system: &WindowSystem,
                          _: &GeneralConfig<'a>) -> WindowManager<'a> {
        let screens : Vec<Screen> = self.workspaces.screens().into_iter().map(|s| {
            let mut ms = s.clone();
            ms.workspace.layout.apply_layout(window_system, ms.screen_detail,
                    &self.workspaces.view(ms.workspace.id).current.workspace.stack
                        .map_or(None, |x| x.filter(|w| !self.workspaces.floating.contains_key(w))));
            ms
        }).collect();

        WindowManager {
            running: self.running,
            dragging: self.dragging.clone(),
            workspaces: self.workspaces.from_current(screens[0].clone()).from_visible(screens.into_iter().skip(1).collect()),
            waiting_unmap: self.waiting_unmap.clone()
        }
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
        fn adjust(RationalRect(x, y, w, h): RationalRect) -> RationalRect {
            if x + w > 1.0 || y + h > 1.0 || x < 0.0 || y < 0.0 {
                RationalRect(0.5 - w / 2.0, 0.5 - h / 2.0, w, h)
            } else {
                RationalRect(x, y, w, h)
            }
        }

        let size_hints = window_system.get_size_hints(window);

        let is_transient = false;
        let is_fixed_size = size_hints.min_size.is_some() && size_hints.min_size == size_hints.max_size;

        debug!("setting focus to newly managed window {}", window);

        let result = if is_transient || is_fixed_size {
            let r = adjust(self.float_location(window_system, window));
            self.windows(window_system, config, |x| x.insert_up(window).float(window, r))
                .focus(window, window_system, config)
        } else {
            self.windows(window_system, config, |x| x.insert_up(window))
                .focus(window, window_system, config)
        };

        debug!("focus is set to {}", window);

        result
    }

    /// Unmanage a window. This happens when a window is closed.
    pub fn unmanage(&self, window_system: &WindowSystem, window: Window,
                    config: &GeneralConfig<'a>) -> WindowManager<'a> {
        if self.workspaces.contains(window) {
            debug!("unmanaging window {}", window);
            self.windows(window_system, config, |x| x.delete(window))
        } else {
            self.clone()
        }
    }

    pub fn focus(&self, window: Window, window_system: &WindowSystem,
                 config: &GeneralConfig<'a>) -> WindowManager<'a> {
        if let Some(screen) = self.workspaces.find_screen(window) {
            if screen.screen_id == self.workspaces.current.screen_id &&
               screen.workspace.peek() != Some(window) {
                return self.windows(window_system, config, |w| w.focus_window(window))
            } else if window == window_system.get_root() {
                return self.windows(window_system, config, |w| w.view(screen.workspace.id))
            }
        }
        self.clone()
    }

    pub fn focus_down(&self) -> WindowManager<'a> {
        self.modify_workspaces(|x| x.focus_down())
    }

    pub fn focus_up(&self) -> WindowManager<'a> {
        self.modify_workspaces(|x| x.focus_up())
    }

    pub fn modify_workspaces<F>(&self, f: F) -> WindowManager<'a> where F : Fn(&Workspaces<'a>) -> Workspaces<'a> {
        WindowManager {
            running: self.running,
            dragging: self.dragging.clone(),
            workspaces: f(&self.workspaces),
            waiting_unmap: self.waiting_unmap.clone()
        }
    }

    pub fn reveal(&self, window_system: &WindowSystem, window: Window) -> WindowManager<'a> {
        window_system.show_window(window);
        self.clone()
    }

    pub fn windows<F>(&self, window_system: &WindowSystem, config: &GeneralConfig<'a>,
                   f: F) -> WindowManager<'a> where F : Fn(&Workspaces<'a>) -> Workspaces<'a> {
        let ws = f(&self.workspaces);

        // Collect all visible and new windows
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

        // Initialize all new windows
        for &window in new_windows.iter() {
            window_system.set_initial_properties(window);
            window_system.set_window_border_width(window, config.border_width);
        }

        // Apply the layout to the current workspace
        let modified = self.modify_workspaces(|x| f(x)).update_layouts(window_system, config);
        let result   = self.modify_workspaces(f).reapply_layout(window_system, config);

        old_visible.iter()
            .fold((),|_, &x| window_system.set_window_border_color(x, config.border_color.clone()));

        old_visible.iter()
            .chain(new_windows.iter())
            .filter(|&&x| !result.iter().any(|&(y, _)| x == y))
            .fold((), |_, &x| window_system.hide_window(x));

        for &(window, rect) in result.iter() {
            WindowManager::tile_window(window_system, config, window, rect);
        }

        //
        match modified.workspaces.peek() {
            Some(focused_window) => {
                window_system.set_window_border_color(focused_window, config.focus_border_color.clone());
                window_system.focus_window(focused_window, self);
            },
            None => window_system.focus_window(window_system.get_root(), self)
        }

        if config.focus_follows_mouse {
            window_system.remove_enter_events();
        }

        old_visible.iter()
            .chain(new_windows.iter())
            .filter(|&&x| !result.iter().any(|&(y, _)| x == y))
            .fold(modified, |a, &x| a.insert_or_update_unmap(x))
    }

    /// Send the given message to the current layout
    pub fn send_layout_message(&self, message: LayoutMessage, window_system: &WindowSystem,
                               config: &GeneralConfig) -> WindowManager<'a> {
        self.modify_workspaces(|w| w.send_layout_message(message, window_system, config))
    }

    /// Kill the currently focused window
    pub fn kill_window(&self, window_system: &WindowSystem) -> WindowManager<'a> {
        self.workspaces.with_focused(|w| window_system.kill_client(w));
        self.clone()
    }

    fn scale_rational_rect(Rectangle(sx, sy, sw, sh): Rectangle,
                           RationalRect(rx, ry, rw, rh): RationalRect) -> Rectangle {
        fn scale(s: u32, r: f32) -> u32 { ((s as f32) * r) as u32 }
        Rectangle(sx + scale(sw, rx), sy + scale(sh, ry), scale(sw, rw), scale(sh, rh))
    }

    fn tile_window(window_system: &WindowSystem, config: &GeneralConfig,
                   window: Window, Rectangle(x, y, w, h): Rectangle) {
        window_system.resize_window(window, w - config.border_width,
                                            h - config.border_width);
        window_system.move_window(window, x, y);
        window_system.show_window(window);
    }

    pub fn float_location(&self, window_system: &WindowSystem, window: Window) -> RationalRect {
        let Rectangle(_, _, sw, sh)   = self.workspaces.current.screen_detail;
        let Rectangle(rx, ry, rw, rh) = window_system.get_geometry(window);

        RationalRect(rx as f32 / sw as f32,
                     ry as f32 / sh as f32,
                     rw as f32 / sw as f32,
                     rh as f32 / sh as f32)
    }

    pub fn float(&self, window_system: &WindowSystem, config: &GeneralConfig<'a>,
                 window: Window) -> WindowManager<'a> {
        let rect = self.float_location(window_system, window);
        let result = self.windows(window_system, config, |w| w.float(window, rect));
        result
    }

    pub fn mouse_drag(&self, window_system: &WindowSystem, f: Box<Fn(u32, u32, WindowManager<'a>, &WindowSystem) -> WindowManager<'a> + 'a>) -> WindowManager<'a> {
        window_system.grab_pointer();

        let motion = Rc::new((box move |&: x, y, window_manager, w| {
            let z = f.call((x, y, window_manager, w));
            w.remove_motion_events();
            z
        }) as MouseDrag<'a>);

        WindowManager {
            running: self.running,
            dragging: Some(motion),
            workspaces: self.workspaces.clone(),
            waiting_unmap: self.waiting_unmap.clone()
        }
    }

    pub fn mouse_move_window(&self, window_system: &WindowSystem, config: &GeneralConfig<'a>,
                             window: Window) -> WindowManager<'a> {
        let (ox, oy) = window_system.get_pointer(window);
        let Rectangle(x, y, _, _) = window_system.get_geometry(window);

        self.mouse_drag(window_system, box move |&: ex: u32, ey: u32, m: WindowManager<'a>, w: &WindowSystem| {
            w.move_window(window, x + (ex - ox), y + (ey - oy));
            m.modify_workspaces(|wsp| wsp.update_floating_rect(window, m.float_location(w, window)))
        }).float(window_system, config, window)
    }

    pub fn mouse_resize_window(&self, window_system: &WindowSystem, config: &GeneralConfig<'a>,
                             window: Window) -> WindowManager<'a> {
        let Rectangle(x, y, w, h) = window_system.get_geometry(window);

        window_system.warp_pointer(window, w, h);
        debug!("warping to {}, {}", w, h);
        self.mouse_drag(window_system, box move |&: ex: u32, ey: u32, m: WindowManager<'a>, w: &WindowSystem| {
            w.resize_window(window, ex - x, ey - y);
            m.modify_workspaces(|wsp| wsp.update_floating_rect(window, m.float_location(w, window)))
        }).float(window_system, config, window)
    }

    pub fn is_waiting_unmap(&self, window: Window) -> bool {
        self.waiting_unmap.contains_key(&window)
    }

    pub fn update_unmap(&self, window: Window) -> WindowManager<'a> {
        if !self.waiting_unmap.contains_key(&window) {
            return self.clone();
        }

        let val = self.waiting_unmap[window];
        let mut new_map = self.waiting_unmap.clone();

        if val == 1 {
            new_map.remove(&window);
        } else {
            new_map[window] = val - 1;
        }

        WindowManager {
            running: self.running,
            dragging: self.dragging.clone(),
            workspaces: self.workspaces.clone(),
            waiting_unmap: new_map
        }
    }

    pub fn insert_or_update_unmap(&self, window: Window) -> WindowManager<'a> {
        let mut new_map = self.waiting_unmap.clone();

        if new_map.contains_key(&window) {
            new_map[window] += 1;
        } else {
            new_map.insert(window, 1);
        }

        WindowManager {
            running: self.running,
            dragging: self.dragging.clone(),
            workspaces: self.workspaces.clone(),
            waiting_unmap: new_map
        }
    }

    pub fn remove_from_unmap(&self, window: Window) -> WindowManager<'a> {
        let mut new_map = self.waiting_unmap.clone();
        if new_map.contains_key(&window) {
            new_map.remove(&window);
        }
        WindowManager {
            running: self.running,
            dragging: self.dragging.clone(),
            workspaces: self.workspaces.clone(),
            waiting_unmap: new_map
        }
    }
}
