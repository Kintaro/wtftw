use std::collections::BTreeMap;
use std::iter::repeat;
use config::GeneralConfig;
use window_manager::ScreenDetail;
use window_system::{ Window, WindowSystem };
use layout::{Layout, LayoutMessage};
use core::screen::Screen;
use core::rational_rect::RationalRect;
use core::workspace::Workspace;
use core::stack::Stack;

pub struct Workspaces {
    /// The currently focused and visible screen
    pub current:  Screen,
    /// The other visible, but non-focused screens
    pub visible:  Vec<Screen>,
    /// All remaining workspaces that are currently hidden
    pub hidden:   Vec<Workspace>,
    /// A list of all floating windows
    pub floating: BTreeMap<Window, RationalRect>
}

impl Clone for Workspaces {
    fn clone(&self) -> Workspaces {
        Workspaces {
            current: self.current.clone(),
            visible: self.visible.clone(),
            hidden:  self.hidden.clone(),
            floating: self.floating.clone()
        }
    }
}

impl Workspaces {
    /// Create a new stackset, of empty stacks, with given tags,
    /// with physical screens whose descriptions are given by 'm'. The
    /// number of physical screens (@length 'm'@) should be less than or
    /// equal to the number of workspace tags.  The first workspace in the
    /// list will be current.
    ///
    /// Xinerama: Virtual workspaces are assigned to physical screens, starting at 0.
    pub fn new(layout: Box<dyn Layout>, tags: Vec<String>, screens: Vec<ScreenDetail>) -> Workspaces {
        debug!("creating new workspaces with {} screen(s)", screens.len());
        let workspaces : Vec<Workspace> = tags.iter()
            .enumerate()
            .map(|(id, tag)| Workspace::new(id as u32, tag.clone(), layout.copy(), None))
            .collect();
        let seen   : Vec<Workspace> = workspaces.iter()
            .take(screens.len())
            .map(|x| x.clone())
            .collect();
        let unseen : Vec<Workspace> = workspaces.iter()
            .skip(screens.len())
            .map(|x| x.clone())
            .collect();
        let current : Vec<Screen> = seen.iter()
            .enumerate()
            .zip(screens.iter())
            .map(|((a, b), c)| Screen::new(b.clone(), a as u32, c.clone()))
            .collect();

        Workspaces {
            current: current[0].clone(),
            visible: current.iter().skip(1).map(|x| x.clone()).collect(),
            hidden: unseen,
            floating: BTreeMap::new()
        }
    }

    pub fn from_current(&self, current: Screen) -> Workspaces {
        Workspaces {
            current: current,
            visible: self.visible.clone(),
            hidden: self.hidden.clone(),
            floating: self.floating.clone()
        }
    }

    pub fn from_visible(&self, visible: Vec<Screen>) -> Workspaces {
        Workspaces {
            current: self.current.clone(),
            visible: visible,
            hidden: self.hidden.clone(),
            floating: self.floating.clone()
        }
    }

    pub fn from_hidden(&self, hidden: Vec<Workspace>) -> Workspaces {
        Workspaces {
            current: self.current.clone(),
            visible: self.visible.clone(),
            hidden: hidden,
            floating: self.floating.clone()
        }
    }

    /// Set focus to the workspace with index \'i\'.
    /// If the index is out of range, return the original 'StackSet'.
    ///
    /// Xinerama: If the workspace is not visible on any Xinerama screen, it
    /// becomes the current screen. If it is in the visible list, it becomes
    /// current.
    pub fn view(&self, index: u32) -> Workspaces {
        debug!("setting focus to {}", index);

        // We are already on the desired workspace. Do nothing
        if self.current.workspace.id == index {
            return self.clone();
        }

        // Desired workspace is visible, switch to it by raising
        // it to current and pushing the current one to visible
        if let Some(screen_pos) = self.visible.iter().position(|s| s.workspace.id == index) {
            let screen = self.visible[screen_pos].clone();
            let visible : Vec<Screen> = self.visible.iter()
                .enumerate()
                .filter(|&(x, _)| x != screen_pos)
                .map(|(_, y)| y.clone())
                .collect();

            self.from_visible(visible.into_iter().chain((vec!(self.current.clone())).into_iter()).collect())
                .from_current(screen)
        // Desired workspace is hidden. Switch it with the current workspace
        } else if let Some(workspace_pos) = self.hidden.iter().position(|w| w.id == index) {
            let hidden : Vec<Workspace> = self.hidden.iter()
                .enumerate()
                .filter(|&(x, _)| x != workspace_pos)
                .map(|(_, y)| y.clone())
                .collect();

            self.from_hidden(hidden.clone().into_iter().chain((vec!(self.current.workspace.clone())).into_iter()).collect())
                .from_current(self.current.map_workspace(|_| self.hidden[workspace_pos].clone()))
        } else {
            self.clone()
        }
    }

