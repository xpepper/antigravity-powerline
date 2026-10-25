# antigravity-powerline

A status line for Google Antigravity CLI that keeps context usage, model quota, prompt-cache health and pull-request state visible while you work.

## Sessions and activity

**Session**:
One Antigravity CLI conversation, identified by its session id.
_Avoid_: Chat, thread

**Refresh**:
One run of the status line, triggered by Antigravity CLI on an event or on its refresh timer.
_Avoid_: Tick, update, render

## Presentation

**Segment**:
One item of the status line, such as Context Window or Pull Request, that the user can enable, order and configure.
_Avoid_: Widget, field, module

## Pull request

**Review State**:
The verdict a pull request's reviews add up to: draft, approved or changes requested. A pull request with no deciding review has none.
_Avoid_: Review status, PR status

**Unresolved Thread**:
A review conversation on a pull request not yet marked resolved, including ones on code that has since changed.
_Avoid_: Open comment, pending review
