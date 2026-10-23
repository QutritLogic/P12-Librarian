//! Bank sheets: a list of patches as CSV, or as an A4 PDF to print and keep
//! next to the synth.

use std::fs;
use std::path::PathBuf;

use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetRow {
    /// `U1:05`, or a File pane label like `edit 3`.
    slot: String,
    name: String,
    layer_b: String,
    favourite: bool,
    tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct SheetSection {
    /// A bank, e.g. `U1`, whose slots are then shown without the `U1:`; or empty.
    bank: String,
    rows: Vec<SheetRow>,
}

#[derive(Deserialize)]
pub struct Sheet {
    /// What the patches are: "Prophet 12", or the file name.
    title: String,
    /// One line under the title, e.g. where and when they were read.
    note: String,
    sections: Vec<SheetSection>,
}

/// Writes a bank sheet as `.csv` or `.pdf`, chosen by the file's extension.
#[tauri::command]
pub fn export_sheet(sheet: Sheet, path: PathBuf) -> Result<(), String> {
    let pdf = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("pdf"));
    let data = if pdf { to_pdf(&sheet) } else { to_csv(&sheet).into_bytes() };
    fs::write(&path, data).map_err(|e| format!("Writing {}: {e}", path.display()))
}

// ───────────────────────────── CSV ─────────────────────────────

fn csv_field(text: &str) -> String {
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_owned()
    }
}

/// RFC 4180 CSV with a header row. Starts with a byte order mark so
/// spreadsheet programs read tags with non-English letters as UTF-8.
fn to_csv(sheet: &Sheet) -> String {
    let mut out = String::from("\u{feff}Slot,Name,Layer B,Favourite,Tags\r\n");
    for row in sheet.sections.iter().flat_map(|s| &s.rows) {
        let fields = [
            row.slot.as_str(),
            &row.name,
            &row.layer_b,
            if row.favourite { "yes" } else { "" },
            &row.tags.join("; "),
        ];
        let line: Vec<String> = fields.iter().map(|f| csv_field(f)).collect();
        out.push_str(&line.join(","));
        out.push_str("\r\n");
    }
    out
}

// ───────────────────────────── PDF ─────────────────────────────

// A4 in points, laid out in two columns of 50 rows: one bank fits a page.
const PAGE: (f32, f32) = (595.0, 842.0);
const MARGIN: f32 = 42.0;
const GUTTER: f32 = 20.0;
const ROWS_PER_COLUMN: usize = 50;
const ROW_HEIGHT: f32 = 13.4;
const TEXT_SIZE: f32 = 8.0;
/// Courier is monospaced: every character is 0.6 em wide.
const CHAR_WIDTH: f32 = 0.6 * TEXT_SIZE;
const GREY: f32 = 0.45;

// The standard PDF fonts, which every reader has, so nothing is embedded.
const SANS: Name = Name(b"F1");
const SANS_BOLD: Name = Name(b"F2");
const MONO: Name = Name(b"F3");
const DINGBATS: Name = Name(b"F4");
/// ★ in ZapfDingbats.
const STAR: &[u8] = b"H";

/// Text in the fonts' WinAnsi encoding: Latin-1 characters as they are, `·`
/// too, anything else as `?`.
fn win_ansi(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| match c as u32 {
            0x20..=0x7e | 0xa0..=0xff => c as u8,
            _ => b'?',
        })
        .collect()
}

/// One page: a heading and up to two columns of rows.
struct PageText<'a> {
    heading: String,
    /// Slot labels with the bank left out where the section is one bank.
    rows: Vec<(String, &'a SheetRow)>,
}

