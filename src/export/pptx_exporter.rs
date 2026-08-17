use crate::core::calendar_gen::CalendarMonth;
use crate::core::text_overlay::{FontFamilyPreset, SlideBackground, SlideElement, TextAlignment, TextBoxStyle};
use crate::core::timeline::Timeline;
use crate::core::track::TrackKind;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use zip::write::FileOptions;
use zip::ZipWriter;

const SLIDE_WIDTH_EMU: i64 = 12_192_000;
const SLIDE_HEIGHT_EMU: i64 = 6_858_000;

/// Exports all slides on the video track to a native Microsoft PowerPoint presentation (.pptx).
pub fn export_to_pptx<P: AsRef<Path>>(timeline: &Timeline, output_path: P) -> std::io::Result<()> {
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // Collect all slides from the video track
    let slides: Vec<_> = timeline
        .tracks
        .iter()
        .filter(|t| t.kind == TrackKind::Video)
        .flat_map(|t| &t.clips)
        .collect();

    let slide_count = slides.len().max(1);

    // 1. [Content_Types].xml
    zip.start_file("[Content_Types].xml", options)?;
    let mut content_types = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Default Extension="png" ContentType="image/png"/>
  <Default Extension="jpg" ContentType="image/jpeg"/>
  <Default Extension="jpeg" ContentType="image/jpeg"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
  <Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
  <Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
"#,
    );
    for i in 1..=slide_count {
        content_types.push_str(&format!(
            r#"  <Override PartName="/ppt/slides/slide{}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
"#,
            i
        ));
    }
    content_types.push_str("</Types>");
    zip.write_all(content_types.as_bytes())?;

    // 2. _rels/.rels
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#,
    )?;

    // 3. ppt/presentation.xml
    zip.start_file("ppt/presentation.xml", options)?;
    let mut pres_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:sldMasterIdLst>
    <p:sldMasterId id="2147483648" r:id="rId1"/>
  </p:sldMasterIdLst>
  <p:sldIdLst>
"#
    );
    for i in 1..=slide_count {
        pres_xml.push_str(&format!(
            r#"    <p:sldId id="{}" r:id="rId{}"/>
"#,
            255 + i,
            i + 1
        ));
    }
    pres_xml.push_str(&format!(
        r#"  </p:sldIdLst>
  <p:sldSz cx="{}" cy="{}" type="custom"/>
  <p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#,
        SLIDE_WIDTH_EMU, SLIDE_HEIGHT_EMU
    ));
    zip.write_all(pres_xml.as_bytes())?;

    // 4. ppt/_rels/presentation.xml.rels
    zip.start_file("ppt/_rels/presentation.xml.rels", options)?;
    let mut pres_rels = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>
"#,
    );
    for i in 1..=slide_count {
        pres_rels.push_str(&format!(
            r#"  <Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{}.xml"/>
"#,
            i + 1,
            i
        ));
    }
    pres_rels.push_str("</Relationships>");
    zip.write_all(pres_rels.as_bytes())?;

    // 5. ppt/slideMasters/slideMaster1.xml & rels
    zip.start_file("ppt/slideMasters/slideMaster1.xml", options)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
    </p:spTree>
  </p:cSld>
  <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
  <p:sldLayoutIdLst>
    <p:sldLayoutId id="2147483649" r:id="rId1"/>
  </p:sldLayoutIdLst>
</p:sldMaster>"#,
    )?;

    zip.start_file("ppt/slideMasters/_rels/slideMaster1.xml.rels", options)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/>
</Relationships>"#,
    )?;

    // 6. ppt/slideLayouts/slideLayout1.xml & rels
    zip.start_file("ppt/slideLayouts/slideLayout1.xml", options)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" type="blank" preserve="1">
  <p:cSld name="Blank">
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
    </p:spTree>
  </p:cSld>
  <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sldLayout>"#,
    )?;

    zip.start_file("ppt/slideLayouts/_rels/slideLayout1.xml.rels", options)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="../slideMasters/slideMaster1.xml"/>
