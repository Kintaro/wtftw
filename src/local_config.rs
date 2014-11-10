pub fn configure(window_manager: &mut WindowManager, window_system: &WindowSystem, config: &mut Config) {
    // Set border color and width
    config.focus_border_color = 0x00ffffff;
    config.mod_mask = Mod1Mask;

    let modm = config.mod_mask;

    // Register key handlers
    config.add_key_handler(KeyCommand { key: String::from_str("Return"), mask: modm | ShiftMask }, 
            box |&: m, w, c| start_terminal(m, w, c));
    config.add_key_handler(KeyCommand { key: String::from_str("p"), mask: modm | ShiftMask },
            box |&: m: WindowManager, w, c| start_launcher(m, w, c));

    for i in range(1u, 10) {
        let ref index = i;
        config.add_key_handler(KeyCommand { key: i.to_string(), mask: modm }, 
            box move |&: m, w, c| switch_to_workspace(m, w, c, i - 1));

        config.add_key_handler(KeyCommand { key: i.to_string(), mask: modm | ShiftMask },
            box move |&: m, w, c| move_window_to_workspace(m, w, c, i - 1));
    }
}