fn pages(sheet: &Sheet) -> Vec<PageText<'_>> {
    let mut pages = Vec::new();
    for section in &sheet.sections {
        let prefix = format!("{}:", section.bank);
        let heading = if section.bank.is_empty() {
            sheet.title.clone()
        } else {
            format!("{} \u{b7} {}", sheet.title, section.bank)
        };
        let rows: Vec<(String, &SheetRow)> = section
            .rows
            .iter()
            .map(|row| {
                let slot = match row.slot.strip_prefix(&prefix) {
                    Some(program) if !section.bank.is_empty() => program,
                    _ => &row.slot,
                };
                (slot.to_owned(), row)
            })
            .collect();
        for chunk in rows.chunks(2 * ROWS_PER_COLUMN) {
            pages.push(PageText {
                heading: heading.clone(),
                rows: chunk.to_vec(),
            });
        }
    }
    if pages.is_empty() {
        pages.push(PageText {
            heading: sheet.title.clone(),
            rows: Vec::new(),
        });
    }
    pages
}

fn text(content: &mut Content, font: Name, size: f32, (x, y): (f32, f32), s: &[u8]) {
    content
        .begin_text()
        .set_font(font, size)
        .next_line(x, y)
        .show(Str(s))
        .end_text();
}

fn page_content(page: &PageText, note: &str, number: usize, count: usize) -> Vec<u8> {
    let mut c = Content::new();
    let top = PAGE.1 - MARGIN;
    text(&mut c, SANS_BOLD, 16.0, (MARGIN, top - 14.0), &win_ansi(&page.heading));
    c.set_fill_gray(GREY);
    text(&mut c, SANS, 9.0, (MARGIN, top - 30.0), &win_ansi(note));

    let column_width = (PAGE.0 - 2.0 * MARGIN - GUTTER) / 2.0;
    let slot_chars = page.rows.iter().map(|(s, _)| s.len()).max().unwrap_or(2).max(2);
    let star_x = slot_chars as f32 * CHAR_WIDTH + 5.0;
    let name_x = star_x + 10.0;
    let layer_b_x = name_x + 21.0 * CHAR_WIDTH;
    let header_y = top - 56.0;
    let first_row_y = header_y - ROW_HEIGHT - 2.0;

    for (column, rows) in page.rows.chunks(ROWS_PER_COLUMN).enumerate() {
        let x = MARGIN + column as f32 * (column_width + GUTTER);
        c.set_fill_gray(GREY);
        text(&mut c, SANS_BOLD, 7.5, (x, header_y), b"Slot");
        text(&mut c, SANS_BOLD, 7.5, (x + name_x, header_y), b"Name");
        text(&mut c, SANS_BOLD, 7.5, (x + layer_b_x, header_y), b"Layer B");
        c.set_stroke_gray(0.7)
            .set_line_width(0.5)
            .move_to(x, header_y - 4.0)
            .line_to(x + column_width, header_y - 4.0)
            .stroke();

        for (i, (slot, row)) in rows.iter().enumerate() {
            let y = first_row_y - i as f32 * ROW_HEIGHT;
            // Every other row shaded, to follow a line across.
            if i % 2 == 1 {
                c.set_fill_gray(0.94)
                    .rect(x - 2.0, y - 4.0, column_width + 4.0, ROW_HEIGHT)
                    .fill_nonzero();
            }
            c.set_fill_gray(GREY);
            text(&mut c, MONO, TEXT_SIZE, (x, y), &win_ansi(slot));
            text(&mut c, MONO, TEXT_SIZE, (x + layer_b_x, y), &win_ansi(&row.layer_b));
            c.set_fill_gray(0.0);
            if row.favourite {
                text(&mut c, DINGBATS, 7.0, (x + star_x, y + 0.5), STAR);
            }
            text(&mut c, MONO, TEXT_SIZE, (x + name_x, y), &win_ansi(&row.name));
        }
    }

    c.set_fill_gray(GREY);
    let footer = format!("P12 Librarian \u{b7} page {number} of {count}");
    text(&mut c, SANS, 8.0, (MARGIN, MARGIN - 14.0), &win_ansi(&footer));
    c.finish().as_slice().to_vec()
}

