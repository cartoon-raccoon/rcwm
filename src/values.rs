use xcb;

use crate::xserver::XWindowID;

pub const ROOT_ATTRS: [(u32, u32); 1] = [
    (
        xcb::CW_EVENT_MASK, 
        xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT | xcb::EVENT_MASK_STRUCTURE_NOTIFY
    )
];

pub fn cursor_attrs(cursor_id: u32) -> [(u32, u32); 1] {
    debug!("Getting cursor attrs for cursor {}", cursor_id);
    return [(xcb::CW_CURSOR, cursor_id)]
}

pub fn disable_events() -> [(u32, u32); 1] {
    debug!("VALUES: attributes no events");
    return [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_NO_EVENT)];
}

pub fn child_events() -> [(u32, u32); 1] {
    debug!("VALUES: attributes child events");
    return [(xcb::CW_EVENT_MASK,
        xcb::EVENT_MASK_ENTER_WINDOW|    // -> Self EnterNotify events
        xcb::EVENT_MASK_STRUCTURE_NOTIFY // -> Self CirculateNotify, ConfigureNotify, DestroyNotify, GravityNotify, MapNotify, ReparentNotify, UnmapNotify events
    )];
}

pub fn configure_move(x: u32, y: u32) -> [(u16, u32); 2] {
    debug!("VALUES: configure move");
    return [(xcb::CONFIG_WINDOW_X as u16, x), (xcb::CONFIG_WINDOW_Y as u16, y)];
}

pub fn configure_resize(width: u32, height: u32) -> [(u16, u32); 2] {
    debug!("VALUES: configure resize");
    return [(xcb::CONFIG_WINDOW_WIDTH as u16, width), (xcb::CONFIG_WINDOW_HEIGHT as u16, height)];
}

pub fn stack_above_sibling(window_id: XWindowID) -> [(u16, u32); 2] {
    debug!("VALUES: configure stack above sibling {}", window_id);
    return [
        (xcb::CONFIG_WINDOW_STACK_MODE as u16, xcb::STACK_MODE_ABOVE),
        (xcb::CONFIG_WINDOW_SIBLING as u16, window_id),
    ];
}