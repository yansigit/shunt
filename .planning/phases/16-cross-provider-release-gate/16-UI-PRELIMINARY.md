# Preliminary Computer review — 2026-09-09

Root used Computer against an owned static server at `http://127.0.0.1:31988`, serving this worktree's built `site/dist` under the serialized isolated wrapper. This is early documentation evidence, not final plan 16-05 acceptance. Screenshots and accessibility observations are recorded in the task's Computer tool results.

- English desktop: corrected plain admission sentence visible; a single active `OpenCode Go` sidebar entry follows Vercel AI Gateway. Title, body, and table of contents occupy separate readable columns.
- Language picker opened, exposed all four locale targets, and selecting Korean navigated to the Korean provider page.
- Korean, Japanese, and Simplified Chinese desktop pages: locale content, plain active navigation label, zero-support heading, canonical endpoint, environment key, and source-only candidate list rendered. No overlap or clipping of the inspected content. Japanese long headings wrap across lines; no new layout change was introduced.
- Simplified Chinese at 390×844: content reflows; desktop sidebar is replaced by the Open navigation button. Opening the drawer and filtering for `OpenCode Go` shows exactly one matching entry. Selecting it closes the drawer and returns to the same locale provider page.
- Restored the browser viewport and closed the owned tab after evaluation. The owned server was stopped by its exact PID; no production service was stopped.

No provider generation requests occurred. No visual claims are made for other provider pages, uninspected lower-page content, or other mobile locale variants. Final gates must reconcile this early observation with the final source/build state.
