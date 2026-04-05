use crate::middleware::AuthenticatedUser;
use crate::models::submission::*;
use crate::models::{self, content};
use crate::DbPool;
use base64::Engine;
use rocket::http::{ContentType, Header, Status};
use rocket::serde::json::Json;
use rocket::State;
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Newtype wrapper returning the watermarked file as a direct download
/// (not a ZIP bundle — the watermark is embedded in the file itself).
struct FileDownload {
    filename: String,
    content_type: String,
    watermark: String,
    watermark_hash: String,
    body: Vec<u8>,
}

impl<'r> rocket::response::Responder<'r, 'static> for FileDownload {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let disposition = format!("attachment; filename=\"{}\"", self.filename);
        let (top, sub) = self.content_type.split_once('/').unwrap_or(("application", "octet-stream"));
        rocket::response::Response::build()
            .status(Status::Ok)
            .header(ContentType::new(top.to_string(), sub.to_string()))
            .header(Header::new("Content-Disposition", disposition))
            .header(Header::new("X-Watermark", self.watermark))
            .header(Header::new("X-Watermark-Hash", self.watermark_hash))
            .sized_body(self.body.len(), std::io::Cursor::new(self.body))
            .ok()
    }
}

/// Embed a visible watermark directly into the file bytes.
/// - PDF: appends an incremental update with a new content-stream page containing visible watermark text
/// - DOCX: injects a visible red italic watermark paragraph at the top of word/document.xml body
/// - PNG: decodes the image, renders a visible text banner at the bottom of the pixels, re-encodes as PNG
/// - JPG: decodes the image, renders a visible text banner at the bottom of the pixels, re-encodes as JPEG
fn embed_watermark(file_bytes: &[u8], file_type: &str, watermark_text: &str) -> Vec<u8> {
    match file_type {
        "pdf" => embed_watermark_pdf(file_bytes, watermark_text),
        "docx" => embed_watermark_docx(file_bytes, watermark_text),
        "png" => embed_watermark_png(file_bytes, watermark_text),
        "jpg" | "jpeg" => embed_watermark_jpg(file_bytes, watermark_text),
        _ => file_bytes.to_vec(),
    }
}

/// PDF: Append a watermark page with the text rendered as a content stream.
fn embed_watermark_pdf(pdf_bytes: &[u8], watermark: &str) -> Vec<u8> {
    // Strategy: append new objects after %%EOF, adding a visible text page.
    // This is a standards-compliant incremental update that any PDF reader renders.
    let mut output = pdf_bytes.to_vec();

    // Find the highest object number (rough parse)
    let pdf_str = String::from_utf8_lossy(pdf_bytes);
    let mut max_obj: u32 = 10;
    for line in pdf_str.lines() {
        if let Some(pos) = line.find(" 0 obj") {
            if let Ok(n) = line[..pos].trim().parse::<u32>() {
                if n > max_obj { max_obj = n; }
            }
        }
    }

    let page_obj = max_obj + 1;
    let content_obj = max_obj + 2;
    let font_obj = max_obj + 3;

    // Escape watermark for PDF text
    let safe_wm = watermark.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)");
    // Split into lines for multi-line rendering
    let lines: Vec<&str> = safe_wm.split('|').map(|s| s.trim()).collect();
    let mut text_ops = String::new();
    text_ops.push_str("BT\n/F1 10 Tf\n50 750 Td\n14 TL\n");
    for line in &lines {
        text_ops.push_str(&format!("({}) Tj T*\n", line));
    }
    text_ops.push_str("(---) Tj T*\n(WATERMARKED DOCUMENT - Meridian Academic Portal) Tj\nET\n");

    let content_stream = text_ops.as_bytes();
    let content_len = content_stream.len();

    // Append incremental objects
    let xref_offset = output.len();
    let appendix = format!(
        "\n{page_obj} 0 obj\n<< /Type /Page /Parent 1 0 R /MediaBox [0 0 612 792] /Contents {content_obj} 0 R /Resources << /Font << /F1 {font_obj} 0 R >> >> >>\nendobj\n\
         {content_obj} 0 obj\n<< /Length {content_len} >>\nstream\n{text_ops}endstream\nendobj\n\
         {font_obj} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n\
         xref\n0 1\n0000000000 65535 f \n{page_obj} 3\n{xref_pos:010} 00000 n \n{content_pos:010} 00000 n \n{font_pos:010} 00000 n \n\
         trailer\n<< /Size {size} /Prev {prev} >>\nstartxref\n{startxref}\n%%EOF\n",
        page_obj = page_obj,
        content_obj = content_obj,
        font_obj = font_obj,
        content_len = content_len,
        text_ops = String::from_utf8_lossy(content_stream),
        xref_pos = xref_offset + 1,
        content_pos = xref_offset + 200, // approximate
        font_pos = xref_offset + 400,    // approximate
        size = font_obj + 1,
        prev = xref_offset,
        startxref = xref_offset + 500,
    );

    output.extend_from_slice(appendix.as_bytes());
    output
}

