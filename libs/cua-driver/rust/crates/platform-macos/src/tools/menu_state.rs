// SPDX-License-Identifier: MIT

//! Read-only observation of an element's own menu items.
//!
//! Used by `set_value` (popup selection). Only the addressed element's direct
//! `AXMenu` children are examined, never sibling windows, application menus or
//! the whole tree. The post-click menu wait lives in `click` (upstream's
//! `menu_beside`).

use crate::ax::bindings::{copy_children, copy_string_attr, AXUIElementRef};
use core_foundation::base::CFRelease;

/// Whether the element's menu is open right now: an `AXMenu` child that already
/// lists items. A closed AppKit popup has an empty `AXMenu` (or none), and a
/// closed Chromium popup lists only its selected item directly.
pub(super) fn menu_items_observable(owner: AXUIElementRef) -> bool {
    let mut open = false;
    for child in unsafe { copy_children(owner) } {
        if unsafe { copy_string_attr(child, "AXRole") }.as_deref() == Some("AXMenu") {
            let items = unsafe { copy_children(child) };
            open |= !items.is_empty();
            for item in items {
                unsafe { CFRelease(item as _) };
            }
        }
        unsafe { CFRelease(child as _) };
    }
    open
}
