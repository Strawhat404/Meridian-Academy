# Meridian Academy — Clarifying Questions & Answers

## Authentication & Accounts

**Q: Can users self-register, or does an admin create accounts?**
A: Administrators create accounts and assign roles. There is no public self-registration to prevent unauthorized access to the academic system.

**Q: What happens after 30 days of soft-delete hold?**
A: The account and all associated data are permanently deleted. A background job or admin-triggered cleanup handles this. The user cannot log in during the hold period.

**Q: Can a user have multiple roles simultaneously?**
A: No. Each user has exactly one role. Administrators can change a user's role, and the change is logged in the audit trail.

**Q: What does "Export My Data" include?**
A: It includes the user's own profile, submissions, orders, reviews, after-sales cases, and notification history. It does not include other users' data, system configuration, or audit logs of other users' actions.

## Submissions

**Q: What happens if a user tries to submit an 11th version?**
A: The system rejects the submission with a clear error message indicating the maximum of 10 resubmissions has been reached.

**Q: What happens if a user tries to submit after the deadline?**
A: The submission is rejected. The deadline is enforced server-side, not just in the UI.

**Q: Are watermarks applied to the original file or generated on download?**
A: Watermarks are applied at download time. The stored file is the original. The watermark (requester name + timestamp) is overlaid when the file is served.

**Q: Can instructors see all student submissions or only those in their courses?**
A: Instructors can only see submissions associated with courses they are assigned to. Administrators can see all submissions.

## Orders & Fulfillment

**Q: Can a single order span multiple subscription periods?**
A: No. Each order has one subscription period (monthly, quarterly, or annual). Users create separate orders for different periods.

**Q: What triggers the abnormal-order flag?**
A: The system flags orders with quantities significantly above the account's historical average, or accounts that have submitted more than 3 refund requests in a 30-day window. Thresholds are configurable by administrators.

**Q: Can a merged order be split again later?**
A: Yes. Split and merge are reversible operations as long as the order has not been fully fulfilled.

## Reviews & After-Sales

**Q: Can a user post a review before receiving the order?**
A: No. Reviews are linked to fulfilled orders. The review option only appears after at least one fulfillment event is logged for the order.

**Q: What happens if the 14-day follow-up window expires?**
A: The follow-up option is hidden and the endpoint returns 403. The original review remains visible.

**Q: Who can move an after-sales case to "Arbitrated" status?**
A: Only Academic Staff and Administrators can set the Arbitrated status, indicating the case has been escalated for a formal decision.

**Q: Are SLA timers paused when a case is in "Awaiting Evidence" status?**
A: Yes. The SLA clock pauses when the case enters Awaiting Evidence and resumes when evidence is submitted or the status changes.

## Content Governance

**Q: Who manages the sensitive-word dictionary?**
A: Only Administrators can add, edit, or remove entries from the sensitive-word dictionary.

**Q: What is the "replace" policy for sensitive words?**
A: The matched word or phrase is automatically replaced with a configurable placeholder (e.g., "[REDACTED]") and the content can still be saved and submitted for review.

**Q: What is the "block" policy?**
A: The content cannot be saved at all until the sensitive word is removed. The user sees an error indicating which word triggered the block.

**Q: How are SEO fields generated deterministically?**
A: Meta title = first 60 characters of the content title. Meta description = first 160 characters of the summary. Slug = lowercase title with spaces replaced by hyphens and special characters removed.

## Payments

**Q: What does "idempotent posting" mean in practice?**
A: Each transaction submission includes an `idempotency_key`. If the same key is submitted twice, the second request returns the result of the first without creating a duplicate transaction.

**Q: What is an "escrow hold"?**
A: A hold reserves funds against an account balance without finalizing the transaction. The hold can be released (converting to a completed transaction) or cancelled (returning the funds).

**Q: When does the nightly reconciliation run?**
A: It runs at midnight local server time. Administrators can also trigger it manually from the admin console.

## Deployment

**Q: Does the system require internet access?**
A: No. The entire system runs on a local network. MySQL, the Rocket backend, and the Dioxus frontend all run in Docker containers on the same machine or local network.

**Q: Can multiple users use the system simultaneously?**
A: Yes. The Rocket backend handles concurrent requests. MySQL manages concurrent database access. The system is designed for multi-user institutional use.
