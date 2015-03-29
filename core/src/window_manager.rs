extern crate collections;

use core::rational_rect::RationalRect;
use core::screen::Screen;
use core::workspace::Workspace;
use core::workspaces::Workspaces;
use config::GeneralConfig;
use layout::LayoutMessage;
use window_system::Rectangle;
use window_system::Window;
use window_system::WindowSystem;

use std::rc::Rc;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::cmp;

pub type ScreenDetail = Rectangle;
pub type MouseDrag = Box<Fn(u32, u32, &mut WindowManager, &WindowSystem)>;

#[derive(Clone)]
pub struct WindowManager {
    pub running: bool,
    pub dragging: Option<Rc<MouseDrag>>,
    pub workspaces: Workspaces,
    pub waiting_unmap: BTreeMap<Window, Window>
}

impl WindowManager {
    /// Create a new window manager for the given window system and configuration
    pub fn new(window_system: &WindowSystem, config: &GeneralConfig) -> WindowManager {
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
    pub fn view(&mut self, window_system: &WindowSystem, index: u32,
                config: &GeneralConfig) {
        if index < self.workspaces.number_workspaces() {
            debug!("switching to workspace {}", config.tags[index as usize].clone());
            self.windows(window_system, config, |w| w.view(index))
        } else {
        }
    }

    pub fn move_window_to_workspace(&mut self, window_system: &WindowSystem,
                                    config: &GeneralConfig,
                                    index: u32) {
        self.windows(window_system, config, |w| w.shift(index))
    }

    /// Rearrange the workspaces across the given screens.
    /// Needs to be called when the screen arrangement changes.
    pub fn rescreen(&mut self, window_system: &WindowSystem) {
        let screens = window_system.get_screen_infos();
        let visible : Vec<Workspace> = (vec!(self.workspaces.current.clone())).iter()
            .chain(self.workspaces.visible.iter())
            .map(|x| x.workspace.clone())
            .collect();
        let sc : Vec<Screen> = visible.iter()
            .chain(self.workspaces.hidden.iter())
            .take(screens.len())
            .map(|x| x.clone())
            .enumerate()
            .zip(screens.iter())
            .map(|((a, b), &c)| Screen::new(b.clone(), a as u32, c))
            .collect();

        self.modify_workspaces(|w: &Workspaces| {
            let mut r = w.clone();
            r.current = sc.first().unwrap().clone();
            r.visible = sc.iter().skip(1).map(|x| x.clone()).collect();
            r
        })
    }

    pub fn update_layouts(&mut self, window_system: &WindowSystem,
                          config: &GeneralConfig) {
        let screens : Vec<Screen> = self.workspaces.screens().into_iter().map(|s| {
            let mut ms = s.clone();
            ms.workspace.layout.apply_layout(window_system, ms.screen_detail, config,
                    &self.workspaces.view(ms.workspace.id).current.workspace.stack
                        .map_or(None, |x| x.filter(|w| !self.workspaces.floating.contains_key(w))));
            ms
        }).collect();

        self.workspaces = self.workspaces.from_current(screens[0].clone()).from_visible(screens.into_iter().skip(1).collect());
    }

    pub fn unfocus_windows(&self, window_system: &WindowSystem, config: &GeneralConfig) {
        for &win in self.workspaces.visible_windows().iter() {
            window_system.set_window_border_color(win, config.border_color);
        }
    }

    /// Manage a new window that was either created just now or already present
    /// when the WM started.
    pub fn manage(&mut self, window_system: &WindowSystem, window: Window,
                  config: &GeneralConfig) {
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
            self.windows(window_system, config, |x| x.insert_up(window).float(window, r));
            self.focus(window, window_system, config)
        } else {
            self.windows(window_system, config, |x| x.insert_up(window));
            self.focus(window, window_system, config)
        };

        debug!("focus is set to {}", window);

        result
    }

    /// Unmanage a window. This happens when a window is closed.
    pub fn unmanage(&mut self, window_system: &WindowSystem, window: Window,
                    config: &GeneralConfig) {
        if self.workspaces.contains(window) {
            debug!("unmanaging window {}", window);
            self.windows(window_system, config, |x| x.delete(window))
        } else {
        }
    }

    pub fn focus(&mut self, window: Window, window_system: &WindowSystem,
                 config: &GeneralConfig) {
        if let Some(screen) = self.workspaces.find_screen(window) {
            if screen.screen_id == self.workspaces.current.screen_id &&
               screen.workspace.peek() != Some(window) {
                return self.windows(window_system, config, |w| w.focus_window(window))
            } else if window == window_system.get_root() {
                return self.windows(window_system, config, |w| w.view(screen.workspace.id))
            }
        }
    }

