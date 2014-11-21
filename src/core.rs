use std::collections::TreeMap;
use std::iter::AdditiveIterator;
use window_manager::ScreenDetail;
use window_system::Window;

#[deriving(Clone)]
pub struct RationalRect(f32, f32, f32, f32);

/// Handles focus tracking on a workspace.
/// `focus` keeps track of the focused window's id
/// and `up` and `down` are the windows above or
/// below the focus stack respectively.
#[deriving(Clone, PartialEq, Eq)]
pub struct Stack<T> {
    pub focus: T,
    pub up:    Vec<T>,
    pub down:  Vec<T>
}

impl<T: Clone + Eq> Stack<T> {
    /// Create a new stack with the given values
    pub fn new<T>(f: T, up: Vec<T>, down: Vec<T>) -> Stack<T> {
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
            down: self.down + vec!(self.focus.clone())
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
    pub fn filter<'r>(&self, f: |&T| : 'r -> bool) -> Option<Stack<T>> {
        let lrs : Vec<T> = (vec!(self.focus.clone()) + self.down).iter()
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
            let filtered : Vec<T> = self.up.iter().map(|x| x.clone()).filter(f).collect();
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
            let tmp : Vec<T> = (vec!(self.focus.clone()) + self.down).iter()
                .rev()
                .map(|x| x.clone())
                .collect();
            let xs : Vec<T> = tmp.iter()
                .skip(1)
                .map(|x| x.clone())
                .collect();

            Stack::<T>::new(tmp[0].clone(), xs, Vec::new())
        } else {
            let down = (vec!(self.focus.clone())) + self.down;
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
            let rs = vec!(x) + self.down;
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
        let rs : Vec<T> = xs + vec!(x) + self.down;

        Stack::<T>::new(self.focus.clone(), Vec::new(), rs)
    }

    /// Reverse the stack by exchanging
    /// the `up` and `down` lists
    pub fn reverse(&self) -> Stack<T> {
        Stack::<T>::new(self.focus.clone(), self.down.clone(), self.up.clone())
    }

    /// Return the number of elements tracked by the stack
    pub fn len(&self) -> uint {
        1 + self.up.len() + self.down.len()
    }

    /// Checks if the given window is tracked by the stack
    pub fn contains(&self, window: T) -> bool {
        self.focus == window || self.up.contains(&window) || self.down.contains(&window)
    }
}

/// Represents a single workspace with a `tag` (name),
/// `id`, a `layout` and a `stack` for all windows
#[deriving(Clone, PartialEq, Eq)]
pub struct Workspace {
    pub id:     u32,
    pub tag:    String,
    pub layout: String,
    pub stack:  Option<Stack<Window>>
}

impl Workspace {
    /// Create a new workspace
    pub fn new(id: u32, tag: String, layout: String, stack: Option<Stack<Window>>) -> Workspace {
        Workspace {
            id: id,
            tag: tag,
            layout: layout,
            stack: stack
        }
    }

    /// Add a new window to the workspace by adding it to the stack.
    /// If the stack doesn't exist yet, create one.
    pub fn add(&self, window: Window) -> Workspace {
        Workspace::new(
            self.id,
            self.tag.clone(),
            self.layout.clone(),
            Some(self.stack.clone().map_or(Stack::from_element(window), |s| s.add(window))))
    }

