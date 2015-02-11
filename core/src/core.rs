use std::collections::BTreeMap;
use std::iter::AdditiveIterator;
use std::iter::repeat;
use config::GeneralConfig;
use window_manager::ScreenDetail;
use window_system::{ Window, WindowSystem };
use layout::{Layout, LayoutMessage};

#[derive(Clone, Copy, Debug)]
pub struct RationalRect(pub f32, pub f32, pub f32, pub f32);

/// Handles focus tracking on a workspace.
/// `focus` keeps track of the focused window's id
/// and `up` and `down` are the windows above or
/// below the focus stack respectively.
#[derive(Clone, PartialEq, Eq)]
pub struct Stack<T> {
    pub focus: T,
    pub up:    Vec<T>,
    pub down:  Vec<T>
}

impl<T: Clone + Eq> Stack<T> {
    /// Create a new stack with the given values
    pub fn new(f: T, up: Vec<T>, down: Vec<T>) -> Stack<T> {
        Stack {
            focus: f,
            up:    up,
            down:  down
        }
    }

    /// Create a new stack with only the given element
    /// as the focused one and initialize the rest to empty.
    pub fn from_element(t: T) -> Stack<T> {
        Stack {
            focus: t,
            up:    Vec::new(),
            down:  Vec::new()
        }
    }

    /// Add a new element to the stack
    /// and automatically focus it.
    pub fn add(&self, t: T) -> Stack<T> {
        Stack {
            focus: t,
            up: self.up.clone(),
            down: self.down.clone() + (vec!(self.focus.clone()).as_slice())
        }
    }

    /// Flatten the stack into a list
    pub fn integrate(&self) -> Vec<T> {
        self.up.iter()
            .rev()
            .chain((vec!(self.focus.clone())).iter())
            .chain(self.down.iter())
            .map(|x| x.clone())
            .collect()
    }

    /// Filter the stack to retain only windows
    /// that yield true in the given filter function
    pub fn filter<F>(&self, f: F) -> Option<Stack<T>> where F : Fn(&T) -> bool {
        let lrs : Vec<T> = (vec!(self.focus.clone()) + self.down.as_slice()).iter()
            .filter(|&x| f(x))
            .map(|x| x.clone())
            .collect();

        if lrs.len() > 0 {
            let first : T         = lrs[0].clone();
            let rest : Vec<T>     = lrs.iter().skip(1).map(|x| x.clone()).collect();
            let filtered : Vec<T> = self.up.iter()
                .filter(|&x| f(x))
                .map(|x| x.clone())
                .collect();
            let stack : Stack<T>  = Stack::<T>::new(first, filtered, rest);

            Some(stack)
        } else {
            let filtered : Vec<T> = self.up.iter().map(|x| x.clone()).filter(|x| f(x)).collect();
            if filtered.len() > 0 {
                let first : T        = filtered[0].clone();
                let rest : Vec<T>    = filtered.iter().skip(1).map(|x| x.clone()).collect();

                Some(Stack::<T>::new(first, rest, Vec::new()))
            } else {
                None
            }
        }
    }

    /// Move the focus to the next element in the `up` list
    pub fn focus_up(&self) -> Stack<T> {
        if self.up.is_empty() {
            let tmp : Vec<T> = (vec!(self.focus.clone()) + self.down.as_slice()).into_iter()
                .rev()
                .collect();
            let xs : Vec<T> = tmp.iter()
                .skip(1)
                .map(|x| x.clone())
                .collect();

            Stack::<T>::new(tmp[0].clone(), xs, Vec::new())
        } else {
            let down = (vec!(self.focus.clone())) + self.down.as_slice();
            let up   = self.up.iter().skip(1).map(|x| x.clone()).collect();
            Stack::<T>::new(self.up[0].clone(), up, down)
        }
    }

    /// Move the focus down
    pub fn focus_down(&self) -> Stack<T> {
        self.reverse().focus_up().reverse()
    }

    pub fn swap_up(&self) -> Stack<T> {
        if self.up.is_empty() {
            Stack::<T>::new(self.focus.clone(), self.down.iter().rev().map(|x| x.clone()).collect(), Vec::new())
        } else {
            let x = self.up[0].clone();
            let xs = self.up.iter().skip(1).map(|x| x.clone()).collect();
            let rs = vec!(x) + self.down.as_slice();
            Stack::<T>::new(self.focus.clone(), xs, rs)
        }
    }

