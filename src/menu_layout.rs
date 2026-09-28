//! Measure and position menu panels independently. A flyout must never change
//! its parent's bounds: moving the parent can trigger another row's hover,
//! close the flyout, and repeat on the next frame with a stationary pointer.
use gpui::{
    canvas, point, px, AnyElement, AvailableSpace, Bounds, IntoElement, Pixels, Point, Size, Styled,
};

fn fit_origin(origin: Point<Pixels>, size: Size<Pixels>, viewport: Size<Pixels>) -> Point<Pixels> {
    point(
        origin
            .x
            .max(px(0.))
            .min((viewport.width - size.width).max(px(0.))),
        origin
            .y
            .max(px(0.))
            .min((viewport.height - size.height).max(px(0.))),
    )
}

fn menu_positions(
    requested: Point<Pixels>,
    root_size: Size<Pixels>,
    flyout: Option<(Pixels, Size<Pixels>)>,
    viewport: Size<Pixels>,
) -> (Bounds<Pixels>, Option<Bounds<Pixels>>) {
    let root = Bounds::new(fit_origin(requested, root_size, viewport), root_size);
    let flyout = flyout.map(|(row_offset, size)| {
        let right_room = viewport.width - root.right();
        let left_room = root.left();
        // Prefer the right, but open to the left when it has room. In a window
        // too narrow for both panels, use the larger side and clamp only the flyout.
        let x = if right_room >= size.width || (left_room < size.width && right_room >= left_room) {
            root.right()
        } else {
            root.left() - size.width
        };
        Bounds::new(
            fit_origin(point(x, root.top() + row_offset), size, viewport),
            size,
        )
    });
    (root, flyout)
}

pub(super) fn menu_layer(
    requested: Point<Pixels>,
    mut root: AnyElement,
    mut flyout: Option<(Pixels, AnyElement)>,
) -> impl IntoElement {
    canvas(
        move |_, window, cx| {
            let viewport = window.viewport_size();
            let root_size = root.layout_as_root(AvailableSpace::min_size(), window, cx);
            let flyout_size = flyout.as_mut().map(|(offset, panel)| {
                (
                    *offset,
                    panel.layout_as_root(AvailableSpace::min_size(), window, cx),
                )
            });
            let (root_bounds, flyout_bounds) =
                menu_positions(requested, root_size, flyout_size, viewport);
            root.prepaint_at(root_bounds.origin, window, cx);
            if let (Some((_, panel)), Some(bounds)) = (flyout.as_mut(), flyout_bounds) {
                panel.prepaint_at(bounds.origin, window, cx);
            }
            (root, flyout)
        },
        |_, (mut root, flyout), window, cx| {
            root.paint(window, cx);
            if let Some((_, mut panel)) = flyout {
                panel.paint(window, cx);
            }
        },
    )
    .absolute()
    .size_full()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::size;

    #[test]
    fn opening_and_switching_flyouts_never_moves_the_hovered_root_row() {
        let viewport = size(px(1100.), px(720.));
        let requested = point(px(1000.), px(650.));
        let root_size = size(px(210.), px(500.));
        let (closed, _) = menu_positions(requested, root_size, None, viewport);
        assert_eq!(closed.origin, point(px(890.), px(220.)));
        let pointer = point(closed.left() + px(60.), closed.top() + px(460.));
        assert!(closed.contains(&pointer));
        for (width, height) in [(210., 234.), (320., 400.), (210., 80.)] {
            let (opened, flyout) = menu_positions(
                requested,
                root_size,
                Some((px(450.), size(px(width), px(height)))),
                viewport,
            );
            assert_eq!(opened, closed);
            let flyout = flyout.unwrap();
            assert!(flyout.right() <= closed.left());
            assert!(flyout.bottom() <= viewport.height);
            assert!(!flyout.contains(&pointer));
        }
    }

    #[test]
    fn flyout_aligns_to_parent_row_when_there_is_space_on_the_right() {
        let (root, flyout) = menu_positions(
            point(px(100.), px(80.)),
            size(px(210.), px(400.)),
            Some((px(120.), size(px(210.), px(234.)))),
            size(px(1100.), px(720.)),
        );
        let flyout = flyout.unwrap();
        assert_eq!(flyout.origin, point(root.right(), root.top() + px(120.)));
    }

    #[test]
    fn bottom_edge_and_narrow_windows_clamp_only_the_flyout() {
        let requested = point(px(90.), px(350.));
        let root_size = size(px(210.), px(360.));
        let viewport = size(px(360.), px(720.));
        let (closed, _) = menu_positions(requested, root_size, None, viewport);
        let (opened, flyout) = menu_positions(
            requested,
            root_size,
            Some((px(300.), size(px(210.), px(234.)))),
            viewport,
        );
        assert_eq!(opened, closed);
        let flyout = flyout.unwrap();
        assert!(flyout.left() >= px(0.) && flyout.right() <= viewport.width);
        assert_eq!(flyout.bottom(), viewport.height);
    }
}
