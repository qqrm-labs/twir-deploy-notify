# Architecture Overview

The crate turns "This Week in Rust" Markdown into Telegram posts.

## Layout
- `src/main.rs` – CLI entrypoint.
- `src/cli.rs` – parses arguments and triggers post generation.
 - `src/shared/` – parser, generator and validator used by the library.
 - `src/generator.rs`, `src/parser.rs`, `src/validator.rs` – thin re-exports of shared modules.
 - `src/bin/verify_posts.rs` – checks posts by sending them to Telegram.
 - `last_sent.txt` – workflow artifact with the last processed issue.

## Processing
1. Markdown files start with `Title:`, `Number:` and `Date:` lines.
2. `pulldown-cmark` splits the rest into sections using `##` headings and list items.
3. Each item becomes Telegram Markdown with special characters escaped.
4. A final link to the web version is derived from the date and number.

## Posts
- Sections are bundled into messages with a conservative 4000-byte budget.
- Long bundles are split, and later messages are prefixed with `*Part X/Y*`.
- Tables stay in one message, including their caption, header, and rows. If a
  table does not fit in the remaining space, it moves to the next message.
  The size budget includes the part prefix and final newline. A table too large
  for a single message fails generation before delivery instead of being split.
- After splitting, fragments of up to 500 bytes are joined to the preceding
  message when they fit, or to the following message when that fits. Joining
  preserves content order and whole tables, includes a blank-line separator in
  the budget, and happens before final part numbering.
- The `--plain` flag strips formatting for plain text destinations.

## Telegram Delivery Flow
1. The CLI always sends posts to the developer chat first. Every part is delivered sequentially, the response payload is parsed to confirm `ok == true`, and the next post is sent only after the acknowledgement arrives.
2. The sender sleeps for `TELEGRAM_DELAY_MS` (currently one second) between posts to avoid spamming Telegram.
3. Developer deliveries are not pinned; once the final acknowledgement is observed the CLI records the exact acknowledgement count and only proceeds when it matches the number of posts prepared for delivery.
4. Production credentials are fetched only after the developer delivery succeeds with a full set of acknowledgements. The exact same posts are then sent to the production chat with the same acknowledgement-and-delay semantics. If any send fails or the acknowledgements do not cover every post, the pipeline aborts before touching the production chat.

## Delivery State Contract

The production workflow is stateful. Its idempotency depends on the following
files and GitHub Actions artifacts being present and passed from one successful
production run to the next:

- `last_sent.txt` contains exactly one path: the TWIR source file successfully
  delivered to the production chat.
- `last-sent-prod` is the production artifact containing `last_sent.txt`.
  Every successful `TWIR Prod summary` run must publish it. A delivery run
  creates it only after the production delivery succeeds; a no-op run copies
  forward the marker it downloaded from the preceding successful run.
- `last-sent-dev-prod` is the developer-stage artifact for a production run.
  It may be used by the developer stage, but it is not the source of truth for
  production deduplication.

The next production run downloads `last-sent-prod` before choosing whether to
send. It must compare the latest TWIR path with the downloaded marker and skip
both deliveries when they match. Do not remove, rename, or make the production
marker conditional on a delivery without updating this contract and its
regression test. In particular, a no-op run without `last-sent-prod` breaks the
state chain and can cause the next run to resend an already delivered issue.
Because the production preflight downloads the marker before checking out any
repository, its GitHub CLI calls must pass `--repo "${{ github.repository }}"`.

Marker recovery reads the latest 100 production workflow runs without
server-side status or branch filters, then selects up to ten successful runs
locally. Those filters have returned stale run indexes. Successful manual
production deliveries from release branches are included, so a recovery's
marker carries forward to scheduled runs on `main`. The first usable marker
is restored; if none is available, delivery proceeds with the Telegram pin
guard still enabled. Failed deliveries do not publish a new marker.

## Telegram Pin Guard

The marker chain above is necessary but not sufficient: GitHub Actions run
listings (both `gh run list` and the REST runs endpoint filtered by status)
intermittently serve stale results, so a run may inherit an outdated or missing
marker and treat an already delivered issue as new. The pin guard is the
last line of defence for the production chat.

Before the production send the CLI calls `getChat` on the production channel
and parses the issue number from the currently pinned message. Every
production delivery pins the first post of the issue, so the pinned message
identifies the last delivered issue. When the pinned number is greater than
or equal to the number of the issue about to be delivered, the CLI aborts
with `PIN GUARD ENGAGED` before touching the production chat. Reaching that
failure means the marker chain failed; the run is intentionally left red to
signal the fallback. The guard fails closed: when the channel pins a known
issue but the input carries no readable issue number, the delivery is
treated as unproven and blocked as well. Only an unknown channel state (no
pin, or a pin with no issue number) lets an unverifiable input through.

## Key crates
- `pulldown-cmark` for Markdown parsing.
- `teloxide` and `reqwest` for Telegram interactions.
