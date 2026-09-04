"use strict";

const assert = require("node:assert/strict");
const test = require("node:test");
let Hierarchy;
let AttachPreflightError;
let ParentReadError;
let sameIssue;
test.before(async () => ({ AttachPreflightError, Hierarchy, ParentReadError, sameIssue } = await import("../src/hierarchy.mjs")));

const config = { repository: "agglayer/agglayer", projectOwner: "agglayer" };
const child = { id: 1747001, issueId: "I_child", repository: config.repository, number: 1747 };
const parent = { issueId: "I_parent", repository: "AggLayer/roadmap", number: 42 };
const headers = { accept: "application/vnd.github+json", "x-github-api-version": "2026-03-10" };

test("parent uses the raw route and normalizes the returned issue", async () => {
  const client = fakeClient(parentIssue());
  const hierarchy = new Hierarchy(client, config);

  assert.deepEqual(await hierarchy.parent(child), {
    id: 42001, issueId: parent.issueId, repository: parent.repository, number: parent.number,
  });
  assert.deepEqual(client.requests, [[
    "GET /repos/{owner}/{repo}/issues/{issue_number}/parent",
    { owner: "agglayer", repo: "agglayer", issue_number: 1747, headers },
  ]]);
});

test("attach and detach use the raw sub-issue routes", async () => {
  const client = fakeClient(parentIssue(), {}, parentIssue(), {}), hierarchy = new Hierarchy(client, config);
  await hierarchy.attach(parent, child, false, async () => child);
  await hierarchy.detach(parent, child, async () => child);

  assert.deepEqual(client.requests, [
    ["GET /repos/{owner}/{repo}/issues/{issue_number}", {
      owner: "AggLayer", repo: "roadmap", issue_number: 42, headers,
    }],
    ["POST /repos/{owner}/{repo}/issues/{issue_number}/sub_issues", {
      owner: "AggLayer", repo: "roadmap", issue_number: 42, headers,
      sub_issue_id: child.id, replace_parent: false,
    }],
    ["GET /repos/{owner}/{repo}/issues/{issue_number}", {
      owner: "AggLayer", repo: "roadmap", issue_number: 42, headers,
    }],
    ["DELETE /repos/{owner}/{repo}/issues/{issue_number}/sub_issue", {
      owner: "AggLayer", repo: "roadmap", issue_number: 42, headers, sub_issue_id: child.id,
    }],
  ]);
});

test("every operation rejects a child outside the exact configured repository before requesting", async () => {
  for (const [method, args] of [
    ["parent", [{ ...child, repository: "AggLayer/agglayer" }]],
    ["attach", [parent, { ...child, repository: "AggLayer/agglayer" }, false, async () => child]],
    ["detach", [parent, { ...child, repository: "AggLayer/agglayer" }, async () => child]],
  ]) {
    const client = fakeClient(), hierarchy = new Hierarchy(client, config);
    await assert.rejects(hierarchy[method](...args), /outside the configured repository/);
    assert.deepEqual(client.requests, []);
  }
});

test("every operation rejects a configured repository outside the project owner before requesting", async () => {
  const externalConfig = { repository: "outside/repo", projectOwner: "agglayer" };
  const externalChild = { ...child, repository: externalConfig.repository };
  for (const [method, args] of [
    ["parent", [externalChild]], ["attach", [parent, externalChild, false, async () => child]],
    ["detach", [parent, externalChild, async () => child]],
  ]) {
    const client = fakeClient(), hierarchy = new Hierarchy(client, externalConfig);
    await assert.rejects(hierarchy[method](...args), /configured project owner/);
    assert.deepEqual(client.requests, []);
  }
});

test("mutations reject parents outside the project owner before requesting", async () => {
  const external = { ...parent, repository: "other/roadmap" };
  for (const [method, args] of [["attach", [external, child, false, async () => child]],
    ["detach", [external, child, async () => child]]]) {
    const client = fakeClient(), hierarchy = new Hierarchy(client, config);
    await assert.rejects(hierarchy[method](...args), /outside the configured project owner/);
    assert.deepEqual(client.requests, []);
  }
  const client = fakeClient(), hierarchy = new Hierarchy(client, config);
  await assert.rejects(hierarchy.parent(child, [external]), /outside the configured project owner/);
  assert.deepEqual(client.requests, []);
});

