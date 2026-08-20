# Grant cross-repository Issues access to the PR review tracker

This is a standalone administrator handoff for the `agglayer/agglayer` PR review tracker.
It supersedes the earlier **Issues: Read-only** and selected-repository guidance for `PROJECTS_TOKEN`.

The existing fine-grained token must be edited in place.
Do not replace or regenerate its value unless repository maintainers have first agreed on a state recovery plan.

## Why this change is required

The tracker creates review-task issues in `agglayer/agglayer`.
After the tracker update is deployed,
each managed review task becomes a sub-issue of the source issue implemented by its pull request.
Source issues can belong to any repository owned by `agglayer`,
so the parent and child can be in different repositories.

Adding or removing a sub-issue requires **Issues: Read and write** on the parent repository.
GitHub documents the permission in its [sub-issues REST API](https://docs.github.com/en/rest/issues/sub-issues?apiVersion=2026-03-10). The API permits a cross-repository relationship only
when the parent and child have the same repository owner.
For this tracker, both sides must therefore be owned by `agglayer`.

The workflow's built-in `GITHUB_TOKEN` is restricted to `agglayer/agglayer`.
GitHub documents that changes to resources outside the workflow repository require a personal access token
or GitHub App.
See [About authentication to GitHub](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/about-authentication-to-github#authenticating-with-the-api).

The same PAT lists Project 47 items, adds generated review tasks to the Project,
and updates their copied fields and Status.
**Projects: Read and write** authorizes those API operations, while the token owner's **Write**
or **Admin** Project role determines which Projects that identity can actually modify.

## Required end state

The credential stored as the `agglayer/agglayer` Actions secret `PROJECTS_TOKEN` must meet every condition below:

- It is the **existing** fine-grained personal access token, edited without changing its value.
- Its **Resource owner** is `agglayer`.
- Its **Repository access** is **All repositories**.
- Its organization permission **Projects** is **Read and write**.
- Its repository permission **Issues** is **Read and write**.
- Its owner has **Write** or **Admin** access to organization Project 47,
  **Agglayer Master Board**.
- It is active, unexpired, and approved by the `agglayer` organization if approval is required.
- Its expiration is at least 30 days after the planned deployment and validation date.

Leave unrelated organization and repository permissions unset.
In particular, this change does not require Contents,
Pull requests, Administration, Actions, or Secrets permission on other repositories.

## Blast radius and safeguards

**All repositories** plus **Issues: Read and write** is a broad permission.
If the token were compromised, it could modify issues in every current and future repository owned by `agglayer`.
This scope is required because Project 47 can contain a source issue from any `agglayer` repository,
including a repository added after the token is configured.

**Projects: Read and write** can also expose every organization Project that the token owner is allowed to modify,
not only Project 47.
The tracker code is fixed to Project 47, but a compromised token would not inherit that application restriction.

The deployed tracker limits use of that repository write permission as follows:

- The PAT's Issues write authority is used only for sub-issue parent operations.
  It is not used to edit, comment on, close, or assign ordinary issues in source repositories.
- Immediately before each add or remove operation, the tracker re-fetches the review task and
  verifies a live, signed marker bound to its repository, pull request, reviewer, database ID,
  node ID, and issue number.
- Confirmed and in-flight parent provenance is stored in signed tracker state.
  The tracker only changes a relationship matching that authenticated provenance.
  If another actor establishes the exact intended relationship while a tracker request is in
  flight, that relationship can satisfy the signed intent and become tracker-managed.
- A signed interruption fence is saved before parent synchronization can mutate anything.
  After an interrupted synchronization, the next run preserves any observed parent and relinquishes
  its ownership claim instead of retrying a destructive write from stale provenance.
- The tracker never directly replaces a parent and never changes a relationship that matches
  neither confirmed nor in-flight signed provenance.
  Such an unexpected parent produces a visible error instead of being overwritten.
- Parent and child repository owners must both be `agglayer`.
- Creating and updating review-task issues and tracker comments in `agglayer/agglayer` continues to
  use the repository-scoped `GITHUB_TOKEN`, not `PROJECTS_TOKEN`.

The incoming event is authorized before the trusted processing job receives the privileged PAT or Claude credential.
Pull-request code never receives either secret.

## Edit the existing token

Only the token owner can edit the requested permissions.
An organization owner can inspect, approve, or revoke the token, but cannot edit it on the owner's behalf.

The organization owner should first identify the credential and its owner:

1. Open the `agglayer` organization.
2. Select **Settings**.
3. Under **Personal access tokens**, select **Active tokens**.
4. Find the fine-grained token used by the `agglayer/agglayer` `PROJECTS_TOKEN` secret.
5. Record its owner, expiration, approval state, exact selected-repository list, and requested
   permissions before changing it.

Repository secret values are intentionally opaque.
Use the organization's credential inventory or existing ownership record to correlate this token with `PROJECTS_TOKEN`.
If the token cannot be identified uniquely, stop and ask the `agglayer/agglayer` maintainers
and the current secret administrators to establish its owner; do not replace the secret to guess.

GitHub documents this view under [Reviewing personal access tokens](https://docs.github.com/en/organizations/managing-programmatic-access-to-your-organization/reviewing-and-revoking-personal-access-tokens-in-your-organization).

The token owner must:

1. Open personal **Settings**.
2. Select **Developer settings**.
3. Select **Personal access tokens**, then **Fine-grained tokens**.
4. Open the existing token used by the `PROJECTS_TOKEN` Actions secret.
5. Confirm that **Resource owner** is `agglayer`.
6. Set **Repository access** to **All repositories**.
7. Under **Organization permissions**, set **Projects** to **Read and write**.
8. Under **Repository permissions**, set **Issues** to **Read and write**.
9. Leave unrelated permissions unset.
10. Save the changes without regenerating or replacing the token value.

GitHub documents token changes under [Managing your personal access tokens](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens#changing-a-fine-grained-personal-access-token).

The required REST permissions are also listed in GitHub's [fine-grained token permission table](https://docs.github.com/en/rest/authentication/permissions-required-for-fine-grained-personal-access-tokens?apiVersion=2026-03-10#repository-permissions-for-issues).

## Confirm Project access

The organization or Project owner must also confirm the token owner's role:

1. Open **Agglayer Master Board**, organization Project 47.
2. Open the Project menu and select **Settings**.
3. Select **Manage access**.
4. Confirm that the token owner has **Write** or **Admin** access directly, through a team, or
   through the Project base role.

If that access is absent, a Project or organization owner must grant **Write** access.
Use the same **Manage access** page and complete the change before deployment.
Do not compensate by granting unrelated token permissions.

See GitHub's [Project access instructions](https://docs.github.com/en/issues/planning-and-tracking-with-projects/managing-your-project/managing-access-to-your-projects).

## Approve the changed token if required

Changing repository access or permissions can return a fine-grained token to a pending state.
If `agglayer` requires approval, an organization owner must:

1. Open the `agglayer` organization.
2. Select **Settings**.
3. Under **Personal access tokens**, select **Pending requests**.
4. Open the request from the tracker token owner.
5. Confirm all requested values:

   - resource owner `agglayer`;
   - repository access **All repositories**;
   - organization **Projects: Read and write**; and
   - repository **Issues: Read and write**.
6. Confirm that unrelated permissions are not requested.
7. Select **Approve**.

GitHub documents this process under [Managing personal access token requests](https://docs.github.com/en/organizations/managing-programmatic-access-to-your-organization/managing-requests-for-personal-access-tokens-in-your-organization).

## Read-only verification

The token owner can confirm effective read access from a private terminal before deployment.
These commands do not mutate issues or Project 47 and do not print the token value:

```bash
(
  set -euo pipefail
  read -rsp 'Tracker token: ' REVIEW_TRACKER_PROJECT_TOKEN
  echo

  GH_TOKEN="$REVIEW_TRACKER_PROJECT_TOKEN" gh api \
    -H 'X-GitHub-Api-Version: 2026-03-10' \
    'orgs/agglayer/projectsV2/47/items?per_page=1' \
    --jq '.[0] | {id, project_url}'

  GH_TOKEN="$REVIEW_TRACKER_PROJECT_TOKEN" gh api \
    -H 'X-GitHub-Api-Version: 2026-03-10' \
    'repos/agglayer/PRIVATE-SOURCE-REPOSITORY/issues/OPEN-ISSUE' \
    --jq '{id, number, repository_url}'

  unset REVIEW_TRACKER_PROJECT_TOKEN
)
```

The first command should return a Project item whose `project_url` ends in `/orgs/agglayer/projectsV2/47`.
Replace the placeholders in the second command with an existing issue from a private source repository.
It should return that issue's database ID, number, and repository URL.

Read requests cannot prove that write access is effective.
Before deployment, an administrator must also inspect the token in GitHub.
The UI must show all three exact settings:

- **All repositories**;
- **Projects: Read and write**; and
- **Issues: Read and write**.

Never paste the token into an issue, pull request, chat, workflow log, command argument, or shell history.

## Deployment gate and rollback ownership

The repository maintainers own deployment and rollback.
The deployment artifact is [PR 1756](https://github.com/agglayer/agglayer/pull/1756), including its follow-up commit.
That commit must be titled `feat(review-tracker): link tasks to source issues`.
That commit is identifiable by the addition of `src/hierarchy.mjs`
and the interruption-fence regressions in `test/tracker.test.cjs`.
Do not run the post-deploy validation below
until maintainers confirm that this exact change is present on the default branch.

If deployment produces unexpected parent mutations,
repository maintainers should disable the **PR Review Tracker** processing workflow
while they revert the hierarchy change.
After the rollback is deployed,
the token owner may reduce **Issues** back to **Read-only** without changing the token value
and restore the exact selected-repository list recorded before this permission change.
Escalate through the [`@agglayer/agglayer-developers`](https://github.com/orgs/agglayer/teams/agglayer-developers) team.
That team owns repository escalation and rollback coordination.

Before repairing relationships, repository maintainers must inventory every tracker run since the deployment,
record the affected PR, review-task issue, and observed source-parent issue IDs,
and agree on the expected relationship for each item.
The same team must then approve and execute the relationship-recovery plan with normal maintainer credentials;
the broad PAT must not be used for ad hoc repair.

Token expiration, revocation, or replacement requires a signed-state recovery or migration plan.
If rotation is approaching,
the token owner must coordinate that plan with repository maintainers before changing the token value.

## Controlled post-deploy validation

Repository maintainers should perform one controlled cross-repository validation after the tracker change is deployed.
Use a real source issue and pull request whose relationship should remain;
do not create a fake issue in a production repository.

1. Choose an open source issue in an `agglayer` repository other than `agglayer/agglayer`.
   Confirm that it is an active, non-generated item in Project 47.
2. On the corresponding `agglayer/agglayer` pull request, set the source explicitly with a trusted
   maintainer comment:

   ```text
   /review-tracker set agglayer/SOURCE-REPOSITORY#SOURCE-ISSUE
   ```

3. Request one individual reviewer.
   Wait for the **PR Review Tracker** workflow to complete.
4. Confirm that the tracker comment reports success and identifies one review-task issue in
   `agglayer/agglayer`.
5. Using a maintainer's normal `gh` login, not `PROJECTS_TOKEN`, read the generated task's database
   ID:

   ```bash
   gh api \
     -H 'X-GitHub-Api-Version: 2026-03-10' \
     'repos/agglayer/agglayer/issues/TASK-ISSUE' \
     --jq '{id, number, repository_url}'
   ```

6. Assign the returned integer `id` to `TASK_DATABASE_ID`, then confirm that the source lists it
   exactly once:

   ```bash
   TASK_DATABASE_ID=123456789

   gh api \
     -H 'X-GitHub-Api-Version: 2026-03-10' \
     'repos/agglayer/SOURCE-REPOSITORY/issues/SOURCE-ISSUE/sub_issues?per_page=100' \
     --jq "map(select(.id == $TASK_DATABASE_ID)) | \
       {count: length, matches: map({id, number, repository_url})}"
   ```

   The result must report `count: 1`.

7. Confirm the reverse relationship from the generated review task:

   ```bash
   gh api \
     -H 'X-GitHub-Api-Version: 2026-03-10' \
     'repos/agglayer/agglayer/issues/TASK-ISSUE/parent' \
     --jq '{id, number, repository_url}'
   ```

8. Comment `/review-tracker reconcile` on the pull request and wait for the workflow to complete.
9. Repeat steps 6 and 7.
   The same task must still appear exactly once, with the same parent, and the tracker comment must
   contain no permission or parent-conflict error.

Stop validation and notify repository maintainers
if the workflow returns `HTTP 403`, `HTTP 404`, or an unexpected-parent error.
Do not broaden permissions further and do not manually replace the task's parent.

## If the existing token cannot be edited

Stop and coordinate with repository maintainers.
Replacing `PROJECTS_TOKEN` changes the HMAC key used for signed tracker state
and task markers, which invalidates existing signatures and can require manual recovery.
A replacement token must not be installed until maintainers have agreed on that recovery plan.