    pub fn swap_down(&self) -> Stack<T> {
        self.reverse().swap_up().reverse()
    }

    pub fn swap_master(&self) -> Stack<T> {
        if self.up.is_empty() {
            return self.clone();
        }

        let r : Vec<T>  = self.up.iter()
            .rev()
            .map(|x| x.clone())
            .collect();
        let x : T       = r[0].clone();
        let xs : Vec<T> = r.iter()
            .skip(1)
            .map(|x| x.clone())
            .collect();
        let rs : Vec<T> = xs + (vec!(x) + self.down.as_slice()).as_slice();

        Stack::<T>::new(self.focus.clone(), Vec::new(), rs)
    }

    /// Reverse the stack by exchanging
    /// the `up` and `down` lists
    pub fn reverse(&self) -> Stack<T> {
        Stack::<T>::new(self.focus.clone(), self.down.clone(), self.up.clone())
    }

    /// Return the number of elements tracked by the stack
    pub fn len(&self) -> usize {
        1 + self.up.len() + self.down.len()
    }

    /// Checks if the given window is tracked by the stack
    pub fn contains(&self, window: T) -> bool {
        self.focus == window || self.up.contains(&window) || self.down.contains(&window)
    }
}

/// Represents a single workspace with a `tag` (name),
/// `id`, a `layout` and a `stack` for all windows
pub struct Workspace<'a> {
    pub id:     u32,
    pub tag:    String,
    pub layout: Box<Layout + 'a>,
    pub stack:  Option<Stack<Window>>
}

impl<'a> Clone for Workspace<'a> {
    fn clone(&self) -> Workspace<'a> {
        Workspace {
            id: self.id,
            tag: self.tag.clone(),
            layout: self.layout.copy(),
            stack: self.stack.clone()
        }
    }
}

impl<'a> Workspace<'a> {
    /// Create a new workspace
    pub fn new(id: u32, tag: String, layout: Box<Layout + 'a>, stack: Option<Stack<Window>>) -> Workspace<'a> {
        Workspace {
            id: id,
            tag: tag,
            layout: layout,
            stack: stack
        }
    }

    /// Add a new window to the workspace by adding it to the stack.
    /// If the stack doesn't exist yet, create one.
    pub fn add(&self, window: Window) -> Workspace<'a> {
        Workspace::new(
            self.id,
            self.tag.clone(),
            self.layout.copy(),
            Some(self.stack.clone().map_or(Stack::from_element(window), |s| s.add(window))))
    }

    /// Returns the number of windows contained in this workspace
    pub fn len(&self) -> usize {
        self.stack.clone().map_or(0, |x| x.len())
    }

    /// Checks if the workspace contains the given window
    pub fn contains(&self, window: Window) -> bool {
        self.stack.clone().map_or(false, |x| x.contains(window))
    }

    pub fn windows(&self) -> Vec<Window> {
        self.stack.clone().map_or(Vec::new(), |s| s.integrate())
    }

    pub fn peek(&self) -> Option<Window> {
        self.stack.clone().map(|s| s.focus)
    }

    pub fn map<F>(&self, f: F) -> Workspace<'a> where F : Fn(Stack<Window>) -> Stack<Window> {
        Workspace::new(self.id, self.tag.clone(), self.layout.copy(),
        self.stack.clone().map(|x| f(x)))
    }

    pub fn map_option<F>(&self, f: F) -> Workspace<'a> where F : Fn(Stack<Window>) -> Option<Stack<Window>> {
        Workspace::new(self.id, self.tag.clone(), self.layout.copy(),
        self.stack.clone().map_or(None, |x| f(x)))
    }

    pub fn map_or<F>(&self, default: Stack<Window>, f: F) -> Workspace<'a> where F : Fn(Stack<Window>) -> Stack<Window> {
        Workspace::new(self.id, self.tag.clone(), self.layout.copy(),
        Some(self.stack.clone().map_or(default, |x| f(x))))
    }

    pub fn send_layout_message<'b>(&self, message: LayoutMessage, window_system: &WindowSystem,
                                   config: &GeneralConfig<'b>) -> Workspace<'a> {
        let mut layout = self.layout.copy();
        layout.apply_message(message, window_system, &self.stack, config);
        Workspace::new(self.id, self.tag.clone(), layout, self.stack.clone())
    }
}

pub struct Screen<'a> {
    pub workspace:     Workspace<'a>,
    pub screen_id:     u32,
    pub screen_detail: ScreenDetail
}