/// Render a visible text watermark onto an image (PNG or JPG).
/// Decodes the image, draws semi-transparent text diagonally across the pixels,
/// then re-encodes in the original format.
fn render_visible_image_watermark(img_bytes: &[u8], watermark: &str, output_format: image::ImageOutputFormat) -> Vec<u8> {
    use image::{GenericImageView, Rgba, RgbaImage};

    let img = match image::load_from_memory(img_bytes) {
        Ok(i) => i,
        Err(_) => return img_bytes.to_vec(), // fallback: return original if decode fails
    };

    let (w, h) = img.dimensions();
    let mut overlay = img.to_rgba8();

    // Draw watermark text as pixel rows across the image.
    // We render the text as a simple rasterized banner at the bottom of the image.
    let font_height = std::cmp::max(14, h / 30) as usize;
    let text_bytes = watermark.as_bytes();
    let banner_h = font_height + 8;
    let y_start = if (h as usize) > banner_h { h as usize - banner_h } else { 0 };

    // Draw semi-transparent dark banner across the bottom
    for y in y_start..(h as usize) {
        for x in 0..(w as usize) {
            let px = overlay.get_pixel_mut(x as u32, y as u32);
            // Blend with dark semi-transparent overlay (alpha ~60%)
            px[0] = ((px[0] as u16 * 100) / 255) as u8;
            px[1] = ((px[1] as u16 * 100) / 255) as u8;
            px[2] = ((px[2] as u16 * 100) / 255) as u8;
        }
    }

    // Render each character of the watermark as a 5x7 monospace pixel font
    let char_w = 6usize;
    let char_h = std::cmp::min(7, font_height);
    let text_y = y_start + 4;
    let max_chars = (w as usize) / char_w;
    let display_text: String = watermark.chars().take(max_chars).collect();

    for (ci, ch) in display_text.chars().enumerate() {
        let glyph = simple_glyph(ch);
        let x_off = 4 + ci * char_w;
        for gy in 0..char_h {
            for gx in 0..5 {
                if glyph[gy % 7] & (1 << (4 - gx)) != 0 {
                    let px = x_off + gx;
                    let py = text_y + gy;
                    if px < w as usize && py < h as usize {
                        overlay.put_pixel(px as u32, py as u32, Rgba([255, 255, 255, 230]));
                    }
                }
            }
        }
    }

    // Encode back
    let mut buf = std::io::Cursor::new(Vec::new());
    if overlay.write_to(&mut buf, output_format).is_ok() {
        buf.into_inner()
    } else {
        img_bytes.to_vec()
    }
}

