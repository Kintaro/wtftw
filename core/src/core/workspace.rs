use crate::config::GeneralConfig;
use crate::core::stack::Stack;
use crate::layout::{Layout, LayoutMessage};
use crate::window_system::{Window, WindowSystem};

/// Represents a single workspace with a `tag` (name),
/// `id`, a `layout` and a `stack` for all windows
pub struct Workspace {
    pub id: u32,
    pub tag: String,
    pub layout: Box<dyn Layout>,
    pub stack: Option<Stack<Window>>,
}

impl Clone for Workspace {
    fn clone(&self) -> Workspace {
        Workspace {
            id: self.id,
            tag: self.tag.clone(),
            layout: self.layout.copy(),
            stack: self.stack.clone(),
        }
    }
}

impl Workspace {
    /// Create a new workspace
    pub fn new(
        id: u32,
        tag: String,
        layout: Box<dyn Layout>,
        stack: Option<Stack<Window>>,
    ) -> Workspace {
        Workspace {
            id,
            tag,
            layout,
            stack,
        }
    }

    /// Add a new window to the workspace by adding it to the stack.
    /// If the stack doesn't exist yet, create one.
    pub fn add(&self, window: Window) -> Workspace {
        Workspace::new(
            self.id,
            self.tag.clone(),
            self.layout.copy(),
            Some(
                self.stack
                    .clone()
                    .map_or(Stack::from_element(window), |s| s.add(window)),
            ),
        )
    }

    /// Returns the number of windows contained in this workspace
    pub fn len(&self) -> usize {
        self.stack.clone().map_or(0usize, |x| x.len())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

    pub fn map<F>(&self, f: F) -> Workspace
    where
        F: Fn(Stack<Window>) -> Stack<Window>,
    {
        Workspace::new(
            self.id,
            self.tag.clone(),
            self.layout.copy(),
            self.stack.clone().map(|x| f(x)),
        )
    }

    pub fn map_option<F>(&self, f: F) -> Workspace
    where
        F: Fn(Stack<Window>) -> Option<Stack<Window>>,
    {
        Workspace::new(
            self.id,
            self.tag.clone(),
            self.layout.copy(),
            self.stack.clone().and_then(|x| f(x)),
        )
    }

    pub fn map_or<F>(&self, default: Stack<Window>, f: F) -> Workspace
    where
        F: Fn(Stack<Window>) -> Stack<Window>,
    {
        Workspace::new(
            self.id,
            self.tag.clone(),
            self.layout.copy(),
            Some(self.stack.clone().map_or(default, |x| f(x))),
        )
    }

    pub fn send_layout_message(
        &self,
        message: LayoutMessage,
        window_system: &dyn WindowSystem,
        config: &GeneralConfig,
    ) -> Workspace {
        let mut layout = self.layout.copy();
        layout.apply_message(message, window_system, &self.stack, config);
        Workspace::new(self.id, self.tag.clone(), layout, self.stack.clone())
    }
}
