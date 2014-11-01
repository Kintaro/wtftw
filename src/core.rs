use std::collections::TreeMap;

#[deriving(Clone)]
struct RationalRect(f32, f32, f32, f32);

#[deriving(Clone)]
struct Window;

#[deriving(Clone)]
struct Stack<T> {
    focus: T,
    up:    Vec<T>,
    down:  Vec<T>
}

#[deriving(Clone)]
struct Workspace {
    tag:    String,
    layout: String,
    stack:  Option<Stack<Window>>
}

#[deriving(Clone)]
impl Workspace {
    fn new(tag: String, layout: String, stack: Option<Stack<Window>>) -> Workspace {
        Workspace {
            tag: tag,
            layout: layout,
            stack: stack
        }
    }
}

#[deriving(Clone)]
struct Screen {
    workspace: Workspace,
    screen:    uint,
    screen_detail: String
}

impl Screen {
    fn new(workspace: Workspace, screen: uint, screen_detail: String) -> Screen {
        Screen {
            workspace: workspace,
            screen: screen,
            screen_detail: screen_detail
        }
    }
}

#[deriving(Clone)]
struct Workspaces {
    current: Screen,
    visible: Vec<Screen>,
    hidden: Vec<Workspace>,
    floating: TreeMap<uint, RationalRect>
}

impl Workspaces {
    fn new(layout: String, tags: Vec<String>, screens: Vec<String>) -> Workspaces {
        let workspaces : Vec<Workspace> = tags.iter()
            .map(|tag| Workspace::new(tag.clone(), layout.clone(), None))
            .collect();
        let seen   : Vec<Workspace> = workspaces.iter().take(screens.len()).map(|x| x.clone()).collect();
        let unseen : Vec<Workspace> = workspaces.iter().skip(screens.len()).map(|x| x.clone()).collect();

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
}
