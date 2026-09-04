"use strict";

const assert = require("node:assert/strict");
const { createHmac } = require("node:crypto");
const test = require("node:test");
let Tracker;
let emptyState;
let parseCommand;
let renderComment;
let taskMarker;
let AttachPreflightError;
let ParentReadError;
test.before(async () => {
  ({ Tracker, emptyState, parseCommand, renderComment, taskMarker } = await import("../src/tracker.mjs"));
  ({ AttachPreflightError, ParentReadError } = await import("../src/hierarchy.mjs"));
});

const config = {
  owner: "agglayer",
  repo: "agglayer",
  projectOwner: "agglayer",
  repository: "agglayer/agglayer",
  repositoryId: "775930816",
  botLogin: "github-actions[bot]",
  serverUrl: "https://github.com",
  runUrl: "https://github.com/run/1",
  readyOptionId: "ready",
  inReviewOptionId: "in-review",
  statusFieldId: 1,
  projectsToken: "projects-secret",
  legacyTasks: [{ issue: 101, issueDatabaseId: 1102, issueNodeId: "I_101", pr: 9, reviewerId: "U_alice" }],
};

test("opening creates one assigned issue per requested person", async () => {
  const fixture = createFixture([reviewer("alice"), reviewer("bob")]);
  const result = await run(fixture, direct("opened"));

  assert.equal(result.errors.length, 0);
  assert.equal(result.state.source.via, "closing");
  assert.deepEqual(
    [...fixture.github.issues.values()].map((issue) => issue.assignees.map(({ login }) => login)),
    [["alice"], ["bob"]],
  );
  assert.deepEqual([...fixture.project.status.values()], ["ready", "ready"]);
  assert.deepEqual([...fixture.hierarchy.parents.values()].map(({ issueId }) => issueId), ["I_source", "I_source"]);
  assert.match(fixture.github.comments[0].body, /@alice: \[#101\]/);
  assert.match(fixture.github.comments[0].body, /\/review-tracker infer/);
});

test("automatic inference requires author write access but trusted infer remains available", async () => {
  const fixture = createFixture([]);
  fixture.github.closing = [];
  fixture.github.permission = "read";
  fixture.project.items = async () => [];

  let result = await run(fixture, direct("opened"));
  assert.equal(result.state.source, null);
  assert.match(result.warnings.at(-1), /reserved for PR authors/);
  assert.deepEqual(fixture.github.permissionReads, ["author"]);

  result = await run(fixture, command("/review-tracker infer"));
  assert.deepEqual(result.state.source, { none: true, via: "no-candidates" });
  assert.deepEqual(fixture.github.permissionReads, ["author"]);
});

test("a surviving lifecycle event runs one-shot inference when the opening signal was lost", async () => {
  const alice = reviewer("alice"), fixture = createFixture([alice]);
  fixture.github.closing = [];
  let modelCalls = 0;
  fixture.anthropic = { messages: { create: async () => {
    modelCalls += 1;
    return { stop_reason: "end_turn", content: [{ type: "text", text: JSON.stringify({ issueId: "I_source" }) }] };
  } } };

  const result = await run(fixture, direct("review_requested", alice));
  assert.equal(result.errors.length, 0);
  assert.equal(result.state.source.via, "model");
  assert.equal(modelCalls, 1);
  assert.deepEqual(fixture.github.permissionReads, ["author"]);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, "I_source");
});

test("a persisted model-none result is not re-spent on later events", async () => {
  const fixture = createFixture([reviewer("alice")]);
  fixture.github.closing = [];
  let modelCalls = 0;
  fixture.anthropic = { messages: { create: async () => {
    modelCalls += 1;
    return { stop_reason: "end_turn", content: [{ type: "text", text: JSON.stringify({ issueId: null }) }] };
  } } };

  let result = await run(fixture, direct("edited"));
  assert.deepEqual(result.state.source, { none: true, via: "model-none" });
  result = await run(fixture, direct("edited"));
  assert.deepEqual(result.state.source, { none: true, via: "model-none" });
  assert.equal(modelCalls, 1);
});

test("removal closes and re-request reopens the same issue at Ready", async () => {
  const alice = reviewer("alice");
  const fixture = createFixture([alice]);
  await run(fixture, direct("opened"));
  await run(fixture, direct("review_request_removed", alice));
  assert.equal(fixture.github.issues.get(101).state, "closed");

  await run(fixture, direct("review_requested", alice));
  assert.equal(fixture.github.issues.size, 1);
  assert.equal(fixture.github.issues.get(101).state, "open");
  assert.equal(fixture.project.status.get(201), "ready");
});

test("stale lifecycle deliveries converge from the current PR state", async () => {
  const alice = reviewer("alice");
  const fixture = createFixture([alice]);
  await run(fixture, direct("opened"));

  fixture.github.pull.requested_reviewers = [];
  await run(fixture, direct("review_requested", alice, false));
  assert.equal(fixture.github.issues.get(101).state, "closed");

  fixture.github.pull.requested_reviewers = [alice];
  await run(fixture, direct("review_request_removed", alice, false));
  assert.equal(fixture.github.issues.get(101).state, "open");

  fixture.github.pull.state = "closed";
  fixture.github.pull.closed_at = "2026-01-02T00:00:00Z";
  await run(fixture, direct("reopened", null, false));
  assert.equal(fixture.github.issues.get(101).state, "closed");

  fixture.github.pull.state = "open";
  fixture.github.pull.closed_at = null;
  await run(fixture, direct("closed", null, false));
  assert.equal(fixture.github.issues.get(101).state, "open");
});

test("a relayed lifecycle event runs lifecycle reconciliation", async () => {
  const alice = reviewer("alice"), fixture = createFixture([alice]);
  await run(fixture, direct("opened"));
  fixture.github.pull.requested_reviewers = [];

  await run(fixture, lifecycleEvent(), null, "review_request_removed");
  assert.equal(fixture.github.issues.get(101).state, "closed");
});

test("each distinct review ID transitions the source once, even for one reviewer", async () => {
  const alice = reviewer("alice");
  const fixture = createFixture([alice]);
  await run(fixture, direct("opened"));
  fixture.github.reviews.set("501", submitted("501", alice, "2026-01-01T01:00:00Z"));

  let result = await run(fixture, reviewEvent(), "501");
  assert.deepEqual(result.state.reviews, ["501"]);
  assert.equal(fixture.project.status.get(201), "in-review");
  assert.deepEqual(fixture.project.sourceReviews, ["2026-01-01T01:00:00Z"]);

  await run(fixture, reviewEvent(), "501");
  assert.equal(fixture.github.reviewReads.size, 0);
  fixture.github.reviews.set("502", submitted("502", alice, "2026-01-01T02:00:00Z"));
  result = await run(fixture, reviewEvent(), "502");
  assert.deepEqual(result.state.reviews, ["501", "502"]);
  assert.deepEqual(fixture.project.sourceReviews, [
    "2026-01-01T01:00:00Z",
    "2026-01-01T02:00:00Z",
  ]);
});

test("a later lifecycle run catches every unprocessed submitted review", async () => {
  const alice = reviewer("alice");
  const fixture = createFixture([alice]);
  await run(fixture, direct("opened"));
  fixture.github.reviews.set("501", submitted("501", alice, "2026-01-01T01:00:00Z"));
  fixture.github.reviews.set("502", submitted("502", alice, "2026-01-01T02:00:00Z"));
  fixture.github.pull.requested_reviewers = [];

  const result = await run(fixture, direct("edited"));
  assert.deepEqual(result.state.reviews, ["501", "502"]);
  assert.deepEqual(fixture.project.sourceReviews, [
    "2026-01-01T01:00:00Z",
    "2026-01-01T02:00:00Z",
  ]);
});

test("unsolicited and removed reviewers cannot move source work", async () => {
  const alice = reviewer("alice");
  const unsolicited = createFixture([]);
  await run(unsolicited, direct("opened"));
  unsolicited.github.reviews.set("501", submitted("501", alice, "2026-01-01T01:00:00Z"));
  let result = await run(unsolicited, reviewEvent(), "501");
  assert.deepEqual(result.state.reviews, ["501"]);
  assert.deepEqual(unsolicited.project.sourceReviews, []);

  const removed = createFixture([alice]);
  await run(removed, direct("opened"));
  await run(removed, direct("review_request_removed", alice));
  removed.github.reviews.set("502", submitted("502", alice, "2026-01-01T02:00:00Z"));
  result = await run(removed, reviewEvent(), "502");
  assert.deepEqual(result.state.reviews, ["502"]);
  assert.equal(removed.project.status.get(201), "ready");
  assert.deepEqual(removed.project.sourceReviews, []);
});

test("a re-request after review moves the existing task back to Ready", async () => {
  const alice = reviewer("alice");
  const fixture = createFixture([alice]);
  await run(fixture, direct("opened"));
  fixture.github.reviews.set("501", submitted("501", alice, "2026-01-01T01:00:00Z"));
  await run(fixture, reviewEvent(), "501");
  await run(fixture, direct("review_requested", alice));
  assert.equal(fixture.project.status.get(201), "ready");
});

test("current-state convergence distinguishes fulfilled reviews from later removals", async () => {
  const alice = reviewer("alice"), fixture = createFixture([alice]);
  await run(fixture, direct("opened"));
  fixture.github.reviews.set("501", submitted("501", alice, "2026-01-01T01:00:00Z"));
  await run(fixture, reviewEvent(), "501");
  fixture.github.pull.requested_reviewers = [];

  await run(fixture, direct("edited"));
  assert.equal(fixture.github.issues.get(101).state, "open");
  assert.equal(fixture.project.status.get(201), "in-review");

  await run(fixture, direct("review_requested", alice));
  assert.equal(fixture.project.status.get(201), "ready");
  await run(fixture, direct("review_request_removed", alice));
  assert.equal(fixture.github.issues.get(101).state, "closed");
});

test("PR reopen restores only tasks that the PR close closed", async () => {
  const alice = reviewer("alice");
  const bob = reviewer("bob");
  const fixture = createFixture([alice, bob]);
  await run(fixture, direct("opened"));
  fixture.github.reviews.set("501", submitted("501", alice, "2026-01-01T01:00:00Z"));
  await run(fixture, reviewEvent(), "501");
  await run(fixture, direct("review_request_removed", bob));
  const closed = await run(fixture, direct("closed"));
  assert.equal(fixture.github.issues.get(101).state, "closed");
  assert.equal(fixture.github.issues.get(102).state, "closed");
  assert.equal(closed.state.tasks.U_alice.reopenStatus, "in-review");
  fixture.project.status.set(201, "done");

  await run(fixture, direct("reopened"));
  assert.equal(fixture.github.issues.get(101).state, "open");
  assert.equal(fixture.github.issues.get(102).state, "closed");
  assert.equal(fixture.project.status.get(201), "in-review");
});

test("source corrections sync closed tasks without changing their review status", async () => {
  const alice = reviewer("alice");
  const fixture = createFixture([alice]);
  await run(fixture, direct("opened"));
  fixture.github.reviews.set("501", submitted("501", alice, "2026-01-01T01:00:00Z"));
  await run(fixture, reviewEvent(), "501");
  await run(fixture, direct("closed"));
  fixture.project.syncs.length = 0;

  await run(fixture, command("/review-tracker none"));
  assert.deepEqual(fixture.project.syncs, [{ item: 201, source: { none: true, via: "manual-none" }, status: null }]);
  assert.equal(fixture.project.status.get(201), "in-review");
  assert.equal(fixture.hierarchy.parents.has(101), false);
  await run(fixture, command("/review-tracker set #7"));
  assert.equal(fixture.github.issues.get(101).state, "closed");
  assert.equal(fixture.hierarchy.parents.get(101).issueId, "I_source");
  await run(fixture, direct("reopened"));
  assert.equal(fixture.github.issues.get(101).state, "open");
  assert.equal(fixture.project.status.get(201), "in-review");
});

test("set and reconcile refresh every task without replaying lifecycle events", async () => {
  const alice = reviewer("alice"), bob = reviewer("bob"), fixture = createFixture([alice, bob]);
  await run(fixture, direct("opened"));
  fixture.github.reviews.set("501", submitted("501", alice, "2026-01-01T01:00:00Z"));
  let result = await run(fixture, reviewEvent(), "501");
  await run(fixture, direct("review_request_removed", bob));

  for (const body of [
    "/review-tracker set agglayer/agglayer#7",
    "/review-tracker set agglayer/agglayer#7",
    "/review-tracker reconcile",
  ]) {
    fixture.project.syncs.length = 0;
    result = await run(fixture, command(body));
    assert.deepEqual(new Set(fixture.project.syncs.map(({ item }) => item)), new Set([201, 202]));
    assert.ok(fixture.project.syncs.every(({ status }) => status === null));
    assert.deepEqual(result.state.reviews, ["501"]);
    assert.equal(fixture.project.status.get(201), "in-review");
    assert.equal(fixture.project.status.get(202), "ready");
    assert.equal(fixture.github.issues.get(102).state, "closed");
    assert.deepEqual(fixture.project.sourceReviews, ["2026-01-01T01:00:00Z"]);
    assert.equal(fixture.github.reviewReads.size, 0);
  }
});

test("the review cap is checked before fetching or mutating a review", async () => {
  const fixture = createFixture([]);
  const tracker = new Tracker({
    github: fixture.github, project: fixture.project, config, core: fixture.core,
    context: { repo: { owner: config.owner, repo: config.repo } },
  });
  tracker.state = { reviews: Array.from({ length: 2_000 }, (_, index) => String(index)) };
  await assert.rejects(tracker.processReview("new"), /processed-review list is too large/);
  assert.equal(fixture.github.reviewReads.size, 0);
});

test("recovery can replay an existing review at the review cap", async () => {
  const alice = reviewer("alice"), fixture = createFixture([]);
  await fixture.github.rest.issues.create({ title: "Review PR #9", body: "task", assignees: [alice.login] });
  fixture.github.reviews.set("501", submitted("501", alice, "2026-01-01T01:00:00Z"));
  const tracker = new Tracker({
    github: fixture.github, project: fixture.project, config, core: fixture.core,
    context: { repo: { owner: config.owner, repo: config.repo } },
  });
  tracker.pull = fixture.github.pull;
  tracker.state = {
    source: { none: true, via: "manual-none" },
    tasks: { U_alice: { login: "alice", issue: 101, item: 201, replayAfter: "2026-01-01T00:00:00Z" } },
    reviews: ["501", ...Array.from({ length: 1_999 }, (_, index) => `old-${index}`)],
  };

  await tracker.processReview("501");
  assert.equal(fixture.project.status.get(201), "in-review");
  assert.equal(fixture.github.reviewReads.get("501"), 1);
});

test("source errors remain visible while review tasks still get created", async () => {
  const fixture = createFixture([reviewer("alice")]);
  fixture.github.graphql = async () => {
    throw Object.assign(new Error("Project source lookup failed"), { status: 503 });
  };
  const result = await run(fixture, direct("opened"));
  assert.equal(fixture.github.issues.size, 1);
  assert.equal(result.errors.length, 1);
  assert.match(fixture.github.comments[0].body, /Project source lookup failed/);
  assert.match(fixture.github.comments[0].body, /HTTP 503/);
  assert.deepEqual(fixture.project.syncs[0], { item: 201, source: null, status: "ready" });
  assert.equal(result.state.tasks.U_alice.hierarchyPending, true);
});

test("an issue-bound marker recovers a task after its first state saves fail", async () => {
  const alice = reviewer("alice"), fixture = createFixture([alice]);
  fixture.github.graphql = async () => { throw new Error("source unavailable"); };
  const createComment = fixture.github.rest.issues.createComment;
  fixture.github.rest.issues.createComment = async () => { throw new Error("state save unavailable"); };

  await assert.rejects(run(fixture, direct("opened")), /state save unavailable/);
  assert.equal(fixture.github.issues.size, 1);
  const issue = fixture.github.issues.get(101);
  assert.match(issue.body, new RegExp(taskMarker(config, 9, alice.node_id, issue).replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));

  fixture.github.rest.issues.createComment = createComment;
  const result = await run(fixture, direct("edited"));
  assert.equal(fixture.github.issues.size, 1);
  assert.equal(result.state.tasks.U_alice.issue, 101);
});

test("parent failures remain retryable without blocking Project synchronization", async () => {
  const fixture = createFixture([reviewer("alice"), reviewer("bob")]);
  fixture.hierarchy.attachFailures = 1;

  let result = await run(fixture, direct("opened"));
  assert.equal(result.errors.length, 1);
  assert.equal(fixture.project.status.get(201), "ready");
  assert.equal(fixture.project.status.get(202), "ready");
  assert.equal(result.state.tasks.U_alice.hierarchyPending, true);
  assert.equal(result.state.tasks.U_bob.hierarchyPending, false);
  assert.equal(fixture.hierarchy.parents.get(102).issueId, "I_source");
  assert.match(fixture.github.comments[0].body, /parent sync pending/);

  result = await run(fixture, direct("edited"));
  assert.equal(result.errors.length, 0);
  assert.equal(result.state.tasks.U_alice.hierarchyPending, false);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, "I_source");
});