    pub fn focus_down(&mut self) {
        self.modify_workspaces(|x| x.focus_down())
    }

    pub fn focus_up(&mut self) {
        self.modify_workspaces(|x| x.focus_up())
    }

    pub fn modify_workspaces<F>(&mut self, f: F) where F : Fn(&Workspaces) -> Workspaces {
        self.workspaces = f(&self.workspaces);
    }

    pub fn reveal(&mut self, window_system: &WindowSystem, window: Window) {
        window_system.show_window(window);
    }

    pub fn windows<F>(&mut self, window_system: &WindowSystem,
                      config: &GeneralConfig, f: F)
            where F : Fn(&Workspaces) -> Workspaces {
        let ws = f(&self.workspaces);
        let old_visible = self.workspaces.visible_windows().into_iter().collect::<BTreeSet<_>>();
        let new_windows = ws.visible_windows().into_iter().collect::<BTreeSet<_>>()
            .difference(&old_visible).map(|&x| x).collect::<Vec<Window>>();

        // Initialize all new windows
        for &window in new_windows.iter() {
            window_system.set_initial_properties(window);
            window_system.set_window_border_width(window, config.border_width);
        }

        let all_screens = ws.screens();
        let summed_visible = (vec!(BTreeSet::new())).into_iter().chain(all_screens.iter().scan(Vec::new(), |acc, x| {
            acc.extend(x.workspace.windows().into_iter());
            Some(acc.clone())
        })
        .map(|x| x.into_iter().collect::<BTreeSet<_>>()))
        .collect::<Vec<_>>();
        error!("{:?}", summed_visible);

        //let rects = all_screens.iter().flat_map(|w| {
        let rects = all_screens.iter().zip(summed_visible.iter()).flat_map(|(w, vis)| {
            let mut wsp = w.workspace.clone();
            let this = ws.view(wsp.id);
            let tiled = this.clone().current.workspace.stack.map_or(None, |x|
                                                      x.filter(|win| !ws.floating.contains_key(win))).map_or(None, |x|
                                                      x .filter(|win| !vis.contains(win)));
                                                               //&& !vis.contains(win)));
            let view_rect = w.screen_detail;

            let rs = wsp.layout.apply_layout(window_system, view_rect, config, &tiled);

            let flt = this.with(Vec::new(), |x| x.integrate()).into_iter()
                .filter(|x| self.workspaces.floating.contains_key(x))
                .map(|x| (x, WindowManager::scale_rational_rect(view_rect, self.workspaces.floating[&x])))
                .collect::<Vec<_>>();

            let vs : Vec<(Window, Rectangle)> = flt.into_iter().chain(rs.into_iter()).collect();
            window_system.restack_windows(vs.iter().map(|x| x.0).collect());

            vs.into_iter()
        }).collect::<Vec<_>>();

        let visible = rects.iter().map(|x| x.0).collect::<Vec<_>>();

        for &(window, rect) in rects.iter() {
            WindowManager::tile_window(window_system, config, window, rect);
        }

        visible.iter().fold((),|_, &x| window_system.set_window_border_color(x, config.border_color.clone()));

        for &win in visible.iter() {
            window_system.show_window(win);
        }

        match ws.peek() {
            Some(focused_window) => {
                window_system.set_window_border_color(focused_window, config.focus_border_color.clone());
                window_system.focus_window(focused_window, self);
            },
            None => window_system.focus_window(window_system.get_root(), self)
        }

        let to_hide = (old_visible.union(&new_windows.into_iter().collect::<BTreeSet<_>>())).map(|&x| x).collect::<BTreeSet<_>>()
                                  .difference(&visible.into_iter().collect::<BTreeSet<_>>()).map(|&x| x).collect::<Vec<_>>();

        for &win in to_hide.iter() {
            window_system.hide_window(win);
        }

        if config.focus_follows_mouse {
            window_system.remove_enter_events();
        }

        self.modify_workspaces(|_| ws.clone());
        self.update_layouts(window_system, config);

        for x in to_hide {
            self.insert_or_update_unmap(x);
        }
    }

    /// Send the given message to the current layout
    pub fn send_layout_message(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                               config: &GeneralConfig) {
        self.modify_workspaces(|w| w.send_layout_message(message, window_system, config))
    }

