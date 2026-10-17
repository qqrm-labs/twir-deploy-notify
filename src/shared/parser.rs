use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::generator::{escape_markdown_url, format_subheading};
use teloxide::utils::markdown::escape;

/// Representation of a single TWIR section.
#[derive(Default)]
pub struct Section {
    pub title: String,
    pub lines: Vec<String>,
    /// Largest rendered table, including its caption and code fences.
    pub(crate) max_table_bytes: usize,
}

fn fix_bare_link(line: &str) -> String {
    if line.contains("](") {
        return line.to_string();
    }
    let plain = line.replace('\\', "");
    let trimmed = plain.trim_end();
    if trimmed.ends_with(')')
        && let Some(start) = trimmed
            .rfind("(https://")
            .or_else(|| trimmed.rfind("(http://"))
    {
        let url = &trimmed[start + 1..trimmed.len() - 1];
        let text = trimmed[..start].trim_end();
        return format!("[{}]({})", escape(text), escape_markdown_url(url));
    }
    line.to_string()
}

fn replace_github_mentions(text: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut bracket_depth = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        match ch {
            '[' => {
                bracket_depth += 1;
                result.push(ch);
                i += 1;
            }
            ']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                result.push(ch);
                i += 1;
            }
            '@' if bracket_depth == 0 => {
                let prev = if i == 0 { None } else { Some(chars[i - 1]) };
                let mut j = i + 1;
                if j < chars.len() && chars[j].is_ascii_alphanumeric() {
                    while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '-') {
                        j += 1;
                    }
                    let next = chars.get(j).copied();
                    if !prev.is_some_and(|c| c.is_ascii_alphanumeric() || c == '/')
                        && !next.is_some_and(|c| c.is_ascii_alphanumeric() || c == '-')
                    {
                        let user: String = chars[i + 1..j].iter().collect();
                        result.push_str(&format!("[{user}](https://github.com/{user})"));
                        i = j;
                        continue;
                    }
                }
                result.push('@');
                i += 1;
            }
            _ => {
                result.push(ch);
                i += 1;
            }
        }
    }
    result
}

fn normalize_table_text(text: &str) -> String {
    text.replace("Regressions", "Reg")
        .replace("Improvements", "Imp")
        .replace('❌', "x")
        .replace('✅', "v")
        .replace("(primary)", "prim")
        .replace("(secondary)", "sec")
        .replace("primary", "prim")
        .replace("secondary", "sec")
}

fn parse_pipe_row(line: &str) -> Option<Vec<&str>> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return None;
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    let cells: Vec<&str> = inner.split('|').map(str::trim).collect();
    if cells.len() < 2 {
        return None;
    }

    Some(cells)
}

fn is_alignment_row(line: &str) -> bool {
    parse_pipe_row(line).is_some_and(|cells| {
        cells.iter().all(|cell| {
            !cell.is_empty() && cell.contains('-') && cell.chars().all(|ch| matches!(ch, '-' | ':'))
        })
    })
}

fn make_alignment_row(columns: usize) -> String {
    let mut row = String::from("|");
    for _ in 0..columns {
        row.push_str(" --- |");
    }
    row
}

fn normalize_compact_tables(text: &str) -> String {
    let source_lines: Vec<&str> = text.lines().collect();
    let mut lines = Vec::with_capacity(source_lines.len());
    let mut in_fenced_code_block = false;
    let mut index = 0usize;

    while index < source_lines.len() {
        let line = source_lines[index];
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fenced_code_block = !in_fenced_code_block;
            lines.push(line.to_string());
            index += 1;
            continue;
        }

        if !in_fenced_code_block && let Some(cells) = parse_pipe_row(line) {
            let next_line = source_lines.get(index + 1).copied();
            let previous_is_pipe_row =
                index > 0 && parse_pipe_row(source_lines[index - 1]).is_some();
            let starts_compact_table = !is_alignment_row(line)
                && !previous_is_pipe_row
                && next_line.is_some_and(|candidate| {
                    !is_alignment_row(candidate)
                        && parse_pipe_row(candidate)
                            .is_some_and(|next_cells| next_cells.len() == cells.len())
                });

            if starts_compact_table {
                if !lines.last().is_none_or(|last| last.is_empty()) {
                    lines.push(String::new());
                }

                lines.push(line.to_string());
                lines.push(make_alignment_row(cells.len()));
                index += 1;

                while index < source_lines.len() {
                    let row_line = source_lines[index];
                    if let Some(row_cells) = parse_pipe_row(row_line)
                        && !is_alignment_row(row_line)
                        && row_cells.len() == cells.len()
                    {
                        lines.push(row_line.to_string());
                        index += 1;
                        continue;
                    }
                    break;
                }

                if source_lines
                    .get(index)
                    .is_some_and(|next| !next.trim().is_empty())
                {
                    lines.push(String::new());
                }

                while source_lines
                    .get(index)
                    .is_some_and(|next| !next.trim().is_empty())
                    && source_lines
                        .get(index)
                        .is_some_and(|next| parse_pipe_row(next).is_none())
                {
                    lines.push(source_lines[index].to_string());
                    index += 1;

                    if source_lines
                        .get(index)
                        .is_some_and(|next| !next.trim().is_empty())
                    {
                        lines.push(String::new());
                    }
                }
                continue;
            }
        }

        lines.push(line.to_string());
        index += 1;
    }

    let mut normalized = lines.join("\n");
    if text.ends_with('\n') {
        normalized.push('\n');
    }
    normalized
}

