//! Own the system Quick Look panel inside the app instead of spawning qlmanage.
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSEvent, NSEventType, NSResponder, NSView, NSWindowDelegate};
use objc2_foundation::{NSObject, NSObjectNSDelayedPerforming, NSObjectProtocol, NSString, NSURL};
use objc2_quick_look_ui::{
    QLPreviewItem, QLPreviewPanel, QLPreviewPanelDataSource, QLPreviewPanelDelegate,
};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

#[derive(Default)]
struct Session {
    paths: Vec<PathBuf>,
    pane: usize,
    stride: usize,
    last_index: usize,
    pending: bool,
}

define_class!(
    // NSResponder has no additional subclassing requirements. AppKit calls
    // these methods on the main thread; all panel state stays on that thread.
    #[unsafe(super(NSResponder))]
    #[thread_kind = MainThreadOnly]
    #[name = "ShuffleFlowQuickLookController"]
    #[ivars = RefCell<Session>]
    struct Controller;

    unsafe impl NSObjectProtocol for Controller {}
    unsafe impl NSWindowDelegate for Controller {}

    impl Controller {
        #[unsafe(method(selectPreviewItem:))]
        fn select_item(&self, _: Option<&NSObject>) {
            select_item(self);
        }
        #[unsafe(method(togglePreviewPanel:))]
        fn toggle_panel(&self, _: Option<&NSObject>) {
            toggle_panel(self);
        }
        #[unsafe(method(acceptsPreviewPanelControl:))]
        fn accepts(&self, _: &QLPreviewPanel) -> bool {
            !self.ivars().borrow().paths.is_empty()
        }
        #[unsafe(method(beginPreviewPanelControl:))]
        fn begin(&self, panel: &QLPreviewPanel) {
            unsafe {
                panel.setDataSource(Some(ProtocolObject::from_ref(self)));
                panel.setDelegate(Some(self));
            }
        }
        #[unsafe(method(endPreviewPanelControl:))]
        fn end(&self, panel: &QLPreviewPanel) {
            unsafe { panel.setDelegate(None); panel.setDataSource(None); }
        }
    }
    unsafe impl QLPreviewPanelDataSource for Controller {
        #[unsafe(method(numberOfPreviewItemsInPreviewPanel:))]
        fn count(&self, _: Option<&QLPreviewPanel>) -> isize {
            self.ivars().borrow().paths.len() as isize
        }
        #[unsafe(method_id(previewPanel:previewItemAtIndex:))]
        fn item(&self, _: Option<&QLPreviewPanel>, index: isize)
            -> Option<Retained<ProtocolObject<dyn QLPreviewItem>>> {
            item_for_index(&self.ivars().borrow().paths, index)
        }
    }
    unsafe impl QLPreviewPanelDelegate for Controller {
        #[unsafe(method(previewPanel:handleEvent:))]
        fn event(&self, panel: Option<&QLPreviewPanel>, event: Option<&NSEvent>) -> bool {
            let stride = self.ivars().borrow().stride;
            handle_event(panel, event, stride)
        }
    }
);

// AppKit can run nested event loops while loading a Quick Look generator.
// Present outside GPUI's App/Entity borrow, on the next native run-loop turn.
fn toggle_panel(controller: &Controller) {
    let index = controller.ivars().borrow().last_index;
    unsafe {
        if let Some(panel) = QLPreviewPanel::sharedPreviewPanel(controller.mtm()) {
            if panel.isVisible() {
                panel.orderOut(None);
            } else {
                panel.updateController();
                panel.reloadData();
                panel.setCurrentPreviewItemIndex(index as isize);
                panel.makeKeyAndOrderFront(None);
            }
        }
        controller.ivars().borrow_mut().pending = false;
    }
}

fn select_item(controller: &Controller) {
    let index = controller.ivars().borrow().last_index;
    unsafe {
        let Some(panel) = QLPreviewPanel::sharedPreviewPanel(controller.mtm()) else {
            return;
        };
        if panel.isVisible() && panel.currentPreviewItemIndex() != index as isize {
            panel.setCurrentPreviewItemIndex(index as isize);
        }
    }
}

fn item_for_index(
    paths: &[PathBuf],
    index: isize,
) -> Option<Retained<ProtocolObject<dyn QLPreviewItem>>> {
    let path = paths.get(usize::try_from(index).ok()?)?;
    let url = NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
    Some(ProtocolObject::from_retained(url))
}

