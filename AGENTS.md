# AGENTS.md — ZeroClaw Solana POS Agent

## Mandatory Pre-Commit Checks

**Commits are FORBIDDEN unless ALL checks pass with ZERO errors and ZERO warnings.**

On every commit, checks run on the **files touched by the diff** (see `git diff --name-only`), NOT the whole tree. The whole tree is not yet clean — the legacy backlog is tracked separately in §Tree-Cleanup Backlog below.

### Lint & Format (touched files only)

```bash
# Must pass with EXIT=0 and NO output on touched files
# E203 ignored: black formats slices as "a[i : i+n]", which flake8 flags but is correct output.
flake8 <touched_py_files> --max-line-length=160 --ignore=E501,W503,E402,E203

# Must pass with EXIT=0 and NO output
black --check <touched_py_files> --line-length=160
```

### Type Check (touched files only)

```bash
# Must pass with EXIT=0 and NO errors (--no-error-summary required)
mypy <touched_py_files> --ignore-missing-imports
```

### Security Audit (touched files only)

```bash
# Must show ZERO Medium and ZERO High issues
bandit <touched_py_files> -ll
```

### Test Suite

```bash
# Must pass with 100% rate (all tests green, no failures)
# Requires a clean tree for verification of interface changes (full suite always valid).
./scripts/verify_all.sh
```

## Tree-Cleanup Backlog

The following legacy debt exists in `scripts/` and is intentionally **NOT** fixed as part of ordinary commits. It must be worked down in a dedicated cleanup PR (incremental, file-by-file). Newly touched files MUST remain clean (zero errors / zero warnings), keeping the backlog non-growing.

| Debt | Scope | Count |
|------|-------|-------|
| File size >400 lines | `tests/test_edge_math_and_blinks.py` (736), `tests/test_squads_multisig.py` (521) | **2 files** |
| legacy `parse_mode="Markdown"` | 7 messages in `bot_ui_handlers.py` (migration to MarkdownV2 tracked) | 7 messages |

Migration note: `i18n.py` was split into `i18n_strings.py` + `i18n_strings_ext.py` (pure data) so both modules stay under 400 lines. All flake8 violations (684→0) and black drift (34→0) have been resolved in this session.

## File Size Rules

- **Maximum 400 lines per file.** No god classes. No monoliths.
- Files exceeding 400 lines MUST be split into smaller modules.
- Each module MUST have a single responsibility.
- Each test module MUST be self-contained with its own `run_suite()`.

## Code Standards

- Zero external dependencies beyond Python stdlib
- All DB connections MUST use `try/finally: conn.close()`
- No `return` inside `try` block without explicit `conn.close()` first
- All SQL MUST use parameterized queries (no f-strings in SQL)
- All Telegram user input MUST go through `sanitize_external_input()`
- All Telegram output text MUST be escaped via `escape_telegram_markdown_v2()` or `t(escape_markdown=True)`
- Manager-only actions MUST check `MANAGER_TELEGRAM_ID` from `bot_ui_utils`

## Project Structure

```
scripts/
  pos_core/          # Domain logic (NO network I/O, NO side effects beyond DB)
    bot_ui_utils.py  # Keyboards, button matching, order parsing, payload builders
    bot_ui_handlers.py # Callback and text message handlers
    db.py            # SQLite WAL DAO layer
    i18n.py          # 13-language internationalization (functions; data in i18n_strings*.py)
    i18n_strings.py  # Pure data: LANG_META + translations (part 1)
    i18n_strings_ext.py  # Pure data: translations (part 2)
    solana_pay.py    # Solana Pay URL generation, refund initiation
    price_feed.py    # Multi-tier fiat rate fallback
    formatters.py    # Pubkey formatting, QR image URLs, Telegram payloads
    nonce_pool.py    # Durable nonce account pool
    verification.py  # Solana transaction verification
    pix_brl.py       # Brazil PIX QR code
    router.py        # HTTP micro-router
    constants.py     # Domain constants
  tests/             # Modular test suite (each module has run_suite())
  telegram_bot_listener.py  # Long-polling adapter (thin I/O layer)
  sanitizer.py       # Input sanitizer, SSRF guard, secret redactor
```

## Test Conventions

- Tests are numbered sequentially across modules
- Each test module has a `run_suite() -> int` returning count of passed tests
- `test_boundary_cases.py` is the master entrypoint
- Test DB uses `data/test_boundary.db`
