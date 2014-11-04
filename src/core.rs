use std::collections::TreeMap;
use std::iter::AdditiveIterator;
use window_manager::ScreenDetail;
use window_system::Window;

#[deriving(Clone)]
pub struct RationalRect(f32, f32, f32, f32);

#[deriving(Clone)]
pub struct Stack<T> {
    pub focus: T,
    pub up:    Vec<T>,
    pub down:  Vec<T>
}

impl<T: Clone + Eq> Stack<T> {
    pub fn new<T>(f: T, up: Vec<T>, down: Vec<T>) -> Stack<T> {
        Stack {
            focus: f,
            up:    up,
            down:  down
        }
    }

    pub fn from_element(t: T) -> Stack<T> {
        Stack {
            focus: t,
            up:    Vec::new(),
            down:  Vec::new()
        }
    }

    pub fn add(&mut self, t: T) {
        self.down.push(self.focus.clone());
        self.focus = t;
    }

    pub fn integrate(&self) -> Vec<T> {
        self.up.iter()
            .rev()
            .chain((vec!(self.focus.clone())).iter())
            .chain(self.down.iter())
            .map(|x| x.clone())
            .collect()
    }

    pub fn filter<'r>(&self, f: |&T| : 'r -> bool) -> Option<Stack<T>> {
        let lrs : Vec<T> = (vec!(self.focus.clone())).iter()
            .chain(self.down.iter())
            .map(|x| x.clone())
            .filter(|x| f(x))
            .collect();

        if lrs.len() > 0 {
            let first : T         = lrs.head().unwrap().clone();
            let rest : Vec<T>     = lrs.iter().skip(1).map(|x| x.clone()).collect();
            let filtered : Vec<T> = self.up.iter().map(|x| x.clone()).filter(f).map(|x| x.clone()).collect();
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

    pub fn len(&self) -> uint {
        1 + self.up.len() + self.down.len()
    }

    pub fn contains(&self, window: T) -> bool {
        self.focus == window || self.up.contains(&window) || self.down.contains(&window)
    }
}

#[deriving(Clone)]
pub struct Workspace {
    pub id:     uint,
    pub tag:    String,
    pub layout: String,
    pub stack:  Option<Stack<Window>>
}

#[deriving(Clone)]
impl Workspace {
    pub fn new(id: uint, tag: String, layout: String, stack: Option<Stack<Window>>) -> Workspace {
        Workspace {
            id: id,
            tag: tag,
            layout: layout,
            stack: stack
        }
    }

    pub fn add(&mut self, window: Window) {
        match self.stack {
            Some(ref mut stack) => stack.add(window),
            _ => self.stack = Some(Stack::from_element(window))
        }
    }

    pub fn len(&self) -> uint {
        match self.stack {
            Some(ref s) => s.len(),
            _       => 0
        }
    }

    pub fn contains(&self, window: Window) -> bool {
        match self.stack {
            Some(ref s) => s.contains(window),
            _       => false
        }
    }
}

#[deriving(Clone)]
pub struct Screen {
    pub workspace:     Workspace,
    pub screen_id:     uint,
    pub screen_detail: ScreenDetail
}

impl Screen {
    pub fn new(workspace: Workspace, screen_id: uint, screen_detail: ScreenDetail) -> Screen {
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
}

#[deriving(Clone)]
pub struct Workspaces {
    pub current:  Screen,
    pub visible:  Vec<Screen>,
    pub hidden:   Vec<Workspace>,
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
        let workspaces : Vec<Workspace> = tags.iter()
            .enumerate()
            .map(|(id, tag)| Workspace::new(id, tag.clone(), layout.clone(), None))
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
            .zip(range(0, seen.len()))
            .zip(screens.iter())
            .map(|((a, b), c)| Screen::new(a.clone(), b, c.clone()))
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
    pub fn view(&mut self, index: uint) {
        debug!("Setting focus to {}", index);
        match self.visible.iter().position(|s| s.workspace.id == index) {
            Some(screen_pos) => {
                let screen = self.visible[screen_pos].clone();
                self.visible.remove(screen_pos);
                self.visible.insert(0, self.current.clone());
                self.current = screen;
                return;
            },
            _ => ()
        }

        match self.hidden.iter().position(|w| w.id == index) {
            Some(workspace_pos) => {
                let current_workspace = self.current.workspace.clone();
                self.current.workspace = self.hidden[workspace_pos].clone();
                self.hidden.insert(0, current_workspace);
                return;
            },
            _ => ()
        }
    }

    /// Set focus to the given workspace.  If that workspace does not exist
    /// in the stackset, the original workspace is returned.  If that workspace is
    /// 'hidden', then display that workspace on the current screen, and move the
    /// current workspace to 'hidden'.  If that workspace is 'visible' on another
    /// screen, the workspaces of the current screen and the other screen are
    /// swapped.
    pub fn greedy_view(&mut self, index: uint) {

    }

    pub fn sink(&mut self, window: Window) {
        self.floating.remove(&window);
    }

    pub fn delete(&mut self, window: Window) {
        self.delete_p(window);
        self.sink(window);
    }

    pub fn delete_p(&mut self, window: Window) {
        self.hidden.iter_mut().fold((), |_, workspace| {
            if workspace.stack.is_some() {
                workspace.stack = workspace.clone().stack.unwrap().filter(|&x| x != window);
            }
        });

        self.visible.iter_mut().fold((), |_, screen| {
            if screen.workspace.stack.is_some() {
                screen.workspace.stack = screen.workspace.clone().stack.unwrap().filter(|&x| x != window);
            }
        });

        self.current.workspace.stack = match self.current.workspace.stack {
            Some(ref s) => s.filter(|&x| x != window),
            _           => self.current.workspace.stack.clone()
        };
    }

    pub fn len(&self) -> uint {
        self.current.len() + 
        self.visible.iter().map(|x| x.len()).sum() + 
        self.hidden.iter().map(|x| x.len()).sum() +
        self.floating.len()
    }

    pub fn contains(&self, window: Window) -> bool {
        self.current.contains(window) ||
        self.visible.iter().any(|x| x.contains(window)) ||
        self.hidden.iter().any(|x| x.contains(window)) ||
        self.floating.contains_key(&window)
    }
}
