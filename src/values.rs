use xcb;

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