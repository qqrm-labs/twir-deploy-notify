use twir_deploy_notify::generator;

use generator::{TELEGRAM_LIMIT, generate_posts, split_posts};
use proptest::prelude::*;
mod common;

fn arb_dash_boundary_short() -> impl Strategy<Value = String> {
    let prefix_re = format!(r"[A-Za-z0-9]{{{}}}", TELEGRAM_LIMIT - 1);
    proptest::string::string_regex(&prefix_re)
        .unwrap()
        .prop_flat_map(|pre| {
            proptest::string::string_regex("[A-Za-z0-9]{0,10}")
                .unwrap()
                .prop_map(move |post| format!("{pre}\\-{post}"))
        })
}

fn arb_long_line() -> impl Strategy<Value = String> {
    let regex = format!(
        r"[A-Za-z0-9\\*_]{{{},{}}}",
        TELEGRAM_LIMIT * 2,
        TELEGRAM_LIMIT * 3
    );
    proptest::string::string_regex(&regex).unwrap()
}

fn arb_escaped_text() -> impl Strategy<Value = String> {
    let lower = TELEGRAM_LIMIT - 5;
    let upper = TELEGRAM_LIMIT + 5;
    let regex = format!(r"(?:[A-Za-z]|\\\\[-!#\\.]){{{lower},{upper}}}");
    proptest::string::string_regex(&regex).unwrap()
}

fn arb_special_line() -> impl Strategy<Value = String> {
    let word = proptest::string::string_regex("[A-Za-z0-9]{1,5}").unwrap();
    let seq = prop_oneof![Just("\\".to_string()), Just("\\-".to_string()),];
    let word2 = proptest::string::string_regex("[A-Za-z0-9]{0,5}").unwrap();
    (word, prop::collection::vec(seq, 1..3), word2)
        .prop_map(|(pre, seqs, post)| format!("{pre}{}{post}", seqs.concat()))
}