test("a failed attach preflight does not block a later source correction", async () => {
  const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
  fixture.hierarchy.attachPreflightFailures = 1;

  let result = await run(fixture, direct("opened"));
  assert.match(result.errors[0], /selected source was replaced/);
  assert.equal(fixture.hierarchy.attaches.length, 0);
  assert.equal(result.state.tasks.U_alice.attemptedParent, undefined);
  assert.equal(result.state.tasks.U_alice.hierarchyInFlight, undefined);
  assert.equal(result.state.tasks.U_alice.hierarchyPending, true);

  fixture.project.find = async () => alternate;
  result = await run(fixture, command("/review-tracker set bridge#8"));
  assert.equal(result.errors.length, 0);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, alternate.issueId);
});

test("a transient parent-read failure preserves ownership and retries", async () => {
  const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
  await run(fixture, direct("opened"));
  fixture.project.find = async () => alternate;
  fixture.hierarchy.parentFailures = 1;

  let result = await run(fixture, command("/review-tracker set bridge#8"));
  assert.match(result.errors[0], /Parent read failed/);
  assert.equal(result.state.tasks.U_alice.hierarchyInFlight, undefined);
  assert.equal(result.state.tasks.U_alice.managedParent.issueId, "I_source");
  assert.equal(fixture.hierarchy.detaches.length, 0);

  result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.errors.length, 0);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, alternate.issueId);
  assert.equal(result.state.tasks.U_alice.managedParent.issueId, alternate.issueId);
  assert.equal(fixture.hierarchy.detaches.length, 1);
});

