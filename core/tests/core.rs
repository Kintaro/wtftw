#![feature(collections)]

extern crate wtftw_core;

use wtftw_core::layout::FullLayout;
use wtftw_core::core::stack::Stack;
use wtftw_core::core::workspace::Workspace;

#[test]
fn stack_add() {
    let s1 = Stack::from_element(42);
    let s2 = s1.add(23);

    assert!(s2.focus == 23);
    assert!(s1.up == s2.up);
    assert!(s1.focus == s2.down[0]);
}

#[test]
fn stack_integrate() {
    let s1 = Stack::new(1, vec!(2, 3), vec!(4, 5, 6));
    assert!(s1.integrate() == vec!(3, 2, 1, 4, 5, 6));
}

#[test]
fn stack_filter() {
    let s1 = Stack::new(1, vec!(2, 3), vec!(4, 5, 6));
    let s2 = s1.filter(|&x| x != 3);

    assert!(s2.is_some());
    assert!(s2.unwrap() == Stack::new(1, vec!(2), vec!(4, 5, 6)));
}

#[test]
fn stack_focus_up() {
    let s1 = Stack::new(1, vec!(2, 3), vec!(4, 5, 6));
    let s2 = s1.focus_up();

    assert!(s2 == Stack::new(2, vec!(3), vec!(1, 4, 5, 6)));
}

#[test]
fn stack_focus_down() {
    let s1 = Stack::new(1, vec!(2, 3), vec!(4, 5, 6));
    let s2 = s1.focus_down();

    assert!(s2 == Stack::new(4, vec!(1, 2, 3), vec!(5, 6)));
}

#[test]
fn stack_reverse() {
    let v1 = vec!(2, 3);
    let v2 = vec!(4, 5, 6);

    let s1 = Stack::new(1, v1.clone(), v2.clone());
    let s2 = Stack::new(1, v2.clone(), v1.clone());

    assert!(s1.reverse() == s2);
    assert!(s2.reverse() == s1);
    assert!(s1.reverse().reverse() == s1);
}

#[test]
fn stack_len() {
    let s1 = Stack::new(1, vec!(2, 3), vec!(4, 5, 6));

    assert!(s1.len() == 6);
}

#[test]
fn stack_contains() {
    let s1 = Stack::new(42, vec!(2, 3), vec!(4, 5, 6));
    let s2 = Stack::new(23, Vec::new(), Vec::new());

    assert!(s1.contains(42));
    assert!(!s1.contains(23));
    assert!(!s2.contains(42));
    assert!(s2.contains(23));
}

#[test]
fn workspace_contains() {
    let s1 = Stack::new(42, vec!(2, 3), vec!(4, 5, 6));
    let w1 = Workspace::new(1, String::from_str("Foo"), Box::new(FullLayout), Some(s1));
    let w2 = Workspace::new(1, String::from_str("Foo"), Box::new(FullLayout), None);

    assert!(w1.contains(42));
    assert!(!w1.contains(23));
    assert!(!w2.contains(2));
}
