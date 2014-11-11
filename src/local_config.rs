pub fn configure(_: &mut WindowManager, _: &WindowSystem, config: &mut Config) {
    let modm = Mod4Mask;

    // Register key handlers
    config.add_key_handler(KeyCommand::new(String::from_str("Return"), modm | ShiftMask), 
            box |&: m, w, c| start_terminal(m, w, c));
    config.add_key_handler(KeyCommand::new(String::from_str("p"), modm),
            box |&: m, w, c| start_launcher(m, w, c));

    config.add_key_handler(KeyCommand::new(String::from_str("j"), modm),
            box |&: m: WindowManager, w: &WindowSystem, c: &Config| { 
                let mut wm = m.clone();
                wm.windows(w, c, |x| {
                    let mut xm = x.clone();
                    xm.focus_down();
                    xm
                });
                wm
            });

    config.add_key_handler(KeyCommand::new(String::from_str("k"), modm),
            box |&: m: WindowManager, w: &WindowSystem, c: &Config| { 
                let mut wm = m.clone();
                wm.windows(w, c, |x| {
                    let mut xm = x.clone();
                    xm.focus_up();
                    xm
                });
                wm
            });


    for i in range(1u, 10) {
        config.add_key_handler(KeyCommand::new(i.to_string(), modm), 
            box move |&: m, w, c| switch_to_workspace(m, w, c, i - 1));

        config.add_key_handler(KeyCommand::new(i.to_string(), modm | ShiftMask),
            box move |&: m, w, c| move_window_to_workspace(m, w, c, i - 1));
    }
}
