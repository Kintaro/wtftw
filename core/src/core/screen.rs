use core::workspace::Workspace;
use core::stack::Stack;
use window_system::{ Window, WindowSystem };
use layout::LayoutMessage;
use config::GeneralConfig;
use window_manager::ScreenDetail;

pub struct Screen {
    pub workspace:     Workspace,
    pub screen_id:     u32,
    pub screen_detail: ScreenDetail
}

impl Clone for Screen {
    fn clone(&self) -> Screen {
        Screen {
            workspace: self.workspace.clone(),
            screen_id: self.screen_id,
            screen_detail: self.screen_detail.clone()
        }
    }
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
    pub fn len(&self) -> usize {
        self.workspace.len()
    }

    pub fn windows(&self) -> Vec<Window> {
        self.workspace.windows()
    }

    pub fn map_workspace<F>(&self, f: F) -> Screen where F : Fn(Workspace) -> Workspace {
        Screen::new(f(self.workspace.clone()), self.screen_id, self.screen_detail)
    }

    pub fn map<F>(&self, f: F) -> Screen where F : Fn(Stack<Window>) -> Stack<Window> {
        Screen::new(self.workspace.map(f), self.screen_id, self.screen_detail)
    }

    pub fn map_option<F>(&self, f: F) -> Screen where F : Fn(Stack<Window>) -> Option<Stack<Window>> {
        Screen::new(self.workspace.map_option(f), self.screen_id, self.screen_detail)
    }

    pub fn map_or<F>(&self, default: Stack<Window>, f: F) -> Screen where F : Fn(Stack<Window>) -> Stack<Window> {
        Screen::new(self.workspace.map_or(default, f), self.screen_id, self.screen_detail)
    }

    pub fn send_layout_message(&self, message: LayoutMessage, window_system: &WindowSystem,
                                   config: &GeneralConfig) -> Screen {
        Screen::new(self.workspace.send_layout_message(message, window_system, config), self.screen_id, self.screen_detail)
    }
}