    /// Returns the number of windows contained in this workspace
    pub fn len(&self) -> uint {
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

    pub fn map(&self, f: |Stack<Window>| -> Stack<Window>) -> Workspace {
        Workspace::new(self.id, self.tag.clone(), self.layout.clone(),
        self.stack.clone().map(f))
    }

    pub fn map_option(&self, f: |Stack<Window>| -> Option<Stack<Window>>) -> Workspace {
        Workspace::new(self.id, self.tag.clone(), self.layout.clone(),
        self.stack.clone().map_or(None, f))
    }

    pub fn map_or(&self, default: Stack<Window>, f: |Stack<Window>| -> Stack<Window>) -> Workspace {
        Workspace::new(self.id, self.tag.clone(), self.layout.clone(),
        Some(self.stack.clone().map_or(default, f)))
    }
}

#[deriving(Clone, PartialEq, Eq)]
pub struct Screen {
    pub workspace:     Workspace,
    pub screen_id:     u32,
    pub screen_detail: ScreenDetail
}

impl Screen {
    /// Create a new screen for the given workspace
    /// and the given dimensions
    pub fn new(workspace: Workspace, screen_id: u32, screen_detail: ScreenDetail) -> Screen {
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
    pub fn len(&self) -> uint {
        self.workspace.len()
    }

    pub fn windows(&self) -> Vec<Window> {
        self.workspace.windows()
    }

    pub fn map_workspace(&self, f: |Workspace| -> Workspace) -> Screen {
        Screen::new(f(self.workspace.clone()), self.screen_id, self.screen_detail)
    }

    pub fn map(&self, f: |Stack<Window>| -> Stack<Window>) -> Screen {
        Screen::new(self.workspace.map(f), self.screen_id, self.screen_detail)
    }

    pub fn map_option(&self, f: |Stack<Window>| -> Option<Stack<Window>>) -> Screen {
        Screen::new(self.workspace.map_option(f), self.screen_id, self.screen_detail)
    }

    pub fn map_or(&self, default: Stack<Window>, f: |Stack<Window>| -> Stack<Window>) -> Screen {
        Screen::new(self.workspace.map_or(default, f), self.screen_id, self.screen_detail)
    }
}

#[deriving(Clone)]
pub struct Workspaces {
    /// The currently focused and visible screen
    pub current:  Screen,
    /// The other visible, but non-focused screens
    pub visible:  Vec<Screen>,
    /// All remaining workspaces that are currently hidden
    pub hidden:   Vec<Workspace>,
    /// A list of all floating windows
    pub floating: TreeMap<Window, RationalRect>
}

impl Workspaces {
    /// Create a new stackset, of empty stacks, with given tags,
    /// with physical screens whose descriptions are given by 'm'. The
    /// number of physical screens (@length 'm'@) should be less than or
    /// equal to the number of workspace tags.  The first workspace in the
    /// list will be current.
    ///
    /// Xinerama: Virtual workspaces are assigned to physical screens, starting at 0.
    pub fn new(layout: String, tags: Vec<String>, screens: Vec<ScreenDetail>) -> Workspaces {
        debug!("creating new workspaces with {} screen(s)", screens.len());
        let workspaces : Vec<Workspace> = tags.iter()
            .enumerate()
            .map(|(id, tag)| Workspace::new(id as u32, tag.clone(), layout.clone(), None))
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
            floating: TreeMap::new()
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
        let w = match self.visible.iter().position(|s| s.workspace.id == index) {
            Some(screen_pos) => {
                let screen = self.visible[screen_pos].clone();
                let visible : Vec<Screen> = self.visible.iter()
                        .enumerate()
                        .filter(|&(x, _)| x != screen_pos)
                        .map(|(_, y)| y.clone())
                        .collect();

                self.from_visible(visible + vec!(self.current.clone()))
                    .from_current(screen)
            },
            _ => self.clone()
        };

        // Desired workspace is hidden. Switch it with the current workspace
        match w.hidden.iter().position(|w| w.id == index) {
            Some(workspace_pos) => {
                let hidden : Vec<Workspace> = w.hidden.iter()
                    .enumerate()
                    .filter(|&(x, _)| x != workspace_pos)
                    .map(|(_, y)| y.clone())
                    .collect();

                w.from_hidden(hidden + vec!(w.current.workspace.clone()))
                 .from_current(w.current.map_workspace(|_| w.hidden[workspace_pos].clone()))
            },
            _ => w.clone()
        }
    }

