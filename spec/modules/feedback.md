# Module: feedback (optional; requires enrollment)

Purpose: instructors send progress notes to guardians, and guardians rate instructors.
Policy: `FeedbackPolicy` (04 §6.11). Tables: `progress_notes`, `instructor_ratings` (06 §4.10).
Legacy reference: `v0-capstone:srs/99-addendum-business-rules.md` §13–14, `backend/app/routers/feedback.py`, `ratings.py`.

## 1. Rules

- **FBK-01. Progress notes.**
  - An instructor writes a note for a student they teach, meaning they had a session together in the last 90 days.
  - Each category in the policy gets a rating from 1 to `scale_max`. Categories with `scale_max=0` are comment-only. The rest of the comment is optional, up to 2000 characters.
  - The note can be linked to a session.
  - Publishing:
    - `publish=Immediate` → `published`, notifying the guardian.
    - `AfterAdminReview` → `pending_review`. An admin publishes or withdraws it.
  - Instructors can withdraw their own note within 24 h of publishing. Withdrawn notes stay in audit.
- **FBK-02. Visibility.** A published note is visible to the student's guardians (all linked guardians), the student (if they have an account), staff, and the authoring instructor.
- **FBK-03. Ratings.**
  - A guardian or adult student rates an instructor who taught one of their students **at least `min_sessions_before` completed sessions**.
  - Limit: one rating per rater per instructor per student per month (DB unique). 1–5 stars, optional comment.
  - With `anonymous_to_instructor`, instructors see averages and comments but never rater identities. Admins see everything.
- **FBK-04. Moderation.** Admins can hide a rating comment (`hidden_at`, reason, audited). The stars still count. *(Add the columns `hidden_at`, `hidden_reason` to `instructor_ratings`.)*

## 2. API

| Method | Path | Perm |
|---|---|---|
| GET/POST | `/students/{id}/notes`; POST `/notes/{id}/publish` \| `/withdraw` | instructor (own students) / admin / guardian read |
| GET | `/me/feedback` (guardian: notes for their students; instructor: own notes + rating summary) | any |
| POST | `/instructors/{id}/ratings` | guardian / adult student |
| GET | `/instructors/{id}/ratings?cursor` | staff (identities), instructor (anonymised) |
| POST | `/ratings/{id}/hide` | admin |

## 3. Acceptance criteria

1. **Eligibility to rate.** A guardian with no completed sessions of their child with instructor X gets 403 `not_eligible` when rating X.
2. **Monthly limit.** A second rating in the same month → 409.
3. **Anonymity.** An instructor's GET of ratings has `rater: null` when anonymous.
4. **Review flow.** `AfterAdminReview` → the note isn't visible to the guardian until published.
