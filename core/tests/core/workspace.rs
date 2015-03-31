#![feature(collections)]

extern crate wtftw_core;

use wtftw_core::layout::FullLayout;
use wtftw_core::core::stack::Stack;
use wtftw_core::core::workspace::Workspace;

#[test]
fn workspace_contains() {
    let s1 = Stack::new(42, vec!(2, 3), vec!(4, 5, 6));
    let w1 = Workspace::new(1, String::from_str("Foo"), Box::new(FullLayout), Some(s1));
    let w2 = Workspace::new(1, String::from_str("Foo"), Box::new(FullLayout), None);

    assert!(w1.contains(42));
    assert!(!w1.contains(23));
    assert!(!w2.contains(2));
}