fn to_pdf(sheet: &Sheet) -> Vec<u8> {
    let pages = pages(sheet);
    let mut pdf = Pdf::new();
    let catalog = Ref::new(1);
    let tree = Ref::new(2);
    let fonts = [
        (SANS, Ref::new(3), "Helvetica"),
        (SANS_BOLD, Ref::new(4), "Helvetica-Bold"),
        (MONO, Ref::new(5), "Courier"),
        (DINGBATS, Ref::new(6), "ZapfDingbats"),
    ];
    let info = Ref::new(7);
    // Each page and its content stream.
    let ids: Vec<(Ref, Ref)> = (0..pages.len() as i32)
        .map(|i| (Ref::new(8 + 2 * i), Ref::new(9 + 2 * i)))
        .collect();

    pdf.catalog(catalog).pages(tree);
    pdf.pages(tree)
        .kids(ids.iter().map(|(page, _)| *page))
        .count(ids.len() as i32);
    for (name, id, base) in fonts {
        let mut font = pdf.type1_font(id);
        font.base_font(Name(base.as_bytes()));
        // ZapfDingbats has its own built-in encoding.
        if name != DINGBATS {
            font.encoding_predefined(Name(b"WinAnsiEncoding"));
        }
    }
    pdf.document_info(info)
        .title(pdf_writer::TextStr(&sheet.title))
        .creator(pdf_writer::TextStr("P12 Librarian"));

    for (number, (text, (page_id, content_id))) in pages.iter().zip(&ids).enumerate() {
        let mut page = pdf.page(*page_id);
        page.media_box(Rect::new(0.0, 0.0, PAGE.0, PAGE.1))
            .parent(tree)
            .contents(*content_id);
        let mut resources = page.resources();
        let mut font_dict = resources.fonts();
        for (name, id, _) in fonts {
            font_dict.pair(name, id);
        }
        font_dict.finish();
        resources.finish();
        page.finish();
        let data = page_content(text, &sheet.note, number + 1, pages.len());
        pdf.stream(*content_id, &data);
    }
    pdf.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(slot: &str, name: &str) -> SheetRow {
        SheetRow {
            slot: slot.into(),
            name: name.into(),
            layer_b: "Init B".into(),
            favourite: false,
            tags: vec![],
        }
    }

    fn sheet(sections: Vec<SheetSection>) -> Sheet {
        Sheet {
            title: "Prophet 12".into(),
            note: "Read 2026-10-23".into(),
            sections,
        }
    }

    #[test]
    fn csv_quotes_where_needed() {
        let mut r = row("U1:01", "Pad, \"warm\"");
        r.favourite = true;
        r.tags = vec!["Pad".into(), "Pärla".into()];
        let csv = to_csv(&sheet(vec![SheetSection {
            bank: "U1".into(),
            rows: vec![r, row("U1:02", "Bass")],
        }]));
        assert_eq!(
            csv,
            "\u{feff}Slot,Name,Layer B,Favourite,Tags\r\n\
             U1:01,\"Pad, \"\"warm\"\"\",Init B,yes,Pad; Pärla\r\n\
             U1:02,Bass,Init B,,\r\n"
        );
    }

    #[test]
    fn a_bank_fits_one_page_and_long_lists_take_more() {
        let bank = SheetSection {
            bank: "U1".into(),
            rows: (1..=99).map(|n| row(&format!("U1:{n:02}"), "Pad")).collect(),
        };
        let list = SheetSection {
            bank: String::new(),
            rows: (1..=150).map(|n| row(&format!("edit {n}"), "Pad")).collect(),
        };
        let s = sheet(vec![bank, list]);
        let pages = pages(&s);
        assert_eq!(pages.len(), 3);
        assert_eq!(pages[0].heading, "Prophet 12 \u{b7} U1");
        assert_eq!(pages[0].rows[4].0, "05");
        assert_eq!(pages[1].heading, "Prophet 12");
        assert_eq!(pages[2].rows[0].0, "edit 101");
        let pdf = to_pdf(&s);
        assert!(pdf.starts_with(b"%PDF-"));
    }

    #[test]
    fn win_ansi_keeps_latin_1() {
        assert_eq!(win_ansi("Pärla ★"), b"P\xe4rla ?");
    }
}
