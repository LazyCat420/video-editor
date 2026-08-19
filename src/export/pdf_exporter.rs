use crate::core::calendar_gen::CalendarMonth;
use crate::core::text_overlay::{FontFamilyPreset, SlideBackground, SlideElement, TextAlignment};
use crate::core::timeline::Timeline;
use crate::core::track::TrackKind;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const PDF_WIDTH: f64 = 960.0;
const PDF_HEIGHT: f64 = 540.0;

/// Exports all slides on the video track to a clean, crisp 16:9 landscape presentation PDF.
pub fn export_to_pdf<P: AsRef<Path>>(timeline: &Timeline, output_path: P) -> std::io::Result<()> {
    let mut file = File::create(output_path)?;

    let slides: Vec<_> = timeline
        .tracks
        .iter()
        .filter(|t| t.kind == TrackKind::Video)
        .flat_map(|t| &t.clips)
        .collect();

    let slide_count = slides.len().max(1);

    // Buffer to accumulate PDF objects and track their byte offsets
    let mut pdf_data = Vec::new();
    let mut offsets = Vec::new();

    // 1. PDF Header
    pdf_data.extend_from_slice(b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n");

    // Standard Font Object (Helvetica / Arial standard type 1)
    let font_obj_id = 1;
    offsets.push(pdf_data.len());
    pdf_data.extend_from_slice(
        format!(
            "{} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
            font_obj_id
        )
        .as_bytes(),
    );

    let font_bold_obj_id = 2;
    offsets.push(pdf_data.len());
    pdf_data.extend_from_slice(
        format!(
            "{} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>\nendobj\n",
            font_bold_obj_id
        )
        .as_bytes(),
    );

    let font_italic_obj_id = 3;
    offsets.push(pdf_data.len());
    pdf_data.extend_from_slice(
        format!(
            "{} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Oblique >>\nendobj\n",
            font_italic_obj_id
        )
        .as_bytes(),
    );

    let font_bold_italic_obj_id = 4;
    offsets.push(pdf_data.len());
    pdf_data.extend_from_slice(
        format!(
            "{} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-BoldOblique >>\nendobj\n",
            font_bold_italic_obj_id
        )
        .as_bytes(),
    );

    let pages_obj_id = 5;
    let mut next_obj_id = 6;
    let mut page_obj_ids = Vec::new();

    for clip in &slides {
        let mut content_stream = String::new();
        let mut xobjects = Vec::new();

        // A. Slide Background
        match &clip.background {
            Some(SlideBackground::Solid(col)) => {
                let r = col.r() as f64 / 255.0;
                let g = col.g() as f64 / 255.0;
                let b = col.b() as f64 / 255.0;
                content_stream.push_str(&format!(
                    "q {:.3} {:.3} {:.3} rg 0 0 {:.1} {:.1} re f Q\n",
                    r, g, b, PDF_WIDTH, PDF_HEIGHT
                ));
            }
            Some(SlideBackground::Picture(p)) => {
                if let Some(obj_id) = embed_image_object(&mut pdf_data, &mut offsets, &mut next_obj_id, p) {
                    let name = format!("Im{}", obj_id);
                    content_stream.push_str(&format!(
                        "q {:.1} 0 0 {:.1} 0 0 cm /{} Do Q\n",
                        PDF_WIDTH, PDF_HEIGHT, name
                    ));
                    xobjects.push((name, obj_id));
                }
            }
            _ => {
                // Default Dark Slate Background
                content_stream.push_str(&format!(
                    "q 0.09 0.10 0.13 rg 0 0 {:.1} {:.1} re f Q\n",
                    PDF_WIDTH, PDF_HEIGHT
                ));
            }
        }

        // B. Render Slide Elements
        for el in &clip.elements {
            match el {
                SlideElement::Picture { path, x, y, w, h } | SlideElement::Video { path, x, y, w, h } => {
                    if let Some(obj_id) = embed_image_object(&mut pdf_data, &mut offsets, &mut next_obj_id, path) {
                        let name = format!("Im{}", obj_id);
                        let pdf_x = *x as f64 * PDF_WIDTH;
                        let pdf_y = PDF_HEIGHT - (*y as f64 * PDF_HEIGHT) - (*h as f64 * PDF_HEIGHT);
                        let pdf_w = *w as f64 * PDF_WIDTH;
                        let pdf_h = *h as f64 * PDF_HEIGHT;

                        content_stream.push_str(&format!(
                            "q {:.1} 0 0 {:.1} {:.1} {:.1} cm /{} Do Q\n",
                            pdf_w, pdf_h, pdf_x, pdf_y, name
                        ));
                        xobjects.push((name, obj_id));
                    }
                }
                SlideElement::Sticker { path, x, y, w, h, .. } => {
                    if let Some(obj_id) = embed_image_object(&mut pdf_data, &mut offsets, &mut next_obj_id, path) {
                        let name = format!("Im{}", obj_id);
                        let pdf_x = *x as f64 * PDF_WIDTH;
                        let pdf_y = PDF_HEIGHT - (*y as f64 * PDF_HEIGHT) - (*h as f64 * PDF_HEIGHT);
                        let pdf_w = *w as f64 * PDF_WIDTH;
                        let pdf_h = *h as f64 * PDF_HEIGHT;

                        content_stream.push_str(&format!(
                            "q {:.1} 0 0 {:.1} {:.1} {:.1} cm /{} Do Q\n",
                            pdf_w, pdf_h, pdf_x, pdf_y, name
                        ));
                        xobjects.push((name, obj_id));
                    }
                }
                SlideElement::Calendar(cal) => {
                    let cal_img = CalendarMonth::render_multi_grid_image(
                        cal.year,
                        cal.start_month,
                        cal.month_count,
                        1920,
                        1080,
                        cal.show_holidays,
                        &cal.holidays,
                    );
                    if let Some(img_obj_id) = embed_raw_image_object(&mut pdf_data, &mut offsets, &mut next_obj_id, &cal_img) {
                        let im_name = format!("Im{}", img_obj_id);
                        let pdf_w = cal.w as f64 * PDF_WIDTH;
                        let pdf_h = cal.h as f64 * PDF_HEIGHT;
                        let pdf_x = cal.x as f64 * PDF_WIDTH;
                        let pdf_y = PDF_HEIGHT - (cal.y as f64 * PDF_HEIGHT) - pdf_h;

                        content_stream.push_str(&format!(
                            "q {:.1} 0 0 {:.1} {:.1} {:.1} cm /{} Do Q\n",
                            pdf_w, pdf_h, pdf_x, pdf_y, im_name
                        ));
                        xobjects.push((im_name, img_obj_id));
                    }
                }
                SlideElement::Placeholder { label, x, y, w, h, .. } => {
                    let pdf_x = *x as f64 * PDF_WIDTH;
                    let pdf_y = PDF_HEIGHT - (*y as f64 * PDF_HEIGHT) - (*h as f64 * PDF_HEIGHT);
                    let pdf_w = *w as f64 * PDF_WIDTH;
                    let pdf_h = *h as f64 * PDF_HEIGHT;

                    // Draw placeholder frame & centered label
                    content_stream.push_str(&format!(
                        "q 0.15 0.18 0.24 rg {:.1} {:.1} {:.1} {:.1} re f Q\n",
                        pdf_x, pdf_y, pdf_w, pdf_h
                    ));
                    content_stream.push_str(&format!(
                        "q 0.38 0.65 0.98 RG 1.5 w {:.1} {:.1} {:.1} {:.1} re S Q\n",
                        pdf_x, pdf_y, pdf_w, pdf_h
                    ));
                    let text_x = pdf_x + pdf_w / 2.0 - 40.0;
                    let text_y = pdf_y + pdf_h / 2.0 - 5.0;
                    content_stream.push_str(&format!(
                        "BT /F2 14 Tf 0.38 0.65 0.98 rg {:.1} {:.1} Td ({}) Tj ET\n",
                        text_x, text_y, escape_pdf_str(label)
                    ));
                }
                SlideElement::Text(overlay) => {
                    let paint = crate::core::TextPaint::from_color32(overlay.text_color);
                    let (r, g, b) = paint.to_pdf_rgb();
                    let pt_size = (overlay.font_size as f64 * 0.75).clamp(10.0, 72.0);
                    let is_bold = overlay.is_bold || overlay.font_family == FontFamilyPreset::Impact;
                    let is_italic = overlay.is_italic;
                    let font_ref = match (is_bold, is_italic) {
                        (false, false) => "/F1",
                        (true, false) => "/F2",
                        (false, true) => "/F3",
                        (true, true) => "/F4",
                    };

                    let formatted = overlay.formatted_text();
                    let lines: Vec<&str> = formatted.lines().collect();
                    let line_height = pt_size * 1.25;
                    let total_h = lines.len() as f64 * line_height;

                    let base_y = PDF_HEIGHT - (overlay.y as f64 * PDF_HEIGHT) + (total_h / 2.0);

                    for (i, line) in lines.iter().enumerate() {
                        let cur_y = base_y - ((i + 1) as f64 * line_height);
                        let estimated_w = line.len() as f64 * (pt_size * 0.55);
                        let cur_x = match overlay.alignment {
                            TextAlignment::Left => (overlay.x as f64 * PDF_WIDTH).clamp(20.0, PDF_WIDTH - 20.0),
                            TextAlignment::Center => ((overlay.x as f64 * PDF_WIDTH) - (estimated_w / 2.0)).clamp(20.0, PDF_WIDTH - 20.0),
                            TextAlignment::Right => ((overlay.x as f64 * PDF_WIDTH) - estimated_w).clamp(20.0, PDF_WIDTH - 20.0),
                        };

                        if overlay.show_shadow {
                            content_stream.push_str(&format!(
                                "BT {} {:.1} Tf 0.05 0.05 0.05 rg {:.1} {:.1} Td ({}) Tj ET\n",
                                font_ref, pt_size, cur_x + 1.2, cur_y - 1.2, escape_pdf_str(line)
                            ));
                        }

                        content_stream.push_str(&format!(
                            "BT {} {:.1} Tf {:.3} {:.3} {:.3} rg {:.1} {:.1} Td ({}) Tj ET\n",
                            font_ref, pt_size, r, g, b, cur_x, cur_y, escape_pdf_str(line)
                        ));
                    }
                }
                SlideElement::Audio { .. } => {}
            }
        }

        // C. Write Page Content Stream Object
        let content_obj_id = next_obj_id;
        next_obj_id += 1;
        offsets.push(pdf_data.len());
        pdf_data.extend_from_slice(
            format!(
                "{} 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
                content_obj_id,
                content_stream.len(),
                content_stream
            )
            .as_bytes(),
        );

        // D. Write Page Object
        let page_obj_id = next_obj_id;
        next_obj_id += 1;
        page_obj_ids.push(page_obj_id);

        let mut xobj_dict = String::new();
        if !xobjects.is_empty() {
            xobj_dict.push_str(" /XObject <<");
            for (name, id) in xobjects {
                xobj_dict.push_str(&format!(" /{} {} 0 R", name, id));
            }
            xobj_dict.push_str(" >>");
        }

        offsets.push(pdf_data.len());
        pdf_data.extend_from_slice(
            format!(
                "{} 0 obj\n<< /Type /Page /Parent {} 0 R /MediaBox [0 0 {:.1} {:.1}] /Contents {} 0 R /Resources << /Font << /F1 1 0 R /F2 2 0 R /F3 3 0 R /F4 4 0 R >>{} >> >>\nendobj\n",
                page_obj_id, pages_obj_id, PDF_WIDTH, PDF_HEIGHT, content_obj_id, xobj_dict
            )
            .as_bytes(),
        );
    }

    // Pages Tree Root Object
    offsets.push(pdf_data.len());
    pdf_data.extend_from_slice(
        format!(
            "{} 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
            pages_obj_id,
            page_obj_ids
                .iter()
                .map(|id| format!("{} 0 R", id))
                .collect::<Vec<_>>()
                .join(" "),
            slide_count
        )
        .as_bytes(),
    );

    // Catalog Object
    let catalog_obj_id = next_obj_id;
    offsets.push(pdf_data.len());
    pdf_data.extend_from_slice(
        format!(
            "{} 0 obj\n<< /Type /Catalog /Pages {} 0 R >>\nendobj\n",
            catalog_obj_id, pages_obj_id
        )
        .as_bytes(),
    );

    // Cross-Reference Table
    let xref_offset = pdf_data.len();
    let total_objects = offsets.len() + 1; // plus obj 0
    pdf_data.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", total_objects).as_bytes());

    for off in &offsets {
        pdf_data.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }

    // Trailer
    pdf_data.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root {} 0 R >>\nstartxref\n{}\n%%EOF\n",
            total_objects, catalog_obj_id, xref_offset
        )
        .as_bytes(),
    );

    file.write_all(&pdf_data)?;
    Ok(())
}