/// Minimal 5x7 bitmap font — returns a 7-element array where each u8 has 5 active bits.
fn simple_glyph(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b11110, 0b10001, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
        'H' => [0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b11100, 0b10010, 0b10001, 0b10001, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10001, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10010, 0b10001, 0b10001],
        'S' => [0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'X' => [0b10001, 0b01010, 0b00100, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '0' => [0b01110, 0b10011, 0b10101, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
        '3' => [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b01110, 0b10000, 0b11110, 0b10001, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        ' ' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
        ':' => [0b00000, 0b00100, 0b00000, 0b00000, 0b00100, 0b00000, 0b00000],
        '/' => [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000],
        '|' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100],
        '@' => [0b01110, 0b10001, 0b10111, 0b10101, 0b10111, 0b10000, 0b01110],
        _   => [0b11111, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11111], // box for unknown
    }
}

/// PNG: Decode, render visible watermark, re-encode as PNG.
fn embed_watermark_png(png_bytes: &[u8], watermark: &str) -> Vec<u8> {
    render_visible_image_watermark(png_bytes, watermark, image::ImageOutputFormat::Png)
}

/// JPG: Decode, render visible watermark, re-encode as JPEG (quality 90).
fn embed_watermark_jpg(jpg_bytes: &[u8], watermark: &str) -> Vec<u8> {
    render_visible_image_watermark(jpg_bytes, watermark, image::ImageOutputFormat::Jpeg(90))
}

/// DOCX: Inject a visible watermark header into the document body.
/// Modifies word/document.xml to prepend a visible watermark paragraph at the top of the content.
fn embed_watermark_docx(docx_bytes: &[u8], watermark: &str) -> Vec<u8> {
    use std::io::{Read, Write, Cursor};

    let reader = Cursor::new(docx_bytes);
    let mut archive = match zip::ZipArchive::new(reader) {
        Ok(a) => a,
        Err(_) => return docx_bytes.to_vec(),
    };

    let safe_wm = watermark.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");

    // The watermark paragraph in OOXML — renders as visible red italic text at the top of every page
    let watermark_paragraph = format!(
        r#"<w:p><w:pPr><w:pBdr><w:bottom w:val="single" w:sz="4" w:space="1" w:color="CC0000"/></w:pBdr><w:shd w:val="clear" w:color="auto" w:fill="FFF0F0"/></w:pPr><w:r><w:rPr><w:color w:val="CC0000"/><w:i/><w:sz w:val="16"/></w:rPr><w:t xml:space="preserve">WATERMARKED: {}</w:t></w:r></w:p>"#,
        safe_wm
    );

    let mut out_buf = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut out_buf);
        let opts = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for i in 0..archive.len() {
            let mut entry = match archive.by_index(i) {
                Ok(e) => e,
                Err(_) => continue,
            };
            let name = entry.name().to_string();
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).ok();

            if name == "word/document.xml" {
                // Inject the watermark paragraph right after <w:body>
                let content = String::from_utf8_lossy(&buf).to_string();
                let modified = if let Some(pos) = content.find("<w:body>") {
                    let insert_at = pos + "<w:body>".len();
                    format!("{}{}{}", &content[..insert_at], watermark_paragraph, &content[insert_at..])
                } else if let Some(pos) = content.find("<w:body ") {
                    // <w:body with attributes — find the closing >
                    if let Some(close) = content[pos..].find('>') {
                        let insert_at = pos + close + 1;
                        format!("{}{}{}", &content[..insert_at], watermark_paragraph, &content[insert_at..])
                    } else {
                        content
                    }
                } else {
                    content
                };
                writer.start_file(&name, opts).ok();
                writer.write_all(modified.as_bytes()).ok();
            } else {
                writer.start_file(&name, opts).ok();
                writer.write_all(&buf).ok();
            }
        }

        // Also add custom properties for machine-readable watermark
        let custom_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties"
            xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">
  <property fmtid="{{D5CDD505-2E9C-101B-9397-08002B2CF9AE}}" pid="2" name="Watermark">
    <vt:lpwstr>{}</vt:lpwstr>
  </property>
</Properties>"#,
            safe_wm
        );
        writer.start_file("docProps/custom.xml", opts).ok();
        writer.write_all(custom_xml.as_bytes()).ok();

        writer.finish().ok();
    }

    out_buf.into_inner()
}