test("a parent validation failure retains the interruption fence", async () => {
  const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
  await run(fixture, direct("opened"));
  fixture.project.find = async () => alternate;
  fixture.hierarchy.parentValidationFailures = 1;

  let result = await run(fixture, command("/review-tracker set bridge#8"));
  assert.match(result.errors[0], /invalid live parent/);
  assert.equal(result.state.tasks.U_alice.hierarchyInFlight, true);
  assert.equal(result.state.tasks.U_alice.managedParent.issueId, "I_source");

  result = await run(fixture, command("/review-tracker reconcile"));
  assert.match(result.errors[0], /after an interrupted sync/);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, "I_source");
  assert.equal(result.state.tasks.U_alice.managedParent, undefined);
  assert.equal(fixture.hierarchy.detaches.length, 0);
});

test("a transient live-parent read before detach preserves ownership and retries", async () => {
  const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
  await run(fixture, direct("opened"));
  fixture.project.find = async () => alternate;
  fixture.hierarchy.detachPreparationFailures = 1;

  let result = await run(fixture, command("/review-tracker set bridge#8"));
  assert.match(result.errors[0], /Live parent read failed/);
  assert.equal(result.state.tasks.U_alice.hierarchyInFlight, undefined);
  assert.equal(result.state.tasks.U_alice.managedParent.issueId, "I_source");
  assert.equal(fixture.hierarchy.detaches.length, 0);

  result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.errors.length, 0);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, alternate.issueId);
  assert.equal(result.state.tasks.U_alice.managedParent.issueId, alternate.issueId);
});

test("a transient child reauthentication failure before detach preserves ownership and retries", async () => {
  const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
  await run(fixture, direct("opened"));
  fixture.project.find = async () => alternate;
  fixture.github.issueGetFailures.add(fixture.github.issueGets + 2);

  let result = await run(fixture, command("/review-tracker set bridge#8"));
  assert.match(result.errors[0], /Child read failed/);
  assert.equal(result.state.tasks.U_alice.hierarchyInFlight, undefined);
  assert.equal(result.state.tasks.U_alice.managedParent.issueId, "I_source");
  assert.equal(fixture.hierarchy.detaches.length, 0);

  result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.errors.length, 0);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, alternate.issueId);
  assert.equal(result.state.tasks.U_alice.managedParent.issueId, alternate.issueId);
});

test("source commands reparent and unlink only the tracker-managed relationship", async () => {
  const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
  await run(fixture, direct("opened"));
  fixture.project.find = async (repository, number) =>
    repository === alternate.repository && number === alternate.number ? alternate : null;

  let result = await run(fixture, command("/review-tracker set bridge#8"));
  assert.equal(result.errors.length, 0);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, alternate.issueId);
  assert.equal(fixture.hierarchy.attaches.at(-1).replaceParent, false);
  assert.equal(fixture.hierarchy.detaches[0].parent.issueId, "I_source");
  assert.deepEqual(result.state.tasks.U_alice.managedParent, {
    issueId: alternate.issueId, repository: alternate.repository, number: alternate.number,
  });

  result = await run(fixture, command("/review-tracker none"));
  assert.equal(result.errors.length, 0);
  assert.equal(fixture.hierarchy.parents.has(101), false);
  assert.equal(result.state.tasks.U_alice.managedParent, undefined);
});

test("an unrelated parent is preserved instead of being replaced or removed", async () => {
  const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
  await run(fixture, direct("opened"));
  fixture.hierarchy.parents.set(101, { issueId: "I_manual", repository: "agglayer/manual", number: 99 });
  fixture.project.find = async () => alternate;

  let result = await run(fixture, command("/review-tracker set bridge#8"));
  assert.match(result.errors[0], /unrelated parent/);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, "I_manual");
  assert.equal(fixture.hierarchy.attaches.length, 1);

  result = await run(fixture, command("/review-tracker none"));
  assert.match(result.errors[0], /unrelated parent/);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, "I_manual");
  assert.equal(fixture.hierarchy.detaches.length, 0);
});

test("observing an unrelated parent permanently relinquishes stale provenance", async () => {
  const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
  await run(fixture, direct("opened"));
  fixture.hierarchy.parents.set(101, { issueId: "I_manual", repository: "agglayer/manual", number: 99 });
  fixture.project.find = async () => alternate;

  let result = await run(fixture, command("/review-tracker set bridge#8"));
  assert.match(result.errors[0], /unrelated parent/);
  assert.equal(result.state.tasks.U_alice.managedParent, undefined);
  assert.equal(result.state.tasks.U_alice.attemptedParent, undefined);

  fixture.hierarchy.parents.set(101, { issueId: "I_source", repository: "agglayer/agglayer", number: 7 });
  result = await run(fixture, command("/review-tracker reconcile"));
  assert.match(result.errors[0], /unrelated parent/);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, "I_source");
  assert.equal(fixture.hierarchy.detaches.length, 0);
});

test("a matching relationship without signed provenance is never later reparented", async () => {
  const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
  const opened = await run(fixture, direct("opened"));
  delete opened.state.tasks.U_alice.managedParent;
  opened.state.tasks.U_alice.hierarchyPending = true;
  fixture.github.comments[0].body = renderComment(config, opened.state);

  let result = await run(fixture, command("/review-tracker reconcile"));
  assert.match(result.warnings.at(-1), /existing relationship was left unmanaged/);
  assert.equal(result.state.tasks.U_alice.managedParent, undefined);
  assert.equal(result.state.tasks.U_alice.hierarchyPending, false);
  fixture.project.find = async () => alternate;

  result = await run(fixture, command("/review-tracker set bridge#8"));
  assert.match(result.errors[0], /unrelated parent/);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, "I_source");
  assert.equal(fixture.hierarchy.detaches.length, 0);
});

test("parent mutation rejects a task whose live authenticated marker was changed", async () => {
  const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
  await run(fixture, direct("opened"));
  fixture.github.issues.get(101).body = "manually replaced body";
  fixture.project.find = async () => alternate;

  const result = await run(fixture, command("/review-tracker set bridge#8"));
  assert.match(result.errors[0], /not an authenticated tracker-owned issue/);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, "I_source");
  assert.equal(fixture.hierarchy.attaches.length, 1);
});

test("the live task marker is rechecked immediately before detach and attach", async () => {
  for (const mutationRead of [2, 3]) {
    const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
    await run(fixture, direct("opened"));
    fixture.project.find = async () => alternate;
    fixture.hierarchy.onParentRead = (read) => {
      if (read === mutationRead) fixture.github.issues.get(101).body = "marker removed during synchronization";
    };

    const result = await run(fixture, command("/review-tracker set bridge#8"));
    assert.match(result.errors[0], /not an authenticated tracker-owned issue/);
    assert.equal(fixture.hierarchy.attaches.length, 1);
    assert.equal(fixture.hierarchy.detaches.length, mutationRead === 2 ? 0 : 1);
  }
});

test("reconcile observes an existing desired parent without posting it again", async () => {
  const fixture = createFixture([reviewer("alice")]);
  await run(fixture, direct("opened"));
  const result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.errors.length, 0);
  assert.equal(fixture.hierarchy.attaches.length, 1);
});

test("an ordinary lifecycle run does not resync a completed parent", async () => {
  const alice = reviewer("alice"), fixture = createFixture([alice]);
  await run(fixture, direct("opened"));
  const reads = fixture.hierarchy.parentReads;
  await run(fixture, direct("review_requested", alice));
  assert.equal(fixture.hierarchy.parentReads, reads);
  assert.equal(fixture.hierarchy.attaches.length, 1);
});

