#![feature(unboxed_closures)]
#![feature(globs)]
#![feature(phase)]
#![feature(macro_rules)]
#[phase(plugin, link)]
extern crate log;
#[phase(plugin)]
extern crate wtftw_core;

extern crate wtftw_core;

use wtftw_core::window_system::*;
use wtftw_core::window_manager::*;
use wtftw_core::handlers::default::*;
use wtftw_core::config::*;
use wtftw_core::util::*;
use wtftw_core::layout::*;
use wtftw_core::layout::Direction::*;
use wtftw_core::layout::LayoutMessage::*;

#[no_mangle]
pub extern fn configure(_: &mut WindowManager, w: &WindowSystem, config: &mut Config) {
    let modm = MOD1MASK;

    config.general.mod_mask = modm;
    config.general.spacing = 0;
    config.general.border_color = 0x3f3f4c;
    config.general.focus_border_color = 0x525263;
    config.general.terminal = (String::from_str("urxvt"), String::from_str(""));
    config.general.layout = LayoutCollection::new(vec!(
        GapLayout::new(16, AvoidStrutsLayout::new(vec!(Direction::Up), ResizableTallLayout::new())),
        GapLayout::new(16, AvoidStrutsLayout::new(vec!(Direction::Up), MirrorLayout::new(ResizableTallLayout::new()))), 
        box FullLayout));

    config.general.tags = (vec!("1: term", "2: web", "3: code",
                                "4: media", "5: steam", "6: latex",
                                "7: music", "8: im", "9: rest"))
        .into_iter().map(String::from_str).collect();

    // Register key handlers

    // Some standard key handlers for starting, restarting, etc.
    add_key_handler_str!(config, w, "q",      modm | SHIFTMASK, exit);
    add_key_handler_str!(config, w, "q",      modm,             restart);
    add_key_handler_str!(config, w, "Return", modm | SHIFTMASK, start_terminal);
    add_key_handler_str!(config, w, "p",      modm,             start_launcher);

    // Focus and window movement
    add_key_handler_str!(config, w, "j", modm, |&: m, w, c| m.windows(w, c, |x| x.focus_down()));
    add_key_handler_str!(config, w, "k", modm, |&: m, w, c| m.windows(w, c, |x| x.focus_up()));
    add_key_handler_str!(config, w, "j", modm | SHIFTMASK, |&: m, w, c| m.windows(w, c, |x| x.swap_down()));
    add_key_handler_str!(config, w, "k", modm | SHIFTMASK, |&: m, w, c| m.windows(w, c, |x| x.swap_up()));
    add_key_handler_str!(config, w, "Return", modm, |&: m, w, c| m.windows(w, c, |x| x.swap_master()));

    // Layout messages
    add_key_handler_str!(config, w, "h", modm,
            |&: m, w, c| { m.send_layout_message(LayoutMessage::Decrease).reapply_layout(w, c) });
    add_key_handler_str!(config, w, "l", modm,
            |&: m, w, c| { m.send_layout_message(LayoutMessage::Increase).reapply_layout(w, c) });
    add_key_handler_str!(config, w, "comma", modm,
            |&: m, w, c| { m.send_layout_message(LayoutMessage::IncreaseMaster).reapply_layout(w, c) });
    add_key_handler_str!(config, w, "period", modm,
            |&: m, w, c| m.send_layout_message(LayoutMessage::DecreaseMaster).reapply_layout(w, c));
    add_key_handler_str!(config, w, "space", modm,
            |&: m, w, c| m.send_layout_message(LayoutMessage::Next).reapply_layout(w, c));

    // Workspace switching and moving
    for i in range(1u, 10) {
        add_key_handler_str!(config, w, i.to_string().as_slice(), modm,
            move |&: m, w, c| switch_to_workspace(m, w, c, i - 1));

        add_key_handler_str!(config, w, i.to_string().as_slice(), modm | SHIFTMASK,
            move |&: m, w, c| move_window_to_workspace(m, w, c, i - 1));
    }

    // Media keys
    add_key_handler_str!(config, w, "j", modm | CONTROLMASK,
            |&: w, _, _| { run("amixer", Some("-q set Master 5%-")); w });
    add_key_handler_str!(config, w, "k", modm | CONTROLMASK,
            |&: w, _, _| { run("amixer", Some("-q set Master 5%+")); w });

    add_key_handler_code!(config, 0x1008ff11, NONEMASK,
            |&: w, _, _| { run("amixer", Some("-q set Master 5%-")); w });
    add_key_handler_code!(config, 0x1008ff13, NONEMASK,
            |&: w, _, _| { run("amixer", Some("-q set Master 5%+")); w });

    add_key_handler_code!(config, 0x1008ff02, NONEMASK,
            |&: w, _, _| { run("xbacklight", Some("+10")); w });
    add_key_handler_code!(config, 0x1008ff03, NONEMASK,
            |&: w, _, _| { run("xbacklight", Some("-10")); w });

    // Place specific applications on specific workspaces
    config.set_manage_hook(box |&: workspaces, window_system, window| {
                match window_system.get_class_name(window).as_slice() {
                    "MPlayer" => spawn_on(workspaces, window_system, window, 3),
                    "vlc"     => spawn_on(workspaces, window_system, window, 3),
                    _         => workspaces.clone()
                }
            });

    // Xmobar handling and formatting
    let mut xmobar = spawn_pipe(config, String::from_str("xmobar"),
                                Some(String::from_str("/home/wollwage/.xmonad/xmobar1.hs")));
    let tags = config.general.tags.clone();
    config.set_log_hook(box move |&mut: m, _| {
        let p = &mut xmobar;
        let tags = &tags;
        let workspaces = tags.clone().iter()
            .enumerate()
            .map(|(i, x)| if i as u32 == m.workspaces.current.workspace.id {
                format!("[<fc=#3279a8>{}</fc>] ", x)
            } else {
                format!("[{}] ", x)
            })
            .fold(String::from_str(""), |a, x| {
                let mut r = a.clone();
                r.push_str(x.as_slice());
                r
            });

        let content = format!("{} {}", workspaces, m.workspaces.current.workspace.layout.description());
        p.stdin.as_mut().unwrap().write_line(content.as_slice()).unwrap();
    });
}

