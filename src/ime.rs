//! Native macOS text input for inline names. Platform ranges use UTF-16;
//! Shuffle's existing cursor/selection use Unicode scalar indices.
use super::*;
use gpui::{
    canvas, ElementInputHandler, EntityInputHandler, Pixels, Point, TextRun, UTF16Selection,
};
use std::ops::Range;

fn to_utf16(text: &str, chars: usize) -> usize {
    text.chars().take(chars).map(char::len_utf16).sum()
}
fn from_utf16(text: &str, offset: usize) -> usize {
    let mut units = 0;
    let mut chars = 0;
    for ch in text.chars() {
        if units + ch.len_utf16() > offset {
            break;
        }
        units += ch.len_utf16();
        chars += 1;
    }
    chars
}
fn from_range(text: &str, range: Range<usize>) -> Range<usize> {
    let start = from_utf16(text, range.start);
    start..from_utf16(text, range.end).max(start)
}
fn to_range(text: &str, range: Range<usize>) -> Range<usize> {
    to_utf16(text, range.start)..to_utf16(text, range.end)
}

impl Rename {
    fn selection(&self) -> Range<usize> {
        let anchor = self.anchor.unwrap_or(self.cursor);
        anchor.min(self.cursor)..anchor.max(self.cursor)
    }
    fn replace(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        marked: bool,
    ) {
        let range = range
            .map(|r| from_range(&self.text, r))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection());
        let start = range.start;
        self.text.replace_range(
            char_byte(&self.text, start)..char_byte(&self.text, range.end),
            text,
        );
        let end = start + text.chars().count();
        self.marked = (marked && !text.is_empty()).then_some(start..end);
        if let Some(selected) = selected {
            let selected = from_range(text, selected);
            self.cursor = start + selected.end;
            self.anchor = (selected.start != selected.end).then_some(start + selected.start);
        } else {
            self.cursor = end;
            self.anchor = None;
        }
        self.input_layout = None;
    }
}

pub(super) fn input_canvas(entity: gpui::Entity<Shuffle>, focus: FocusHandle) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, cx| {
            window.handle_input(&focus, ElementInputHandler::new(bounds, entity.clone()), cx);
            entity.update(cx, |this, _| {
                if let Some(r) = this.rename.as_mut() {
                    let style = window.text_style();
                    let line = window.text_system().shape_line(
                        r.text.clone().into(),
                        style.font_size.to_pixels(window.rem_size()),
                        &[TextRun {
                            len: r.text.len(),
                            font: style.font(),
                            color: style.color,
                            background_color: None,
                            underline: None,
                            strikethrough: None,
                        }],
                        None,
                    );
                    r.input_layout = Some((bounds, line));
                }
            });
        },
    )
    .absolute()
    .size_full()
}

impl EntityInputHandler for Shuffle {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let r = self.rename.as_ref()?;
        let range = from_range(&r.text, range);
        *actual = Some(to_range(&r.text, range.clone()));
        Some(r.text[char_byte(&r.text, range.start)..char_byte(&r.text, range.end)].into())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let r = self.rename.as_ref()?;
        Some(UTF16Selection {
            range: to_range(&r.text, r.selection()),
            reversed: r.anchor.is_some_and(|a| a > r.cursor),
        })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        let r = self.rename.as_ref()?;
        r.marked.clone().map(|range| to_range(&r.text, range))
    }
    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(r) = self.rename.as_mut() {
            r.marked = None;
        }
        cx.notify();
    }
    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(r) = self.rename.as_mut() {
            let text: String = text.chars().filter(|c| !c.is_control()).collect();
            r.replace(range, &text, None, false);
            cx.notify();
        }
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(r) = self.rename.as_mut() {
            r.replace(range, text, selected, true);
            cx.notify();
        }
    }
    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        fallback: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let r = self.rename.as_ref()?;
        let Some((bounds, line)) = &r.input_layout else {
            return Some(fallback);
        };
        let range = from_range(&r.text, range);
        Some(Bounds::from_corners(
            point(
                bounds.left() + line.x_for_index(char_byte(&r.text, range.start)),
                bounds.top(),
            ),
            point(
                bounds.left() + line.x_for_index(char_byte(&r.text, range.end)),
                bounds.bottom(),
            ),
        ))
    }
    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let r = self.rename.as_ref()?;
        let (bounds, line) = r.input_layout.as_ref()?;
        let index = line.closest_index_for_x(point.x - bounds.left());
        Some(r.text[..index].encode_utf16().count())
    }
}

impl Shuffle {
    pub(super) fn text_editing(&self) -> bool {
        self.rename.is_some()
            || self.palette_open
            || self.active_tab().editing_path.is_some()
            || self.active_tab().find_query.is_some()
            || self.term_focused
            || self.server_dialog.is_some()
            || self.group_dialog.is_some()
            || self.batch_rename.is_some()
    }
    pub(super) fn dispatch_file_menu(
        &mut self,
        action: KeyAction,
        key: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.text_editing() {
            self.on_key(
                &KeyDownEvent {
                    keystroke: gpui::Keystroke::parse(&format!("cmd-{key}")).unwrap(),
                    is_held: false,
                },
                window,
                cx,
            );
        } else if self.context_menu.is_none()
            && self.confirm_delete.is_none()
            && self.info_panel.is_none()
        {
            self.run_key_action(action, window, cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rename(text: &str) -> Rename {
        Rename {
            pane: 0,
            path: PathBuf::from("文件.txt"),
            text: text.into(),
            cursor: text.chars().count(),
            anchor: Some(0),
            marked: None,
            input_layout: None,
        }
    }
    #[test]
    fn composition_replaces_selection_and_commits_once() {
        let mut r = rename("untitled folder");
        r.replace(None, "zhong", Some(5..5), true);
        r.replace(None, "中文", Some(2..2), true);
        assert_eq!(r.text, "中文");
        r.replace(None, "中文资料", None, false);
        assert_eq!((r.text.as_str(), r.cursor), ("中文资料", 4));
        assert!(r.marked.is_none());
    }
    #[test]
    fn utf16_replacement_handles_emoji_and_mixed_names() {
        let mut r = rename("中😀文.txt");
        r.replace(Some(1..3), "国", None, false);
        assert_eq!(r.text, "中国文.txt");
        assert_eq!(to_range("中😀文", 1..2), 1..3);
        assert_eq!(from_range("中😀文", 1..3), 1..2);
        assert_eq!(from_range("中😀文", 99..100), 3..3);
    }
    #[test]
    fn marked_selection_is_relative_to_inserted_text() {
        let mut r = rename("前缀.txt");
        r.cursor = 2;
        r.anchor = None;
        r.replace(None, "😀中文", Some(2..3), true);
        assert_eq!(r.text, "前缀😀中文.txt");
        assert_eq!(r.selection(), 3..4);
        r.replace(None, "资料", None, false);
        assert_eq!(r.text, "前缀资料.txt");
    }
}