</Relationships>"#,
    )?;

    // 7. ppt/theme/theme1.xml
    zip.start_file("ppt/theme/theme1.xml", options)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office Theme">
  <a:themeElements>
    <a:clrScheme name="Office">
      <a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1>
      <a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1>
      <a:dk2><a:srgbClr val="1F497D"/></a:dk2>
      <a:lt2><a:srgbClr val="EEECE1"/></a:lt2>
      <a:accent1><a:srgbClr val="4F81BD"/></a:accent1>
      <a:accent2><a:srgbClr val="C0504D"/></a:accent2>
      <a:accent3><a:srgbClr val="9BBB59"/></a:accent3>
      <a:accent4><a:srgbClr val="8064A2"/></a:accent4>
      <a:accent5><a:srgbClr val="4BACC6"/></a:accent5>
      <a:accent6><a:srgbClr val="F79646"/></a:accent6>
      <a:hlink><a:srgbClr val="0000FF"/></a:hlink>
      <a:folHlink><a:srgbClr val="800080"/></a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="Office">
      <a:majorFont><a:latin typeface="Calibri"/></a:majorFont>
      <a:minorFont><a:latin typeface="Calibri"/></a:minorFont>
    </a:fontScheme>
    <a:fmtScheme name="Office"><a:fillStyleLst/><a:lnStyleLst/><a:effectStyleLst/><a:bgFillStyleLst/></a:fmtScheme>
  </a:themeElements>