fn arb_multiline_special() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_special_line(), 2..5).prop_map(|lines| lines.join("\n"))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn split_posts_handles_long_lines(lines in prop::collection::vec(arb_long_line(), 1..3)) {
        let input = lines.join("\n");
        let posts = split_posts(&input, TELEGRAM_LIMIT);
        prop_assert!(!posts.is_empty());
        for p in posts {
            prop_assert!(p.len() <= TELEGRAM_LIMIT);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn escaped_chunks_are_valid(line in arb_escaped_text()) {
        let posts = split_posts(&line, TELEGRAM_LIMIT);
        prop_assert!(!posts.is_empty());
        for p in posts {
            common::assert_valid_markdown(&p);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn dash_boundary_preserves_escape(text in arb_dash_boundary_short()) {
        let posts = split_posts(&text, TELEGRAM_LIMIT);
        prop_assert!(!posts.is_empty());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn dash_escape_at_boundary(input in arb_dash_boundary_short()) {
        let posts = split_posts(&input, TELEGRAM_LIMIT);
        prop_assert!(posts.len() >= 2);
        for p in posts {
            prop_assert!(!p.starts_with('-'));
            common::assert_valid_markdown(&p);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn special_sequences_are_valid(input in arb_multiline_special()) {
        let posts = split_posts(&input, TELEGRAM_LIMIT);
        prop_assert!(!posts.is_empty());
        for p in posts {
            prop_assert!(!p.is_empty());
            common::assert_valid_markdown(&p);
        }
    }
}

#[test]
fn boundary_escape_preserved() {
    let mut input = "a".repeat(TELEGRAM_LIMIT - 1);
    input.push('\\');
    input.push_str("-b");
    let posts = split_posts(&input, TELEGRAM_LIMIT);
    assert!(posts.len() >= 2);
    assert!(!posts[1].starts_with('-'));
    for p in posts {
        common::assert_valid_markdown(&p);
    }
}

#[test]
fn single_section_has_expected_prefix() {
    let input = "Title: Test\nNumber: 1\nDate: 2025-01-01\n\n## News\n- item\n";
    let posts = generator::generate_posts(input.to_string()).unwrap();
    assert_eq!(posts.len(), 1);
    assert!(!posts[0].starts_with("*Part"));
}

#[test]
fn quote_section_spacing() {
    let posts =
        generator::generate_posts(include_str!("2025-07-02-this-week-in-rust.md").to_string())
            .unwrap();
    assert!(posts.len() >= 5);
    let lines: Vec<_> = posts[4].lines().collect();
    let idx = lines
        .iter()
        .position(|l| l.contains("Quote of the Week"))
        .unwrap();
    assert!(lines.get(idx + 1).is_some_and(|l| l.is_empty()));
    assert!(lines.get(idx + 2).is_some_and(|l| !l.is_empty()));
    let author_idx = lines
        .iter()
        .position(|l| l.trim_start().starts_with('–'))
        .unwrap();
    assert!(lines.get(author_idx + 1).is_some_and(|l| l.is_empty()));
}

#[test]
fn multiple_quotes_formatted_uniformly() {
    let input = r#"Title: Test
Number: 1
Date: 2025-01-01

## Jobs

# Quote of the Week

> First quote line

– Author One

> Second quote line

– Author Two

Despite a lack of suggestions, we picked these.
Please submit quotes and vote for next week!
Thanks to everyone who suggested quotes.
"#;

    let posts = generator::generate_posts(input.to_string()).unwrap();
    let combined = posts.join("\n");
    let quote_block = combined
        .split("Quote of the Week")
        .nth(1)
        .expect("quote section present");
    assert!(quote_block.contains("_First quote line_"));
    assert!(quote_block.contains("_Second quote line_"));
    assert!(quote_block.contains("– Author One"));
    assert!(quote_block.contains("– Author Two"));
    assert!(!quote_block.contains("Please submit quotes"));
    assert!(!quote_block.contains("Thanks to"));
    assert!(!quote_block.contains("Despite a lack of suggestions"));
}

#[test]
fn quote_containing_section_name_is_preserved() {
    let input = r#"Title: This Week in Rust 672
Number: 672
Date: 2026-10-07

## Jobs

# Quote of the Week

> There ain't no rules here in Quote of the Week - it's survival of the wittest

– [Simon Buchan on rust-users](https://users.rust-lang.org/t/twir-quote-of-the-week/328/1814?u=llogiq)
"#;

    let posts = generate_posts(input.to_string()).unwrap();
    let combined = posts.join("\n");
    assert!(combined.contains(
        r"_There ain't no rules here in Quote of the Week \- it's survival of the wittest_"
    ));
    assert_eq!(combined.matches("**Quote of the Week:** 💬").count(), 1);
    assert!(combined.contains(
        r"– [Simon Buchan on rust\-users](https://users.rust-lang.org/t/twir-quote-of-the-week/328/1814?u=llogiq)"
    ));
    for post in &posts {
        common::assert_valid_markdown(post);
    }
}

#[test]
fn section_name_in_prose_does_not_start_quote_section() {
    let input = "Title: Test\nNumber: 1\nDate: 2025-01-01\n\n## News\nRead Quote of the Week for more\n\nThanks to our contributors\n";

    let posts = generate_posts(input.to_string()).unwrap();
    let combined = posts.join("\n");
    assert!(combined.contains("Read Quote of the Week for more"));
    assert!(combined.contains("Thanks to our contributors"));
    assert!(!combined.contains("**Quote of the Week:** 💬"));
    for post in &posts {
        common::assert_valid_markdown(post);
    }
}

#[test]
fn jobs_url_simplified() {
    let input = "Title: Test\nNumber: 1\nDate: 2025-01-01\n\n## Jobs\nPlease see the latest [Who's Hiring thread on r/rust](https://example.com/thread)\n";
    let posts = generator::generate_posts(input.to_string()).unwrap();
    let combined = posts.join("\n");
    assert!(combined.contains("[Rust Job Reddit Thread](https://example.com/thread)"));
}

#[test]
fn table_compact_format() {
    let input = "Title: Test\nNumber: 1\nDate: 2024-01-01\n\n## Table\n| Name | Score |\n|------|------|\n| Foo | 10 |\n| Bar | 20 |\n";
    let posts = generate_posts(input.to_string()).unwrap();
    assert!(posts[0].contains("```\n"));
    assert!(
        posts[0].contains("| Name | Score |\n| ---- | ----- |\n| Foo  | 10    |\n| Bar  | 20    |")
    );
    assert!(!posts[0].contains("\\|"));
    common::assert_valid_markdown(&posts[0]);
}

#[test]
fn table_moves_whole_when_remaining_space_is_one_byte_short() {
    let sections = twir_deploy_notify::parser::parse_sections(include_str!("regression_table.md"));
    let table = &sections[0].lines[0];
    let prefix = "intro";
    let input = format!("{prefix}\n{table}");
    assert_eq!(split_posts(&input, input.len()), vec![input.clone()]);
    assert_eq!(
        split_posts(&input, input.len() - 1),
        vec![prefix.to_string(), table.clone()]
    );
    assert_eq!(split_posts(table, table.len()), vec![table.clone()]);
}

#[test]
fn consecutive_tables_remain_complete_without_empty_posts() {
    let table = "```\n| A | B |\n| - | - |\n| 1 | 2 |\n```";
    let input = format!("{table}\n{table}\nafter");
    assert_eq!(split_posts(&input, input.len()), vec![input.clone()]);
    let posts = split_posts(&input, table.len());
    assert_eq!(posts, vec![table, table, "after"]);
    for post in &posts {
        common::assert_valid_markdown(post);
    }
}

#[test]
fn generated_performance_table_stays_in_one_message() {
    let source = include_str!("regression_table.md");
    let sections = twir_deploy_notify::parser::parse_sections(source);
    let table = &sections[0].lines[0];
    let input = format!(
        "Title: Test\nNumber: 1\nDate: 2026-10-14\n\n## Perf\n{}\n\n{}",
        "a".repeat(TELEGRAM_LIMIT - 150),
        source.strip_prefix("## Perf\n").unwrap()
    );
    let posts = generate_posts(input).unwrap();
    assert!(!posts[0].contains("(instructions:u)"));
    assert!(posts[1].contains(table));
    assert_eq!(posts.iter().filter(|post| post.contains(table)).count(), 1);
    for post in &posts {
        assert!(post.len() <= TELEGRAM_LIMIT);
        common::assert_valid_markdown(post);
    }
}

#[test]
fn table_that_cannot_fit_with_part_header_fails_generation() {
    // The rendered table fits the raw limit, but not a numbered message.
    let input = format!(
        "## Table\n| A | B |\n| - | - |\n| {} | 1 |\n",
        "x".repeat(1321)
    );
    let sections = twir_deploy_notify::parser::parse_sections(&input);
    let table = &sections[0].lines[0];
    assert!(table.len() < TELEGRAM_LIMIT);
    assert!(table.len() + "*Part 2/2*\n\n".len() > TELEGRAM_LIMIT);
    let error = generate_posts(input).unwrap_err();
    assert!(error.to_string().contains("refusing to split the table"));
}

#[test]
fn oversized_table_fails_generation() {
    let input = format!(
        "## Table\n| A | B |\n| - | - |\n{}",
        "| row | value |\n".repeat(300)
    );
    let error = generate_posts(input).unwrap_err();
    assert!(error.to_string().contains("refusing to split the table"));
}

#[test]
fn unicode_posts_reserve_space_for_two_digit_part_numbers() {
    let input = (0..10)
        .map(|index| format!("## Section {index}\n{}\n", "Я".repeat(1978)))
        .collect::<String>();
    let posts = generate_posts(input).unwrap();
    assert!(posts.len() >= 10);
    assert!(posts.last().unwrap().starts_with("*Part 10/10*"));
    for post in &posts {
        assert!(post.len() <= TELEGRAM_LIMIT);
        common::assert_valid_markdown(post);
    }
}

#[test]
fn compact_summary_without_separator_renders_as_code_block() {
    let input = "Title: Test\nNumber: 1\nDate: 2025-01-01\n\n## Perf\nSummary:\n| (instructions:u) | mean | range | count |\n| Reg x  (prim) | 0.4% | [0.4%, 0.5%] | 3 |\n| Reg x  (sec) | 0.6% | [0.1%, 1.2%] | 8 |\n| Imp v  (prim) | -0.9% | [-2.5%, -0.1%] | 110 |\n| Imp v  (sec) | -0.8% | [-2.7%, -0.1%] | 77 |\n| All xv (prim) | -0.9% | [-2.5%, 0.5%] | 113 |\n0 Regressions, 6 Improvements, 3 Mixed; 5 of them in rollups\n31 artifact comparisons made in total\n";
    let posts = generate_posts(input.to_string()).unwrap();
    let combined = posts.join("\n");
    assert!(combined.contains("Summary:"));
    assert!(combined.contains("```\n(instructions:u)\n| kind"));
    assert!(combined.contains("| Reg x (prm)"));
    assert!(combined.contains("0 Regressions, 6 Improvements, 3 Mixed; 5 of them in rollups"));
    common::assert_valid_markdown(&combined);
}

#[test]
fn heading_parentheses_are_escaped() {
    let input = "Title: Test\nNumber: 1\nDate: 2025-01-01\n\n## Tracking\n##### [Compiler Team](https://example.com) [(MCPs only)](https://example.org)\n* [Issue](https://example.com/issue)\n";
    let posts = generate_posts(input.to_string()).unwrap();
    assert!(posts.iter().any(|p| p.contains("\\(MCPs only\\)")));
    for post in posts {
        common::assert_valid_markdown(&post);
    }
}

#[test]
fn latest_issue_call_for_testing_is_formatted() {
    let posts =
        generate_posts(include_str!("2025-10-22-this-week-in-rust.md").to_string()).unwrap();
    assert!(posts.len() >= 5);
    let combined = posts.join("\n");
    assert!(combined.contains(
        "• [Tracking Issue for cargo\\-script RFC 3424](https://github.com/rust-lang/cargo/issues/12207)"
    ));
    assert!(combined.contains(
        "  • [Testing Steps](https://github.com/rust-lang/cargo/issues/12207#issuecomment-3412997290)"
    ));
    assert!(combined.contains(
        "• No calls for testing were issued this week by [Rust](https://github.com/rust-lang/rust/labels/call-for-testing)"
    ));
}