    /// Set focus to the given workspace.  If that workspace does not exist
    /// in the stackset, the original workspace is returned.  If that workspace is
    /// 'hidden', then display that workspace on the current screen, and move the
    /// current workspace to 'hidden'.  If that workspace is 'visible' on another
    /// screen, the workspaces of the current screen and the other screen are
    /// swapped.
    pub fn greedy_view(&self, index: u32) -> Workspaces {
        if self.hidden.iter().any(|x| x.id == index) {
            self.view(index)
        } else if let Some(s) = self.visible.iter().find(|x| x.workspace.id == index) {
            let mut current_screen = self.current.clone();
            let old_workspace = current_screen.workspace.clone();
            let mut screen_with_requested_workspace = s.clone();
            let desired_workspace = screen_with_requested_workspace.workspace.clone();

            current_screen.workspace = desired_workspace;
            screen_with_requested_workspace.workspace = old_workspace;

            self.from_current(current_screen.clone())
                .from_visible(self.visible.iter()
                              .filter(|x| x.screen_id != screen_with_requested_workspace.screen_id)
                              .map(|x| x.clone())
                              .chain((vec!(screen_with_requested_workspace.clone())).into_iter())
                              .collect())

        } else {
            self.clone()
        }
    }

    pub fn float(&self, window: Window, rect: RationalRect) -> Workspaces {
        let mut w = self.clone();
        w.floating.insert(window, rect);
        w
    }

    pub fn sink(&self, window: Window) -> Workspaces {
        let mut w = self.clone();
        w.floating.remove(&window);
        w
    }

    pub fn delete(&self, window: Window) -> Workspaces {
        self.delete_p(window).sink(window)
    }

    pub fn delete_p(&self, window: Window) -> Workspaces {
        fn remove_from_workspace(stack: Stack<Window>, window: Window) -> Option<Stack<Window>> {
            stack.filter(|&x| x != window)
        }

        self.modify_stack_option(|x| remove_from_workspace(x, window))
            .modify_hidden_option(|x| remove_from_workspace(x, window))
            .modify_visible_option(|x| remove_from_workspace(x, window))
    }

    pub fn focus_window(&self, window: Window) -> Workspaces {
        if self.peek() == Some(window) {
            return self.clone();
        }

        match self.find_tag(window) {
            Some(tag) => {
                let mut s = self.view(tag);
                while s.peek() != Some(window) {
                    s = s.focus_up();
                }
                s
            },
            _ => self.clone()
        }
    }

    /// Move the focus of the currently focused workspace down
    pub fn focus_down(&self) -> Workspaces {
        self.modify_stack(|x| x.focus_down())
    }

    pub fn focus_up(&self) -> Workspaces {
        self.modify_stack(|x| x.focus_up())
    }

    pub fn swap_down(&self) -> Workspaces {
        self.modify_stack(|x| x.swap_down())
    }

    pub fn swap_up(&self) -> Workspaces {
        self.modify_stack(|x| x.swap_up())
    }

    pub fn swap_master(&self) -> Workspaces {
        self.modify_stack(|x| x.swap_master())
    }

    pub fn modify_stack<F>(&self, f: F) -> Workspaces
            where F : Fn(Stack<Window>) -> Stack<Window> {
        self.from_current(self.current.map(|s| f(s)))
    }

