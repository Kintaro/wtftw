
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
