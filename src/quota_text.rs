//! Fixed-column typography for the compact quota display.
use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, RECT};
use windows::Win32::Graphics::Gdi::*;

use crate::native_interop::{self, Color};

const DIGIT_CELL: i32 = 8;
const PERCENT_CELL: i32 = 12;
const PERCENT_WIDTH: i32 = DIGIT_CELL * 3 + PERCENT_CELL;
const COLUMN_GAP: i32 = 8;

fn parts(text: &str) -> (&str, &str) {
    if let Some(end) = text.find('%') {
        if text[..end].chars().all(|c| c.is_ascii_digit()) && end > 0 {
            return (
                &text[..=end],
                text[end + 1..].trim_start_matches([' ', '·']),
            );
        }
    }
    (text, "")
}

fn percentage_cells(text: &str) -> Option<String> {
    let digits = text.strip_suffix('%')?;
    if digits.is_empty() || digits.len() > 3 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("{digits:>3}%"))
}

fn text(hdc: HDC, value: &str, mut rect: RECT, flags: DRAW_TEXT_FORMAT) {
    let mut wide: Vec<u16> = value.encode_utf16().collect();
    unsafe {
        let _ = DrawTextW(
            hdc,
            &mut wide,
            &mut rect,
            flags | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
        );
    }
}

/// Each digit occupies a fixed cell; reset text starts at the same x for
/// 0%, 9%, 99% and 100%. Status labels remain in the percentage column.
pub fn draw(
    hdc: HDC,
    x: i32,
    y: i32,
    height: i32,
    width: i32,
    value: &str,
    primary: &Color,
    is_dark: bool,
    scale: impl Fn(i32) -> i32,
) {
    let (percentage, reset) = parts(value);
    let rect = RECT {
        left: x,
        top: y,
        right: x + scale(PERCENT_WIDTH),
        bottom: y + height,
    };
    unsafe {
        let _ = SetTextColor(hdc, COLORREF(primary.to_colorref()));
    }
    if let Some(cells) = percentage_cells(percentage) {
        for (index, glyph) in cells.chars().enumerate() {
            if glyph == ' ' {
                continue;
            }
            let left = x + scale(index as i32 * DIGIT_CELL);
            let cell_width = if glyph == '%' {
                PERCENT_CELL
            } else {
                DIGIT_CELL
            };
            text(
                hdc,
                &glyph.to_string(),
                RECT {
                    left,
                    right: left + scale(cell_width),
                    ..rect
                },
                DT_CENTER,
            );
        }
    } else {
        text(hdc, percentage, rect, DT_RIGHT);
    }
    if reset.is_empty() {
        return;
    }

    let reset_x = x + scale(PERCENT_WIDTH + COLUMN_GAP);
    let face = native_interop::wide_str("Microsoft YaHei UI");
    unsafe {
        let font = CreateFontW(
            -scale(12),
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            OUT_TT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            CLEARTYPE_QUALITY.0 as u32,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            PCWSTR(face.as_ptr()),
        );
        let old_font = SelectObject(hdc, font);
        let secondary = Color::from_hex(if is_dark { "#B5B5B5" } else { "#666666" });
        let _ = SetTextColor(hdc, COLORREF(secondary.to_colorref()));
        text(
            hdc,
            reset,
            RECT {
                left: reset_x,
                right: x + scale(width),
                ..rect
            },
            DT_LEFT,
        );
        SelectObject(hdc, old_font);
        let _ = DeleteObject(font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentages_keep_the_same_digit_and_percent_slots() {
        assert_eq!(percentage_cells("2%"), Some("  2%".into()));
        assert_eq!(percentage_cells("87%"), Some(" 87%".into()));
        assert_eq!(percentage_cells("100%"), Some("100%".into()));
        assert_eq!(percentage_cells("--"), None);
    }

    #[test]
    fn separates_percentage_from_reset_without_changing_status_labels() {
        assert_eq!(parts("2%  01:59重置"), ("2%", "01:59重置"));
        assert_eq!(parts("87%  10/08重置"), ("87%", "10/08重置"));
        assert_eq!(parts("18% · 2h 34m"), ("18%", "2h 34m"));
        assert_eq!(parts("--"), ("--", ""));
        assert_eq!(parts("网络"), ("网络", ""));
    }
}