test("a response-lost parent add is preserved but left unmanaged after a source change", async () => {
  const fixture = createFixture([reviewer("alice")]), second = alternateSource();
  const third = { ...alternateSource(), item: 12, itemNode: "PVTI_third", issueId: "I_third",
    repository: "agglayer/pm", number: 9 };
  await run(fixture, direct("opened"));
  fixture.project.find = async (repository, number) => [second, third]
    .find((source) => source.repository === repository && source.number === number) ?? null;
  fixture.hierarchy.attachResponseLosses = 1;

  let result = await run(fixture, command("/review-tracker set bridge#8"));
  assert.match(result.errors[0], /Parent add response was lost/);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, second.issueId);
  assert.equal(result.state.tasks.U_alice.attemptedParent.issueId, second.issueId);
  assert.equal(result.state.tasks.U_alice.hierarchyInFlight, true);

  result = await run(fixture, command("/review-tracker set pm#9"));
  assert.match(result.errors[0], /after an interrupted sync/);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, second.issueId);
  assert.equal(result.state.tasks.U_alice.managedParent, undefined);
  assert.equal(result.state.tasks.U_alice.attemptedParent, undefined);
  assert.equal(result.state.tasks.U_alice.hierarchyInFlight, undefined);
  assert.equal(fixture.hierarchy.detaches.length, 1);

  fixture.hierarchy.parents.delete(101);
  result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.errors.length, 0);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, third.issueId);
  assert.equal(result.state.tasks.U_alice.managedParent.issueId, third.issueId);
});

test("orphaned in-flight provenance never authorizes a later detach", async () => {
  const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
  const opened = await run(fixture, direct("opened"));
  opened.state.tasks.U_alice.attemptedParent = alternate;
  delete opened.state.tasks.U_alice.hierarchyInFlight;
  opened.state.tasks.U_alice.hierarchyPending = true;
  fixture.github.comments[0].body = renderComment(config, opened.state);
  fixture.hierarchy.parents.set(101, alternate);
  fixture.project.find = async () => ({ ...alternateSource(), issueId: "I_third", repository: "agglayer/pm", number: 9 });

  const result = await run(fixture, command("/review-tracker set pm#9"));
  assert.match(result.errors[0], /after an interrupted sync/);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, alternate.issueId);
  assert.equal(fixture.hierarchy.detaches.length, 0);
  assert.equal(result.state.tasks.U_alice.attemptedParent, undefined);
});

test("a concurrent manual reparent after detach is preserved", async () => {
  const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
  await run(fixture, direct("opened"));
  fixture.project.find = async () => alternate;
  fixture.hierarchy.parentAfterDetach = { issueId: "I_manual", repository: "agglayer/manual", number: 99 };

  const result = await run(fixture, command("/review-tracker set bridge#8"));
  assert.match(result.errors[0], /acquired an unrelated parent/);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, "I_manual");
  assert.equal(fixture.hierarchy.attaches.length, 1);
});

test("a cross-owner source is rejected before any hierarchy request", async () => {
  const fixture = createFixture([reviewer("alice")]);
  await run(fixture, direct("opened"));
  fixture.project.find = async () => ({ ...alternateSource(), repository: "outside/bridge" });
  const reads = fixture.hierarchy.parentReads;

  const result = await run(fixture, command("/review-tracker set outside/bridge#8"));
  assert.match(result.errors[0], /parent routing state is invalid/);
  assert.equal(fixture.hierarchy.parentReads, reads);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, "I_source");
});

test("cross-owner signed parent provenance is rejected before any hierarchy request", async () => {
  for (const field of ["managedParent", "attemptedParent"]) {
    const fixture = createFixture([reviewer("alice")]), opened = await run(fixture, direct("opened"));
    opened.state.tasks.U_alice[field] = { issueId: "I_outside", repository: "outside/private", number: 9 };
    opened.state.tasks.U_alice.hierarchyPending = true;
    fixture.github.comments[0].body = renderComment(config, opened.state);
    const reads = fixture.hierarchy.parentReads;

    const result = await run(fixture, command("/review-tracker reconcile"));
    assert.match(result.errors[0], /parent routing state is invalid/);
    assert.equal(fixture.hierarchy.parentReads, reads);
  }
});

test("a lost final state save leaves the observed parent unmanaged", async () => {
  const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
  await run(fixture, direct("opened"));
  fixture.project.find = async () => alternate;
  const updateComment = fixture.github.rest.issues.updateComment;
  let updates = 0;
  fixture.github.rest.issues.updateComment = async (params) => {
    if (++updates >= 4) throw new Error("final state save failed");
    return updateComment(params);
  };

  await assert.rejects(run(fixture, command("/review-tracker set bridge#8")), /final state save failed/);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, alternate.issueId);
  fixture.github.rest.issues.updateComment = updateComment;
  const result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.errors.length, 0);
  assert.equal(result.state.tasks.U_alice.managedParent, undefined);
  assert.equal(result.state.tasks.U_alice.attemptedParent, undefined);
  assert.equal(result.state.tasks.U_alice.hierarchyPending, false);
  assert.match(result.warnings.at(-1), /interrupted sync.*left unmanaged/);
});

test("a hierarchy preflight save failure performs no parent I/O", async () => {
  const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
  await run(fixture, direct("opened"));
  fixture.project.find = async () => alternate;
  const updateComment = fixture.github.rest.issues.updateComment;
  let updates = 0;
  fixture.github.rest.issues.updateComment = async (params) => {
    if (++updates === 2) throw new Error("hierarchy preflight save failed");
    return updateComment(params);
  };
  const reads = fixture.hierarchy.parentReads, detaches = fixture.hierarchy.detaches.length;

  let result = await run(fixture, command("/review-tracker set bridge#8"));
  assert.match(result.errors[0], /hierarchy preflight save failed/);
  assert.equal(fixture.hierarchy.parentReads, reads);
  assert.equal(fixture.hierarchy.detaches.length, detaches);
  assert.equal(result.state.tasks.U_alice.hierarchyInFlight, undefined);

  fixture.github.rest.issues.updateComment = updateComment;
  result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.errors.length, 0);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, alternate.issueId);
});

test("an interrupted conflict cannot revive stale parent ownership", async () => {
  const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
  await run(fixture, direct("opened"));
  fixture.project.find = async () => alternate;
  fixture.hierarchy.parents.set(101, { issueId: "I_manual", repository: "agglayer/manual", number: 99 });
  const updateComment = fixture.github.rest.issues.updateComment;
  let updates = 0;
  fixture.github.rest.issues.updateComment = async (params) => {
    if (++updates >= 3) throw new Error("conflict state save failed");
    return updateComment(params);
  };

  await assert.rejects(run(fixture, command("/review-tracker set bridge#8")), /conflict state save failed/);
  fixture.github.rest.issues.updateComment = updateComment;
  fixture.hierarchy.parents.set(101, { issueId: "I_source", repository: "agglayer/agglayer", number: 7 });
  const detaches = fixture.hierarchy.detaches.length;

  const result = await run(fixture, command("/review-tracker reconcile"));
  assert.match(result.errors[0], /after an interrupted sync/);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, "I_source");
  assert.equal(fixture.hierarchy.detaches.length, detaches);
  assert.equal(result.state.tasks.U_alice.managedParent, undefined);
});

test("a transferred review issue cannot be reparented with the organization token", async () => {
  const fixture = createFixture([reviewer("alice")]), alternate = alternateSource();
  await run(fixture, direct("opened"));
  fixture.github.issues.get(101).repository_url = "https://api.github.com/repos/agglayer/transferred";
  fixture.project.find = async () => alternate;
  const parentReads = fixture.hierarchy.parentReads;
  const detaches = fixture.hierarchy.detaches.length;

  const result = await run(fixture, command("/review-tracker set bridge#8"));
  assert.match(result.errors[0], /not an authenticated tracker-owned issue/);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, "I_source");
  assert.equal(fixture.hierarchy.parentReads, parentReads);
  assert.equal(fixture.hierarchy.detaches.length, detaches);
});

test("an interrupted detach never removes a manually restored parent twice", async () => {
  const fixture = createFixture([reviewer("alice")]);
  await run(fixture, direct("opened"));
  const updateComment = fixture.github.rest.issues.updateComment;
  let updates = 0;
  fixture.github.rest.issues.updateComment = async (params) => {
    if (++updates >= 3) throw new Error("detach state save failed");
    return updateComment(params);
  };

  await assert.rejects(run(fixture, command("/review-tracker none")), /detach state save failed/);
  assert.equal(fixture.hierarchy.parents.has(101), false);
  fixture.github.rest.issues.updateComment = updateComment;
  fixture.hierarchy.parents.set(101, { issueId: "I_source", repository: "agglayer/agglayer", number: 7 });
  const detaches = fixture.hierarchy.detaches.length;

  const result = await run(fixture, command("/review-tracker reconcile"));
  assert.match(result.errors[0], /after an interrupted sync/);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, "I_source");
  assert.equal(fixture.hierarchy.detaches.length, detaches);
});