fn unescape_telegram_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                out.push(next);
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn render_table_as_code_block(table: &[Vec<String>]) -> Option<String> {
    if table.is_empty() {
        return None;
    }

    let mut rows: Vec<Vec<String>> = table.to_vec();
    if rows.len() > 1 && rows[0] == rows[1] {
        rows.remove(1);
    }

    let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
    if columns == 0 {
        return None;
    }

    for row in &mut rows {
        row.resize(columns, String::new());
        for cell in row {
            *cell = unescape_telegram_markdown(cell).trim().to_string();
        }
    }

    // Keep the metric above performance tables so it does not widen every row.
    let caption = if rows[0] == ["(instructions:u)", "mean", "range", "count"] {
        let metric = std::mem::replace(&mut rows[0][0], "kind".to_string());
        rows[0][3] = "n".to_string();
        for row in rows.iter_mut().skip(1) {
            row[0] = row[0]
                .split_whitespace()
                .map(|part| match part {
                    "prim" => "prm",
                    "(prim)" => "(prm)",
                    _ => part,
                })
                .collect::<Vec<_>>()
                .join(" ");
            row[2] = row[2].replace(", ", ",");
        }
        Some(metric)
    } else {
        None
    };

    let mut widths = vec![1usize; columns];
    for row in &rows {
        for (idx, cell) in row.iter().enumerate() {
            widths[idx] = widths[idx].max(cell.chars().count());
        }
    }

    let mut lines = Vec::with_capacity(rows.len() + 1);
    if let Some(caption) = caption {
        lines.push(caption);
    }
    for (row_index, row) in rows.iter().enumerate() {
        let mut line = String::new();
        line.push('|');
        for (col_index, cell) in row.iter().enumerate() {
            let width = widths[col_index];
            let len = cell.chars().count();
            line.push(' ');
            line.push_str(cell);
            if len < width {
                line.push_str(&" ".repeat(width - len));
            }
            line.push(' ');
            line.push('|');
        }
        lines.push(line);

        if row_index == 0 && rows.len() > 1 {
            let mut separator = String::new();
            separator.push('|');
            for width in &widths {
                separator.push(' ');
                separator.push_str(&"-".repeat(*width));
                separator.push(' ');
                separator.push('|');
            }
            lines.push(separator);
        }
    }

    Some(format!("```\n{}\n```", lines.join("\n")))
}

