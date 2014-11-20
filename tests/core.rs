extern crate wtftw;

mod stack {
    use wtftw::core::Stack;
    #[test]
    fn from_element() {
        let stack = Stack::<u32>::from_element(42);
        assert!(stack.focus == 42);
        assert!(stack.up.is_empty());
        assert!(stack.down.is_empty());
    }

    #[test]
    fn add() {
        let stack1 = Stack::<u32>::from_element(42);
        let stack2 = stack1.add(23);

        assert!(stack2.down.head().unwrap() == &stack1.focus);
        assert!(stack2.focus == 23);
    }

    #[test]
    fn integrate() {
        let stack = Stack::<u32>::from_element(42);
        assert!(stack.integrate() == vec!(42));

        let stack2 : Stack<u32> = Stack {
            focus: 1,
            up: vec!(2, 3),
            down: vec!(4, 5)
        };
        assert!(stack2.integrate() == vec!(3, 2, 1, 4, 5));
    }

    #[test]
    fn filter_some() {
        let mut stack = Stack::<u32>::from_element(42);
        for i in range(0, 50) {
            stack = stack.add(2 * i + 1);
        }

        assert!(stack.filter(|&x| x % 2 == 0) == Some(Stack::<u32>::from_element(42)))
    }

    #[test]
    fn filter_none() {
        let mut stack = Stack::<u32>::from_element(43);
        for i in range(0, 50) {
            stack = stack.add(2 * i + 1);
        }

        assert!(stack.filter(|&x| x % 2 == 0) == None)
    }

    #[test]
    fn len() {
        let mut stack = Stack::<u32>::from_element(100);
        for i in range(0, 50) {
            stack = stack.add(i);
        }
        assert!(stack.len() == 51);
    }

    #[test]
    fn contains() {
        let stack = Stack::<u32>::from_element(42);
        assert!( stack.contains(42));
        assert!(!stack.contains(23));
    }
}

mod workspace {
    use wtftw::core::Stack;
    use wtftw::core::Workspace;

    #[test]
    fn new() {
        let stack = Stack::<u64>::from_element(42);
        let workspace = Workspace::new(42, String::from_str("test"), String::from_str("tall"), Some(stack));

        assert!(workspace.id == 42);
    }

    #[test]
    fn windows_some() {
        let stack = Stack::<u64>::from_element(42);
        let workspace = Workspace::new(42, String::from_str("test"), String::from_str("tall"), Some(stack.clone()));

        assert!(workspace.windows() == stack.integrate())
    }

    #[test]
    fn windows_none() {
        let workspace = Workspace::new(42, String::from_str("test"), String::from_str("tall"), None);
        assert!(workspace.windows() == Vec::new())
    }
}

mod screen {
    #[test]
    fn new() {
    }
}