test("unmanage relinquishes unverifiable parent provenance without touching relationships", async () => {
  const fixture = createFixture([reviewer("alice")]);
  await run(fixture, direct("opened"));
  fixture.hierarchy.parent = async () => {
    throw new ParentReadError(Object.assign(
      new Error("The child has no visible parent, but a recorded parent could not be verified."), { status: 404 }));
  };
  let result = await run(fixture, command("/review-tracker reconcile"));
  assert.match(result.errors[0], /could not be verified/);
  assert.equal(result.state.tasks.U_alice.managedParent.issueId, "I_source");
  assert.equal(result.state.tasks.U_alice.hierarchyInFlight, undefined);
  assert.equal(result.state.tasks.U_alice.hierarchyPending, true);

  result = await run(fixture, command("/review-tracker unmanage"));
  assert.equal(result.errors.length, 0);
  assert.match(result.warnings.at(-1), /relinquished/);
  assert.equal(result.state.tasks.U_alice.managedParent, undefined);
  assert.equal(result.state.tasks.U_alice.hierarchyPending, false);
  assert.equal(fixture.hierarchy.parents.get(101).issueId, "I_source");
  assert.equal(fixture.hierarchy.detaches.length, 0);
});

test("a failed Project add preserves and retries the created review issue", async () => {
  const fixture = createFixture([reviewer("alice")]);
  const calls = [];
  const createIssue = fixture.github.rest.issues.create;
  fixture.github.rest.issues.create = async (params) => { calls.push("issue"); return createIssue(params); };
  const createComment = fixture.github.rest.issues.createComment;
  fixture.github.rest.issues.createComment = async (params) => { calls.push("state"); return createComment(params); };
  const updateComment = fixture.github.rest.issues.updateComment;
  fixture.github.rest.issues.updateComment = async (params) => { calls.push("state"); return updateComment(params); };
  const ensureIssue = fixture.project.ensureIssue.bind(fixture.project);
  fixture.project.ensureIssue = async (issue) => { calls.push("project"); return ensureIssue(issue); };
  fixture.project.ensureIssueFailures = 2;

  let result = await run(fixture, direct("opened"));
  assert.equal(result.errors.length, 2);
  assert.deepEqual(calls.slice(0, 4), ["state", "issue", "state", "project"]);
  assert.equal(fixture.github.issues.size, 1);
  assert.deepEqual(result.state.tasks.U_alice, {
    login: "alice", issue: 101, pending: true, closedByPr: false, fulfilled: false,
    hierarchyPending: false,
    managedParent: { issueId: "I_source", repository: "agglayer/agglayer", number: 7 },
  });
  assert.match(fixture.github.comments[0].body, /@alice: \[#101\].*Project sync pending/);

  result = await run(fixture, direct("edited"));
  assert.equal(result.errors.length, 0);
  assert.equal(fixture.github.issues.size, 1);
  assert.equal(result.state.tasks.U_alice.item, 201);
  assert.equal(result.state.tasks.U_alice.pending, undefined);
  assert.equal(fixture.project.status.get(201), "ready");
});

test("PR close and reopen converge while Project attachment is pending", async () => {
  const fixture = createFixture([reviewer("alice")]);
  fixture.project.ensureIssueFailures = 2;
  let result = await run(fixture, direct("opened"));
  assert.equal(result.state.tasks.U_alice.item, undefined);

  result = await run(fixture, direct("closed"));
  assert.equal(result.errors.length, 0);
  assert.equal(fixture.github.issues.get(101).state, "closed");
  assert.equal(result.state.tasks.U_alice.closedByPr, true);

  result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.state.tasks.U_alice.item, 201);
  assert.equal(result.state.tasks.U_alice.pending, undefined);
  assert.equal(fixture.project.status.get(201), undefined);

  result = await run(fixture, direct("reopened"));
  assert.equal(result.errors.length, 0);
  assert.equal(fixture.github.issues.get(101).state, "open");
  assert.equal(result.state.tasks.U_alice.item, 201);
  assert.equal(result.state.tasks.U_alice.pending, undefined);
  assert.equal(fixture.project.status.get(201), "ready");
});

test("PR close retries a failed Project status snapshot before closing", async () => {
  const fixture = createFixture([reviewer("alice")]);
  await run(fixture, direct("opened"));
  fixture.project.getItemFailures = 1;

  let result = await run(fixture, direct("closed"));
  assert.equal(result.errors.length, 1);
  assert.equal(fixture.github.issues.get(101).state, "open");
  assert.equal(result.state.tasks.U_alice.closedByPr, false);

  result = await run(fixture, direct("closed"));
  assert.equal(result.errors.length, 0);
  assert.equal(fixture.github.issues.get(101).state, "closed");
  assert.equal(result.state.tasks.U_alice.closedByPr, true);
});

test("PR close persists reopen intent before closing the review issue", async () => {
  const alice = reviewer("alice"), fixture = createFixture([alice]);
  await run(fixture, direct("opened"));
  fixture.github.reviews.set("501", submitted("501", alice, "2026-01-01T01:00:00Z"));
  await run(fixture, reviewEvent(), "501");
  fixture.github.pull.requested_reviewers = [];
  fixture.github.pull.state = "closed";
  fixture.github.pull.closed_at = "2026-01-02T00:00:00Z";

  const updateComment = fixture.github.rest.issues.updateComment;
  let saves = 0;
  fixture.github.rest.issues.updateComment = async (params) => {
    if (++saves === 2) throw new Error("final state save failed");
    return updateComment(params);
  };
  await assert.rejects(run(fixture, direct("closed")), /final state save failed/);
  assert.equal(fixture.github.issues.get(101).state, "closed");

  fixture.github.rest.issues.updateComment = updateComment;
  fixture.github.pull.state = "open";
  fixture.github.pull.closed_at = null;
  const result = await run(fixture, direct("reopened"));
  assert.equal(result.errors.length, 0);
  assert.equal(fixture.github.issues.get(101).state, "open");
  assert.equal(fixture.project.status.get(201), "in-review");
});

test("a task-item 404 clears routing while a missing source preserves it", async () => {
  const fixture = createFixture([reviewer("alice")]);
  await run(fixture, direct("opened"));

  fixture.project.sync = async () => {
    throw Object.assign(new Error("source item gone"), { status: 404, sourceMissing: true });
  };
  let result = await run(fixture, command("/review-tracker reconcile"));
  assert.match(result.errors[0], /source item gone/);
  assert.equal(result.state.tasks.U_alice.item, 201);
  assert.equal(result.state.tasks.U_alice.pending, true);

  fixture.project.sync = async () => { throw Object.assign(new Error("task item gone"), { status: 404 }); };
  result = await run(fixture, command("/review-tracker reconcile"));
  assert.match(result.errors[0], /task item gone/);
  assert.equal(result.state.tasks.U_alice.item, undefined);
});

test("a failed Project sync keeps its review live through reviewer removal", async () => {
  const alice = reviewer("alice"), fixture = createFixture([alice]);
  fixture.project.syncFailures = 5;
  let result = await run(fixture, direct("opened"));
  assert.equal(result.state.tasks.U_alice.pending, true);
  fixture.github.reviews.set("501", submitted("501", alice, "2026-01-01T01:00:00Z"));

  result = await run(fixture, reviewEvent(), "501");
  assert.deepEqual(result.state.reviews, []);
  assert.equal(result.state.tasks.U_alice.pending, true);
  assert.equal(result.state.tasks.U_alice.fulfilled, true);

  result = await run(fixture, direct("edited"));
  assert.deepEqual(result.state.reviews, []);
  assert.equal(fixture.github.issues.get(101).state, "open");
  assert.equal(result.state.tasks.U_alice.fulfilled, true);

  result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.errors.length, 0);
  assert.deepEqual(result.state.reviews, ["501"]);
  assert.equal(result.state.tasks.U_alice.fulfilled, true);
  assert.equal(result.state.tasks.U_alice.pending, undefined);
  assert.equal(fixture.project.status.get(201), "in-review");
  assert.equal(fixture.github.issues.get(101).state, "open");
  assert.equal(fixture.github.issues.size, 1);
});

