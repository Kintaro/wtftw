use std::collections::TreeMap;
use std::iter::AdditiveIterator;
use std::mem::swap;
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
        let mut s = self.clone();
        s.down.push(self.focus.clone());
        s.focus = t;
        s
    }

    /// Flatten the stack into a list
    pub fn integrate(&self) -> Vec<T> {
        self.up.iter()
            .rev()
            .chain((vec!(self.focus.clone())).iter())
            .chain(self.down.iter())
            .map(|x| x.clone())
            //.rev()
            .collect()
    }

    /// Filter the stack to retain only windows
    /// that yield true in the given filter function
    pub fn filter<'r>(&self, f: |&T| : 'r -> bool) -> Option<Stack<T>> {
        let lrs : Vec<T> = (vec!(self.focus.clone())).iter()
            .chain(self.down.iter())
            .map(|x| x.clone())
            .filter(|x| f(x))
            .collect();

        if lrs.len() > 0 {
            let first : T         = lrs.head().unwrap().clone();
            let rest : Vec<T>     = lrs.iter().skip(1).map(|x| x.clone()).collect();
            let filtered : Vec<T> = self.up.iter()
                .map(|x| x.clone())
                .filter(f)
                .map(|x| x.clone())
                .collect();
            let stack : Stack<T>  = Stack::<T>::new(first, filtered, rest);

            Some(stack)
        } else {
            let filtered : Vec<T> = self.up.iter().map(|x| x.clone()).filter(f).collect();
            if filtered.len() > 0 {
                let first : T        = filtered.head().unwrap().clone();
                let rest : Vec<T>    = filtered.iter().skip(1).map(|x| x.clone()).collect();
                let stack : Stack<T> = Stack::<T>::new(first, rest, Vec::new());

                Some(stack)
            } else {
                None
            }
        }
    }

    /// Move the focus to the next element in the `down` list
    pub fn focus_down(&self) -> Stack<T> {
        let mut s = self.clone();
        if self.up.is_empty() {
            let tmp : Vec<T> = (vec!(s.focus.clone())).iter()
                .chain(s.down.iter())
                .rev()
                .map(|x| x.clone())
                .collect();
            let x = tmp.head().unwrap();
            let xs : Vec<T> = tmp.iter()
                .skip(1)
                .map(|x| x.clone())
                .collect();

            s.focus = x.clone();
            s.up = xs;
            s.down = Vec::new();
        } else {
            s.down.insert(0, s.focus.clone());
            s.focus = s.up.head().unwrap().clone();
            s.up.remove(0);
        }

        s
    }

    /// Move the focus up
    pub fn focus_up(&self) -> Stack<T> {
        self.reverse().focus_down().reverse()
    }

    /// Reverse the stack by exchanging
    /// the `up` and `down` lists
    pub fn reverse(&self) -> Stack<T> {
        let mut s = self.clone();
        swap(&mut s.up, &mut s.down);
        s
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
#[deriving(Clone)]
pub struct Workspace {
    pub id:     u32,
    pub tag:    String,
    pub layout: String,
    pub stack:  Option<Stack<Window>>
}

#[deriving(Clone)]
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
        let mut w = self.clone();
        w.stack = Some(self.stack.clone().map_or(Stack::from_element(window), |s| s.add(window)));
        w
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
}

#[deriving(Clone)]
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
            current: current.head().unwrap().clone(),
            visible: current.iter().skip(1).map(|x| x.clone()).collect(),
            hidden: unseen,
            floating: TreeMap::new()
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
        if self.current.workspace.id == index {
            return self.clone();
        }

        let mut w = self.clone();

        match w.visible.iter().position(|s| s.workspace.id == index) {
            Some(screen_pos) => {
                let screen = self.visible[screen_pos].clone();
                w.visible.remove(screen_pos);
                w.visible.insert(0, self.current.clone());
                w.current = screen;
                return w;
            },
            _ => ()
        }

        match self.hidden.iter().position(|w| w.id == index) {
            Some(workspace_pos) => {
                let current_workspace = self.current.workspace.clone();
                w.current.workspace = self.hidden[workspace_pos].clone();
                w.hidden.remove(workspace_pos);
                w.hidden.insert(0, current_workspace);
                w
            },
            _ => w
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
        let mut w = self.clone();

        let remove_from_workspace = |workspace: &mut Workspace| {
            if workspace.stack.is_some() {
                workspace.stack = workspace.clone().stack.unwrap().filter(|&x| x != window);
            }
        };

        remove_from_workspace(&mut w.current.workspace);

        w.hidden.iter_mut().fold((), |_, workspace| {
            remove_from_workspace(workspace)
        });

        w.visible.iter_mut().fold((), |_, screen| {
            remove_from_workspace(&mut screen.workspace)
        });

        w
    }

    /// Move the focus of the currently focused workspace down
    pub fn focus_down(&self) -> Workspaces {
        let mut w = self.clone();
        w.current.workspace.stack = w.current.workspace.stack.map(|s| s.focus_down());
        w
    }

    pub fn focus_up(&self) -> Workspaces {
        let mut w = self.clone();
        w.current.workspace.stack = w.current.workspace.stack.map(|s| s.focus_up());
        w
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
        debug!("insert_up");
        if self.contains(window) {
            return self.clone();
        }
        debug!("yep");

        let mut w = self.clone();
        match w.current.workspace.stack {
            Some(ref mut s) => {
                s.down.insert(0, s.focus.clone());
                s.focus = window;
            },
            None => w.current.workspace.stack = Some(Stack::from_element(window))
        }
        w
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

    /// Flatten all workspaces into a list
    pub fn workspaces(&self) -> Vec<Workspace> {
        let v : Vec<Workspace> = self.visible.iter().map(|x| x.workspace.clone()).collect();
        (vec!(self.current.workspace.clone())).iter()
            .chain(v.iter())
            .chain(self.hidden.iter())
            .map(|x| x.clone())
            .collect()
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
                (*b).call(((*a).call((self.clone(),)),))
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

    pub fn visible_windows(&self) -> Vec<Window> {
        let visible : Vec<Window> = self.visible.iter()
            .map(|x| x.windows())
            .flat_map(|x| x.into_iter())
            .collect();
        self.current.windows().iter()
            .chain(visible.iter())
            .map(|x| x.clone())
            .collect()
    }

    pub fn all_windows(&self) -> Vec<Window> {
        let hidden : Vec<Window> = self.hidden.iter()
            .map(|x| x.windows())
            .flat_map(|x| x.into_iter())
            .collect();

        self.visible_windows().iter()
            .chain(hidden.iter())
            .map(|x| x.clone())
            .collect()
    }
}