    pub fn modify_stack_option<F>(&self, f: F) -> Workspaces
            where F : Fn(Stack<Window>) -> Option<Stack<Window>> {
        self.from_current(self.current.map_option(|s| f(s)))
    }

    pub fn modify_hidden<F>(&self, f: F) -> Workspaces
            where F : Fn(Stack<Window>) -> Stack<Window> {
        self.from_hidden(self.hidden.iter().map(|x| x.map(|s| f(s))).collect())
    }

    pub fn modify_hidden_option<F>(&self, f: F) -> Workspaces
            where F : Fn(Stack<Window>) -> Option<Stack<Window>> {
        self.from_hidden(self.hidden.iter().map(|x| x.map_option(|s| f(s))).collect())
    }


    pub fn modify_visible<F>(&self, f: F) -> Workspaces
            where F : Fn(Stack<Window>) -> Stack<Window> {
        self.from_visible(self.visible.iter().map(|x| x.map(|s| f(s))).collect())
    }

    pub fn modify_visible_option<F>(&self, f: F) -> Workspaces
            where F : Fn(Stack<Window>) -> Option<Stack<Window>> {
        self.from_visible(self.visible.iter().map(|x| x.map_option(|s| f(s))).collect())
    }

    pub fn get_focus_window(&self) -> Option<Window> {
        self.current.workspace.stack.clone().map(|s| s.focus)
    }

    /// Retrieve the currently focused workspace's
    /// focus element. If there is none, return None.
    pub fn peek(&self) -> Option<Window> {
        self.with(None, |s| Some(s.focus))
    }

    /// Apply the given function to the currently focused stack
    /// or return a default if the stack is empty
    pub fn with<T, F>(&self, default: T, f: F) -> T
            where F : Fn(&Stack<Window>) -> T {
        self.clone().current.workspace.stack.map_or(default, |x| f(&x))
    }

    /// Return the number of windows
    /// contained in all workspaces, including floating windows
    pub fn len(&self) -> usize {
        self.current.len() +
            self.visible.iter().map(|x| x.len()).fold(0, |a, x| a + x) +
            self.hidden.iter().map(|x| x.len()).fold(0, |a, x| a + x) +
            self.floating.len()
    }

    /// Checks if any of the workspaces contains the
    /// given window
    pub fn contains(&self, window: Window) -> bool {
        self.current.contains(window) ||
            self.visible.iter().any(|x| x.contains(window)) ||
            self.hidden.iter().any(|x| x.contains(window)) ||
            self.floating.contains_key(&window)
    }

    /// Get the number of managed workspaces.
    /// This is mostly used for out-of-bounds checking.
    pub fn number_workspaces(&self) -> u32 {
        (1 + self.visible.len() + self.hidden.len()) as u32
    }

    /// Shift the currently focused window to the given workspace
    pub fn shift(&self, index: u32) -> Workspaces {
        // Get current window
        self.peek()
            // and move it
            .map_or(self.clone(), |w| self.shift_window(index, w))
    }

    pub fn shift_master(&self) -> Workspaces {
        self.modify_stack(|s| if s.up.len() == 0 {
            s.clone()
        } else {
            let rev : Vec<Window> = s.up.iter()
                .rev()
                .chain(s.down.iter())
                .map(|x| x.clone())
                .collect();
            Stack::<Window>::new(s.focus, Vec::<Window>::new(), rev)
        })
    }

    pub fn insert_up(&self, window: Window) -> Workspaces {
        if self.contains(window) {
            return self.clone();
        }

        self.from_current(self.current.map_or(Stack::from_element(window), |s| {
            Stack::<Window>::new(window, s.up, (vec!(s.focus.clone())).into_iter().chain(s.down.clone().into_iter()).collect())
        }))
    }

    /// Retrieve the currently focused workspace's id
    pub fn current_tag(&self) -> u32 {
        self.current.workspace.id
    }

    /// Retrieve the tag of the workspace the given window
    /// is contained in. If it is not contained anywhere,
    /// return None.
    pub fn find_tag(&self, window: Window) -> Option<u32> {
        debug!("trying to find tag of workspace with window {}", window);
        self.workspaces().iter()
            .filter(|x| x.contains(window))
            .map(|x| x.id)
            .nth(0)
    }

