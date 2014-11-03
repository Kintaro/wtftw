use std::collections::TreeMap;
use window_manager::ScreenDetail;
use window_system::Window;

#[deriving(Clone)]
pub struct RationalRect(f32, f32, f32, f32);

#[deriving(Clone)]
pub struct Stack<T> {
    focus: T,
    up:    Vec<T>,
    down:  Vec<T>
}

#[deriving(Clone)]
pub struct Workspace {
    id:     uint,
    tag:    String,
    layout: String,
    stack:  Option<Stack<Window>>
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
}

#[deriving(Clone)]
pub struct Screen {
    workspace: Workspace,
    screen_id:    uint,
    screen_detail: ScreenDetail
}

impl Screen {
    pub fn new(workspace: Workspace, screen_id: uint, screen_detail: ScreenDetail) -> Screen {
        Screen {
            workspace: workspace,
            screen_id: screen_id,
            screen_detail: screen_detail
        }
    }
}

#[deriving(Clone)]
pub struct Workspaces {
    current: Screen,
    visible: Vec<Screen>,
    hidden: Vec<Workspace>,
    floating: TreeMap<uint, RationalRect>
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
}