fn embed_image_object(
    pdf_data: &mut Vec<u8>,
    offsets: &mut Vec<usize>,
    next_obj_id: &mut usize,
    path: &Path,
) -> Option<usize> {
    if let Ok(mut f) = File::open(path) {
        let mut bytes = Vec::new();
        if f.read_to_end(&mut bytes).is_ok() {
            if let Ok(img) = image::load_from_memory(&bytes) {
                let rgb = img.to_rgb8();
                let width = rgb.width();
                let height = rgb.height();
                let raw_rgb = rgb.into_raw();

                let obj_id = *next_obj_id;
                *next_obj_id += 1;
                offsets.push(pdf_data.len());

                let header = format!(
                    "{} 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length {} >>\nstream\n",
                    obj_id, width, height, raw_rgb.len()
                );
                pdf_data.extend_from_slice(header.as_bytes());
                pdf_data.extend_from_slice(&raw_rgb);
                pdf_data.extend_from_slice(b"\nendstream\nendobj\n");
                return Some(obj_id);
            }
        }
    }

    // Video poster frame fallback: extract frame at t=0
    if let Ok(color_img) = crate::media::extract_thumbnail(path, 0.0) {
        return embed_raw_image_object(pdf_data, offsets, next_obj_id, &color_img);
    }

    None
}

fn embed_raw_image_object(
    pdf_data: &mut Vec<u8>,
    offsets: &mut Vec<usize>,
    next_obj_id: &mut usize,
    img: &egui::ColorImage,
) -> Option<usize> {
    let width = img.width();
    let height = img.height();
    let mut raw_rgb = Vec::with_capacity(width * height * 3);
    for p in &img.pixels {
        raw_rgb.push(p.r());
        raw_rgb.push(p.g());
        raw_rgb.push(p.b());
    }

    let obj_id = *next_obj_id;
    *next_obj_id += 1;
    offsets.push(pdf_data.len());

    let header = format!(
        "{} 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length {} >>\nstream\n",
        obj_id, width, height, raw_rgb.len()
    );
    pdf_data.extend_from_slice(header.as_bytes());
    pdf_data.extend_from_slice(&raw_rgb);
    pdf_data.extend_from_slice(b"\nendstream\nendobj\n");
    Some(obj_id)
}

fn escape_pdf_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}