test("legacy empty state recovers an unsigned orphan and replays its review", async () => {
  const alice = reviewer("alice"), fixture = createFixture([]);
  await fixture.github.rest.issues.create({
    title: "Review PR #9", body: legacyTaskMarker(9, alice.node_id), assignees: [alice.login],
  });
  fixture.github.reviews.set("501", submitted("501", alice, "2026-01-01T01:00:00Z"));
  const state = emptyState(config, 9);
  state.source = { none: true, via: "manual-none" };
  state.reviews = ["501"];
  delete state.taskRecovery;
  fixture.github.comments.push({ id: 1, user: { login: config.botLogin }, body: renderComment(config, state) });

  fixture.github.pull.state = "closed";
  fixture.github.pull.closed_at = "2026-01-02T00:00:00Z";
  const result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.errors.length, 0);
  assert.deepEqual(result.warnings, []);
  assert.equal(fixture.github.issues.size, 1);
  assert.equal(result.state.tasks.U_alice.issue, 101);
  assert.equal(result.state.tasks.U_alice.fulfilled, true);
  assert.equal(result.state.tasks.U_alice.replayAfter, undefined);
  assert.deepEqual(result.state.reviews, ["501"]);
  assert.equal(fixture.project.status.get(201), "in-review");
  assert.equal(fixture.github.issues.get(101).state, "closed");
  assert.equal(result.state.tasks.U_alice.closedByPr, true);
  const issue = fixture.github.issues.get(101);
  assert.match(issue.body, new RegExp(taskMarker(config, 9, alice.node_id, issue).replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
});

test("legacy recovery rejects unsigned task issues outside its allowlist", async () => {
  const alice = reviewer("alice"), fixture = createFixture([]);
  await fixture.github.rest.issues.create({ title: "Unrelated bot issue", body: "No task marker." });
  await fixture.github.rest.issues.create({
    title: "Review PR #9", body: legacyTaskMarker(9, alice.node_id), assignees: [alice.login],
  });
  const state = emptyState(config, 9);
  delete state.taskRecovery;
  fixture.github.comments.push({ id: 1, user: { login: config.botLogin }, body: renderComment(config, state) });

  const result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.state.tasks.U_alice, undefined);
});

test("legacy recovery rejects an unsigned issue-bound v2 marker", async () => {
  const alice = reviewer("alice"), fixture = createFixture([]);
  await fixture.github.rest.issues.create({ title: "Unrelated bot issue", body: "No task marker." });
  const { data: issue } = await fixture.github.rest.issues.create({
    title: "Review PR #9", body: "placeholder", assignees: [alice.login],
  });
  const payload = Buffer.from(JSON.stringify({
    v: 2, repositoryId: config.repositoryId, pr: 9, reviewerId: alice.node_id,
    issueDatabaseId: issue.id, issueNodeId: issue.node_id, issue: issue.number,
  })).toString("base64url");
  issue.body = `<!-- review-tracker-task:${payload} -->`;
  const state = emptyState(config, 9);
  delete state.taskRecovery;
  fixture.github.comments.push({ id: 1, user: { login: config.botLogin }, body: renderComment(config, state) });

  const result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.state.tasks.U_alice, undefined);
});

test("legacy recovery replays every consumed review for one task", async () => {
  const alice = reviewer("alice"), fixture = createFixture([]);
  await fixture.github.rest.issues.create({
    title: "Review PR #9", body: legacyTaskMarker(9, alice.node_id), assignees: [alice.login],
  });
  fixture.github.reviews.set("501", submitted("501", alice, "2026-01-01T01:00:00Z"));
  fixture.github.reviews.set("502", submitted("502", alice, "2026-01-01T02:00:00Z"));
  const state = emptyState(config, 9);
  state.source = sourceItem();
  state.reviews = ["501", "502"];
  delete state.taskRecovery;
  fixture.github.comments.push({ id: 1, user: { login: config.botLogin }, body: renderComment(config, state) });

  const result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.errors.length, 0);
  assert.deepEqual(fixture.project.sourceReviews, ["2026-01-01T01:00:00Z", "2026-01-01T02:00:00Z"]);
  assert.equal(result.state.tasks.U_alice.replayAfter, undefined);
});

test("legacy recovery replays a review for a closed orphan without reopening it", async () => {
  const alice = reviewer("alice"), fixture = createFixture([]);
  const { data: issue } = await fixture.github.rest.issues.create({
    title: "Review PR #9", body: legacyTaskMarker(9, alice.node_id), assignees: [alice.login],
  });
  issue.state = "closed";
  issue.closed_at = "2026-01-02T01:00:00Z";
  fixture.github.reviews.set("501", submitted("501", alice, "2026-01-01T01:00:00Z"));
  const state = emptyState(config, 9);
  state.source = sourceItem();
  state.reviews = ["501"];
  delete state.taskRecovery;
  fixture.github.comments.push({ id: 1, user: { login: config.botLogin }, body: renderComment(config, state) });
  fixture.github.pull.state = "closed";
  fixture.github.pull.closed_at = "2026-01-02T00:00:00Z";

  const result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.errors.length, 0);
  assert.equal(result.state.tasks.U_alice.fulfilled, true);
  assert.equal(result.state.tasks.U_alice.closedByPr, true);
  assert.equal(result.state.tasks.U_alice.reopenStatus, "in-review");
  assert.equal(result.state.tasks.U_alice.replayAfter, undefined);
  assert.equal(fixture.github.issues.get(101).state, "closed");
  assert.equal(fixture.project.status.get(201), "in-review");
  assert.deepEqual(fixture.project.sourceReviews, ["2026-01-01T01:00:00Z"]);
});

test("current state recovers signed task markers but rejects unsigned ones", async () => {
  const alice = reviewer("alice");
  for (const [body, recovered] of [
    [taskMarker(config, 9, alice.node_id, { id: 1102, node_id: "I_101", number: 101 }), true],
    [signedTaskMarkerV1(9, alice.node_id), false],
    [legacyTaskMarker(9, alice.node_id), false],
  ]) {
    const fixture = createFixture([]);
    await fixture.github.rest.issues.create({ title: "Review PR #9", body, assignees: [alice.login] });
    const state = emptyState(config, 9);
    state.source = { none: true, via: "manual-none" };
    fixture.github.comments.push({ id: 1, user: { login: config.botLogin }, body: renderComment(config, state) });

    const result = await run(fixture, command("/review-tracker reconcile"));
    assert.equal(Boolean(result.state.tasks.U_alice), recovered);
  }
});

