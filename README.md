# TWIR Deploy Notify

TWIR Deploy Notify is a small Rust CLI that turns the latest This Week in Rust issue into a series of Telegram posts.

What it does:

1. Finds the newest TWIR Markdown issue file in `this-week-in-rust/content`.
2. Parses the issue into sections.
3. Generates Telegram friendly messages:
   - one Telegram post per section
   - automatically splits sections and overly long lines to fit Telegram limits
   - appends the issue link (derived from issue number and date) to each post
4. Sends the posts to the configured Telegram chat.
5. Pins the first sent message and removes the service notification.
6. Cleans up any previously generated `output_*.md` files before writing new ones.

## Why it exists

- TWIR issues are long, and Telegram has message size limits.
- You want consistent formatting and splitting without manual editing.
- You want a repeatable way to publish the latest issue to one or more chats.

## Requirements

- Rust toolchain (stable).
- A local clone of `rust-lang/this-week-in-rust` with the `content` directory available.
- Telegram bot token and chat id.

## Setup for local use

Clone TWIR into a `twir` subdirectory (sparse checkout of `content`):

```bash
git clone --depth 1 --filter=blob:none --sparse https://github.com/rust-lang/this-week-in-rust twir && cd twir && git sparse-checkout set content && cd ..
```

## Run

```bash
cargo run --bin twir-deploy-notify -- twir/content/<file-name>.md
```

Generate plain text instead of Telegram markup:

```bash
cargo run --bin twir-deploy-notify -- --plain twir/content/<file-name>.md
```

Enable detailed logs:

```bash
RUST_LOG=info cargo run --bin twir-deploy-notify -- twir/content/<file-name>.md
```

## Configuration

Environment variables for sending to Telegram:

- `DEV_BOT_TOKEN` and `DEV_CHAT_ID` dev chat credentials
- `PROD_BOT_TOKEN` and `PROD_CHAT_ID` production chat credentials
  - If `*_CHAT_ID` is numeric, it is automatically prefixed with `-100` when sending requests to Telegram
- `TWIR_SKIP_DEVELOPER_SEND` optional boolean that skips the developer send
- `TWIR_SKIP_PRODUCTION_SEND` optional boolean that skips the production send

If neither credential pair is fully configured, the CLI exits with an error. Partial configuration (only one variable from a pair) is also treated as an error.

## License

See `LICENSE_QQRM_LAPOCHKA`.

## Delivery checkpoints and recovery

Production delivery stores its checkpoint in the separate `delivery-state` branch using GitHub's contents API. The workflow token is scoped to the restore/send steps and consumed by the checkpoint helper; checkout does not persist credentials. The remote file is a versioned JSON envelope containing base64-encoded application state and a `pending` SHA-256 digest. No token, chat ID, or API response is stored.

Before each send, a pending operation is committed. Only an acknowledged send promotes the application state and clears pending in the same API write. Authentication, network, corrupt-state and checkpoint conflicts stop delivery. A missing branch is the only first-run condition; existing legacy state is migrated where available. A missing file in an existing state branch is an error. Caches and run artifacts are no longer the authority after migration.

A timeout, process crash, or failed checkpoint after an acknowledgement leaves pending set. Subsequent scheduled runs stop before sending. This provides safe interruption, not exactly-once delivery: Telegram has no idempotency key for this operation.

To reconcile:

1. Pause the production workflow and inspect the failed run and Telegram channel. Do not blindly retry or remove state.
2. Read the checkpoint file on `delivery-state` and decode its `state` field locally. If the pending message was delivered, add its confirmed entry to the application state. If it was definitely not delivered, retain the previous application state. If the outcome is still unknown, leave pending intact.
3. Encode the reconciled state, set `pending` to null, and update that same file with its current blob SHA to reject concurrent changes. Keep tokens, chat IDs and response bodies out of the commit. Record the evidence in the recovery change.
4. For local runs, update the local state consistently and remove only its `.pending` sidecar after reconciliation. Resume the workflow. Already confirmed entries are skipped.

## Dependency and workflow policy

Cargo and Actions updates share a weekly Dependabot maintenance batch with a seven-day cooldown; security fixes use a dedicated group. Rust toolchain updates remain tracked separately. Native auto-merge requires the exact PR head to pass `ci-gate` and `workflow-security-gate`, with no review requirement. Failed or skipped required jobs cannot pass a gate. The privileged controller reads metadata only; it never checks out PR code or consumes PR artifacts.

Existing release and delivery triggers are preserved. A merge performed using `GITHUB_TOKEN` does not guarantee a follow-up push workflow. Guaranteed automatic post-merge releases need a separate GitHub App token or verified release controller; this change does not claim that coverage.

TWIR checkpoints contain only content/target hashes, part count and acknowledged Telegram message IDs. A changed unfinished issue or target is rejected. The existing channel pin guard remains active; pinning happens after the entire issue is durable. Only a service message whose chat and pinned message ID match is eligible for deletion. For reconciliation, append a verified message ID to the ordered `message_ids` list; never invent one.

Every mutation compares the remote blob SHA with the snapshot recorded at restore (or the prior successful write). A stale runner cannot replace newer progress, even across branches; a SHA conflict stops before another send.
