# 2026-10-24/25 — Fly prod verification, and the coding-job hang

## Deployment state
- App `mbilinyitech-tz`, machine `8e25d6b7d99018`, region jnb, single
  shared-cpu-2x:2048MB machine with volume `vol_40o3185pyo8n13m4` at /data.
- DNS is settled: authoritative NS are ns1/ns2.rodlinehost.com and the apex
  resolves to 66.241.125.44 (AAAA 2a09:8280:1::1cb:226c0). All GET/HEAD reach
  Fly. Older 404s were a stale LiteSpeed cache on the cPanel host
  (91.204.209.30), not a routing fault.
- Deploy command that works from the sandbox:
  `flyctl deploy --remote-only --ha=false -a mbilinyitech-tz`
  with `FLY_API_TOKEN` from /workspace/.secrets/fly.token. Remote builder, so
  no local Docker needed. A deploy restarts the machine; expect 30-90s of
  connection resets right after "Machine is now in a good state".

## The real coding-tool bug (fixed, 9c3ee08)
The panel could create a real OpenHands conversation but a job that hit an
error stayed `running` forever with an empty result. Cause: the poll read
only MessageEvent(agent) and ActionEvent(finish). A conversation that dies
produces neither — it emits ConversationErrorEvent. Now a non-retryable
ConversationErrorEvent sets status `failed` and the detail becomes the
result. Extracted as `read_events()` so it is unit-tested (4 tests).

## Why the conversations were dying at all
Not our code. `GET https://app.all-hands.dev/api/v1/users/me` for the
account's API key shows the org is on `openhands/deepseek-v4.1-flash` via
`https://llm-proxy.app.all-hands.dev/`, and `GET /api/billing/credits`
returns `{"credits":"0.00"}`. With no credits the managed proxy key is
rejected: the conversation emits `LLMAuthenticationError` /
`OpenAIError 401 Incorrect API key provided: sk-_qdwV...`. Earlier jobs
succeeded only because credits had not yet run out.
=> OpenHands *hosted* coding works; it needs credits on the account (or the
   CEO's own provider key) to actually run an agent.

## LLM / fine-tuning on our own site
Independent of the above. `api/llm.php` (status/save/clear_key/test) stores
a provider key in settings; `ready=false, keySet=false` until the CEO sets
one in Admin -> Settings. Verified save/clear round-trip on prod. With no
key, `engine.php action=run_job` for an AI tool records the run as `blocked`
with a clear message ("No AI provider is configured...") rather than hanging.
So the AI/fine-tuning family is wired and fails honestly; it needs a key.

## Traps found
- Fly machine IPv6 is not routable: `curl -6` fails, `-4` works. Only a
  problem if something forces v6. `app.all-hands.dev` resolves A-only here.
- The site is behind Cloudflare-ish edge; occasional
  `Connection reset by peer` during/after a deploy is normal. Use a retrying
  client (see /workspace/.secrets/prod_client.py) rather than treating one
  reset as an outage.
- Admin login is a two-step: auth.php `login` then `verify-login-pin`
  (the CEO PIN), each needing a fresh CSRF token from index.php.
