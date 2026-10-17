*Part 3/6*

📰 **UPDATES FROM THE RUST PROJECT** 📰
369 pull requests were [merged in the last week](https://github.com/search?q=is%3Apr+org%3Arust-lang+is%3Amerged+merged%3A2025-10-14..2025-10-21)

**Compiler:** 🛠️
• [add a \!\= check to ChunkedBitSet::union](https://github.com/rust-lang/rust/pull/147619)
• [bitset cleanups](https://github.com/rust-lang/rust/pull/147630)
• [deduced\_param\_attrs: check Freeze on monomorphic types](https://github.com/rust-lang/rust/pull/147695)
• [deny\-by\-default never type lints](https://github.com/rust-lang/rust/pull/146167)
• [improve error message for ambiguous numeric types in closure parameters](https://github.com/rust-lang/rust/pull/147577)
• [remove boxes from AST list elements](https://github.com/rust-lang/rust/pull/146221)
• [TaskDeps improvements](https://github.com/rust-lang/rust/pull/147508)
• [unused\_must\_use: Don't warn on Result<\(\), Uninhabited\> or ControlFlow<Uninhabited, \(\)\>](https://github.com/rust-lang/rust/pull/147382)
• [use regular Vec in BitSet](https://github.com/rust-lang/rust/pull/147644)

**Library:** 📚
• [const mem::drop](https://github.com/rust-lang/rust/pull/147708)
• [constify basic Clone impls](https://github.com/rust-lang/rust/pull/146976)
• [iter repeat: panic on last](https://github.com/rust-lang/rust/pull/147258)
• [stabilise rotate\_left and rotate\_right in \[\_\] as const fn items](https://github.com/rust-lang/rust/pull/146841)
• [stabilize rwlock\_downgrade library feature](https://github.com/rust-lang/rust/pull/143191)

**Cargo:** 📦
• [check: Fix suggested command for bin package](https://github.com/rust-lang/cargo/pull/16127)
• [script: Remove name sanitiztion outside what is strictly required](https://github.com/rust-lang/cargo/pull/16120)
• [script: Tweak cargo script build\-dir / target\-dir](https://github.com/rust-lang/cargo/pull/16086)

**Rustdoc:** 📖
• [search: stringdex 0\.0\.2](https://github.com/rust-lang/rust/pull/147660)
• [fix passes order so intra\-doc links are collected after stripping passes](https://github.com/rust-lang/rust/pull/147809)

**Clippy:** 🔧
• [empty\_enum: don't lint if all variants happen to be cfg\-d out](https://github.com/rust-lang/rust-clippy/pull/15911)
• [option\_option: split part of diagnostic message into help message](https://github.com/rust-lang/rust-clippy/pull/15870)
• [unnecessary\_safety\_comment Some fixes regarding comments above attributes](https://github.com/rust-lang/rust-clippy/pull/15678)
• [allow explicit\_write in tests](https://github.com/rust-lang/rust-clippy/pull/15862)
• [dereference argument of manual\_div\_ceil\(\) if needed](https://github.com/rust-lang/rust-clippy/pull/15706)
• [manual\_rotate: also recognize non\-consts](https://github.com/rust-lang/rust-clippy/pull/15402)
• [overhaul mutex\_\{atomic,integer\}](https://github.com/rust-lang/rust-clippy/pull/15632)

**Rust\-Analyzer:** 🤖
• [parser: Don't error on frontmatter](https://github.com/rust-lang/rust-analyzer/pull/20854)
• [improve fixture support](https://github.com/rust-lang/rust-analyzer/pull/20855)
• [fix invalid RestPat for convert\_tuple\_struct\_to\_named\_struct](https://github.com/rust-lang/rust-analyzer/pull/20880)
• [fix missing RestPat for convert\_named\_struct\_to\_tuple\_struct](https://github.com/rust-lang/rust-analyzer/pull/20872)
• [don't make convert\_to\_guarded\_return applicable on let\-else](https://github.com/rust-lang/rust-analyzer/pull/20838)
• [fix signature\_help to proto conversion creating invalid utf16 offsets](https://github.com/rust-lang/rust-analyzer/pull/20876)
• [support break with value in completions](https://github.com/rust-lang/rust-analyzer/pull/20673)
• [support else blocks with \! return type in convert\_to\_guarded\_return](https://github.com/rust-lang/rust-analyzer/pull/20758)
• [support match inside if in pull\_assignment\_up](https://github.com/rust-lang/rust-analyzer/pull/20772)