impl<'a> Clone for Screen<'a> {
    fn clone(&self) -> Screen<'a> {
        Screen {
            workspace: self.workspace.clone(),
            screen_id: self.screen_id,
            screen_detail: self.screen_detail.clone()
        }
    }
}

impl<'a> Screen<'a> {
    /// Create a new screen for the given workspace
    /// and the given dimensions
    pub fn new(workspace: Workspace<'a>, screen_id: u32, screen_detail: ScreenDetail) -> Screen<'a> {
        Screen {
            workspace: workspace,
            screen_id: screen_id,
            screen_detail: screen_detail
        }
    }

    /// Checks if the screen's workspace contains
    /// the given window
    pub fn contains(&self, window: Window) -> bool {
        self.workspace.contains(window)
    }

    /// Returns the number of windows in the
    /// screen's workspace
    pub fn len(&self) -> usize {
        self.workspace.len()
    }

    pub fn windows(&self) -> Vec<Window> {
        self.workspace.windows()
    }

    pub fn map_workspace<F>(&self, f: F) -> Screen<'a> where F : Fn(Workspace<'a>) -> Workspace<'a> {
        Screen::new(f(self.workspace.clone()), self.screen_id, self.screen_detail)
    }

    pub fn map<F>(&self, f: F) -> Screen<'a> where F : Fn(Stack<Window>) -> Stack<Window> {
        Screen::new(self.workspace.map(f), self.screen_id, self.screen_detail)
    }

    pub fn map_option<F>(&self, f: F) -> Screen<'a> where F : Fn(Stack<Window>) -> Option<Stack<Window>> {
        Screen::new(self.workspace.map_option(f), self.screen_id, self.screen_detail)
    }

    pub fn map_or<F>(&self, default: Stack<Window>, f: F) -> Screen<'a> where F : Fn(Stack<Window>) -> Stack<Window> {
        Screen::new(self.workspace.map_or(default, f), self.screen_id, self.screen_detail)
    }

    pub fn send_layout_message<'b>(&self, message: LayoutMessage, window_system: &WindowSystem,
                                   config: &GeneralConfig<'b>) -> Screen<'a> {
        Screen::new(self.workspace.send_layout_message(message, window_system, config), self.screen_id, self.screen_detail)
    }
}

pub struct Workspaces<'a> {
    /// The currently focused and visible screen
    pub current:  Screen<'a>,
    /// The other visible, but non-focused screens
    pub visible:  Vec<Screen<'a>>,
    /// All remaining workspaces that are currently hidden
    pub hidden:   Vec<Workspace<'a>>,
    /// A list of all floating windows
    pub floating: BTreeMap<Window, RationalRect>
}

impl<'a> Clone for Workspaces<'a> {
    fn clone(&self) -> Workspaces<'a> {
        Workspaces {
            current: self.current.clone(),
            visible: self.visible.clone(),
            hidden:  self.hidden.clone(),
            floating: self.floating.clone()
        }
    }
}