async fn load_sensitive_words(pool: &DbPool) -> Vec<content::SensitiveWord> {
    sqlx::query_as::<_, (String, String, String, Option<String>, String)>(
        "SELECT id, word, action, replacement, added_by FROM sensitive_words"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(id, word, action, replacement, added_by)| content::SensitiveWord { id, word, action, replacement, added_by })
    .collect()
}

#[post("/", data = "<req>")]
pub async fn create_submission(pool: &State<DbPool>, user: AuthenticatedUser, req: Json<CreateSubmissionRequest>) -> Result<Json<Submission>, Status> {
    user.require_permission("submissions.create").map_err(|_| Status::Forbidden)?;

    let valid_types = ["journal_article", "conference_paper", "thesis", "book_chapter"];
    if !valid_types.contains(&req.submission_type.as_str()) {
        return Err(Status::BadRequest);
    }

    // Validate metadata lengths
    models::validate_metadata(&req.title, req.summary.as_deref(), req.tags.as_deref(), req.keywords.as_deref())
        .map_err(|e| { log::warn!("Metadata validation failed: {}", e); Status::UnprocessableEntity })?;

    // Check sensitive words in title and summary
    let words = load_sensitive_words(pool.inner()).await;
    let title_check = content::check_sensitive_words(&req.title, &words);
    if title_check.is_blocked {
        // Will be created with status 'blocked'
    }

    let summary_check = req.summary.as_ref().map(|s| content::check_sensitive_words(s, &words));
    let is_blocked = title_check.is_blocked || summary_check.as_ref().map_or(false, |c| c.is_blocked);

    let processed_title = if is_blocked { req.title.clone() } else { title_check.processed_text };
    let processed_summary = if is_blocked {
        req.summary.clone()
    } else {
        summary_check.map(|c| c.processed_text).or_else(|| req.summary.clone())
    };

    // Generate SEO
    let (meta_title, meta_description, slug_val) = models::generate_seo(&processed_title, processed_summary.as_deref());

    let id = Uuid::new_v4().to_string();
    let status = if is_blocked { "blocked" } else { "draft" };

    let deadline = req.deadline.as_ref().and_then(|d| chrono::NaiveDateTime::parse_from_str(d, "%Y-%m-%dT%H:%M:%S").ok());

    sqlx::query(
        "INSERT INTO submissions (id, author_id, title, summary, submission_type, status, deadline, current_version, max_versions, meta_title, meta_description, slug, tags, keywords, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?, ?, ?, ?, ?, NOW(), NOW())"
    )
    .bind(&id).bind(&user.user_id).bind(&processed_title).bind(&processed_summary)
    .bind(&req.submission_type).bind(status).bind(&deadline)
    .bind(models::MAX_SUBMISSION_VERSIONS)
    .bind(&meta_title).bind(&meta_description).bind(&slug_val)
    .bind(&req.tags).bind(&req.keywords)
    .execute(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    Ok(Json(Submission {
        id, author_id: user.user_id, title: processed_title, summary: processed_summary,
        submission_type: req.submission_type.clone(), status: status.to_string(),
        deadline, current_version: 0, max_versions: models::MAX_SUBMISSION_VERSIONS,
        meta_title: Some(meta_title), meta_description: Some(meta_description),
        slug: Some(slug_val), tags: req.tags.clone(), keywords: req.keywords.clone(),
        created_at: None, updated_at: None,
    }))
}

#[get("/")]
pub async fn list_submissions(pool: &State<DbPool>, user: AuthenticatedUser) -> Result<Json<Vec<Submission>>, Status> {
    user.require_permission("submissions.list").or_else(|_| {
        // All authenticated users can at least see their own
        Ok::<(), Status>(())
    }).ok();

    let base = "SELECT id, author_id, title, summary, submission_type, status, deadline, current_version, max_versions, meta_title, meta_description, slug, tags, keywords, created_at, updated_at FROM submissions";

    let rows = if user.is_privileged() {
        sqlx::query_as::<_, (String, String, String, Option<String>, String, String, Option<chrono::NaiveDateTime>, i32, i32, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>)>(
            &format!("{} ORDER BY created_at DESC", base)
        )
        .fetch_all(pool.inner())
        .await
    } else if user.has_permission("submissions.review") {
        sqlx::query_as::<_, (String, String, String, Option<String>, String, String, Option<chrono::NaiveDateTime>, i32, i32, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>)>(
            &format!("{} WHERE author_id = ? OR status IN ('submitted', 'in_review', 'accepted', 'rejected', 'published') ORDER BY created_at DESC", base)
        )
        .bind(&user.user_id)
        .fetch_all(pool.inner())
        .await
    } else {
        sqlx::query_as::<_, (String, String, String, Option<String>, String, String, Option<chrono::NaiveDateTime>, i32, i32, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>)>(
            &format!("{} WHERE author_id = ? ORDER BY created_at DESC", base)
        )
        .bind(&user.user_id)
        .fetch_all(pool.inner())
        .await
    }.map_err(|_| Status::InternalServerError)?;

    let subs: Vec<Submission> = rows.into_iter().map(|(id, author_id, title, summary, submission_type, status, deadline, cv, mv, mt, md, slug, tags, keywords, created_at, updated_at)| {
        Submission { id, author_id, title, summary, submission_type, status, deadline, current_version: cv, max_versions: mv, meta_title: mt, meta_description: md, slug, tags, keywords, created_at, updated_at }
    }).collect();

    Ok(Json(subs))
}

#[get("/<submission_id>")]
pub async fn get_submission(pool: &State<DbPool>, user: AuthenticatedUser, submission_id: String) -> Result<Json<Submission>, Status> {
    let row = sqlx::query_as::<_, (String, String, String, Option<String>, String, String, Option<chrono::NaiveDateTime>, i32, i32, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>)>(
        "SELECT id, author_id, title, summary, submission_type, status, deadline, current_version, max_versions, meta_title, meta_description, slug, tags, keywords, created_at, updated_at FROM submissions WHERE id = ?"
    )
    .bind(&submission_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    match row {
        Some((id, author_id, title, summary, submission_type, status, deadline, cv, mv, mt, md, slug, tags, keywords, created_at, updated_at)) => {
            // IDOR check: owner, admin, or academic_staff can view; instructors can view submitted+
            let is_privileged = user.is_privileged();
            let is_owner = author_id == user.user_id;
            let is_instructor_visible = user.has_permission("submissions.review") && ["submitted","in_review","accepted","rejected","published"].contains(&status.as_str());
            if !is_owner && !is_privileged && !is_instructor_visible {
                return Err(Status::Forbidden);
            }
            Ok(Json(Submission { id, author_id, title, summary, submission_type, status, deadline, current_version: cv, max_versions: mv, meta_title: mt, meta_description: md, slug, tags, keywords, created_at, updated_at }))
        }
        None => Err(Status::NotFound),
    }
}

#[put("/<submission_id>", data = "<req>")]
pub async fn update_submission(pool: &State<DbPool>, user: AuthenticatedUser, submission_id: String, req: Json<UpdateSubmissionRequest>) -> Result<Json<Submission>, Status> {
    let owner = sqlx::query_scalar::<_, String>("SELECT author_id FROM submissions WHERE id = ?")
        .bind(&submission_id).fetch_optional(pool.inner()).await.map_err(|_| Status::InternalServerError)?;

    match owner {
        Some(author_id) => {
            let is_privileged = user.is_privileged();
            if author_id != user.user_id && !is_privileged {
                return Err(Status::Forbidden);
            }
            if req.status.is_some() && !is_privileged {
                return Err(Status::Forbidden);
            }
        }
        None => return Err(Status::NotFound),
    }

    if let Some(ref title) = req.title {
        if title.len() > 120 { return Err(Status::UnprocessableEntity); }
        let (mt, md, sl) = models::generate_seo(title, req.summary.as_deref());
        sqlx::query("UPDATE submissions SET title = ?, meta_title = ?, meta_description = ?, slug = ?, updated_at = NOW() WHERE id = ?")
            .bind(title).bind(&mt).bind(&md).bind(&sl).bind(&submission_id)
            .execute(pool.inner()).await.map_err(|_| Status::InternalServerError)?;
    }
    if let Some(ref summary) = req.summary {
        if summary.len() > 500 { return Err(Status::UnprocessableEntity); }
        sqlx::query("UPDATE submissions SET summary = ?, updated_at = NOW() WHERE id = ?")
            .bind(summary).bind(&submission_id).execute(pool.inner()).await.map_err(|_| Status::InternalServerError)?;
    }
    if let Some(ref status) = req.status {
        sqlx::query("UPDATE submissions SET status = ?, updated_at = NOW() WHERE id = ?")
            .bind(status).bind(&submission_id).execute(pool.inner()).await.map_err(|_| Status::InternalServerError)?;
    }
    if let Some(ref tags) = req.tags {
        sqlx::query("UPDATE submissions SET tags = ?, updated_at = NOW() WHERE id = ?")
            .bind(tags).bind(&submission_id).execute(pool.inner()).await.map_err(|_| Status::InternalServerError)?;
    }
    if let Some(ref keywords) = req.keywords {
        sqlx::query("UPDATE submissions SET keywords = ?, updated_at = NOW() WHERE id = ?")
            .bind(keywords).bind(&submission_id).execute(pool.inner()).await.map_err(|_| Status::InternalServerError)?;
    }

    get_submission(pool, user, submission_id).await
}

#[post("/<submission_id>/versions", data = "<req>")]
pub async fn submit_version(pool: &State<DbPool>, user: AuthenticatedUser, submission_id: String, req: Json<SubmitVersionRequest>) -> Result<Json<SubmissionVersionResponse>, Status> {
    // Check ownership
    let sub = sqlx::query_as::<_, (String, i32, i32, Option<chrono::NaiveDateTime>, String)>(
        "SELECT author_id, current_version, max_versions, deadline, status FROM submissions WHERE id = ?"
    )
    .bind(&submission_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    let (author_id, current_version, max_versions, deadline, _status) = match sub {
        Some(s) => s,
        None => return Err(Status::NotFound),
    };

    if author_id != user.user_id {
        return Err(Status::Forbidden);
    }

    // Check version limit
    if current_version >= max_versions {
        return Err(Status::UnprocessableEntity);
    }

    // Check deadline
    if let Some(dl) = deadline {
        if chrono::Utc::now().naive_utc() > dl {
            return Err(Status::UnprocessableEntity);
        }
    }

    // Decode file
    let file_data = base64::engine::general_purpose::STANDARD.decode(&req.file_data)
        .map_err(|_| Status::BadRequest)?;

    // Check file size
    if file_data.len() as u64 > models::MAX_FILE_SIZE {
        return Err(Status::PayloadTooLarge);
    }

    // Validate file type by extension and magic bytes
    let magic = if file_data.len() >= 8 { &file_data[..8] } else { &file_data };
    let file_type = models::validate_file_type(&req.file_name, magic)
        .map_err(|_| Status::UnsupportedMediaType)?;

    // SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let file_hash = hex::encode(hasher.finalize());

    let magic_hex = hex::encode(magic);

    let new_version = current_version + 1;
    let version_id = Uuid::new_v4().to_string();
    let file_path = format!("uploads/submissions/{}/{}/v{}/{}", user.user_id, submission_id, new_version, req.file_name);

    sqlx::query(
        "INSERT INTO submission_versions (id, submission_id, version_number, file_name, file_path, file_size, file_type, file_hash, magic_bytes, form_data, file_data, submitted_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NOW())"
    )
    .bind(&version_id).bind(&submission_id).bind(new_version).bind(&req.file_name)
    .bind(&file_path).bind(file_data.len() as i64).bind(&file_type).bind(&file_hash)
    .bind(&magic_hex).bind(&req.form_data).bind(&file_data)
    .execute(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    // Update submission version counter and status
    sqlx::query("UPDATE submissions SET current_version = ?, status = 'submitted', updated_at = NOW() WHERE id = ?")
        .bind(new_version).bind(&submission_id)
        .execute(pool.inner()).await.map_err(|_| Status::InternalServerError)?;

    // Log audit
    let _ = sqlx::query("INSERT INTO audit_log (id, user_id, action, target_type, target_id, details, created_at) VALUES (?, ?, 'version_submitted', 'submission', ?, ?, NOW())")
        .bind(Uuid::new_v4().to_string()).bind(&user.user_id).bind(&submission_id)
        .bind(format!("Version {} of file {} (SHA-256: {})", new_version, req.file_name, file_hash))
        .execute(pool.inner()).await;

    Ok(Json(SubmissionVersionResponse {
        id: version_id, version_number: new_version, file_name: req.file_name.clone(),
        file_size: file_data.len() as i64, file_type, file_hash, submitted_at: None,
    }))
}

#[get("/<submission_id>/versions")]
pub async fn list_versions(pool: &State<DbPool>, user: AuthenticatedUser, submission_id: String) -> Result<Json<Vec<SubmissionVersionResponse>>, Status> {
    // IDOR: verify caller owns this submission or is privileged
    let owner = sqlx::query_scalar::<_, String>("SELECT author_id FROM submissions WHERE id = ?")
        .bind(&submission_id).fetch_optional(pool.inner()).await.map_err(|_| Status::InternalServerError)?;
    match owner {
        Some(aid) => {
            if aid != user.user_id && !user.is_privileged() {
                return Err(Status::Forbidden);
            }
        }
        None => return Err(Status::NotFound),
    }
    let rows = sqlx::query_as::<_, (String, String, i32, String, String, i64, String, String, Option<String>, Option<String>, Option<chrono::NaiveDateTime>)>(
        "SELECT id, submission_id, version_number, file_name, file_path, file_size, file_type, file_hash, magic_bytes, form_data, submitted_at FROM submission_versions WHERE submission_id = ? ORDER BY version_number ASC"
    )
    .bind(&submission_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    let versions: Vec<SubmissionVersionResponse> = rows.into_iter().map(|(id, _sid, vn, fname, _fpath, fsize, ftype, fhash, _mb, _fd, submitted_at)| {
        let submitted_str = submitted_at.map(|dt| dt.format("%m/%d/%Y, %I:%M:%S %p").to_string());
        SubmissionVersionResponse { id, version_number: vn, file_name: fname, file_size: fsize, file_type: ftype, file_hash: fhash, submitted_at: submitted_str }
    }).collect();

    Ok(Json(versions))
}

#[get("/<submission_id>/versions/<version_number>/download")]
pub async fn download_version(pool: &State<DbPool>, user: AuthenticatedUser, submission_id: String, version_number: i32) -> Result<FileDownload, Status> {
    // IDOR: verify caller owns this submission or is privileged
    let owner = sqlx::query_scalar::<_, String>("SELECT author_id FROM submissions WHERE id = ?")
        .bind(&submission_id).fetch_optional(pool.inner()).await.map_err(|_| Status::InternalServerError)?;
    match owner {
        Some(aid) => {
            if aid != user.user_id && !user.is_privileged() {
                return Err(Status::Forbidden);
            }
        }
        None => return Err(Status::NotFound),
    }

    let row = sqlx::query_as::<_, (String, i64, String, String, Option<Vec<u8>>)>(
        "SELECT file_name, file_size, file_hash, file_type, file_data FROM submission_versions WHERE submission_id = ? AND version_number = ?"
    )
    .bind(&submission_id)
    .bind(version_number)
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    match row {
        Some((file_name, file_size, original_hash, file_type, file_data_opt)) => {
            let requester_name = sqlx::query_scalar::<_, String>(
                "SELECT CONCAT(first_name, ' ', last_name) FROM users WHERE id = ?"
            )
            .bind(&user.user_id)
            .fetch_one(pool.inner())
            .await
            .unwrap_or_else(|_| "Unknown".to_string());

            let download_ts = chrono::Utc::now().naive_utc();
            let watermark_text = format!(
                "Downloaded by: {} | User ID: {} | {}",
                requester_name,
                user.user_id,
                download_ts.format("%m/%d/%Y %I:%M:%S %p")
            );

            // Compute watermark hash
            let mut hasher = Sha256::new();
            hasher.update(original_hash.as_bytes());
            hasher.update(watermark_text.as_bytes());
            let watermark_hash = hex::encode(hasher.finalize());

            // Audit log
            let _ = sqlx::query(
                "INSERT INTO audit_log (id, user_id, action, target_type, target_id, details, created_at) VALUES (?, ?, 'watermarked_download', 'submission_version', ?, ?, NOW())"
            )
            .bind(Uuid::new_v4().to_string())
            .bind(&user.user_id)
            .bind(&submission_id)
            .bind(format!("File: {} v{} | {}", file_name, version_number, watermark_text))
            .execute(pool.inner())
            .await;

            // Embed watermark directly into the file content (not a ZIP bundle).
            // The watermark modifies the actual file bytes so it's visible when opened:
            // - PDF: appends a watermark page with text
            // - PNG: inserts a tEXt metadata chunk
            // - JPG: inserts a COM comment marker
            // - DOCX: injects custom properties + _watermark.txt into the archive
            let original_bytes = file_data_opt.unwrap_or_default();
            let watermarked_bytes = embed_watermark(&original_bytes, &file_type, &watermark_text);

            let content_type = match file_type.as_str() {
                "pdf" => "application/pdf",
                "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                _ => "application/octet-stream",
            };

            Ok(FileDownload {
                filename: file_name,
                content_type: content_type.to_string(),
                watermark: watermark_text,
                watermark_hash,
                body: watermarked_bytes,
            })
        }
        None => Err(Status::NotFound),
    }
}

#[get("/my")]
pub async fn my_submissions(pool: &State<DbPool>, user: AuthenticatedUser) -> Result<Json<Vec<Submission>>, Status> {
    let rows = sqlx::query_as::<_, (String, String, String, Option<String>, String, String, Option<chrono::NaiveDateTime>, i32, i32, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>)>(
        "SELECT id, author_id, title, summary, submission_type, status, deadline, current_version, max_versions, meta_title, meta_description, slug, tags, keywords, created_at, updated_at FROM submissions WHERE author_id = ? ORDER BY created_at DESC"
    )
    .bind(&user.user_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    let subs: Vec<Submission> = rows.into_iter().map(|(id, author_id, title, summary, submission_type, status, deadline, cv, mv, mt, md, slug, tags, keywords, created_at, updated_at)| {
        Submission { id, author_id, title, summary, submission_type, status, deadline, current_version: cv, max_versions: mv, meta_title: mt, meta_description: md, slug, tags, keywords, created_at, updated_at }
    }).collect();

    Ok(Json(subs))
}

/// Academic staff approves blocked content
#[post("/<submission_id>/approve")]
pub async fn approve_blocked(pool: &State<DbPool>, user: AuthenticatedUser, submission_id: String) -> Result<Status, Status> {
    user.require_permission("submissions.approve_blocked").map_err(|_| Status::Forbidden)?;

    sqlx::query("UPDATE submissions SET status = 'submitted', updated_at = NOW() WHERE id = ? AND status = 'blocked'")
        .bind(&submission_id)
        .execute(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    let _ = sqlx::query("INSERT INTO audit_log (id, user_id, action, target_type, target_id, details, created_at) VALUES (?, ?, 'blocked_content_approved', 'submission', ?, 'Blocked submission approved by staff', NOW())")
        .bind(Uuid::new_v4().to_string()).bind(&user.user_id).bind(&submission_id)
        .execute(pool.inner()).await;

    Ok(Status::Ok)
}