</a:theme>"#,
    )?;

    // 8. Generate each slide and its media
    let mut media_counter = 1;

    for (slide_idx, clip) in slides.iter().enumerate() {
        let slide_num = slide_idx + 1;
        let mut slide_rels = Vec::new();
        slide_rels.push((
            "rId1".to_string(),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout".to_string(),
            "../slideLayouts/slideLayout1.xml".to_string(),
        ));

        let mut shape_tree_xml = String::new();
        let mut sp_id_counter = 2;

        // A. Slide Background
        let mut bg_xml = String::new();
        match &clip.background {
            Some(SlideBackground::Solid(col)) => {
                let hex = format!("{:02X}{:02X}{:02X}", col.r(), col.g(), col.b());
                bg_xml = format!(
                    r#"    <p:bg>
      <p:bgPr>
        <a:solidFill><a:srgbClr val="{}"/></a:solidFill>
        <a:effectLst/>
      </p:bgPr>
    </p:bg>
"#,
                    hex
                );
            }
            Some(SlideBackground::Picture(path)) => {
                if let Ok(mut img_file) = File::open(path) {
                    let mut img_bytes = Vec::new();
                    if img_file.read_to_end(&mut img_bytes).is_ok() {
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
                        let media_name = format!("image{}.{}", media_counter, ext);
                        media_counter += 1;
                        zip.start_file(format!("ppt/media/{}", media_name), options)?;
                        zip.write_all(&img_bytes)?;

                        let rel_id = format!("rId{}", slide_rels.len() + 1);
                        slide_rels.push((
                            rel_id.clone(),
                            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image".to_string(),
                            format!("../media/{}", media_name),
                        ));

                        bg_xml = format!(
                            r#"    <p:bg>
      <p:bgPr>
        <a:blipFill>
          <a:blip r:embed="{}"/>
          <a:stretch><a:fillRect/></a:stretch>
        </a:blipFill>
        <a:effectLst/>
      </p:bgPr>
    </p:bg>
"#,
                            rel_id
                        );
                    }
                }
            }
            None => {
                // Default dark elegant background
                bg_xml = r#"    <p:bg>
      <p:bgPr>
        <a:solidFill><a:srgbClr val="121218"/></a:solidFill>
        <a:effectLst/>
      </p:bgPr>
    </p:bg>
"#
                .to_string();
            }
        }

        // B. Slide Elements (Pictures, Videos, Calendars, Text)
        for el in &clip.elements {
            match el {
                SlideElement::Picture { path, x, y, w, h }
                | SlideElement::Sticker { path, x, y, w, h, .. }
                | SlideElement::Video { path, x, y, w, h } => {
                    let mut img_bytes = Vec::new();
                    let mut ext = "png".to_string();

                    if let Ok(mut f) = File::open(path) {
                        let _ = f.read_to_end(&mut img_bytes);
                        ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png").to_string();
                    }

                    // Fallback to extract thumbnail frame if direct file read failed
                    if img_bytes.is_empty() {
                        if let Ok(img) = crate::media::thumbnail::extract_thumbnail(path, 0.0) {
                            let mut cursor = Cursor::new(Vec::new());
                            let raw_bytes: Vec<u8> = img.pixels.iter().flat_map(|p: &egui::Color32| p.to_array()).collect();
                            if let Some(rgba) = image::RgbaImage::from_raw(img.width() as u32, img.height() as u32, raw_bytes) {
                                let _ = rgba.write_to(&mut cursor, image::ImageFormat::Png);
                                img_bytes = cursor.into_inner();
                                ext = "png".to_string();
                            }
                        }
                    }

                    if !img_bytes.is_empty() {
                        let media_name = format!("image{}.{}", media_counter, ext);
                        media_counter += 1;
                        zip.start_file(format!("ppt/media/{}", media_name), options)?;
                        zip.write_all(&img_bytes)?;

                        let rel_id = format!("rId{}", slide_rels.len() + 1);
                        slide_rels.push((
                            rel_id.clone(),
                            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image".to_string(),
                            format!("../media/{}", media_name),
                        ));

                        let emu_x = (*x as f64 * SLIDE_WIDTH_EMU as f64).round() as i64;
                        let emu_y = (*y as f64 * SLIDE_HEIGHT_EMU as f64).round() as i64;
                        let emu_w = (*w as f64 * SLIDE_WIDTH_EMU as f64).round() as i64;
                        let emu_h = (*h as f64 * SLIDE_HEIGHT_EMU as f64).round() as i64;

                        shape_tree_xml.push_str(&format!(
                            r#"      <p:pic>
        <p:nvPicPr>
          <p:cNvPr id="{}" name="Picture {}"/>
          <p:cNvPicPr><a:picLocks noChangeAspect="0"/></p:cNvPicPr>
          <p:nvPr/>
        </p:nvPicPr>
        <p:blipFill>
          <a:blip r:embed="{}"/>
          <a:stretch><a:fillRect/></a:stretch>
        </p:blipFill>
        <p:spPr>
          <a:xfrm>
            <a:off x="{}" y="{}"/>
            <a:ext cx="{}" cy="{}"/>
          </a:xfrm>
          <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
        </p:spPr>
      </p:pic>
"#,
                            sp_id_counter, sp_id_counter, rel_id, emu_x, emu_y, emu_w, emu_h
                        ));
                        sp_id_counter += 1;
                    }
                }
                SlideElement::Calendar(cal) => {
                    // Render sharp vector calendar PNG to embed in PPTX
                    let cal_img = CalendarMonth::render_multi_grid_image(
                        cal.year,
                        cal.start_month,
                        cal.month_count,
                        1920,
                        1080,
                        cal.show_holidays,
                        &cal.holidays,
                    );
                    let mut cursor = Cursor::new(Vec::new());
                    let raw_bytes: Vec<u8> = cal_img.pixels.iter().flat_map(|p: &egui::Color32| p.to_array()).collect();
                    if let Some(rgba) = image::RgbaImage::from_raw(1920, 1080, raw_bytes) {
                        let _ = rgba.write_to(&mut cursor, image::ImageFormat::Png);
                        let img_bytes = cursor.into_inner();

                        let media_name = format!("image{}.png", media_counter);
                        media_counter += 1;
                        zip.start_file(format!("ppt/media/{}", media_name), options)?;
                        zip.write_all(&img_bytes)?;

                        let rel_id = format!("rId{}", slide_rels.len() + 1);
                        slide_rels.push((
                            rel_id.clone(),
                            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image".to_string(),
                            format!("../media/{}", media_name),
                        ));

                        let emu_x = (cal.x as f64 * SLIDE_WIDTH_EMU as f64).round() as i64;
                        let emu_y = (cal.y as f64 * SLIDE_HEIGHT_EMU as f64).round() as i64;
                        let emu_w = (cal.w as f64 * SLIDE_WIDTH_EMU as f64).round() as i64;
                        let emu_h = (cal.h as f64 * SLIDE_HEIGHT_EMU as f64).round() as i64;

                        shape_tree_xml.push_str(&format!(
                            r#"      <p:pic>
        <p:nvPicPr>
          <p:cNvPr id="{}" name="Calendar Grid {}"/>
          <p:cNvPicPr><a:picLocks noChangeAspect="0"/></p:cNvPicPr>
          <p:nvPr/>
        </p:nvPicPr>
        <p:blipFill>
          <a:blip r:embed="{}"/>
          <a:stretch><a:fillRect/></a:stretch>
        </p:blipFill>
        <p:spPr>
          <a:xfrm>
            <a:off x="{}" y="{}"/>
            <a:ext cx="{}" cy="{}"/>
          </a:xfrm>
          <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
        </p:spPr>
      </p:pic>
"#,
                            sp_id_counter, sp_id_counter, rel_id, emu_x, emu_y, emu_w, emu_h
                        ));
                        sp_id_counter += 1;
                    }
                }
                SlideElement::Placeholder { label, x, y, w, h, .. } => {
                    let emu_x = (*x as f64 * SLIDE_WIDTH_EMU as f64).round() as i64;
                    let emu_y = (*y as f64 * SLIDE_HEIGHT_EMU as f64).round() as i64;
                    let emu_w = (*w as f64 * SLIDE_WIDTH_EMU as f64).round() as i64;
                    let emu_h = (*h as f64 * SLIDE_HEIGHT_EMU as f64).round() as i64;

                    shape_tree_xml.push_str(&format!(
                        r#"      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="{}" name="Placeholder {}"/>
          <p:cNvSpPr/>
          <p:nvPr/>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm>
            <a:off x="{}" y="{}"/>
            <a:ext cx="{}" cy="{}"/>
          </a:xfrm>
          <a:prstGeom prst="roundRect"><a:avLst><a:gd name="adj" fmla="val 2000"/></a:avLst></a:prstGeom>
          <a:solidFill><a:srgbClr val="1E283C"/></a:solidFill>
          <a:ln w="25400"><a:solidFill><a:srgbClr val="3A82F6"/></a:solidFill><a:prstDash val="dash"/></a:ln>
        </p:spPr>
        <p:txBody>
          <a:bodyPr anchor="ctr"/>
          <a:lstStyle/>
          <a:p>
            <a:pPr algn="ctr"/>
            <a:r>
              <a:rPr lang="en-US" sz="1800" b="1">
                <a:solidFill><a:srgbClr val="60A5FA"/></a:solidFill>
              </a:rPr>
              <a:t>{}</a:t>
            </a:r>
          </a:p>
        </p:txBody>
      </p:sp>
"#,
                        sp_id_counter, sp_id_counter, emu_x, emu_y, emu_w, emu_h, html_escape(label)
                    ));
                    sp_id_counter += 1;
                }
                SlideElement::Text(overlay) => {
                    let width_estimate = 0.50f64;
                    let height_estimate = (overlay.font_size as f64 * 1.5) / 540.0;
                    let left = (overlay.x as f64 - width_estimate / 2.0).clamp(0.0, 0.95);
                    let top = (overlay.y as f64 - height_estimate / 2.0).clamp(0.0, 0.95);

                    let emu_x = (left * SLIDE_WIDTH_EMU as f64) as i64;
                    let emu_y = (top * SLIDE_HEIGHT_EMU as f64) as i64;
                    let emu_w = (width_estimate * SLIDE_WIDTH_EMU as f64) as i64;
                    let emu_h = (height_estimate.max(0.06) * SLIDE_HEIGHT_EMU as f64) as i64;

                    let sz = (overlay.font_size * 100.0).round() as u32;
                    let hex_color = format!(
                        "{:02X}{:02X}{:02X}",
                        overlay.text_color.r(),
                        overlay.text_color.g(),
                        overlay.text_color.b()
                    );
                    let typeface = match overlay.font_family {
                        FontFamilyPreset::SansSerif => "Arial",
                        FontFamilyPreset::Serif => "Georgia",
                        FontFamilyPreset::Monospace => "Courier New",
                        FontFamilyPreset::Impact => "Impact",
                        FontFamilyPreset::Handwritten => "Segoe Print",
                        FontFamilyPreset::Condensed => "Arial Narrow",
                        FontFamilyPreset::Display => "Trebuchet MS",
                        FontFamilyPreset::VintageSerif => "Palatino Linotype",
                        FontFamilyPreset::Script => "Lucida Handwriting",
                        FontFamilyPreset::Futuristic => "Century Gothic",
                    };
                    let align = match overlay.alignment {
                        TextAlignment::Left => "l",
                        TextAlignment::Center => "ctr",
                        TextAlignment::Right => "r",
                    };

                    let bg_fill = if overlay.box_style != TextBoxStyle::None {
                        r#"<a:solidFill><a:srgbClr val="000000"><a:alpha val="60000"/></a:srgbClr></a:solidFill>"#
                    } else {
                        "<a:noFill/>"
                    };

                    let mut paragraphs_xml = String::new();
                    for line in overlay.text.lines() {
                        paragraphs_xml.push_str(&format!(
                            r#"          <a:p>
            <a:pPr algn="{}"/>
            <a:r>
              <a:rPr lang="en-US" sz="{}" b="{}">
                <a:solidFill><a:srgbClr val="{}"/></a:solidFill>
                <a:latin typeface="{}"/>
              </a:rPr>
              <a:t>{}</a:t>
            </a:r>
          </a:p>
"#,
                            align,
                            sz,
                            if overlay.font_family == FontFamilyPreset::Impact { 1 } else { 0 },
                            hex_color,
                            typeface,
                            html_escape(line)
                        ));
                    }

                    shape_tree_xml.push_str(&format!(
                        r#"      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="{}" name="Text Box {}"/>
          <p:cNvSpPr txBox="1"/>
          <p:nvPr/>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm>
            <a:off x="{}" y="{}"/>
            <a:ext cx="{}" cy="{}"/>
          </a:xfrm>
          <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
          {}
        </p:spPr>
        <p:txBody>
          <a:bodyPr wrap="square" rtlCol="0">
            <a:spAutoFit/>
          </a:bodyPr>
          <a:lstStyle/>
{}
        </p:txBody>
      </p:sp>
"#,
                        sp_id_counter, sp_id_counter, emu_x, emu_y, emu_w, emu_h, bg_fill, paragraphs_xml
                    ));
                    sp_id_counter += 1;
                }
                SlideElement::Audio { .. } => {}
            }
        }

        // C. Write ppt/slides/slide{N}.xml
        zip.start_file(format!("ppt/slides/slide{}.xml", slide_num), options)?;
        let slide_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld name="{}">
{}    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
{}    </p:spTree>
  </p:cSld>
  <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sld>"#,
            html_escape(&clip.name),
            bg_xml,
            shape_tree_xml
        );
        zip.write_all(slide_xml.as_bytes())?;

        // D. Write ppt/slides/_rels/slide{N}.xml.rels
        zip.start_file(format!("ppt/slides/_rels/slide{}.xml.rels", slide_num), options)?;
        let mut srels_xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
"#,
        );
        for (r_id, r_type, target) in slide_rels {
            srels_xml.push_str(&format!(
                r#"  <Relationship Id="{}" Type="{}" Target="{}"/>
"#,
                r_id, r_type, target
            ));
        }
        srels_xml.push_str("</Relationships>");
        zip.write_all(srels_xml.as_bytes())?;
    }

    // Finish writing the zip archive
    zip.finish()?;
    Ok(())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&apos;")
}
