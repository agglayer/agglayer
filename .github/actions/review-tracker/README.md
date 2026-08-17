# PR review tracker

This local action creates one same-repository issue for each person requested to review a PR.
It assigns the issue to that reviewer and updates Project 47 as review events arrive.

The maintained production implementation is capped at 675 physical lines.
That count includes `src/`, `action.yml`, `package.json`, and both runtime workflows.
It excludes tests, documentation, `package-lock.json`, and the generated `dist/` bundle.

The action runs natively on Node.js 24.
It uses GitHub's official Actions Toolkit, GitHub's official Octokit wrapper,
the official Anthropic SDK, and GitHub's Projects REST API.
Exact dependency versions and the lockfile are committed.
The generated bundle is also committed so a workflow run never installs packages.

## Security boundary

`pr-review-tracker-signal.yml` receives PR lifecycle and `pull_request_review.submitted` events.
It has no token permissions, secrets, checkout, cache, or artifacts.
Its run name carries only the PR number, event action, and review ID when present.

`pr-review-tracker-process.yml` runs a trusted base/default-branch commit.
Before its privileged job receives either secret, an unprivileged job:

- rejects comments that lack a trusted GitHub author association or command prefix, then checks
  the author's exact effective repository permission through GitHub's API;
- validates the signal workflow, repository, event type, action, target PR, head repository, and
  branch; and
- for a submitted review, fetches the exact review ID through GitHub's API.

The privileged job never checks out or executes PR code.
Fork, Dependabot, and same-repository pull requests use the same powerless relay.
Every relay is bound to its PR through the run's recorded head repository and branch, never
through the `pull_requests` payload, which GitHub can leave empty for fork heads.
Lifecycle processing fetches the current PR after validating the base-repository signal.
Events are serialized per PR, so one noisy PR cannot block tracking for unrelated PRs.

## Lifecycle

- A review request creates one issue for that person, assigns it, adds it to Project 47,
  and sets Status to `Ready`.
- Removing a request closes that issue.
- Re-requesting review reopens the same issue and returns it to `Ready`.
- Every distinct submitted review ID sets the review issue to `In Review`.
- If the source issue's Status was `In Review` or `Blocked` when that review was submitted,
  it moves to `Ready`.
  A later human Status edit is not overwritten.
- Closing or merging the PR closes every open review issue.
- Reopening the PR reopens only issues that the PR closure closed and reapplies each task's
  recorded pre-close Project Status, preserving `Ready` versus `In Review`.

Every lifecycle run converges the recorded review issues from the PR's current open or closed
state and current individual reviewer requests, rather than trusting event-delivery order.
Lifecycle and review runs also enumerate submitted reviews and process every ID not already in
the signed state, so a later run catches a missed review signal.

Team requests and unsolicited reviews without a recorded review task are not tracked.
Multiple reviews by one reviewer are independent because the durable key is the review ID,
not the reviewer ID.

## Source matching

Exactly one eligible `closingIssuesReferences` result is accepted without AI.
A plain issue mention is not an explicit closing relationship.
Closing relationships added later are rechecked on PR edits and synchronizations unless a
maintainer has set the mapping explicitly.
Removing a closing relationship, or making it ambiguous, preserves the previous mapping and does
not resynchronize review issues; a trusted command can correct it.

Otherwise, automatic Claude inference runs on the opening lifecycle signal only when the PR
author has effective repository write permission. A trusted user can explicitly request it on
any PR with an exact
`/review-tracker infer` comment. Claude Sonnet 5 at medium effort then receives every active
Project issue assigned to the PR author, regardless of issue state or Status.
For each candidate it receives the issue response, all issue comments, and configured Project
values, including Notes.
It also receives the PR response, top-level PR issue comments, and the available code diff.
Tracker state comments are excluded.

The model must return one validated candidate node ID or `null` through a structured output.
User-authored data is explicitly marked as untrusted.
There is no candidate-count limit.
Issue conversations are fetched in batches of five to bound concurrent API usage.
Requests larger than 3.5 MB fail visibly and require a maintainer command.

