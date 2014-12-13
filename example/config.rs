#![feature(unboxed_closures)]
#![feature(globs)]
#![feature(phase)]
#[phase(plugin, link)]

extern crate log;
extern crate wtftw_core;

use wtftw_core::core::*;
use wtftw_core::window_system::*;
use wtftw_core::window_manager::*;
use wtftw_core::handlers::*;
use wtftw_core::handlers::default::*;
use wtftw_core::config::*;
use wtftw_core::util::*;
use wtftw_core::layout::*;
use wtftw_core::layout::Direction::*;
use wtftw_core::layout::LayoutMessage::*;

use std::io::process::Command;

#[no_mangle]
pub extern fn configure(m: &mut WindowManager, w: &WindowSystem, config: &mut Config) {
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
    config.add_key_handler(w.get_keycode_from_string("q"), modm.clone() | SHIFTMASK,
            box |&: m, w, c| exit(m, w, c));
    config.add_key_handler(w.get_keycode_from_string("q"), modm,
            box |&: m, w, c| restart(m, w, c));
    config.add_key_handler(w.get_keycode_from_string("Return"), modm | SHIFTMASK,
            box |&: m, w, c| start_terminal(m, w, c));
    config.add_key_handler(w.get_keycode_from_string("p"), modm,
            box |&: m, w, c| start_launcher(m, w, c));

    config.add_key_handler(w.get_keycode_from_string("j"), modm,
            box |&: m: WindowManager, w: &WindowSystem, c: &GeneralConfig| {
                m.windows(w, c, |x| x.focus_down())
            });

    config.add_key_handler(w.get_keycode_from_string("k"), modm,
            box |&: m: WindowManager, w: &WindowSystem, c: &GeneralConfig| {
                m.windows(w, c, |x| x.focus_up())
            });

    config.add_key_handler(w.get_keycode_from_string("j"), modm | SHIFTMASK,
            box |&: m: WindowManager, w: &WindowSystem, c: &GeneralConfig| {
                m.windows(w, c, |x| x.swap_down())
            });

    config.add_key_handler(w.get_keycode_from_string("k"), modm | SHIFTMASK,
            box |&: m: WindowManager, w: &WindowSystem, c: &GeneralConfig| {
                m.windows(w, c, |x| x.swap_up())
            });

    config.add_key_handler(w.get_keycode_from_string("Return"), modm,
            box |&: m: WindowManager, w: &WindowSystem, c: &GeneralConfig| {
                m.windows(w, c, |x| x.swap_master())
            });

    config.add_key_handler(w.get_keycode_from_string("h"), modm,
            box |&: m: WindowManager, w: &WindowSystem, c: &GeneralConfig| {
                m.send_layout_message(LayoutMessage::Decrease)
                 .reapply_layout(w, c)
            });

    config.add_key_handler(w.get_keycode_from_string("l"), modm,
            box |&: m: WindowManager, w: &WindowSystem, c: &GeneralConfig| {
                m.send_layout_message(LayoutMessage::Increase)
                 .reapply_layout(w, c)
            });

    config.add_key_handler(w.get_keycode_from_string("comma"), modm,
            box |&: m: WindowManager, w: &WindowSystem, c: &GeneralConfig| {
                m.send_layout_message(LayoutMessage::IncreaseMaster)
                 .reapply_layout(w, c)
            });

    config.add_key_handler(w.get_keycode_from_string("period"), modm,
            box |&: m: WindowManager, w: &WindowSystem, c: &GeneralConfig| {
                m.send_layout_message(LayoutMessage::DecreaseMaster)
                 .reapply_layout(w, c)
            });
    config.add_key_handler(w.get_keycode_from_string("space"), modm,
            box |&: m: WindowManager, w: &WindowSystem, c: &GeneralConfig| {
                m.send_layout_message(LayoutMessage::Next)
                 .reapply_layout(w, c)
            });

    for i in range(1u, 10) {
        config.add_key_handler(w.get_keycode_from_string(i.to_string().as_slice()), modm,
            box move |&: m, w, c| switch_to_workspace(m, w, c, i - 1));

        config.add_key_handler(w.get_keycode_from_string(i.to_string().as_slice()), modm | SHIFTMASK,
            box move |&: m, w, c| move_window_to_workspace(m, w, c, i - 1));
    }

    config.add_key_handler(w.get_keycode_from_string("j"), modm | CONTROLMASK,
            box |&: w: WindowManager, _: &WindowSystem, _: &GeneralConfig| {
                run(String::from_str("amixer"), Some(String::from_str("-q set Master 5%-")));
                w
            });

    config.add_key_handler(w.get_keycode_from_string("k"), modm | CONTROLMASK,
            box |&: w: WindowManager, _: &WindowSystem, _: &GeneralConfig| {
                run(String::from_str("amixer"), Some(String::from_str("-q set Master 5%+")));
                w
            });

    config.add_key_handler(0x1008ff11, NONEMASK,
            box |&: w: WindowManager, _: &WindowSystem, _: &GeneralConfig| {
                run(String::from_str("amixer"), Some(String::from_str("-q set Master 5%-")));
                w
            });
    config.add_key_handler(0x1008ff13, NONEMASK,
            box |&: w: WindowManager, _: &WindowSystem, _: &GeneralConfig| {
                run(String::from_str("amixer"), Some(String::from_str("-q set Master 5%+")));
                w
            });

    config.add_key_handler(0x1008ff02, NONEMASK,
            box |&: w: WindowManager, _: &WindowSystem, _: &GeneralConfig| {
                run(String::from_str("xbacklight"), Some(String::from_str("+10")));
                w
            });

    config.add_key_handler(0x1008ff03, NONEMASK,
            box |&: w: WindowManager, _: &WindowSystem, _: &GeneralConfig| {
                run(String::from_str("xbacklight"), Some(String::from_str("-10")));
                w
            });

    config.set_manage_hook(box |&: workspaces: Workspaces, window_system: &WindowSystem,
                           window: Window| {
                match window_system.get_class_name(window).as_slice() {
                    "MPlayer" => spawn_on(workspaces, window_system, window, 3),
                    "vlc"     => spawn_on(workspaces, window_system, window, 3),
                    _         => workspaces.clone()
                }
            });

    let mut xmobar = spawn_pipe(config, String::from_str("xmobar"),
                                Some(String::from_str("/home/wollwage/.xmonad/xmobar1.hs")));
    let tags = config.general.tags.clone();
    let color = config.general.focus_border_color.clone();
    config.set_log_hook(box move |&mut: m: WindowManager, w: &WindowSystem| {
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
        p.stdin.as_mut().unwrap().write_line(content.as_slice());
    });
}

