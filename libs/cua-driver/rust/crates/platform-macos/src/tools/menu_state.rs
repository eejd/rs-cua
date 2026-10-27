//! Read-only observation of an element's own menu items.
//!
//! Shared by `set_value` (popup selection) and `click` (settling a successful
//! menu-opening action). Only the addressed element's direct `AXMenu` children
//! are examined, never sibling windows, application menus or the whole tree.

use std::time::{Duration, Instant};

use crate::ax::bindings::{
    copy_children, copy_string_attr, AXUIElementRef, AXUIElementSetMessagingTimeout,
};
use core_foundation::base::CFRelease;

/// Nominal observation budget after a successful menu-opening action.
pub(super) const MENU_PUBLICATION_BUDGET: Duration = Duration::from_millis(1000);
const POLL_INTERVAL: Duration = Duration::from_millis(50);
const QUERY_TIMEOUT: Duration = Duration::from_millis(100);
/// Consecutive positive reads required before reporting `Observed`.
const REQUIRED_POSITIVE_READS: u32 = 2;

/// Outcome of waiting for a menu to publish its items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MenuPublication {
    /// Items became observable after the action.
    Observed,
    /// No items were observed within the budget.
    NotObserved,
    /// Items were observable before the action, so no wait was performed.
    AlreadyObservable,
}

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

/// Wait for `owner`'s menu items to become observable after a successful
/// action. `already_observable` is the pre-dispatch baseline. Never dispatches
/// input; an unreadable or never-published menu ends as `NotObserved`.
pub(super) fn wait_for_menu_publication(
    owner: AXUIElementRef,
    already_observable: bool,
) -> MenuPublication {
    let start = Instant::now();
    poll_publication(
        already_observable,
        MENU_PUBLICATION_BUDGET,
        |remaining| menu_items_observable_before(owner, Instant::now() + remaining),
        || start.elapsed(),
        std::thread::sleep,
    )
}

/// Polling policy, independent of AX so its transitions are testable with an
/// injected clock. `read` receives the remaining budget.
fn poll_publication(
    already_observable: bool,
    budget: Duration,
    mut read: impl FnMut(Duration) -> bool,
    mut elapsed: impl FnMut() -> Duration,
    mut sleep: impl FnMut(Duration),
) -> MenuPublication {
    if already_observable {
        return MenuPublication::AlreadyObservable;
    }
    let mut positive_reads = 0;
    loop {
        let spent = elapsed();
        if spent >= budget {
            return MenuPublication::NotObserved;
        }
        if read(budget - spent) {
            positive_reads += 1;
            if positive_reads >= REQUIRED_POSITIVE_READS {
                return MenuPublication::Observed;
            }
        } else {
            positive_reads = 0;
        }
        // Never sleep past the budget.
        sleep(POLL_INTERVAL.min(budget.saturating_sub(elapsed())));
    }
}

/// Resets a temporary AX messaging timeout to the system default on drop.
struct TimeoutReset(AXUIElementRef);

impl Drop for TimeoutReset {
    fn drop(&mut self) {
        unsafe { AXUIElementSetMessagingTimeout(self.0, 0.0) };
    }
}

/// A per-query AX timeout: the smaller of `QUERY_TIMEOUT` and the time left,
/// never below 1 ms (an AX timeout of 0 means "system default", which would
/// lift the bound). `None` once the deadline has passed.
fn query_timeout(deadline: Instant) -> Option<f32> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return None;
    }
    Some(
        QUERY_TIMEOUT
            .min(remaining)
            .max(Duration::from_millis(1))
            .as_secs_f32(),
    )
}

/// Children of `element`, with the query bounded by [`query_timeout`]. Empty
/// once the deadline has passed.
fn children_before(element: AXUIElementRef, deadline: Instant) -> Vec<AXUIElementRef> {
    let Some(bound) = query_timeout(deadline) else {
        return Vec::new();
    };
    unsafe { AXUIElementSetMessagingTimeout(element, bound) };
    let _reset = TimeoutReset(element);
    unsafe { copy_children(element) }
}

/// Deadline-aware variant of [`menu_items_observable`]: same owner → `AXMenu`
/// → children shape, but every native query is time-bounded and no reference
/// survives, including unvisited children when the deadline expires.
fn menu_items_observable_before(owner: AXUIElementRef, deadline: Instant) -> bool {
    let mut open = false;
    for child in children_before(owner, deadline) {
        if !open {
            let role = query_timeout(deadline).and_then(|bound| {
                unsafe { AXUIElementSetMessagingTimeout(child, bound) };
                let _reset = TimeoutReset(child);
                unsafe { copy_string_attr(child, "AXRole") }
            });
            if role.as_deref() == Some("AXMenu") {
                for item in children_before(child, deadline) {
                    open = true;
                    unsafe { CFRelease(item as _) };
                }
            }
        }
        unsafe { CFRelease(child as _) };
    }
    open
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    const BUDGET: Duration = Duration::from_millis(1000);

    /// Runs the policy on a fake clock that only advances through `sleep`.
    /// `script(elapsed)` answers each read; returns the outcome, elapsed time
    /// at return and the read count.
    fn run(already: bool, script: impl Fn(Duration) -> bool) -> (MenuPublication, Duration, u32) {
        let clock = Cell::new(Duration::ZERO);
        let reads = Cell::new(0);
        let outcome = poll_publication(
            already,
            BUDGET,
            |_| {
                reads.set(reads.get() + 1);
                script(clock.get())
            },
            || clock.get(),
            |d| clock.set(clock.get() + d),
        );
        (outcome, clock.get(), reads.get())
    }

    #[test]
    fn delayed_publication_is_not_reported_early() {
        let delay = Duration::from_millis(300);
        let (outcome, finished, _) = run(false, |t| t >= delay);
        assert_eq!(outcome, MenuPublication::Observed);
        assert!(
            finished >= delay,
            "returned at {finished:?}, before items published"
        );
    }

    #[test]
    fn positive_then_empty_resets_readiness() {
        // Positive only at 0ms and from 200ms on: the early lone read must not count.
        let (outcome, finished, _) = run(false, |t| t.is_zero() || t >= Duration::from_millis(200));
        assert_eq!(outcome, MenuPublication::Observed);
        assert!(finished >= Duration::from_millis(200));
    }

    #[test]
    fn single_positive_read_is_insufficient() {
        let (outcome, _, _) = run(false, |t| t.is_zero());
        assert_eq!(outcome, MenuPublication::NotObserved);
    }

    #[test]
    fn persistent_empty_ends_not_observed_within_budget() {
        let (outcome, finished, reads) = run(false, |_| false);
        assert_eq!(outcome, MenuPublication::NotObserved);
        assert_eq!(finished, BUDGET, "the wait must not sleep past the budget");
        assert!(reads > 1);
    }

    #[test]
    fn late_positive_cannot_extend_the_budget() {
        let (outcome, finished, _) = run(false, |t| t >= Duration::from_millis(960));
        assert_eq!(outcome, MenuPublication::NotObserved);
        assert_eq!(finished, BUDGET);
    }

    #[test]
    fn preexisting_items_skip_the_wait() {
        let (outcome, finished, reads) = run(true, |_| true);
        assert_eq!(outcome, MenuPublication::AlreadyObservable);
        assert_eq!((finished, reads), (Duration::ZERO, 0));
    }
}