    /// Set focus to the given workspace.  If that workspace does not exist
    /// in the stackset, the original workspace is returned.  If that workspace is
    /// 'hidden', then display that workspace on the current screen, and move the
    /// current workspace to 'hidden'.  If that workspace is 'visible' on another
    /// screen, the workspaces of the current screen and the other screen are
    /// swapped.
    pub fn greedy_view(&self, _: uint) {

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
        let remove_from_workspace = |stack: Stack<Window>| -> Option<Stack<Window>> {
            stack.filter(|&x| x != window)
        };

        self.modify_stack_option(|x| remove_from_workspace(x))
            .modify_hidden_option(|x| remove_from_workspace(x))
            .modify_visible_option(|x| remove_from_workspace(x))
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

    pub fn modify_stack(&self, f: |Stack<Window>| -> Stack<Window>) -> Workspaces {
        self.from_current(self.current.map(|s| f(s)))
    }

    pub fn modify_stack_option(&self, f: |Stack<Window>| -> Option<Stack<Window>>) -> Workspaces {
        self.from_current(self.current.map_option(|s| f(s)))
    }

    pub fn modify_hidden(&self, f: |Stack<Window>| -> Stack<Window>) -> Workspaces {
        self.from_hidden(self.hidden.iter().map(|x| x.map(|s| f(s))).collect())
    }

    pub fn modify_hidden_option(&self, f: |Stack<Window>| -> Option<Stack<Window>>) -> Workspaces {
        self.from_hidden(self.hidden.iter().map(|x| x.map_option(|s| f(s))).collect())
    }


    pub fn modify_visible(&self, f: |Stack<Window>| -> Stack<Window>) -> Workspaces {
        self.from_visible(self.visible.iter().map(|x| x.map(|s| f(s))).collect())
    }

    pub fn modify_visible_option(&self, f: |Stack<Window>| -> Option<Stack<Window>>) -> Workspaces {
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
    pub fn with<T>(&self, default: T, f: |&Stack<Window>| -> T) -> T {
        match self.current.workspace.stack {
            Some(ref s) => f(s),
            None        => default
        }
    }

    /// Return the number of windows
    /// contained in all workspaces, including floating windows
    pub fn len(&self) -> uint {
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
    pub fn shift(&self, index: u32) -> Workspaces {
        // Get current window
        self.peek()
            // and move it
            .map_or(self.clone(), |w| self.shift_window(index, w))
    }

    pub fn insert_up(&self, window: Window) -> Workspaces {
        if self.contains(window) {
            return self.clone();
        }

        self.from_current(self.current.map_or(Stack::from_element(window), |s| {
            Stack::<Window>::new(window, s.up, (vec!(s.focus.clone())) + s.down)
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
        self.workspaces().iter()
            .filter(|x| x.contains(window))
            .map(|x| x.id as u32)
            .nth(0)
    }

    pub fn find_screen(&self, window: Window) -> Screen {
        if self.current.contains(window) {
            self.current.clone()
        } else {
            self.visible.iter()
                .filter(|x| x.contains(window))
                .nth(0).unwrap().clone()
        }
    }

    /// Flatten all workspaces into a list
    pub fn workspaces(&self) -> Vec<Workspace> {
        let v : Vec<Workspace> = self.visible.iter().map(|x| x.workspace.clone()).collect();
        (vec!(self.current.workspace.clone())) + v + self.hidden
    }

    /// Shift the given window to the given workspace
    pub fn shift_window(&self, index: u32, window: Window) -> Workspaces {
        let first_closure = (box move |&: w: Workspaces| {
            w.delete(window)
        }) as Box<Fn<(Workspaces,), Workspaces> + 'static>;

        let second_closure = (box move |&: w: Workspaces| {
            w.insert_up(window)
        }) as Box<Fn<(Workspaces,), Workspaces> + 'static>;

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
    pub fn on_workspace(&self, index: u32, f: Box<Fn<(Workspaces,), Workspaces> + 'static>)
        -> Box<Fn<(Workspaces,), Workspaces> + 'static> {
            (box move |&: x: Workspaces| {
                let current_tag = x.current_tag();
                (*f).call((x.view(index),)).view(current_tag)
            }) as Box<Fn<(Workspaces,), Workspaces> + 'static>
        }

    /// Return a list of all visible windows.
    /// This is just a convenience function.
    pub fn visible_windows(&self) -> Vec<Window> {
        let visible : Vec<Window> = self.visible.iter()
            .map(|x| x.windows())
            .flat_map(|x| x.into_iter())
            .collect();

        self.current.windows() + visible
    }

    /// Return a list of all windows, hidden, visible and floating.
    pub fn all_windows(&self) -> Vec<Window> {
        let hidden : Vec<Window> = self.hidden.iter()
            .map(|x| x.windows())
            .flat_map(|x| x.into_iter())
            .collect();

        self.visible_windows() + hidden
    }

    pub fn all_windows_with_workspaces(&self) -> Vec<(Window, u32)> {
        let visible : Vec<(Window, u32)> = self.visible.iter()
            .map(|x| {
                let t : Vec<(Window, u32)> = x.windows().iter()
                    .map(|&w| (w, x.workspace.id))
                    .collect(); t
            })
            .flat_map(|x| x.into_iter())
            .collect();
        let hidden : Vec<(Window, u32)> = self.hidden.iter()
            .map(|x| {
                let t : Vec<(Window, u32)> = x.windows().iter()
                    .map(|&w| (w, x.id))
                    .collect(); t
            })
            .flat_map(|x| x.into_iter())
            .collect();
        let current : Vec<(Window, u32)> = self.current.windows().iter()
            .map(|&x| (x, self.current.workspace.id))
            .collect();

        current + visible + hidden
    }

    /// Return a list of all screens and their workspaces.
    /// Mostly used by layout.
    pub fn screens(&self) -> Vec<Screen> {
        (vec!(self.current.clone())) + self.visible
    }
}