test("a mapped signed v1 task is upgraded to its issue-bound marker", async () => {
  const alice = reviewer("alice"), fixture = createFixture([]);
  const { data: issue } = await fixture.github.rest.issues.create({
    title: "Review PR #9", body: signedTaskMarkerV1(9, alice.node_id), assignees: [alice.login],
  });
  const state = emptyState(config, 9);
  state.source = { none: true, via: "manual-none" };
  state.tasks.U_alice = {
    login: alice.login, issue: issue.number, item: 201, fulfilled: true,
    closedByPr: false, hierarchyPending: true,
  };
  fixture.github.comments.push({ id: 1, user: { login: config.botLogin }, body: renderComment(config, state) });

  const result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.errors.length, 0);
  assert.match(issue.body, new RegExp(taskMarker(config, 9, alice.node_id, issue).replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
});

test("a mapped unsigned v1 production task is upgraded via its signed state binding", async () => {
  const alice = reviewer("alice"), fixture = createFixture([]);
  await fixture.github.rest.issues.create({ title: "Unrelated bot issue", body: "No task marker." });
  const { data: issue } = await fixture.github.rest.issues.create({
    title: "Review PR #9", body: legacyTaskMarker(9, alice.node_id), assignees: [alice.login],
  });
  const state = emptyState(config, 9);
  state.source = { none: true, via: "manual-none" };
  state.tasks.U_alice = {
    login: alice.login, issue: issue.number, item: 201, fulfilled: true,
    closedByPr: false, hierarchyPending: true,
  };
  fixture.github.comments.push({ id: 1, user: { login: config.botLogin }, body: renderComment(config, state) });

  const result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.errors.length, 0);
  assert.match(issue.body, new RegExp(taskMarker(config, 9, alice.node_id, issue).replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
});

test("recovery rejects a valid task marker copied onto another bot issue", async () => {
  const alice = reviewer("alice"), fixture = createFixture([alice]);
  await fixture.github.rest.issues.create({ title: "Other bot issue", body: "Unrelated.", assignees: [alice.login] });
  await run(fixture, direct("opened"));
  fixture.github.issues.get(101).body = fixture.github.issues.get(102).body;
  fixture.hierarchy.parents.clear();
  const state = emptyState(config, 9);
  state.source = { none: true, via: "manual-none" };
  fixture.github.comments[0].body = renderComment(config, state);

  const result = await run(fixture, command("/review-tracker reconcile"));
  assert.equal(result.errors.length, 0);
  assert.equal(result.state.tasks.U_alice.issue, 102);
});

test("failed inference preserves the current source and copied fields", async () => {
  const fixture = createFixture([reviewer("alice")]);
  let result = await run(fixture, direct("opened"));
  const source = result.state.source;
  const syncs = fixture.project.syncs.length;
  fixture.github.graphql = async () => {
    throw new Error("inference unavailable");
  };
  result = await run(fixture, command("/review-tracker infer"));
  assert.deepEqual(result.state.source, source);
  assert.equal(fixture.project.syncs.length, syncs);
  assert.match(result.errors[0], /inference unavailable/);
});

test("a later closing reference replaces an inferred none mapping", async () => {
  const fixture = createFixture([reviewer("alice")]);
  fixture.github.closing = [];
  fixture.project.items = async () => {
    const item = sourceItem();
    item.assignees = [];
    return [item];
  };
  let result = await run(fixture, direct("opened"));
  assert.equal(result.state.source.via, "no-candidates");
  fixture.github.closing = [{ id: "I_source" }];
  fixture.project.items = async () => [sourceItem()];
  result = await run(fixture, direct("edited"));
  assert.equal(result.state.source.via, "closing");
});

test("removing a closing reference preserves its mapping without resyncing tasks", async () => {
  const fixture = createFixture([reviewer("alice")]);
  const opened = await run(fixture, direct("opened"));
  fixture.github.closing = [];
  fixture.project.syncs.length = 0;

  const edited = await run(fixture, direct("edited"));
  assert.deepEqual(edited.state.source, opened.state.source);
  assert.deepEqual(fixture.project.syncs, []);
});

test("a newly resolved source is copied to every existing review task", async () => {
  const fixture = createFixture([reviewer("alice"), reviewer("bob")]);
  const graphql = fixture.github.graphql.bind(fixture.github);
  fixture.github.graphql = async () => { throw new Error("temporary source failure"); };
  await run(fixture, direct("opened"));

  fixture.github.graphql = graphql;
  fixture.project.syncs.length = 0;
  await run(fixture, direct("review_requested", reviewer("alice")));
  assert.deepEqual(new Set(fixture.project.syncs.map(({ item }) => item)), new Set([201, 202]));
  assert.ok(fixture.project.syncs.every(({ source }) => source.item === 10));
});

test("trusted commands use exact syntax", () => {
  const current = { owner: "agglayer", repo: "agglayer" };
  assert.deepEqual(parseCommand("/review-tracker none"), { kind: "none" });
  assert.deepEqual(parseCommand("/review-tracker unmanage"), { kind: "unmanage" });
  assert.deepEqual(parseCommand("/review-tracker set #7", current), {
    kind: "set", repository: "agglayer/agglayer", number: 7,
  });
  assert.deepEqual(parseCommand("/review-tracker set bridge#7", current), {
    kind: "set", repository: "agglayer/bridge", number: 7,
  });
  assert.deepEqual(parseCommand("/review-tracker set agglayer/agglayer#7"), {
    kind: "set",
    repository: "agglayer/agglayer",
    number: 7,
  });
  assert.deepEqual(parseCommand("/review-tracker set https://github.com/agglayer/agglayer/issues/7"), {
    kind: "set", repository: "agglayer/agglayer", number: 7,
  });
  assert.throws(() => parseCommand("/review-tracker set #7"), /invalid/);
  assert.throws(() => parseCommand(`/review-tracker set #${Number.MAX_SAFE_INTEGER + 1}`, current), /invalid/);
  for (const body of ["/review-tracker set #0", "/review-tracker set #01", "/review-tracker set bridge/issues/7",
    "/review-tracker set https://github.com/bridge#7", "/review-tracker set .#7",
    "/review-tracker set agglayer/..#7", "/review-tracker set #7\n"])
    assert.throws(() => parseCommand(body, current), /invalid/);
  assert.throws(() => parseCommand("please /review-tracker none"), /invalid/);
});

test("the set shorthand is resolved against the current PR repository", async () => {
  const fixture = createFixture([]);
  const result = await run(fixture, command("/review-tracker set #7"));
  assert.equal(result.errors.length, 0);
  assert.equal(result.state.source.repository, "agglayer/agglayer");
  assert.equal(result.state.source.number, 7);
});

test("invalid trusted commands are reported in the canonical comment", async () => {
  const fixture = createFixture([]);
  const result = await run(fixture, command("/review-tracker typo"));
  assert.match(result.errors[0], /command: The review-tracker command is invalid/);
  assert.equal(fixture.github.comments.length, 1);
  assert.match(fixture.github.comments[0].body, /Tracking completed with errors/);
});

test("a command comment lost to concurrency is applied on the next event", async () => {
  const fixture = createFixture([reviewer("alice")]);
  await run(fixture, direct("opened"));
  fixture.github.comments.push({ id: 5000, user: { login: "maintainer" }, body: "/review-tracker none" });

  const result = await run(fixture, direct("edited"));
  assert.equal(result.errors.length, 0);
  assert.deepEqual(result.state.source, { none: true, via: "manual-none" });
  assert.equal(result.state.lastCommand, "5000");
  assert.deepEqual(fixture.github.permissionReads, ["author", "maintainer"]);
  assert.equal(fixture.hierarchy.parents.has(101), false);
});

test("a recovered older command applies before the triggering newer command", async () => {
  const fixture = createFixture([]);
  fixture.github.comments.push({ id: 30, user: { login: "maintainer" }, body: "/review-tracker none" });

  const result = await run(fixture, command("/review-tracker set agglayer/agglayer#7", 40));
  assert.equal(result.errors.length, 0);
  assert.equal(result.state.source.via, "manual");
  assert.equal(result.state.source.number, 7);
  assert.equal(result.state.lastCommand, "40");
});

test("scanned commands without write access are ignored once with a warning", async () => {
  const fixture = createFixture([]);
  await run(fixture, direct("opened"));
  fixture.github.permission = "read";
  fixture.github.comments.push({ id: 6000, user: { login: "drive-by" }, body: "/review-tracker none" });

  let result = await run(fixture, direct("edited"));
  assert.equal(result.warnings.some((warning) => /Ignoring review-tracker command from @drive-by/.test(warning)), true);
  assert.equal(result.state.source.via, "closing");
  assert.equal(result.state.lastCommand, "6000");

  const reads = fixture.github.permissionReads.length;
  result = await run(fixture, direct("edited"));
  assert.equal(fixture.github.permissionReads.length, reads);
  assert.equal(result.warnings.some((warning) => /drive-by/.test(warning)), false);
});

test("newer command comments cannot be overwritten by delayed older commands", async () => {
  const fixture = createFixture([]);
  let result = await run(fixture, command("/review-tracker none", 20));
  assert.deepEqual(result.state.source, { none: true, via: "manual-none" });

  result = await run(fixture, command("/review-tracker set agglayer/agglayer#7", 10));
  assert.deepEqual(result.state.source, { none: true, via: "manual-none" });
  assert.equal(result.state.lastCommand, "20");
  assert.match(result.warnings[0], /out-of-order command comment 10/);
});

async function run(fixture, context, reviewId = null, eventAction = null) {
  if (context.eventName === "workflow_run" && reviewId) {
    const submittedReview = fixture.github.reviews.get(String(reviewId));
    if (submittedReview?.user?.node_id) fixture.github.pull.requested_reviewers =
      fixture.github.pull.requested_reviewers.filter(({ node_id }) => node_id !== submittedReview.user.node_id);
  }
  if (context.eventName === "pull_request_target" && context.applyCurrent !== false) {
    const reviewer = context.payload.requested_reviewer;
    if (context.payload.action === "review_request_removed" && reviewer)
      fixture.github.pull.requested_reviewers = fixture.github.pull.requested_reviewers
        .filter(({ node_id }) => node_id !== reviewer.node_id);
    if (context.payload.action === "review_requested" && reviewer &&
      !fixture.github.pull.requested_reviewers.some(({ node_id }) => node_id === reviewer.node_id))
      fixture.github.pull.requested_reviewers.push(reviewer);
    if (context.payload.action === "closed") {
      fixture.github.pull.state = "closed"; fixture.github.pull.closed_at = "2026-01-02T00:00:00Z";
    }
    if (context.payload.action === "reopened") {
      fixture.github.pull.state = "open"; fixture.github.pull.closed_at = null;
    }
  }
  const tracker = new Tracker({
    github: fixture.github,
    project: fixture.project,
    hierarchy: fixture.hierarchy,
    anthropic: fixture.anthropic,
    config,
    context,
    core: fixture.core,
    apiKey: "",
    reviewId,
    eventAction,
  });
  return tracker.run(9);
}

function createFixture(requested) {
  const github = new FakeGithub(requested);
  return { github, project: new FakeProject(), hierarchy: new FakeHierarchy(), core: { error() {} } };
}

class FakeGithub {
  constructor(requested) {
    this.comments = [];
    this.issues = new Map();
    this.reviews = new Map();
    this.reviewReads = new Map();
    this.permission = "write";
    this.permissionReads = [];
    this.issueGets = 0;
    this.issueGetFailures = new Set();
    this.closing = [{ id: "I_source" }];
    this.request = async () => ({ data: "diff --git a/old b/new" });
    this.pull = {
      number: 9,
      title: "Implement settlement safety",
      html_url: "https://github.com/agglayer/agglayer/pull/9",
      state: "open",
      user: { login: "author", node_id: "U_author" },
      base: { repo: { id: 775930816 } },
      requested_reviewers: requested,
    };
    this.rest = {
      repos: { getCollaboratorPermissionLevel: async ({ username }) => {
        this.permissionReads.push(username);
        return { data: { permission: this.permission } };
      } },
      pulls: {
        get: async () => ({ data: this.pull }),
        listRequestedReviewers: async () => ({ data: {
          users: this.pull.requested_reviewers, teams: this.pull.requested_teams ?? [],
        } }),
        listReviews: "listReviews",
        getReview: async ({ review_id }) => {
          const id = String(review_id);
          this.reviewReads.set(id, (this.reviewReads.get(id) ?? 0) + 1);
          return { data: this.reviews.get(id) };
        },
      },
      issues: {
        listComments: "listComments",
        listForRepo: "listForRepo",
        create: async (params) => {
          const number = 101 + this.issues.size;
          const issue = { id: 1001 + number, node_id: `I_${number}`, number, state: "open", ...params,
            assignees: (params.assignees ?? []).map(reviewer), user: { login: config.botLogin },
            repository_url: `https://api.github.com/repos/${config.repository}`,
            created_at: "2026-01-01T00:00:00Z" };
          this.issues.set(number, issue);
          return { data: issue };
        },
        get: async ({ issue_number }) => {
          if (this.issueGetFailures.has(++this.issueGets))
            throw Object.assign(new Error("Child read failed"), { status: 503 });
          return { data: this.issues.get(issue_number) };
        },
        update: async ({ issue_number, ...changes }) => {
          if (changes.assignees) changes.assignees = changes.assignees.map(reviewer);
          Object.assign(this.issues.get(issue_number), changes);
          return { data: this.issues.get(issue_number) };
        },
        createComment: async ({ body }) => {
          const comment = { id: this.comments.length + 1, user: { login: config.botLogin }, body };
          this.comments.push(comment);
          return { data: comment };
        },
        updateComment: async ({ comment_id, body }) => {
          this.comments.find(({ id }) => id === comment_id).body = body;
          return { data: {} };
        },
      },
    };
  }

  async paginate(method) {
    if (method === "listComments") return this.comments;
    if (method === "listForRepo") return [...this.issues.values()];
    if (method === "listReviews") return [...this.reviews.values()];
    throw new Error(`Unexpected pagination method ${method}`);
  }

  async graphql() {
    return {
      repository: {
        pullRequest: { closingIssuesReferences: {
          nodes: this.closing, pageInfo: { hasNextPage: false, endCursor: null },
        } },
      },
    };
  }
}

class FakeProject {
  constructor() {
    this.client = { rest: { issues: { listComments: "listComments" } }, paginate: async () => [] };
    this.status = new Map();
    this.issueItems = new Map();
    this.syncs = [];
    this.sourceReviews = [];
    this.ensureIssueFailures = 0;
    this.syncFailures = 0;
    this.getItemFailures = 0;
  }

  async items() {
    return [sourceItem()];
  }

  async ensureIssue(issue) {
    if (this.ensureIssueFailures-- > 0) throw Object.assign(new Error("Project add failed"), { status: 503 });
    if (!this.issueItems.has(issue.node_id)) this.issueItems.set(issue.node_id, 201 + this.issueItems.size);
    const id = this.issueItems.get(issue.node_id);
    return { id, nodeId: `PVTI_${id}` };
  }

  async sync(item, source, status = null) {
    if (this.syncFailures-- > 0) throw Object.assign(new Error("Project sync failed"), { status: 503 });
    this.syncs.push({ item, source, status });
    if (status) this.status.set(item, status);
  }

  async setStatus(item, status) {
    this.status.set(item, status);
  }

  async getItem(item) {
    if (this.getItemFailures-- > 0) throw Object.assign(new Error("Project read failed"), { status: 503 });
    return { fields: [{ id: config.statusFieldId, value: { id: this.status.get(item) } }] };
  }

  async moveSourceToReady(_source, reviewedAt) {
    this.sourceReviews.push(reviewedAt);
  }

  async find(repository, number) {
    const source = sourceItem();
    return source.repository === repository && source.number === number ? source : null;
  }
}

class FakeHierarchy {
  constructor() {
    this.parents = new Map();
    this.attaches = [];
    this.detaches = [];
    this.parentFailures = 0;
    this.parentValidationFailures = 0;
    this.attachFailures = 0;
    this.attachPreflightFailures = 0;
    this.detachFailures = 0;
    this.detachPreparationFailures = 0;
    this.attachResponseLosses = 0;
    this.parentAfterDetach = null;
    this.parentReads = 0;
    this.onParentRead = null;
  }

  async parent(child) {
    this.parentReads += 1;
    if (this.parentFailures-- > 0)
      throw new ParentReadError(Object.assign(new Error("Parent read failed"), { status: 503 }));
    if (this.parentValidationFailures-- > 0) throw new Error("invalid live parent");
    this.onParentRead?.(this.parentReads);
    return this.parents.get(child.number) ?? null;
  }

  async attach(parent, child, replaceParent, authenticate) {
    if (this.attachPreflightFailures-- > 0)
      throw new AttachPreflightError(Object.assign(new Error("selected source was replaced"), { status: 404 }));
    child = await authenticate();
    if (this.attachFailures-- > 0) throw Object.assign(new Error("Parent add failed"), { status: 503 });
    if (this.parents.has(child.number) && !replaceParent) throw new Error("replace_parent was required");
    this.attaches.push({ parent, child, replaceParent });
    this.parents.set(child.number, parent);
    if (this.attachResponseLosses-- > 0) throw Object.assign(new Error("Parent add response was lost"), { status: 503 });
  }

  async detach(parent, child, authenticate) {
    if (this.detachPreparationFailures-- > 0)
      throw new ParentReadError(Object.assign(new Error("Live parent read failed"), { status: 503 }));
    child = await authenticate();
    if (this.detachFailures-- > 0) throw Object.assign(new Error("Parent removal failed"), { status: 503 });
    assert.equal(this.parents.get(child.number)?.issueId, parent.issueId);
    this.detaches.push({ parent, child });
    this.parents.delete(child.number);
    if (this.parentAfterDetach) {
      this.parents.set(child.number, this.parentAfterDetach);
      this.parentAfterDetach = null;
    }
  }
}

function sourceItem() {
  return {
    item: 10,
    itemNode: "PVTI_source",
    archived: false,
    generated: false,
    issueId: "I_source",
    repository: "agglayer/agglayer",
    number: 7,
    assignees: [{ login: "author", node_id: "U_author" }],
    fields: [],
    content: { body: "source" },
  };
}

function alternateSource() {
  return { ...sourceItem(), item: 11, itemNode: "PVTI_alternate", issueId: "I_alternate",
    repository: "agglayer/bridge", number: 8 };
}

function reviewer(login) {
  return { login, node_id: `U_${login}` };
}

function submitted(id, user, submittedAt) {
  return { id: Number(id), state: "APPROVED", submitted_at: submittedAt, user };
}

function legacyTaskMarker(pr, reviewerId) {
  const payload = Buffer.from(JSON.stringify({ v: 1, repositoryId: config.repositoryId, pr, reviewerId })).toString("base64url");
  return `<!-- review-tracker-task:${payload} -->`;
}

function signedTaskMarkerV1(pr, reviewerId) {
  const payload = Buffer.from(JSON.stringify({ v: 1, repositoryId: config.repositoryId, pr, reviewerId })).toString("base64url");
  const signature = createHmac("sha256", config.projectsToken).update(payload).digest("base64url");
  return `<!-- review-tracker-task:${payload}.${signature} -->`;
}

function direct(action, requestedReviewer = null, applyCurrent = true) {
  return {
    applyCurrent,
    eventName: "pull_request_target",
    repo: { owner: "agglayer", repo: "agglayer" },
    payload: {
      action,
      repository: { id: 775930816 },
      requested_reviewer: requestedReviewer,
    },
  };
}

function reviewEvent() {
  return {
    eventName: "workflow_run",
    repo: { owner: "agglayer", repo: "agglayer" },
    payload: { repository: { id: 775930816 } },
  };
}

function lifecycleEvent() {
  return {
    eventName: "workflow_run",
    repo: { owner: "agglayer", repo: "agglayer" },
    payload: { repository: { id: 775930816 } },
  };
}

let nextCommandId = 1_000;
function command(body, id = ++nextCommandId) {
  return {
    eventName: "issue_comment",
    repo: { owner: "agglayer", repo: "agglayer" },
    payload: { repository: { id: 775930816 }, comment: { id, body } },
  };
}
