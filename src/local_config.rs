pub fn configure(_: &mut WindowManager, w: &WindowSystem, config: &mut Config) {
    let modm = MOD1MASK;

    config.border_color = 0x7e9014;
    config.focus_border_color = 0xafc81c;
    //config.focus_border_color = 0xaf0000;
    config.terminal = (String::from_str("xfce4-terminal"), String::from_str(""));

    // Register key handlers
    config.add_key_handler(w.get_keycode_from_string("q"), modm | SHIFTMASK,
            box |&: m, w, c| exit(m, w, c));
    config.add_key_handler(w.get_keycode_from_string("q"), modm,
            box |&: m, w, c| restart(m, w, c));
    config.add_key_handler(w.get_keycode_from_string("Return"), modm | SHIFTMASK,
            box |&: m, w, c| start_terminal(m, w, c));
    config.add_key_handler(w.get_keycode_from_string("p"), modm,
            box |&: m, w, c| start_launcher(m, w, c));

    config.add_key_handler(w.get_keycode_from_string("j"), modm,
            box |&: m: WindowManager, w: &WindowSystem, c: &Config| {
                m.windows(w, c, |x| x.focus_down())
            });

    config.add_key_handler(w.get_keycode_from_string("k"), modm,
            box |&: m: WindowManager, w: &WindowSystem, c: &Config| {
                m.windows(w, c, |x| x.focus_up())
            });

    config.add_key_handler(w.get_keycode_from_string("j"), modm | SHIFTMASK,
            box |&: m: WindowManager, w: &WindowSystem, c: &Config| {
                m.windows(w, c, |x| x.swap_down())
            });

    config.add_key_handler(w.get_keycode_from_string("k"), modm | SHIFTMASK,
            box |&: m: WindowManager, w: &WindowSystem, c: &Config| {
                m.windows(w, c, |x| x.swap_up())
            });

    config.add_key_handler(w.get_keycode_from_string("Return"), modm,
            box |&: m: WindowManager, w: &WindowSystem, c: &Config| {
                m.windows(w, c, |x| x.swap_master())
            });


    for i in range(1u, 10) {
        config.add_key_handler(w.get_keycode_from_string(i.to_string().as_slice()), modm,
            box move |&: m, w, c| switch_to_workspace(m, w, c, i - 1));

        config.add_key_handler(w.get_keycode_from_string(i.to_string().as_slice()), modm | SHIFTMASK,
            box move |&: m, w, c| move_window_to_workspace(m, w, c, i - 1));
    }

    config.add_key_handler(w.get_keycode_from_string("j"), modm | CONTROLMASK,
            box |&: w: WindowManager, _: &WindowSystem, _: &Config| {
                run(String::from_str("amixer"), Some(String::from_str("-q set Master 5%-")));
                w
            });

    config.add_key_handler(w.get_keycode_from_string("k"), modm | CONTROLMASK,
            box |&: w: WindowManager, _: &WindowSystem, _: &Config| {
                run(String::from_str("amixer"), Some(String::from_str("-q set Master 5%+")));
                w
            });

    config.add_key_handler(0x1008ff11, NONEMASK,
            box |&: w: WindowManager, _: &WindowSystem, _: &Config| {
                run(String::from_str("amixer"), Some(String::from_str("-q set Master 5%-")));
                w
            });
    config.add_key_handler(0x1008ff13, NONEMASK,
            box |&: w: WindowManager, _: &WindowSystem, _: &Config| {
                run(String::from_str("amixer"), Some(String::from_str("-q set Master 5%+")));
                w
            });

    config.set_manage_hook(box |&: workspaces: Workspaces, window_system: &WindowSystem, window: Window| {
                match window_system.get_class_name(window).as_slice() {
                    "MPlayer" => spawn_on(workspaces, window_system, window, 3),
                    _         => workspaces.clone()
                }
            });
}
