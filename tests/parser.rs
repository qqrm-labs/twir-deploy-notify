use twir_deploy_notify::parser;

use parser::parse_sections;
mod common;

#[test]
fn code_block_before_next_heading() {
    let input = "## First\n```\nline1\nline2\n```\n\n## Second";
    let sections = parse_sections(input);
    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].title, "First");
    assert_eq!(sections[0].lines, vec!["```\nline1\nline2\n```"]);
}

#[test]
fn bare_link_with_parentheses() {
    let input = "## Section\n- Some text (https://example.com/path(1))";
    let sections = parse_sections(input);
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].title, "Section");
    assert_eq!(
        sections[0].lines,
        vec!["• [Some text](https://example.com/path\\(1\\))"]
    );
    common::assert_valid_markdown(&sections[0].lines[0]);
}

#[test]
fn fenced_code_block_round_trip() {
    let input = "## Section\n```\nfn main() {\n    println!(\"Hello_world\");\n}\n```";
    let sections = parse_sections(input);
    assert_eq!(sections.len(), 1);
    assert_eq!(
        sections[0].lines,
        vec!["```\nfn main() {\n    println!(\"Hello_world\");\n}\n```"]
    );
}

#[test]
fn loose_list_preserves_parent_items() {
    let input = "## Section\n[Cargo](https://example.com/cargo)\n* [Parent](https://example.com/parent)\n  * [Child](https://example.com/child)\n\n* *No calls for testing were issued by [Rust](https://example.com/rust).*";
    let sections = parse_sections(input);
    assert_eq!(sections.len(), 1);
    let lines = &sections[0].lines;
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0], "[Cargo](https://example.com/cargo)");
    assert_eq!(lines[1], "• [Parent](https://example.com/parent)");
    assert_eq!(lines[2], "  • [Child](https://example.com/child)");
    assert_eq!(
        lines[3],
        "• No calls for testing were issued by [Rust](https://example.com/rust)\\."
    );
}

#[test]
fn regression_table_ascii_aligned() {
    let input = include_str!("regression_table.md");
    let sections = parse_sections(input);
    assert_eq!(sections.len(), 1);
    let lines = &sections[0].lines;
    assert_eq!(lines.len(), 1);
    let block = &lines[0];
    assert!(block.starts_with("```\n"));
    assert!(block.ends_with("\n```"));
    let body: Vec<&str> = block.lines().filter(|line| line.starts_with('|')).collect();
    assert_eq!(body.len(), 7);
    let separators: Vec<_> = body[0].match_indices('|').map(|(idx, _)| idx).collect();
    for line in body {
        assert!(line.is_ascii());
        assert_eq!(
            line.match_indices('|')
                .map(|(idx, _)| idx)
                .collect::<Vec<_>>(),
            separators
        );
    }
}

#[test]
fn regression_table_compact() {
    let input = include_str!("regression_table.md");
    let sections = parse_sections(input);
    assert_eq!(sections.len(), 1);
    let lines = &sections[0].lines;
    assert_eq!(lines.len(), 1);
    let block = &lines[0];
    assert!(block.starts_with("```"));
    assert!(block.contains("\n(instructions:u)\n| kind"));
    assert!(
        block
            .lines()
            .filter(|line| line.starts_with('|'))
            .all(|line| line.len() == 45)
    );
    assert!(block.contains("| Reg x prim  | 0.4%  | [0.1%,0.9%]   | 47  |"));
    assert!(block.contains("| All xv prim | -0.5% | [-4.1%,0.9%]  | 169 |"));
    assert!(block.contains("| -----"));
    assert!(!block.contains("\\|"));
}

#[test]
fn compact_summary_table_without_separator_becomes_code_block() {
    let input = "## Perf\nSummary:\n| (instructions:u) | mean | range | count |\n| Reg x  (prim) | 0.4% | [0.4%, 0.5%] | 3 |\n| Reg x  (sec) | 0.6% | [0.1%, 1.2%] | 8 |\n| Imp v  (prim) | -0.9% | [-2.5%, -0.1%] | 110 |\n| Imp v  (sec) | -0.8% | [-2.7%, -0.1%] | 77 |\n| All xv (prim) | -0.9% | [-2.5%, 0.5%] | 113 |\n0 Regressions, 6 Improvements, 3 Mixed; 5 of them in rollups\n31 artifact comparisons made in total\n";
    let sections = parse_sections(input);
    assert_eq!(sections.len(), 1);

    let lines = &sections[0].lines;
    assert_eq!(lines[0], "Summary:");
    assert!(lines[1].starts_with("```\n"));
    assert!(lines[1].contains("\n(instructions:u)\n| kind"));
    assert!(lines[1].contains("| Reg x (prim)"));
    assert!(lines[1].contains("| ----"));
    assert_eq!(
        lines[2],
        "0 Regressions, 6 Improvements, 3 Mixed; 5 of them in rollups"
    );
    assert_eq!(lines[3], "31 artifact comparisons made in total");
}