    pub fn find_screen(&self, window: Window) -> Option<Screen> {
        if self.current.contains(window) {
            Some(self.current.clone())
        } else {
            self.visible.iter().filter(|x| x.contains(window)).map(|x| x.clone()).nth(0).clone()
        }
    }

    /// Flatten all workspaces into a list
    pub fn workspaces(&self) -> Vec<Workspace> {
        let v : Vec<Workspace> = self.visible.iter().map(|x| x.workspace.clone()).collect();
        (vec!(self.current.workspace.clone())).into_iter().chain(v.into_iter()).chain(self.hidden.clone().into_iter()).collect()
    }

    /// Shift the given window to the given workspace
    pub fn shift_window(&self, index: u32, window: Window) -> Workspaces {
        let first_closure = (Box::new(move |w: Workspaces| {
            w.delete(window)
        })) as Box<Fn(Workspaces,) -> Workspaces + 'static>;

        let second_closure = (Box::new(move |w: Workspaces| {
            w.insert_up(window)
        })) as Box<Fn(Workspaces,) -> Workspaces + 'static>;

        match self.find_tag(window) {
            Some(from) => {
                let a = self.on_workspace(from, first_closure);
                let b = self.on_workspace(index, second_closure);

                debug!("shifting window from {} to {}", from, index);
                b(a(self.clone()))
            },
            None => self.clone()
        }
    }

    /// Apply the given function to the given workspace
    pub fn on_workspace(&self, index: u32, f: Box<Fn(Workspaces,) -> Workspaces + 'static>)
        -> Box<Fn(Workspaces,) -> Workspaces + 'static> {
            (Box::new(move |x: Workspaces| {
                let current_tag = x.current_tag();
                (*f)(x.view(index)).view(current_tag)
            })) as Box<Fn(Workspaces,) -> Workspaces + 'static>
    }

    /// Return a list of all visible windows.
    /// This is just a convenience function.
    pub fn visible_windows(&self) -> Vec<Window> {
        self.current.windows().into_iter().chain(self.visible.iter()
            .flat_map(|x| x.windows().into_iter()))
            .collect()
    }

    /// Return a list of all windows, hidden, visible and floating.
    pub fn all_windows(&self) -> Vec<Window> {
        self.visible_windows().into_iter().chain(self.hidden.iter()
            .flat_map(|x| x.windows().into_iter()))
            .collect()
    }

    /// Returns a list of all windows as tuples together with their
    /// respective workspace IDs
    pub fn all_windows_with_workspaces(&self) -> Vec<(Window, u32)> {
        self.current.windows().into_iter().rev()
            .zip(repeat(self.current.workspace.id))
            .chain(self.visible.clone().into_iter()
            .flat_map(|x| x.windows().into_iter().rev().zip(repeat(x.workspace.id))))
            .chain(self.hidden.iter()
            .flat_map(|x| x.windows().into_iter().rev().zip(repeat(x.id))))
            .collect()
    }

    /// Return a list of all screens and their workspaces.
    /// Mostly used by layout.
    pub fn screens(&self) -> Vec<Screen> {
        (vec!(self.current.clone())).into_iter().chain(self.visible.clone().into_iter()).collect()
    }

    pub fn send_layout_message(&self, message: LayoutMessage, window_system: &WindowSystem,
                                   config: &GeneralConfig) -> Workspaces {
        self.from_current(self.current.send_layout_message(message, window_system, config))
    }

    pub fn with_focused<F>(&self, f: F) -> Workspaces where F : Fn(Window) {
        if let Some(window) = self.peek() {
            f(window);
        }

        self.clone()
    }

    pub fn update_floating_rect(&self, window: Window, rect: RationalRect) -> Workspaces {
        let mut map = self.floating.clone();
        map.insert(window, rect);
        Workspaces {
            current: self.current.clone(),
            visible: self.visible.clone(),
            hidden: self.hidden.clone(),
            floating: map
        }
    }
}
