*Part 4/6*

• [migrate more stuff to the next solver](https://github.com/rust-lang/rust-analyzer/pull/20841)
• [migrate variance to the next solver and remove lint allows from its stuff](https://github.com/rust-lang/rust-analyzer/pull/20867)
• [rip Chalk out of the codebase 🎉](https://github.com/rust-lang/rust-analyzer/pull/20873)
• [support underscore suffix parameter hide inlayHints](https://github.com/rust-lang/rust-analyzer/pull/20858)
• [use FileId::MAX for id assertion in PathInterner::intern](https://github.com/rust-lang/rust-analyzer/pull/20757)

**Rust Compiler Performance Triage:** 📊
Fairly busy week, with lots of mixed results\. However, overall we ended with a slight improvement on average\.
Triage done by [simulacrum](https://github.com/simulacrum)\. Revision range: [956f47c3\.\.4068bafe](https://perf.rust-lang.org/?start=956f47c32f1bd97b22cd702d7ccf78f0f0d42c34&end=4068bafedd8ba724e332a5221c06a6fa531a30d2&absolute=false&stat=instructions%3Au)
2 Regressions, 5 Improvements, 10 Mixed; 5 of them in rollups
39 artifact comparisons made in total
[Full report here](https://github.com/rust-lang/rustc-perf/blob/master/triage/2025/2025-10-20.md)
**[Approved RFCs](https://github.com/rust-lang/rfcs/commits/master)**
Changes to Rust follow the Rust [RFC \(request for comments\) process](https://github.com/rust-lang/rfcs#rust-rfcs)\. These are the RFCs that were approved for implementation this week:
• No RFCs were approved this week\.
**Final Comment Period**
Every week, [the team](https://www.rust-lang.org/team.html) announces the 'final comment period' for RFCs and key PRs which are reaching a decision\. Express your opinions now\.

**Tracking Issues & PRs:** 📌
[Rust](https://github.com/rust-lang/rust/issues?q=is%3Aopen+label%3Afinal-comment-period+sort%3Aupdated-desc)
• [Tracking Issue for NEON fp16 intrinsics](https://github.com/rust-lang/rust/issues/136306)
• [Change Location<'\_\> lifetime to 'static in Panic\[Hook\]Info](https://github.com/rust-lang/rust/pull/146561)
• [Tracking Issue for substr\_range and related methods](https://github.com/rust-lang/rust/issues/126769)
• [repr\(transparent\): do not consider repr\(C\) types to be 1\-ZST](https://github.com/rust-lang/rust/pull/147185)
• [Don't require T: RefUnwindSafe for vec::IntoIter<T\>: UnwindSafe](https://github.com/rust-lang/rust/pull/145665)
• [Stabilize \-Zno\-jump\-tables into \-Cjump\-tables\=bool](https://github.com/rust-lang/rust/pull/145974)
• [Tracking issue for alloc\_layout\_extra](https://github.com/rust-lang/rust/issues/55724)
• [Add warn\-by\-default lint for visibility on const \_ declarations](https://github.com/rust-lang/rust/pull/147136)
• [Tracking Issue for debug\_closure\_helpers](https://github.com/rust-lang/rust/issues/117729)
• [fully deprecate the legacy integral modules](https://github.com/rust-lang/rust/pull/146882)
• [Tracking Issue for fmt\_from\_fn](https://github.com/rust-lang/rust/issues/146705)
• [Make IoSlice and IoSliceMut methods unstably const](https://github.com/rust-lang/rust/pull/144090)
• [Tracking Issue for VecDeque::pop\_front\_if & VecDeque::pop\_back\_if](https://github.com/rust-lang/rust/issues/135889)
• \[disposition: unspecified\] [\[std\]\[BTree\] Fix behavior of ::append to match documentation, ::insert, and ::extend](https://github.com/rust-lang/rust/pull/145628)
• [Impls and impl items inherit dead\_code lint level of the corresponding traits and trait items](https://github.com/rust-lang/rust/pull/144113)
• [Document MaybeUninit bit validity](https://github.com/rust-lang/rust/pull/140463)
[Compiler Team](https://github.com/rust-lang/compiler-team/issues?q=label%3Amajor-change%20%20label%3Afinal-comment-period) [\(MCPs only\)](https://forge.rust-lang.org/compiler/mcp.html)
• [Move unreachable code lint from HIR type check to a proper lint](https://github.com/rust-lang/compiler-team/issues/931)