impl<'a> Workspaces<'a> {
    /// Create a new stackset, of empty stacks, with given tags,
    /// with physical screens whose descriptions are given by 'm'. The
    /// number of physical screens (@length 'm'@) should be less than or
    /// equal to the number of workspace tags.  The first workspace in the
    /// list will be current.
    ///
    /// Xinerama: Virtual workspaces are assigned to physical screens, starting at 0.
    pub fn new(layout: Box<Layout + 'a>, tags: Vec<String>, screens: Vec<ScreenDetail>) -> Workspaces<'a> {
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

    pub fn from_current(&self, current: Screen<'a>) -> Workspaces<'a> {
        Workspaces {
            current: current,
            visible: self.visible.clone(),
            hidden: self.hidden.clone(),
            floating: self.floating.clone()
        }
    }

    pub fn from_visible(&self, visible: Vec<Screen<'a>>) -> Workspaces<'a> {
        Workspaces {
            current: self.current.clone(),
            visible: visible,
            hidden: self.hidden.clone(),
            floating: self.floating.clone()
        }
    }

    pub fn from_hidden(&self, hidden: Vec<Workspace<'a>>) -> Workspaces<'a> {
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
    pub fn view(&self, index: u32) -> Workspaces<'a> {
        debug!("setting focus to {}", index);

        // We are already on the desired workspace. Do nothing
        if self.current.workspace.id == index {
            return self.clone();
        }

        // Desired workspace is visible, switch to it by raising
        // it to current and pushing the current one to visible
        if let Some(screen_pos) = self.visible.iter().position(|s| s.workspace.id == index) {
            let screen = self.visible[screen_pos].clone();
            let visible : Vec<Screen<'a>> = self.visible.iter()
                .enumerate()
                .filter(|&(x, _)| x != screen_pos)
                .map(|(_, y)| y.clone())
                .collect();

            self.from_visible(visible + (vec!(self.current.clone())).as_slice())
                .from_current(screen)
        // Desired workspace is hidden. Switch it with the current workspace
        } else if let Some(workspace_pos) = self.hidden.iter().position(|w| w.id == index) {
            let hidden : Vec<Workspace<'a>> = self.hidden.iter()
                .enumerate()
                .filter(|&(x, _)| x != workspace_pos)
                .map(|(_, y)| y.clone())
                .collect();

            self.from_hidden(hidden + (vec!(self.current.workspace.clone())).as_slice())
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
    pub fn greedy_view(&self, index: u32) -> Workspaces<'a> {
        if self.hidden.iter().any(|x| x.id == index) {
            self.view(index)
        } else if let Some(s) = self.visible.iter().find(|x| x.workspace.id == index) {
            let screen = self.current.clone();
            self.from_current(s.clone())
                .from_visible(self.visible.iter()
                              .filter(|x| x.workspace.id != index)
                              .map(|x| x.clone())
                              .collect::<Vec<_>>() + (vec!(screen)).as_slice())
        } else {
            self.clone()
        }
    }

    pub fn float(&self, window: Window, rect: RationalRect) -> Workspaces<'a> {
        let mut w = self.clone();
        w.floating.insert(window, rect);
        w
    }

    pub fn sink(&self, window: Window) -> Workspaces<'a> {
        let mut w = self.clone();
        w.floating.remove(&window);
        w
    }

    pub fn delete(&self, window: Window) -> Workspaces<'a> {
        self.delete_p(window).sink(window)
    }

    pub fn delete_p(&self, window: Window) -> Workspaces<'a> {
        fn remove_from_workspace(stack: Stack<Window>, window: Window) -> Option<Stack<Window>> {
            stack.filter(|&x| x != window)
        }

        self.modify_stack_option(|x| remove_from_workspace(x, window))
            .modify_hidden_option(|x| remove_from_workspace(x, window))
            .modify_visible_option(|x| remove_from_workspace(x, window))
    }

    pub fn focus_window(&self, window: Window) -> Workspaces<'a> {
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
    pub fn focus_down(&self) -> Workspaces<'a> {
        self.modify_stack(|x| x.focus_down())
    }

    pub fn focus_up(&self) -> Workspaces<'a> {
        self.modify_stack(|x| x.focus_up())
    }

