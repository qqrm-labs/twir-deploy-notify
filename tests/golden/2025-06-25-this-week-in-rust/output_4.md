*Part 4/6*

• [rust\-analyzer: add fn parent\(self, db\) → GenericDef to hir::TypeParam](https://github.com/rust-lang/rust-analyzer/pull/20046)
• [rust\-analyzer: cleanup folding\_ranges and support more things](https://github.com/rust-lang/rust-analyzer/pull/20080)
• [rust\-analyzer: do not default to 'static for trait object lifetimes](https://github.com/rust-lang/rust-analyzer/pull/20036)
• [rust\-analyzer: closure capturing for let exprs](https://github.com/rust-lang/rust-analyzer/pull/20039)
• [rust\-analyzer: fix cargo project manifest not pointing to the workspace root](https://github.com/rust-lang/rust-analyzer/pull/20069)
• [rust\-analyzer: in "Wrap return type" assist, don't wrap exit points if they already have the right type](https://github.com/rust-lang/rust-analyzer/pull/20061)
• [rust\-analyzer: respect \.cargo/config\.toml build\.target\-dir](https://github.com/rust-lang/rust-analyzer/pull/20072)
• [rust\-analyzer: temporarily disable \+ typing handler as it moves the cursor position](https://github.com/rust-lang/rust-analyzer/pull/20042)
• [rust\-analyzer: use ROOT hygiene for args inside new format\_args\! expansion](https://github.com/rust-lang/rust-analyzer/pull/20073)
• [rust\-analyzer: hide imported privates if private editable is disabled](https://github.com/rust-lang/rust-analyzer/pull/20025)
• [rust\-analyzer: mimic rustc's new format\_args\! expansion](https://github.com/rust-lang/rust-analyzer/pull/20056)

**Rust Compiler Performance Triage:** 📊
A week dominated by the landing of a large patch implementing [RFC\#3729](https://github.com/rust-lang/rfcs/pull/3729) which unfortunately introduced rather sizeable performance regressions \(avg of \~1% instruction count on 111 primary benchmarks\)\. This was deemed worth it so that the patch could land and performance could be won back in follow up PRs\.
Triage done by [rylev](https://github.com/rylev)\. Revision range: [45acf54e\.\.42245d34](https://perf.rust-lang.org/?start=45acf54eea118ed27927282b5e0bfdcd80b7987c&end=42245d34d22ade32b3f276dcf74deb826841594c&absolute=false&stat=instructions%3Au)
Summary:
```
(instructions:u)
| kind          | mean  | range          | n   |
| ------------- | ----- | -------------- | --- |
| Reg x (prim)  | 1.1%  | [0.2%,9.1%]    | 123 |
| Reg x (sec)   | 1.0%  | [0.1%,4.6%]    | 86  |
| Imp v (prim)  | -3.8% | [-7.3%,-0.3%]  | 2   |
| Imp v (sec)   | -2.3% | [-18.5%,-0.2%] | 44  |
| All xv (prim) | 1.0%  | [-7.3%,9.1%]   | 125 |
```
2 Regressions, 4 Improvements, 10 Mixed; 7 of them in rollups 40 artifact comparisons made in total
[Full report here](https://github.com/rust-lang/rustc-perf/blob/a63db4d1799853b334e4106d914fba24e49c8782/triage/2025/2025-06-24.md)
**[Approved RFCs](https://github.com/rust-lang/rfcs/commits/master)**
Changes to Rust follow the Rust [RFC \(request for comments\) process](https://github.com/rust-lang/rfcs#rust-rfcs)\. These are the RFCs that were approved for implementation this week:
• No RFCs were approved this week\.
**Final Comment Period**
Every week, [the team](https://www.rust-lang.org/team.html) announces the 'final comment period' for RFCs and key PRs which are reaching a decision\. Express your opinions now\.

**Tracking Issues & PRs:** 📌
[Rust](https://github.com/rust-lang/rust/issues?q=is%3Aopen+label%3Afinal-comment-period+sort%3Aupdated-desc)
• [Use lld by default on x86\_64\-unknown\-linux\-gnu stable](https://github.com/rust-lang/rust/pull/140525)
• [Allow \#\[must\_use\] on associated types to warn on unused values in generic contexts](https://github.com/rust-lang/rust/pull/142590)
• [Fix proc\_macro::Ident 's handling of $crate](https://github.com/rust-lang/rust/pull/141996)
• [Ensure non\-empty buffers for large vectored I/O](https://github.com/rust-lang/rust/pull/138879)
[Rust RFCs](https://github.com/rust-lang/rfcs/labels/final-comment-period)
• [RFC: \-\-crate\-attr](https://github.com/rust-lang/rfcs/pull/3791)
