pub fn configure(_: &mut WindowManager, _: &WindowSystem, config: &mut Config) {
    let modm = MOD1MASK;

    config.border_color = 0x7e9014;
    config.focus_border_color = 0xafc81c;
    //config.focus_border_color = 0xaf0000;
    config.terminal = (String::from_str("xfce4-terminal"), String::from_str(""));

    // Register key handlers
    config.add_key_handler(String::from_str("q"), modm | SHIFTMASK,
            box |&: m, w, c| restart(m, w, c));
    config.add_key_handler(String::from_str("Return"), modm | SHIFTMASK,
            box |&: m, w, c| start_terminal(m, w, c));
    config.add_key_handler(String::from_str("p"), modm,
            box |&: m, w, c| start_launcher(m, w, c));

    config.add_key_handler(String::from_str("k"), modm,
            box |&: m: WindowManager, w: &WindowSystem, c: &Config| {
                m.windows(w, c, |x| x.focus_down())
            });

    config.add_key_handler(String::from_str("j"), modm,
            box |&: m: WindowManager, w: &WindowSystem, c: &Config| {
                m.windows(w, c, |x| x.focus_up())
            });


    for i in range(1u, 10) {
        config.add_key_handler(i.to_string(), modm,
            box move |&: m, w, c| switch_to_workspace(m, w, c, i - 1));

        config.add_key_handler(i.to_string(), modm | SHIFTMASK,
            box move |&: m, w, c| move_window_to_workspace(m, w, c, i - 1));
    }
}