The model is fixed to `claude-sonnet-5`, adaptive thinking, and medium effort.
The workflow uses only the `CLAUDE_API_KEY` repository or organization secret.

## Project fields

The action source stores stable numeric field IDs and a Status node ID at its top.
Renaming fields does not affect the tracker.
The five copied fields are:

- Component
- Mission
- Target
- Risk / Likelihood
- Impact

Estimate is always set to `0`.
Notes is included in matching context but is never copied, cleared, or mutated.
When no source is selected, the five copied fields are cleared and Status remains lifecycle-owned.

GitHub's Projects REST API reads all configured values and applies field changes in one batch.
A small GraphQL read remains solely for the Status value's `updatedAt` timestamp.

## Comment and commands

One bot-authored PR comment shows the selected source, links every review issue, reports
sanitized warnings and errors, and repeats these exact commands:

```text
/review-tracker set OWNER/REPOSITORY#123
/review-tracker none
/review-tracker infer
/review-tracker reconcile
```

Only users with effective repository write permission reach the privileged command processor.
The workflow author-association filter is an optimization, not the authorization boundary;
the processor requires GitHub's effective `write`, `maintain`, or `admin` permission.
`set` and `reconcile` reapply the current source fields to every recorded review issue,
including closed issues, without replaying reviews or changing lifecycle Status.
The hidden versioned state contains routing IDs, issue numbers, reviewer logins, lifecycle and
review-fulfillment flags, pre-close Status IDs, processed review IDs, and the greatest processed
command-comment ID.
It never stores titles, bodies, comments, Notes, copied field values, diffs, prompts,
or model output.
Its integrity is authenticated with `PROJECTS_TOKEN`.
Repository workflows share the `github-actions[bot]` identity, so authorship alone cannot prove
that another workflow did not edit the comment and forge Project mutation targets.
The HMAC lets the tracker reject state that was not written by a holder of the Project secret.

## Deliberate recovery boundary

Normal operations are idempotent, lifecycle runs converge from current GitHub state, and review
IDs are caught up from the PR's submitted-review history.
Commands are applied in increasing comment-ID order, so a delayed older command cannot replace a
newer choice.
GitHub and Project mutations are not transactional, however.
Interruption during task creation can leave an orphan or cause a duplicate on retry, and an issue
mutation followed by a failed state-comment save can lose close provenance.

The tracker does not reconstruct deleted state comments or corrupted markers.
`/review-tracker reconcile` reapplies the current mapping and copied fields; it deliberately does
not change lifecycle-owned Status.
Changing a source after reviews have been processed likewise does not replay those reviews.
There is no dedicated watcher for later source-field changes.
Rotating `PROJECTS_TOKEN` invalidates existing state signatures and requires manual repair.

## Porting

To install the tracker elsewhere:

1. Copy the local action, both runtime workflows, and the test workflow.
2. Replace the Project, field, and option IDs in `src/tracker.mjs`.
   Update the fixed-ID assertions in `test/workflow.test.cjs` at the same time.
3. Update the Project-specific action description and concurrency-group label.
4. Preserve the signal workflow's name, path, and run-name grammar as one coupled interface,
   or update the processor and relay tests together.
5. Add a stable fine-grained PAT as `PROJECTS_TOKEN`, with organization Projects write access
   and Issues read access only for repositories that can contain candidate issues.
   The token also authenticates durable state, so short-lived GitHub App installation tokens
   are not supported without adding a separate stable state-signing secret.
6. Make `CLAUDE_API_KEY` available to the repository.

The repository `GITHUB_TOKEN` manages same-repository issues and comments.
Only the privileged job receives the Project and Claude credentials.
Avoid a classic PAT with broad `repo` scope.

After changing source or dependency versions, run these commands from the action directory:

```text
npm ci --ignore-scripts
npm test
npm run notices
npm run build
```

The test workflow regenerates `THIRD_PARTY_NOTICES.txt`, rebuilds `dist/`,
and fails if either committed artifact is stale.