    pub fn swap_down(&self) -> Workspaces<'a> {
        self.modify_stack(|x| x.swap_down())
    }

    pub fn swap_up(&self) -> Workspaces<'a> {
        self.modify_stack(|x| x.swap_up())
    }

    pub fn swap_master(&self) -> Workspaces<'a> {
        self.modify_stack(|x| x.swap_master())
    }

    pub fn modify_stack<F>(&self, f: F) -> Workspaces<'a> where F : Fn(Stack<Window>) -> Stack<Window> {
        self.from_current(self.current.map(|s| f(s)))
    }

    pub fn modify_stack_option<F>(&self, f: F) -> Workspaces<'a> where F : Fn(Stack<Window>) -> Option<Stack<Window>> {
        self.from_current(self.current.map_option(|s| f(s)))
    }

    pub fn modify_hidden<F>(&self, f: F) -> Workspaces<'a> where F : Fn(Stack<Window>) -> Stack<Window> {
        self.from_hidden(self.hidden.iter().map(|x| x.map(|s| f(s))).collect())
    }

    pub fn modify_hidden_option<F>(&self, f: F) -> Workspaces<'a> where F : Fn(Stack<Window>) -> Option<Stack<Window>> {
        self.from_hidden(self.hidden.iter().map(|x| x.map_option(|s| f(s))).collect())
    }


    pub fn modify_visible<F>(&self, f: F) -> Workspaces<'a> where F : Fn(Stack<Window>) -> Stack<Window> {
        self.from_visible(self.visible.iter().map(|x| x.map(|s| f(s))).collect())
    }

    pub fn modify_visible_option<F>(&self, f: F) -> Workspaces<'a> where F : Fn(Stack<Window>) -> Option<Stack<Window>> {
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
    pub fn with<T, F>(&self, default: T, f: F) -> T where F : Fn(&Stack<Window>) -> T {
        self.clone().current.workspace.stack.map_or(default, |x| f(&x))
    }

    /// Return the number of windows
    /// contained in all workspaces, including floating windows
    pub fn len(&self) -> usize {
        self.current.len() +
            self.visible.iter().map(|x| x.len()).sum() +
            self.hidden.iter().map(|x| x.len()).sum() +
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
    pub fn shift(&self, index: u32) -> Workspaces<'a> {
        // Get current window
        self.peek()
            // and move it
            .map_or(self.clone(), |w| self.shift_window(index, w))
    }

    pub fn shift_master(&self) -> Workspaces<'a> {
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

    pub fn insert_up(&self, window: Window) -> Workspaces<'a> {
        if self.contains(window) {
            return self.clone();
        }

        self.from_current(self.current.map_or(Stack::from_element(window), |s| {
            Stack::<Window>::new(window, s.up, (vec!(s.focus.clone())) + s.down.as_slice())
        }))
    }

    /// Retrieve the currently focused workspace's id
    pub fn current_tag(&self) -> u32 {
        self.current.workspace.id as u32
    }

    /// Retrieve the tag of the workspace the given window
    /// is contained in. If it is not contained anywhere,
    /// return None.
    pub fn find_tag(&self, window: Window) -> Option<u32> {
        debug!("trying to find tag of workspace with window {}", window);
        self.workspaces().iter()
            .filter(|x| x.contains(window))
            .map(|x| x.id as u32)
            .nth(0)
    }

    pub fn find_screen(&self, window: Window) -> Option<Screen<'a>> {
        if self.current.contains(window) {
            Some(self.current.clone())
        } else {
            self.visible.iter().filter(|x| x.contains(window)).map(|x| x.clone()).nth(0).clone()
        }
    }

    /// Flatten all workspaces into a list
    pub fn workspaces(&self) -> Vec<Workspace<'a>> {
        let v : Vec<Workspace> = self.visible.iter().map(|x| x.workspace.clone()).collect();
        (vec!(self.current.workspace.clone())) + v.as_slice() + self.hidden.as_slice()
    }

    /// Shift the given window to the given workspace
    pub fn shift_window(&self, index: u32, window: Window) -> Workspaces<'a> {
        let first_closure = (box move |&: w: Workspaces<'a>| {
            w.delete(window)
        }) as Box<Fn(Workspaces<'a>,) -> Workspaces<'a> + 'static>;

        let second_closure = (box move |&: w: Workspaces<'a>| {
            w.insert_up(window)
        }) as Box<Fn(Workspaces<'a>,) -> Workspaces<'a> + 'static>;

        match self.find_tag(window) {
            Some(from) => {
                let a = self.on_workspace(from, first_closure);
                let b = self.on_workspace(index, second_closure);

                debug!("shifting window from {} to {}", from, index);
                b.call((a.call((self.clone(),)),))
            },
            None => self.clone()
        }
    }

    /// Apply the given function to the given workspace
    pub fn on_workspace(&self, index: u32, f: Box<Fn(Workspaces<'a>,) -> Workspaces<'a> + 'static>)
        -> Box<Fn(Workspaces<'a>,) -> Workspaces<'a> + 'static> {
            (box move |&: x: Workspaces<'a>| {
                let current_tag = x.current_tag();
                (*f).call((x.view(index),)).view(current_tag)
            }) as Box<Fn(Workspaces<'a>,) -> Workspaces<'a> + 'static>
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
    pub fn screens(&self) -> Vec<Screen<'a>> {
        (vec!(self.current.clone())) + self.visible.as_slice()
    }

    pub fn send_layout_message<'b>(&self, message: LayoutMessage, window_system: &WindowSystem,
                                   config: &GeneralConfig<'b>) -> Workspaces<'a> {
        self.from_current(self.current.send_layout_message(message, window_system, config))
    }

    pub fn with_focused<F>(&self, f: F) -> Workspaces<'a> where F : Fn(Window) {
        if let Some(window) = self.peek() {
            f(window);
        }

        self.clone()
    }

    pub fn update_floating_rect(&self, window: Window, rect: RationalRect) -> Workspaces<'a> {
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