test("invalid IDs, numbers, repositories, and self-parenting are rejected before requesting", async () => {
  const cases = [
    ["parent", [{ ...child, id: 0 }], /child issue ID/],
    ["parent", [{ ...child, number: Number.MAX_SAFE_INTEGER + 1 }], /child issue number/],
    ["parent", [{ ...child, repository: "agglayer/agglayer/extra" }], /child repository/],
    ["attach", [{ ...parent, number: -1 }, child, false, async () => child], /parent issue number/],
    ["attach", [{ ...parent, repository: "agglayer/bad repo" }, child, false, async () => child], /parent repository/],
    ["attach", [parent, child, true, async () => child], /directly is forbidden/],
    ["attach", [{ issueId: child.issueId, repository: child.repository, number: child.number }, child, false,
      async () => child], /own parent/],
    ["attach", [{ issueId: "I_inconsistent", repository: child.repository, number: child.number }, child, false,
      async () => child], /own parent/],
  ];
  for (const [method, args, pattern] of cases) {
    const client = fakeClient(), hierarchy = new Hierarchy(client, config);
    await assert.rejects(hierarchy[method](...args), pattern);
    assert.deepEqual(client.requests, []);
  }
});

test("parent confirms the exact child before treating HTTP 404 as no parent", async () => {
  const missing = httpError(404, "no parent"), gone = httpError(410, "gone");
  const client = fakeClient(missing, childIssue());
  assert.equal(await new Hierarchy(client, config).parent(child), null);
  assert.deepEqual(client.requests, [
    ["GET /repos/{owner}/{repo}/issues/{issue_number}/parent", {
      owner: "agglayer", repo: "agglayer", issue_number: 1747, headers,
    }],
    ["GET /repos/{owner}/{repo}/issues/{issue_number}", {
      owner: "agglayer", repo: "agglayer", issue_number: 1747, headers,
    }],
  ]);
  await assert.rejects(new Hierarchy(fakeClient(gone), config).parent(child),
    (error) => error instanceof ParentReadError && error.cause === gone);
});

test("an unconfirmed parent 404 is classified as an ambiguous read", async () => {
  for (const confirmation of [
    childIssue({ id: 999 }), childIssue({ node_id: "I_other" }), childIssue({ number: 999 }),
    childIssue({ repository_url: "https://api.github.com/repos/agglayer/other" }),
  ]) {
    const missing = httpError(404, "ambiguous parent lookup");
    await assert.rejects(new Hierarchy(fakeClient(missing, confirmation), config).parent(child),
      (error) => error instanceof ParentReadError && /child issue does not match/.test(error.message));
  }
  for (const confirmation of [childIssue({ node_id: "" }), httpError(404, "child unavailable"), httpError(403, "forbidden")]) {
    const missing = httpError(404, "ambiguous parent lookup");
    await assert.rejects(new Hierarchy(fakeClient(missing, confirmation), config).parent(child),
      (error) => error instanceof ParentReadError && !/no visible parent/.test(error.message));
  }
});

test("parent confirms every known parent is visible before accepting a 404", async () => {
  const alternate = { issueId: "I_alternate", repository: "agglayer/bridge", number: 84 };
  const missing = httpError(404, "no parent"), client = fakeClient(
    missing, childIssue(), parentIssue(), issueFor(alternate, 84001),
  );

  assert.equal(await new Hierarchy(client, config).parent(child, [parent, alternate]), null);
  assert.deepEqual(client.requests.slice(2), [
    ["GET /repos/{owner}/{repo}/issues/{issue_number}", {
      owner: "AggLayer", repo: "roadmap", issue_number: 42, headers,
    }],
    ["GET /repos/{owner}/{repo}/issues/{issue_number}", {
      owner: "agglayer", repo: "bridge", issue_number: 84, headers,
    }],
  ]);
});

test("an invisible or mismatched known parent is reported as unverifiable", async () => {
  const confirmations = [
    parentIssue({ node_id: "I_other" }),
    parentIssue({ repository_url: "https://api.github.com/repos/outside/other" }),
    httpError(404, "parent unavailable"), httpError(403, "forbidden"),
  ];
  for (const confirmation of confirmations) {
    const missing = httpError(404, "no parent");
    await assert.rejects(new Hierarchy(fakeClient(missing, childIssue(), confirmation), config)
      .parent(child, [parent]), (error) => error instanceof ParentReadError &&
        /recorded parent could not be verified/.test(error.message) && /unmanage/.test(error.message));
  }
  const alternate = { issueId: "I_alternate", repository: "agglayer/bridge", number: 84 };
  const missing = httpError(404, "no parent");
  await assert.rejects(new Hierarchy(fakeClient(missing, childIssue(), parentIssue(), httpError(403, "forbidden")), config)
    .parent(child, [parent, alternate]), (error) => error instanceof ParentReadError && error.status === 403);
});

test("known-parent visibility follows the stable node ID across an agglayer move", async () => {
  const missing = httpError(404, "no parent");
  const moved = parentIssue({ number: 99, repository_url: "https://api.github.com/repos/agglayer/renamed" });
  assert.equal(await new Hierarchy(fakeClient(missing, childIssue(), moved), config)
    .parent(child, [parent]), null);
});