/// Parse TWIR Markdown into logical sections using `pulldown-cmark`.
///
/// # Parameters
/// - `text`: Full Markdown source from a TWIR issue.
///
/// # Returns
/// A list of [`Section`]s preserving the order found in the input.
pub fn parse_sections(text: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current: Option<Section> = None;
    let mut buffer = String::new();
    let normalized = normalize_compact_tables(text);
    let parser = Parser::new_ext(&normalized, Options::ENABLE_TABLES);
    let mut link_dest: Option<String> = None;
    let mut list_depth: usize = 0;
    let mut in_code_block = false;
    let mut in_heading = false;
    let mut heading_raw = String::new();
    let mut heading_sanitized = String::new();
    let mut table: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                heading_raw.clear();
                heading_sanitized.clear();
                if level == HeadingLevel::H2 {
                    if let Some(ref mut sec) = current {
                        let line = buffer.trim();
                        if !line.is_empty() {
                            sec.lines.push(line.to_string());
                        }
                    }
                    if let Some(sec) = current.take() {
                        sections.push(sec);
                    }
                    buffer.clear();
                } else if matches!(
                    level,
                    HeadingLevel::H1 | HeadingLevel::H3 | HeadingLevel::H4
                ) {
                    if let Some(ref mut sec) = current {
                        let line = buffer.trim_end();
                        if !line.is_empty() {
                            sec.lines.push(line.to_string());
                        }
                    }
                    buffer.clear();
                }
            }
            Event::End(TagEnd::Heading(level)) => {
                in_heading = false;
                let raw = heading_raw.trim();
                if level == HeadingLevel::H2 {
                    current = Some(Section {
                        title: raw.to_string(),
                        ..Section::default()
                    });
                    buffer.clear();
                } else if matches!(
                    level,
                    HeadingLevel::H1 | HeadingLevel::H3 | HeadingLevel::H4
                ) {
                    if let Some(ref mut sec) = current
                        && !raw.is_empty()
                    {
                        sec.lines.push(format_subheading(raw));
                    }
                    buffer.clear();
                } else if let Some(ref mut sec) = current {
                    let sanitized = heading_sanitized.trim();
                    if !sanitized.is_empty() {
                        let fixed = replace_github_mentions(&fix_bare_link(sanitized));
                        sec.lines.push(fixed);
                    }
                    buffer.clear();
                } else {
                    buffer.clear();
                }
                heading_raw.clear();
                heading_sanitized.clear();
            }
            Event::Start(Tag::List(_)) => {
                if let Some(ref mut sec) = current {
                    let line = buffer.trim_end();
                    if !line.is_empty() {
                        let fixed = replace_github_mentions(&fix_bare_link(line));
                        let indent = "  ".repeat(list_depth.saturating_sub(1));
                        sec.lines.push(format!("{indent}• {fixed}"));
                        buffer.clear();
                    }
                }
                list_depth += 1;
            }
            Event::End(TagEnd::List(_)) => {
                list_depth = list_depth.saturating_sub(1);
            }
            Event::Start(Tag::Item) => {
                buffer.clear();
            }
            Event::End(TagEnd::Item) => {
                if let Some(ref mut sec) = current {
                    let line = buffer.trim_end();
                    if !line.is_empty() {
                        let fixed = replace_github_mentions(&fix_bare_link(line));
                        let indent = "  ".repeat(list_depth.saturating_sub(1));
                        sec.lines.push(format!("{indent}• {fixed}"));
                    }
                }
                buffer.clear();
            }
            Event::Start(Tag::Table(_)) => {
                table.clear();
                if !buffer.trim().is_empty() {
                    if let Some(ref mut sec) = current {
                        sec.lines.push(buffer.trim().to_string());
                    }
                    buffer.clear();
                }
            }
            Event::Start(Tag::TableHead) => {
                row.clear();
            }
            Event::End(TagEnd::TableHead) => {
                table.push(row.clone());
            }
            Event::End(TagEnd::Table) => {
                if let Some(ref mut sec) = current
                    && let Some(block) = render_table_as_code_block(&table)
                {
                    sec.max_table_bytes = sec.max_table_bytes.max(block.len());
                    sec.lines.push(block);
                }
                table.clear();
            }
            Event::Start(Tag::TableRow) => {
                row.clear();
            }
            Event::End(TagEnd::TableRow) => {
                table.push(row.clone());
            }
            Event::Start(Tag::TableCell) => {
                buffer.clear();
            }
            Event::End(TagEnd::TableCell) => {
                row.push(normalize_table_text(buffer.trim()).to_string());
                buffer.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                if list_depth > 0 {
                    let trimmed = buffer.trim_end().to_string();
                    buffer.clear();
                    if !trimmed.is_empty() {
                        buffer.push_str(&trimmed);
                        buffer.push('\n');
                    }
                } else if let Some(ref mut sec) = current {
                    let line = buffer.trim_end();
                    if !line.is_empty() {
                        let fixed = replace_github_mentions(&fix_bare_link(line));
                        sec.lines.push(fixed);
                    }
                    buffer.clear();
                } else {
                    buffer.clear();
                }
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                if in_heading {
                    heading_raw.push('[');
                    heading_sanitized.push('[');
                } else {
                    buffer.push('[');
                }
                link_dest = Some(dest_url.to_string());
            }
            Event::End(TagEnd::Link) => {
                if let Some(d) = link_dest.take() {
                    if in_heading {
                        heading_raw.push_str("](");
                        heading_raw.push_str(&d);
                        heading_raw.push(')');
                        heading_sanitized.push_str("](");
                        heading_sanitized.push_str(&escape_markdown_url(&d));
                        heading_sanitized.push(')');
                    } else {
                        buffer.push_str("](");
                        buffer.push_str(&escape_markdown_url(&d));
                        buffer.push(')');
                    }
                }
            }
            Event::Start(Tag::BlockQuote(_)) => {
                if !buffer.is_empty() && !buffer.ends_with('\n') {
                    buffer.push('\n');
                }
                buffer.push_str("\\> ");
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                buffer.push('\n');
            }
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
                buffer.push_str("```\n");
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                if !buffer.ends_with('\n') {
                    buffer.push('\n');
                }
                buffer.push_str("```");
            }
            Event::Text(t) | Event::Code(t) => {
                if in_heading {
                    heading_raw.push_str(&t);
                    heading_sanitized.push_str(&escape(&t));
                } else if in_code_block {
                    buffer.push_str(&t);
                } else {
                    buffer.push_str(&escape(&t));
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_code_block {
                    buffer.push('\n');
                } else if in_heading {
                    heading_raw.push(' ');
                    heading_sanitized.push(' ');
                } else {
                    buffer.push(' ');
                }
            }
            Event::Html(html) if html.trim_start().starts_with("<br") => {
                if in_heading {
                    heading_raw.push(' ');
                    heading_sanitized.push(' ');
                } else {
                    buffer.push(' ');
                }
            }
            _ => {}
        }
    }
    if let Some(mut sec) = current {
        if !buffer.trim().is_empty() {
            let fixed = replace_github_mentions(&fix_bare_link(buffer.trim()));
            sec.lines.push(fixed);
        }
        sections.push(sec);
    }
    sections
}
