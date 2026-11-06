# Markdown Parsing Guidelines

- Always use established crates like `pulldown-cmark` for reading and parsing Markdown files.
- Avoid implementing Markdown parsing logic manually unless absolutely necessary.
- Review all Markdown files in the repository before starting new tasks, as they may contain important instructions or data for the project.
- Generated Telegram MarkdownV2 must follow the [Telegram Bot API rules](https://core.telegram.org/bots/api#markdownv2-style).
- Plain-text escaping is shared in `src/shared/markdown.rs`. Changes to this
  helper must be compared with an external reference implementation such as
  `teloxide::utils::markdown::escape`, including reserved characters,
  backslashes, and Unicode.
- `teloxide` provides escaping helpers but no complete MarkdownV2 validator.
  The project retains its minimal validator in
  [`src/validator.rs`](src/validator.rs) for the supported formatting subset,
  with golden fixtures checking complete generated messages.