    /// Kill the currently focused window
    pub fn kill_window(&mut self, window_system: &WindowSystem) {
        self.workspaces.with_focused(|w| window_system.kill_client(w));
    }

    fn scale_rational_rect(Rectangle(sx, sy, sw, sh): Rectangle,
                           RationalRect(rx, ry, rw, rh): RationalRect) -> Rectangle {
        fn scale(s: u32, r: f32) -> u32 { ((s as f32) * r) as u32 }
        Rectangle(sx + scale(sw, rx) as i32, sy + scale(sh, ry) as i32, scale(sw, rw), scale(sh, rh))
    }

    fn tile_window(window_system: &WindowSystem, config: &GeneralConfig,
                   window: Window, Rectangle(x, y, w, h): Rectangle) {
        window_system.resize_window(window, w - 2 * config.border_width,
                                            h - 2 * config.border_width);
        window_system.move_window(window, x, y);
        window_system.show_window(window);
    }

    pub fn float_location(&self, window_system: &WindowSystem, window: Window) -> RationalRect {
        let Rectangle(sx, sy, sw, sh)   = self.workspaces.current.screen_detail;
        let Rectangle(rx, ry, rw, rh) = window_system.get_geometry(window);

        RationalRect((rx as f32 - sx as f32) / sw as f32,
                     (ry as f32 - sy as f32) / sh as f32,
                      rw as f32 / sw as f32,
                      rh as f32 / sh as f32)
    }

    pub fn float(&mut self, window_system: &WindowSystem, config: &GeneralConfig, window: Window) {
        let rect = self.float_location(window_system, window);
        self.windows(window_system, config, |w| w.float(window, rect));
    }

    pub fn mouse_drag(&mut self, window_system: &WindowSystem,
                      f: Box<Fn(u32, u32, &mut WindowManager, &WindowSystem)>) {
        window_system.grab_pointer();

        let motion = Rc::new((box move |x, y, window_manager, w| {
            f(x, y, window_manager, w);
            w.remove_motion_events();
        }) as MouseDrag);

        self.dragging = Some(motion);
    }

    pub fn mouse_move_window(&mut self, window_system: &WindowSystem,
                             config: &GeneralConfig, window: Window) {
        let (ox, oy) = window_system.get_pointer(window);
        let Rectangle(x, y, _, _) = window_system.get_geometry(window);

        self.mouse_drag(window_system, box move |ex: u32, ey: u32, m: &mut WindowManager, w: &WindowSystem| {
            w.move_window(window, x + (ex as i32 - ox as i32), y + (ey as i32 - oy as i32));
            let location = m.float_location(w, window);
            m.modify_workspaces(|wsp| wsp.update_floating_rect(window, location));
        });
        self.float(window_system, config, window)
    }

    pub fn mouse_resize_window(&mut self, window_system: &WindowSystem, config: &GeneralConfig,
                             window: Window) {
        let Rectangle(x, y, w, h) = window_system.get_geometry(window);

        window_system.warp_pointer(window, w, h);
        debug!("warping to {}, {}", w, h);
        self.mouse_drag(window_system, box move |ex: u32, ey: u32, m: &mut WindowManager, w: &WindowSystem| {
            debug!("resizing to {}x{} at ({}, {})", ex - x as u32, ey - y as u32, x, y);
            let nx = cmp::max(0, ex as i32 - x) as u32;
            let ny = cmp::max(0, ey as i32 - y) as u32;
            w.resize_window(window, nx, ny);
            let location = m.float_location(w, window);
            m.modify_workspaces(|wsp| wsp.update_floating_rect(window, location));
        });
        self.float(window_system, config, window)
    }

    pub fn is_waiting_unmap(&self, window: Window) -> bool {
        self.waiting_unmap.contains_key(&window)
    }

    pub fn update_unmap(&mut self, window: Window) {
        let remove = match self.waiting_unmap.get_mut(&window) {
            Some(ref val) if **val < 2 => true,
            Some(val) => { *val -= 1; false },
            None => false
        };
        if remove { self.waiting_unmap.remove(&window); }
    }

    pub fn insert_or_update_unmap(&mut self, window: Window) {
        let insert = match self.waiting_unmap.get_mut(&window) {
            Some(val) => { *val += 1; false },
            None => true
        };
        if insert { self.waiting_unmap.insert(window, 1); }
    }

    pub fn remove_from_unmap(&mut self, window: Window) {
        self.waiting_unmap.remove(&window);
    }
}