fn handle_event(panel: Option<&QLPreviewPanel>, event: Option<&NSEvent>, stride: usize) -> bool {
    let (Some(panel), Some(event)) = (panel, event) else {
        return false;
    };
    if event.r#type() != NSEventType::KeyDown {
        return false;
    }
    let delta = match event.keyCode() {
        126 => -(stride.max(1) as isize),
        125 => stride.max(1) as isize,
        49 | 53 => {
            panel.orderOut(None);
            return true;
        }
        _ => return false,
    };
    let index = unsafe { panel.currentPreviewItemIndex() };
    let count = unsafe {
        panel
            .dataSource()
            .map_or(0, |s| s.numberOfPreviewItemsInPreviewPanel(Some(panel)))
    };
    let next = step(index, delta, count.max(0) as usize);
    unsafe {
        panel.setCurrentPreviewItemIndex(next as isize);
    }
    true
}

thread_local! {
    static CONTROLLER: RefCell<Option<Retained<Controller>>> = const { RefCell::new(None) };
}

fn step(index: isize, delta: isize, count: usize) -> usize {
    (index.saturating_add(delta)).clamp(0, count.saturating_sub(1) as isize) as usize
}

pub(super) fn show(
    view: *mut std::ffi::c_void,
    pane: usize,
    paths: Vec<PathBuf>,
    selected: &Path,
    stride: usize,
) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    if paths.is_empty() || view.is_null() {
        return;
    }
    CONTROLLER.with(|cell| {
        let mut slot = cell.borrow_mut();
        let controller = slot.get_or_insert_with(|| {
            let allocated = Controller::alloc(mtm).set_ivars(RefCell::new(Session::default()));
            unsafe { msg_send![super(allocated), init] }
        });
        let index = paths.iter().position(|p| p == selected).unwrap_or(0);
        *controller.ivars().borrow_mut() = Session {
            paths,
            pane,
            stride,
            last_index: index,
            pending: true,
        };
        // Insert our controller into the native view's responder chain, keeping
        // GPUI's existing responder intact. The controller is retained above.
        unsafe {
            let view = &*(view as *const NSView);
            let next = view.nextResponder();
            let already_attached = next
                .as_ref()
                .is_some_and(|r| std::ptr::eq::<NSResponder>(&**r, &***controller));
            if !already_attached {
                controller.setNextResponder(next.as_deref());
                view.setNextResponder(Some(controller));
            }
            controller.performSelector_withObject_afterDelay(sel!(togglePreviewPanel:), None, 0.0);
        }
    });
}

pub(super) fn select(path: &Path) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    CONTROLLER.with(|cell| {
        let slot = cell.borrow();
        let Some(controller) = slot.as_ref() else {
            return;
        };
        unsafe {
            if !QLPreviewPanel::sharedPreviewPanelExists(mtm) {
                return;
            }
            let Some(panel) = QLPreviewPanel::sharedPreviewPanel(mtm) else {
                return;
            };
            if !panel.isVisible() {
                return;
            }
            let mut state = controller.ivars().borrow_mut();
            if let Some(index) = state.paths.iter().position(|p| p == path) {
                state.last_index = index;
                drop(state);
                controller.performSelector_withObject_afterDelay(
                    sel!(selectPreviewItem:),
                    None,
                    0.0,
                );
            }
        }
    });
}

/// Called only while the panel is open. Synchronizes native Left/Right and
/// delegate Up/Down navigation back to the file browser, and frees the session
/// paths when the panel closes.
pub(super) fn poll() -> (bool, Option<(usize, PathBuf)>) {
    let Some(mtm) = MainThreadMarker::new() else {
        return (false, None);
    };
    CONTROLLER.with(|cell| {
        let slot = cell.borrow();
        let Some(controller) = slot.as_ref() else {
            return (false, None);
        };
        let mut state = controller.ivars().borrow_mut();
        if state.pending {
            return (true, None);
        }
        unsafe {
            let panel = QLPreviewPanel::sharedPreviewPanel(mtm);
            if !panel.as_ref().is_some_and(|p| p.isVisible()) {
                state.paths = Vec::new();
                return (false, None);
            }
            let index = panel.unwrap().currentPreviewItemIndex();
            let Ok(index) = usize::try_from(index) else {
                return (true, None);
            };
            if index == state.last_index {
                return (true, None);
            }
            state.last_index = index;
            (
                true,
                state
                    .paths
                    .get(index)
                    .cloned()
                    .map(|path| (state.pane, path)),
            )
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arrows_stop_at_edges_and_follow_icon_rows() {
        assert_eq!(step(0, -1, 5), 0);
        assert_eq!(step(4, 1, 5), 4);
        assert_eq!(step(1, 3, 8), 4);
        assert_eq!(step(4, -3, 8), 1);
    }
}
