# Restore Project access for the PR review tracker

This is an administrator handoff for the `agglayer/agglayer` PR review tracker.
It explains the required GitHub organization and token changes without exposing any secret value.

> [!IMPORTANT]
> The repository-access guidance in this incident handoff has been superseded by
> [Grant cross-repository Issues access to the PR review tracker](review-tracker-org-issues-write.md).
> **Issues: Read-only** and selected-repository access are no longer sufficient for the current
> tracker design.

## Incident summary

The tracker can create repository issues, but its `PROJECTS_TOKEN` currently receives `HTTP 404`
from the organization Project API.
The failing operations are listing, adding, and updating items in organization Project 47,
**Agglayer Master Board**.
The Project number, API route, and API version have been independently verified.
The observed example is
[PR 1746](https://github.com/agglayer/agglayer/pull/1746), whose intended review task is
[issue 1747](https://github.com/agglayer/agglayer/issues/1747).

GitHub requires the fine-grained **Projects** organization permission at read level to list items
and at write level to add or update them.
Selecting **Read and write** grants both requirements.
See GitHub's
[Project item permission reference](https://docs.github.com/en/rest/projects/items?apiVersion=2026-03-10)
and
[fine-grained token permission table](https://docs.github.com/en/rest/authentication/permissions-required-for-fine-grained-personal-access-tokens?apiVersion=2026-03-10#organization-permissions-for-projects).

## Required end state

The credential stored as the repository Actions secret `PROJECTS_TOKEN` must meet all of these
conditions:

- It is a fine-grained personal access token whose resource owner is `agglayer`.
- Its organization permission **Projects** is **Read and write**.
- Its owner has at least **Write** access to organization Project 47.
- It is active, unexpired, and approved by the `agglayer` organization if approval is required.
- Its repository access and Issues permission match the superseding
  [cross-repository access handoff](review-tracker-org-issues-write.md):
  **All repositories** and **Issues: Read and write**.

The workflow still uses its repository `GITHUB_TOKEN`,
not `PROJECTS_TOKEN`, to create and update same-repository review issues and comments.
The broader PAT permission is reserved for sub-issue parent operations in source repositories.

## Preferred repair: edit the current token in place

Do not replace or regenerate the token value if its owner can still edit it.
The tracker uses the token value as an HMAC key for durable state.
Replacing the value invalidates existing PR state and task signatures.

The organization owner should first identify the credential and its owner:

1. Open the `agglayer` organization.
2. Select **Settings**.
3. Under **Personal access tokens**, select **Active tokens**.
4. Find the fine-grained token used by the review tracker.
   Review its owner, expiration, resource access, and permissions.

GitHub documents this view under
[Reviewing personal access tokens](https://docs.github.com/en/organizations/managing-programmatic-access-to-your-organization/reviewing-and-revoking-personal-access-tokens-in-your-organization).
An organization owner can inspect or revoke a fine-grained token, but only its owner can edit its
requested permissions.

The token owner should then:

1. Open personal **Settings**.
2. Select **Developer settings**.
3. Select **Personal access tokens**, then **Fine-grained tokens**.
4. Open the existing tracker token.
5. Confirm that **Resource owner** is `agglayer`.
6. Under **Organization permissions**, set **Projects** to **Read and write**.
7. Under **Repository access**, select **All repositories**.
8. Under **Repository permissions**, grant **Issues: Read and write** and leave unrelated
   permissions unset.
9. Save the token changes without regenerating the token value.

The organization or Project owner must also confirm the token owner's Project role:

1. Open **Agglayer Master Board**.
2. Open the Project menu and select **Settings**.
3. Select **Manage access**.
4. Confirm that the token owner has **Write** or **Admin** access, either directly, through a team,
   or through the Project base role.

See GitHub's
[Project access instructions](https://docs.github.com/en/issues/planning-and-tracking-with-projects/managing-your-project/managing-access-to-your-projects).

## Approve the permission request

Changing a fine-grained token's permissions may place it into a pending state.
If the `agglayer` organization requires approval, an organization owner must:

1. Open the `agglayer` organization.
2. Select **Settings**.
3. Under **Personal access tokens**, select **Pending requests**.
4. Open the request from the tracker token owner.
5. Confirm that it requests **Projects: Read and write**, **All repositories**, and
   **Issues: Read and write**, with unrelated permissions unset.
6. Select **Approve**.

GitHub documents this workflow under
[Managing personal access token requests](https://docs.github.com/en/organizations/managing-programmatic-access-to-your-organization/managing-requests-for-personal-access-tokens-in-your-organization).

## Read-only validation

The token owner can validate effective read access from a private terminal.
This command does not print the token or mutate the Project:

```bash
read -rsp 'Tracker token: ' REVIEW_TRACKER_PROJECT_TOKEN
echo
GH_TOKEN="$REVIEW_TRACKER_PROJECT_TOKEN" gh api \
  -H 'X-GitHub-Api-Version: 2026-03-10' \
  'orgs/agglayer/projectsV2/47/items?per_page=1' \
  --jq '.[0] | {id, project_url}'
unset REVIEW_TRACKER_PROJECT_TOKEN
```

Success is an HTTP 200 response containing a Project item ID and a `project_url` ending in
`/orgs/agglayer/projectsV2/47`.
Do not paste the token into an issue, pull request, chat, workflow log, or shell history.

GitHub has no non-mutating write probe for this endpoint.
The organization owner should therefore verify **Projects: Read and write** in the token UI,
then ask a repository maintainer to perform the tracker recovery below.

## Maintainer recovery after access is restored

Deploy the review-tracker recovery change before repairing affected PRs.
For PR 1746, the existing task is issue 1747 and the detected source is unresolved.
A maintainer should first choose the correct source mapping with one of these PR comments:

```text
/review-tracker set OWNER/REPOSITORY#123
/review-tracker none
```

After that command succeeds, run:

```text
/review-tracker reconcile
```

Reconciliation should reuse issue 1747, attach or locate its existing Project item, synchronize
its fields, and replay the already-submitted review.
Because PR 1746 and issue 1747 are both already closed,
the replay restores the review's recorded effects,
including the task's fulfillment and Project Status,
without reopening the closed issue.
The tracker comment should report success, list issue 1747, and no longer show
`Project sync pending`.

## If the current token cannot be edited

Stop and coordinate with the repository maintainers before replacing `PROJECTS_TOKEN`.
A new fine-grained token must use the permissions and approval steps above, but replacing the
secret value invalidates all state signed with the old value.
The repository secret can be updated under **Settings** → **Secrets and variables** → **Actions** →
**Repository secrets** → `PROJECTS_TOKEN` only after a recovery plan is agreed.

GitHub documents repository secret administration under
[Using secrets in GitHub Actions](https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets).
