# antigravity-powerline

A status line for Google Antigravity CLI that keeps context usage, model quota, prompt-cache health and pull-request state visible while you work.

## Sessions and activity

**Session**:
One Antigravity CLI conversation, identified by its conversation id.
_Avoid_: Chat, thread

**Step**:
A single model call within a Session; the smallest unit of activity the status line can tell apart.
_Avoid_: Call, turn, request, generation

**Refresh**:
One run of the status line, triggered by Antigravity CLI on an event: a Step finishing, the agent state changing, the user starting to type into an empty prompt, or exit. There is no refresh timer, so nothing refreshes while a Session sits idle. Several Refreshes may report the same Step.
_Avoid_: Tick, update, render

**Idle Time**:
How long the Session has gone without a Step. Typing, reading and long tool runs all count as idle. With no refresh timer, it is measured on the first Refresh after the break, not counted up while the Session sits idle.
_Avoid_: Inactivity, break time

## Prompt cache

**Prompt Cache**:
The model provider's cache of recently sent context; tokens read from it cost less than tokens sent fresh.
_Avoid_: Cache (unqualified, when other caches are in play)

**Cache Hit Rate**:
The share of the latest Step's prompt tokens read from the Prompt Cache.
_Avoid_: Cache ratio, cache efficiency

**Cache Trend**:
Whether the latest Step hit the Prompt Cache clearly more or clearly less than the Session's earlier Steps did on average.

**Cache Miss**:
A Step that sent a large number of input tokens not read from the Prompt Cache, such as the first Step after the cache expired.
_Avoid_: Cold start, cache flush

**Cold Cache**:
A Prompt Cache that has most likely expired, so the next Step re-reads the whole context at full price. Inferred from Idle Time, never observed directly.
_Avoid_: Expired cache

**Long-Break Reminder**:
The `cache_expiry` Segment: shown once Idle Time passes common Prompt Cache lifetimes, saying the cache is likely cold.
_Avoid_: TTL warning

## Presentation

**Segment**:
One item of the status line, such as Context Window or Pull Request, that the user can enable, order and configure.
_Avoid_: Widget, field, module

## Pull request

**Review State**:
Where an open pull request stands: draft, or the verdict its reviews add up to (approved or changes requested). A merged or closed pull request, or one with no deciding review, has none.
_Avoid_: Review status, PR status

**Unresolved Thread**:
A review conversation on a pull request not yet marked resolved, including ones on code that has since changed.
_Avoid_: Open comment, pending review
