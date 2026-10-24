# ADR-005: Unofficial WhatsApp API with Email Fallback

## Status

Accepted

## Context

Equalizer-Stake sends notifications to guardians about invoices, payment reminders, session updates, and attendance alerts. Indonesia has ~87% WhatsApp penetration — it's the primary communication channel for parents. The system needs reliable notification delivery with a fallback for when WhatsApp fails.

## Decision

Use **unofficial WhatsApp gateway** (e.g., WABlas, Fonnte, WA-Cloud API) as primary notification channel, with **SMTP email as automatic fallback**.

## Rationale

### Why unofficial WhatsApp

| Factor | Official WhatsApp Business API | Unofficial Gateway | Email (SMTP) |
|--------|-------------------------------|-------------------|--------------|
| Cost | $0.005-0.08 per message (Meta pricing) | ~$5-15/month flat | ~free (SMTP relay) |
| Approval | Meta business verification (weeks) | Minutes (self-hosted or SaaS) | Immediate |
| Template pre-approval | Required for each message type | Not required | N/A |
| Indonesia adoption | Same reach | Same reach | ~40% open rate |
| Rate limits | 1000 messages/24h (tier 1) | Varies by provider | Typically unlimited |
| Reliability | SLA-backed | Varies, circuit breaker handles failures | Reliable |
| Setup complexity | High (Meta verification, webhook) | Low (REST API, API key) | Low |

### Key factors:

1. **Cost at scale**: Music school with 50-200 guardians, ~4 notifications/guardian/month = 200-800 messages/month. Official API at $0.05/msg = $10-40/month. Unofficial at $10/month flat — cheaper and predictable.

2. **Meta verification friction**: Small music school may not have business registration documents Meta requires. Unofficial gateways accept any WhatsApp number.

3. **Template restrictions**: Official API requires pre-approved templates for every message type. Invoice notifications, payment reminders, and attendance alerts would each need separate template approval. Unofficial sends free-form text.

4. **Indonesia-specific**: Unofficial gateways like Fonnte and WABlas are built for Indonesian market, have local support, and understand local use patterns.

### Why email fallback

1. **WhatsApp delivery unreliable**: Unofficial APIs can be blocked by WhatsApp, experience downtime, or hit rate limits. Email is the reliable backup.

2. **Financial notifications**: Invoice and payment records are legally relevant. Email provides a paper trail with timestamps that guardians can reference.

3. **Not all guardians use WhatsApp**: ~95% do, but email covers the remaining 5% and provides redundancy.

### Implementation

```python
# Notification delivery flow
async def deliver_notification(notification: Notification):
 # 1. Try WhatsApp (with circuit breaker + retry)
 if whatsapp_circuit.allow_request():
 try:
 await whatsapp.send(phone, message) # 3 retries, exponential backoff
 whatsapp_circuit.record_success()
 notification.channel = "whatsapp"
 notification.status = "sent"
 return
 except Exception:
 whatsapp_circuit.record_failure()

 # 2. Fallback to email
 try:
 await email.send(user.email, subject, html_template)
 notification.channel = "email"
 notification.status = "sent"
 except Exception:
 notification.channel = None
 notification.status = "failed"
```

### Circuit breaker state machine

```
CLOSED (normal) ──[5 failures]──► OPEN (block requests, 60s)
 │
 [60s elapsed]
 ▼
 HALF_OPEN (allow 1 probe)
 │
 ┌───────┴───────┐
 [success] [failure]
 │ │
 CLOSED OPEN
```

## Consequences

- **No delivery SLA**: Unofficial gateway has no guaranteed uptime. Circuit breaker + email fallback mitigates.
- **WhatsApp may block**: Unofficial APIs risk being blocked by WhatsApp. Mitigation: use reputable provider, monitor delivery rates, have email fallback always ready.
- **No read receipts**: Unofficial API may not provide delivery/read confirmation. Track via `Notification.status` and `sent_at`.
- **Message format**: Plain text only (no rich templates). Email fallback can send HTML with invoice details, payment links.
- **Phone number required**: Guardian must have WhatsApp number on file. Missing number → skip to email immediately.

## Alternatives Considered

- **Official WhatsApp Business API**: Rejected — Meta verification friction, per-message cost, template pre-approval overhead for a small school
- **Telegram Bot API**: Free, reliable, but ~15% adoption in Indonesia vs 87% WhatsApp
- **SMS (Twilio)**: Universal but expensive (~$0.05/msg), no rich content, declining usage
- **Push notifications only**: PWA push limited on iOS, requires PWA installation, not reliable as primary
- **WhatsApp only (no fallback)**: Rejected — financial notifications (invoices, payments) require reliable delivery. Email ensures no notification is lost.