test("parent rejects an invalid known-parent list before requesting", async () => {
  for (const known of [null, {}, [{ ...parent, number: 0 }]]) {
    const client = fakeClient(), hierarchy = new Hierarchy(client, config);
    await assert.rejects(hierarchy.parent(child, known), /known parent list|parent issue number/);
    assert.deepEqual(client.requests, []);
  }
});

test("parent validates the returned issue identifiers", async () => {
  for (const issue of [
    parentIssue({ id: 0 }), parentIssue({ node_id: "" }), parentIssue({ number: Number.MAX_SAFE_INTEGER + 1 }),
  ]) await assert.rejects(new Hierarchy(fakeClient(issue), config).parent(child), /returned issue/);
  await assert.rejects(new Hierarchy(fakeClient(parentIssue({
    repository_url: "https://api.github.com/repos/outside/private",
  })), config).parent(child), /outside the configured project owner/);
});

test("a 422 add race succeeds only when a fresh parent is the target", async () => {
  const raced = httpError(422, "already attached"), client = fakeClient(parentIssue(), raced, parentIssue());
  await new Hierarchy(client, config).attach(parent, child, false, async () => child);
  assert.deepEqual(client.requests.map(([route]) => route), [
    "GET /repos/{owner}/{repo}/issues/{issue_number}",
    "POST /repos/{owner}/{repo}/issues/{issue_number}/sub_issues",
    "GET /repos/{owner}/{repo}/issues/{issue_number}/parent",
  ]);
});

test("an unconfirmed 422 add race rethrows the original failure", async () => {
  const raced = httpError(422, "validation failed");
  for (const confirmation of [parentIssue({ node_id: "I_other" }), httpError(404, "no parent"), httpError(503, "unavailable")]) {
    const hierarchy = new Hierarchy(fakeClient(parentIssue(), raced, confirmation), config);
    await assert.rejects(hierarchy.attach(parent, child, false, async () => child), (error) => error === raced);
  }
});

test("mutations verify the live parent identity and use its same-owner moved route", async () => {
  const reused = fakeClient(parentIssue({ node_id: "I_reused" }));
  await assert.rejects(new Hierarchy(reused, config).attach(parent, child, false, async () => child),
    (error) => error instanceof AttachPreflightError && /live parent issue does not match/.test(error.message));
  assert.deepEqual(reused.requests.map(([route]) => route), ["GET /repos/{owner}/{repo}/issues/{issue_number}"]);

  const moved = parentIssue({ repository_url: "https://api.github.com/repos/agglayer/renamed", number: 99 });
  const client = fakeClient(moved, {});
  await new Hierarchy(client, config).attach(parent, child, false, async () => child);
  assert.deepEqual(client.requests[1], ["POST /repos/{owner}/{repo}/issues/{issue_number}/sub_issues", {
    owner: "agglayer", repo: "renamed", issue_number: 99, headers,
    sub_issue_id: child.id, replace_parent: false,
  }]);
});

test("mutations reauthenticate the exact child after resolving the live parent", async () => {
  const client = fakeClient(parentIssue()), moved = { ...child, repository: "agglayer/transferred" };
  await assert.rejects(new Hierarchy(client, config).detach(parent, child, async () => moved),
    /outside the configured repository/);
  assert.deepEqual(client.requests.map(([route]) => route), ["GET /repos/{owner}/{repo}/issues/{issue_number}"]);
});

test("a live-parent transport failure is classified before any mutation", async () => {
  const unavailable = httpError(503, "unavailable"), hierarchy = new Hierarchy(fakeClient(unavailable), config);
  await assert.rejects(hierarchy.detach(parent, child, async () => child),
    (error) => error instanceof ParentReadError && error.cause === unavailable);
});

test("issue identity uses the stable node ID", () => {
  assert.equal(sameIssue(parent, { ...parent, repository: "agglayer/renamed", number: 99 }), true);
  assert.equal(sameIssue(parent, { ...parent, issueId: "I_other" }), false);
  assert.equal(sameIssue(parent, null), false);
});

function parentIssue(overrides = {}) {
  return {
    id: 42001, node_id: parent.issueId, number: parent.number,
    repository_url: `https://api.github.com/repos/${parent.repository}`, ...overrides,
  };
}
function childIssue(overrides = {}) {
  return {
    id: child.id, node_id: child.issueId, number: child.number,
    repository_url: `https://api.github.com/repos/${child.repository}`, ...overrides,
  };
}
function issueFor(issue, id) {
  return { id, node_id: issue.issueId, number: issue.number,
    repository_url: `https://api.github.com/repos/${issue.repository}` };
}
function fakeClient(...results) {
  return {
    requests: [],
    async request(route, params) {
      this.requests.push([route, params]);
      const result = results.shift() ?? {};
      if (result instanceof Error) throw result;
      return { data: result };
    },
  };
}
function httpError(status, message) { return Object.assign(new Error(message), { status }); }
