#![feature(unboxed_closures)]
#![feature(plugin)]
#![feature(box_syntax)]
#![feature(std_misc, collections, path_ext)]
#[macro_use]
#[macro_use]
extern crate wtftw;
extern crate wtftw_contrib;

use std::fs::PathExt;
use std::io::Write;
use std::path::Path;
use std::env;
use std::ffi::AsOsStr;
use wtftw::window_system::*;
use wtftw::window_manager::*;
use wtftw::handlers::default::*;
use wtftw::config::*;
use wtftw::util::*;
use wtftw::layout::Direction;
use wtftw::layout::LayoutMessage;
use wtftw_contrib::layout::{ AvoidStrutsLayout, LayoutCollection, BinarySpacePartition, GapLayout, MirrorLayout, NoBordersLayout, FullLayout };


#[no_mangle]
pub extern fn configure(_: &mut WindowManager, w: &WindowSystem, config: &mut Config) {
    let modm = MOD1MASK;

    config.general.mod_mask = modm;
    config.general.border_color = 0x404040;
    config.general.focus_border_color = 0xebebeb;
    config.general.border_width = 1;
    config.general.terminal = (String::from_str("termite"), String::from_str(""));
    config.general.layout = LayoutCollection::new(vec!(
            GapLayout::new(0, AvoidStrutsLayout::new(vec!(Direction::Up, Direction::Down), BinarySpacePartition::new())),
            GapLayout::new(0, AvoidStrutsLayout::new(vec!(Direction::Up, Direction::Down), MirrorLayout::new(BinarySpacePartition::new()))),
            NoBordersLayout::new(box FullLayout)));

    config.general.tags = (vec!("一: ターミナル", "二: ウェブ", "三: コード",
                                "四: メディア", "五: スチーム", "六: ラテック",
                                "七: 音楽", "八: im", "九: 残り"))
        .into_iter().map(String::from_str).collect();

    // Register key handlers

    // Some standard key handlers for starting, restarting, etc.
    add_key_handler_str!(config, w, "q",      modm | SHIFTMASK, exit);
    add_key_handler_str!(config, w, "q",      modm,             restart);
    add_key_handler_str!(config, w, "Return", modm | SHIFTMASK, start_terminal);
    add_key_handler_str!(config, w, "p",      modm,             start_launcher);

    // Focus and window movement
    add_key_handler_str!(config, w, "j", modm,             |m, w, c| m.windows(w, c, |x| x.focus_down()));
    add_key_handler_str!(config, w, "k", modm,             |m, w, c| m.windows(w, c, |x| x.focus_up()));
    add_key_handler_str!(config, w, "j", modm | SHIFTMASK, |m, w, c| m.windows(w, c, |x| x.swap_down()));
    add_key_handler_str!(config, w, "k", modm | SHIFTMASK, |m, w, c| m.windows(w, c, |x| x.swap_up()));
    add_key_handler_str!(config, w, "Return", modm,        |m, w, c| m.windows(w, c, |x| x.swap_master()));
    add_key_handler_str!(config, w, "c", modm, |m, w, c| m.kill_window(w).windows(w, c, |x| x.clone()));

    add_key_handler_str!(config, w, "t", modm, |m, w, c| {
        match m.workspaces.peek() {
            Some(window) => m.windows(w, c, |x| x.sink(window)),
            None => m.clone()
        }
    });

    // Layout messages
    add_key_handler_str!(config, w, "h",      modm,             send_layout_message!(LayoutMessage::Decrease));
    add_key_handler_str!(config, w, "l",      modm,             send_layout_message!(LayoutMessage::Increase));
    add_key_handler_str!(config, w, "z",      modm,             send_layout_message!(LayoutMessage::DecreaseSlave));
    add_key_handler_str!(config, w, "a",      modm,             send_layout_message!(LayoutMessage::IncreaseSlave));
    add_key_handler_str!(config, w, "x",      modm | SHIFTMASK, send_layout_message!(LayoutMessage::IncreaseGap));
    add_key_handler_str!(config, w, "s",      modm | SHIFTMASK, send_layout_message!(LayoutMessage::DecreaseGap));
    add_key_handler_str!(config, w, "comma",  modm,             send_layout_message!(LayoutMessage::IncreaseMaster));
    add_key_handler_str!(config, w, "period", modm,             send_layout_message!(LayoutMessage::DecreaseMaster));
    add_key_handler_str!(config, w, "space",  modm,             send_layout_message!(LayoutMessage::Next));
    add_key_handler_str!(config, w, "space",  modm | SHIFTMASK, send_layout_message!(LayoutMessage::Prev));
    add_key_handler_str!(config, w, "r",      modm,             send_layout_message!(LayoutMessage::TreeRotate));
    add_key_handler_str!(config, w, "s",      modm,             send_layout_message!(LayoutMessage::TreeSwap));
    add_key_handler_str!(config, w, "u",      modm | SHIFTMASK, send_layout_message!(LayoutMessage::TreeExpandTowards(Direction::Left)));
    add_key_handler_str!(config, w, "p",      modm | SHIFTMASK, send_layout_message!(LayoutMessage::TreeExpandTowards(Direction::Right)));
    add_key_handler_str!(config, w, "i",      modm | SHIFTMASK, send_layout_message!(LayoutMessage::TreeExpandTowards(Direction::Down)));
    add_key_handler_str!(config, w, "o",      modm | SHIFTMASK, send_layout_message!(LayoutMessage::TreeExpandTowards(Direction::Up)));
    add_key_handler_str!(config, w, "u",      modm | CONTROLMASK, send_layout_message!(LayoutMessage::TreeShrinkFrom(Direction::Left)));
    add_key_handler_str!(config, w, "p",      modm | CONTROLMASK, send_layout_message!(LayoutMessage::TreeShrinkFrom(Direction::Right)));
    add_key_handler_str!(config, w, "i",      modm | CONTROLMASK, send_layout_message!(LayoutMessage::TreeShrinkFrom(Direction::Down)));
    add_key_handler_str!(config, w, "o",      modm | CONTROLMASK, send_layout_message!(LayoutMessage::TreeShrinkFrom(Direction::Up)));


    // Workspace switching and moving
    for i in (1usize..10) {
        add_key_handler_str!(config, w, &i.to_string(), modm,
        move |m, w, c| switch_to_workspace(m, w, c, i - 1));

        add_key_handler_str!(config, w, &i.to_string(), modm | SHIFTMASK,
        move |m, w, c| move_window_to_workspace(m, w, c, i - 1));
    }

    // Media keys
    add_key_handler_str!(config, w, "j", modm | CONTROLMASK, run!("amixer", "-q set Master 5%-"));
    add_key_handler_str!(config, w, "k", modm | CONTROLMASK, run!("amixer", "-q set Master 5%+"));

    add_key_handler_code!(config, 0x1008ff11, NONEMASK, run!("amixer", "-q set Master 5%-"));
    add_key_handler_code!(config, 0x1008ff13, NONEMASK, run!("amixer", "-q set Master 5%+"));

    add_key_handler_code!(config, 0x1008ff02, NONEMASK, run!("xbacklight", "+10"));
    add_key_handler_code!(config, 0x1008ff03, NONEMASK, run!("xbacklight", "-10"));

    add_mouse_handler!(config, BUTTON1, modm,
                       |m, w, c, s| {
                           m.focus(s, w, c).mouse_move_window(w, c, s).windows(w, c, |x| x.shift_master())
                       });
    add_mouse_handler!(config, BUTTON3, modm,
                       |m, w, c, s| {
                           m.focus(s, w, c).mouse_resize_window(w, c, s).windows(w, c, |x| x.shift_master())
                       });

    // Place specific applications on specific workspaces
    config.set_manage_hook(box |workspaces, window_system, window| {
        match &window_system.get_class_name(window)[..] {
            "MPlayer" => spawn_on(workspaces, window_system, window, 3),
            "vlc"     => spawn_on(workspaces, window_system, window, 3),
            _         => workspaces.clone()
        }
    });

    // Xmobar handling and formatting
    let home_dir = env::home_dir().unwrap();
    let home = home_dir.as_os_str().to_str().unwrap();
    let xmobar_config = format!("{}/.xmonad/xmobar1.hs", home);

    if Path::new(&xmobar_config.clone()).is_file() {
        let mut xmobar = spawn_pipe(config, "xmobar",
                                    vec!(xmobar_config));
        let tags = config.general.tags.clone();
        config.set_log_hook(box move |m, w| {
            let p = &mut xmobar;
            let tags = &tags;
            let workspaces = tags.clone().iter()
                .enumerate()
                .map(|(i, _)| if i as u32 == m.workspaces.current.workspace.id {
                    format!("<fc=#fb4934>■</fc>")
                } else if m.workspaces.visible.iter().any(|w| w.workspace.id == i as u32) {
                    format!("<fc=#fabd2f>■</fc>")
                } else {
                    format!("■")
                })
            .fold(String::from_str(""), |a, x| {
                let mut r = a.clone();
                r.push_str(&x);
                r
            });

            let name = match m.workspaces.peek() {
                None => String::from_str(""),
                Some(window) => w.get_window_name(window)
            };
            let content = format!("{} {} {}\n", workspaces, m.workspaces.current.workspace.layout.description(), name);
            match p.write().unwrap().stdin.as_mut() {
                Some(pin) => {
                    match pin.write(content.as_bytes()) {
                        _      => ()
                    };
                    pin.flush().unwrap();
                },
                _ => ()
            }
        });
    };
}
