#[macro_use]
extern crate wtftw;
extern crate wtftw_contrib;

use std::ops::Deref;
//use std::ffi::AsOsStr;
use wtftw::window_system::*;
use wtftw::window_manager::*;
use wtftw::handlers::default::*;
use wtftw::config::*;
use wtftw::util::*;
use wtftw::layout::Direction;
use wtftw::layout::LayoutMessage;
use wtftw_contrib::layout::{ AvoidStrutsLayout, LayoutCollection, BinarySpacePartition, GapLayout, MirrorLayout, NoBordersLayout, FullLayout };


#[no_mangle]
pub extern fn configure(_: &mut WindowManager, w: &dyn WindowSystem, config: &mut Config) {
    let modm = KeyModifiers::MOD1MASK;

    config.general.mod_mask = modm;
    config.general.border_color = 0x404040;
    config.general.focus_border_color = 0xebebeb;
    config.general.border_width = 2;
    config.general.terminal = (String::from("urxvt"), String::from(""));
    config.general.layout = LayoutCollection::boxed_new(vec!(
            GapLayout::boxed_new(8, AvoidStrutsLayout::boxed_new(vec!(Direction::Up, Direction::Down), BinarySpacePartition::boxed_new())),
            GapLayout::boxed_new(8, AvoidStrutsLayout::boxed_new(vec!(Direction::Up, Direction::Down), MirrorLayout::boxed_new(BinarySpacePartition::boxed_new()))),
            NoBordersLayout::boxed_new(Box::new(FullLayout))));

    config.general.tags = (vec!("一: ターミナル", "二: ウェブ", "三: コード",
                                "四: メディア", "五: スチーム", "六: ラテック",
                                "七: 音楽", "八: im", "九: 残り"))
        .into_iter().map(String::from).collect();

    // Register key handlers

    // Some standard key handlers for starting, restarting, etc.
    add_key_handler_str!(config, w, "q",      modm | KeyModifiers::SHIFTMASK, exit);
    add_key_handler_str!(config, w, "q",      modm,             restart);
    add_key_handler_str!(config, w, "Return", modm | KeyModifiers::SHIFTMASK, start_terminal);
    add_key_handler_str!(config, w, "p",      modm,             start_launcher);

    // Focus and window movement
    add_key_handler_str!(config, w, "j", modm,             |m, w, c| m.windows(w.deref(), c, &|x| x.focus_down()));
    add_key_handler_str!(config, w, "k", modm,             |m, w, c| m.windows(w.deref(), c, &|x| x.focus_up()));
    add_key_handler_str!(config, w, "j", modm | KeyModifiers::SHIFTMASK, |m, w, c| m.windows(w.deref(), c, &|x| x.swap_down()));
    add_key_handler_str!(config, w, "k", modm | KeyModifiers::SHIFTMASK, |m, w, c| m.windows(w.deref(), c, &|x| x.swap_up()));
    add_key_handler_str!(config, w, "Return", modm,        |m, w, c| m.windows(w.deref(), c, &|x| x.swap_master()));
    add_key_handler_str!(config, w, "c", modm, |m, w, c| m.kill_window(w.deref()).windows(w.deref(), c, &|x| x.clone()));

    add_key_handler_str!(config, w, "t", modm, |m, w, c| {
        match m.workspaces.peek() {
            Some(window) => m.windows(w.deref(), c, &|x| x.sink(window)),
            None => m
        }
    });

    // Layout messages
    add_key_handler_str!(config, w, "h",      modm,             send_layout_message!(LayoutMessage::Decrease));
    add_key_handler_str!(config, w, "l",      modm,             send_layout_message!(LayoutMessage::Increase));
    add_key_handler_str!(config, w, "z",      modm,             send_layout_message!(LayoutMessage::DecreaseSlave));
    add_key_handler_str!(config, w, "a",      modm,             send_layout_message!(LayoutMessage::IncreaseSlave));
    add_key_handler_str!(config, w, "x",      modm | KeyModifiers::SHIFTMASK, send_layout_message!(LayoutMessage::IncreaseGap));
    add_key_handler_str!(config, w, "s",      modm | KeyModifiers::SHIFTMASK, send_layout_message!(LayoutMessage::DecreaseGap));
    add_key_handler_str!(config, w, "comma",  modm,             send_layout_message!(LayoutMessage::IncreaseMaster));
    add_key_handler_str!(config, w, "period", modm,             send_layout_message!(LayoutMessage::DecreaseMaster));
    add_key_handler_str!(config, w, "space",  modm,             send_layout_message!(LayoutMessage::Next));
    add_key_handler_str!(config, w, "space",  modm | KeyModifiers::SHIFTMASK, send_layout_message!(LayoutMessage::Prev));
    add_key_handler_str!(config, w, "r",      modm,             send_layout_message!(LayoutMessage::TreeRotate));
    add_key_handler_str!(config, w, "s",      modm,             send_layout_message!(LayoutMessage::TreeSwap));
    add_key_handler_str!(config, w, "u",      modm | KeyModifiers::SHIFTMASK, send_layout_message!(LayoutMessage::TreeExpandTowards(Direction::Left)));
    add_key_handler_str!(config, w, "p",      modm | KeyModifiers::SHIFTMASK, send_layout_message!(LayoutMessage::TreeExpandTowards(Direction::Right)));
    add_key_handler_str!(config, w, "i",      modm | KeyModifiers::SHIFTMASK, send_layout_message!(LayoutMessage::TreeExpandTowards(Direction::Down)));
    add_key_handler_str!(config, w, "o",      modm | KeyModifiers::SHIFTMASK, send_layout_message!(LayoutMessage::TreeExpandTowards(Direction::Up)));
    add_key_handler_str!(config, w, "u",      modm | KeyModifiers::CONTROLMASK, send_layout_message!(LayoutMessage::TreeShrinkFrom(Direction::Left)));
    add_key_handler_str!(config, w, "p",      modm | KeyModifiers::CONTROLMASK, send_layout_message!(LayoutMessage::TreeShrinkFrom(Direction::Right)));
    add_key_handler_str!(config, w, "i",      modm | KeyModifiers::CONTROLMASK, send_layout_message!(LayoutMessage::TreeShrinkFrom(Direction::Down)));
    add_key_handler_str!(config, w, "o",      modm | KeyModifiers::CONTROLMASK, send_layout_message!(LayoutMessage::TreeShrinkFrom(Direction::Up)));


    // Workspace switching and moving
    for i in 1usize..10 {
        add_key_handler_str!(config, w, &i.to_string(), modm,
        move |m, w, c| switch_to_workspace(m, w, c, i - 1));

        add_key_handler_str!(config, w, &i.to_string(), modm | KeyModifiers::SHIFTMASK,
        move |m, w, c| move_window_to_workspace(m, w, c, i - 1));
    }

    // Media keys
    add_key_handler_str!(config, w, "j", modm | KeyModifiers::CONTROLMASK, run!("amixer", "-q set Master 5%-"));
    add_key_handler_str!(config, w, "k", modm | KeyModifiers::CONTROLMASK, run!("amixer", "-q set Master 5%+"));

    add_key_handler_code!(config, 0x1008ff11, KeyModifiers::NONEMASK, run!("amixer", "-q set Master 5%-"));
    add_key_handler_code!(config, 0x1008ff13, KeyModifiers::NONEMASK, run!("amixer", "-q set Master 5%+"));

    add_key_handler_code!(config, 0x1008ff02, KeyModifiers::NONEMASK, run!("xbacklight", "+10"));
    add_key_handler_code!(config, 0x1008ff03, KeyModifiers::NONEMASK, run!("xbacklight", "-10"));

    add_mouse_handler!(config, BUTTON1, modm,
                       |m, w, c, s| {
                           m.focus(s, w.deref(), c).mouse_move_window(w.deref(), c, s).windows(w.deref(), c, &|x| x.shift_master())
                       });
    add_mouse_handler!(config, BUTTON3, modm,
                       |m, w, c, s| {
                           m.focus(s, w.deref(), c).mouse_resize_window(w.deref(), c, s).windows(w.deref(), c, &|x| x.shift_master())
                       });
}
