"use strict";

const assert = require("node:assert/strict");
const test = require("node:test");
let Tracker;
let parseCommand;
test.before(async () => ({ Tracker, parseCommand } = await import("../src/tracker.mjs")));

const config = {
  owner: "agglayer",
  repo: "agglayer",
  repository: "agglayer/agglayer",
  repositoryId: "775930816",
  botLogin: "github-actions[bot]",
  serverUrl: "https://github.com",
  runUrl: "https://github.com/run/1",
  readyOptionId: "ready",
  inReviewOptionId: "in-review",
  statusFieldId: 1,
  projectsToken: "projects-secret",
};

test("opening creates one assigned issue per requested person", async () => {
  const fixture = createFixture([reviewer("alice"), reviewer("bob")]);
  const result = await run(fixture, direct("opened"));

  assert.equal(result.errors.length, 0);
  assert.equal(result.state.source.via, "closing");
  assert.deepEqual(
    [...fixture.github.issues.values()].map((issue) => issue.assignees),
    [["alice"], ["bob"]],
  );
  assert.deepEqual([...fixture.project.status.values()], ["ready", "ready"]);
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
  assert.deepEqual(fixture.github.permissionReads, ["author"]);

  result = await run(fixture, command("/review-tracker infer"));
  assert.deepEqual(result.state.source, { none: true, via: "model-none" });
  assert.deepEqual(fixture.github.permissionReads, ["author"]);
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
  assert.equal(fixture.github.reviewReads.get("501"), 1);
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
    assert.equal(fixture.github.reviewReads.get("501"), 1);
  }
});

test("the review cap is checked before fetching or mutating a review", async () => {
  const fixture = createFixture([]);
  const tracker = new Tracker({ github: fixture.github, project: fixture.project, config, core: fixture.core });
  tracker.state = { reviews: Array.from({ length: 2_000 }, (_, index) => String(index)) };
  await assert.rejects(tracker.processReview("new"), /processed-review list is too large/);
  assert.equal(fixture.github.reviewReads.size, 0);
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

test("a later closing reference replaces a model-none mapping", async () => {
  const fixture = createFixture([reviewer("alice")]);
  fixture.github.closing = [];
  fixture.project.items = async () => {
    const item = sourceItem();
    item.assignees = [];
    return [item];
  };
  let result = await run(fixture, direct("opened"));
  assert.equal(result.state.source.via, "model-none");
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
  assert.deepEqual(parseCommand("/review-tracker none"), { kind: "none" });
  assert.deepEqual(parseCommand("/review-tracker set agglayer/agglayer#7"), {
    kind: "set",
    repository: "agglayer/agglayer",
    number: 7,
  });
  assert.throws(() => parseCommand("please /review-tracker none"), /invalid/);
});

test("invalid trusted commands are reported in the canonical comment", async () => {
  const fixture = createFixture([]);
  const result = await run(fixture, command("/review-tracker typo"));
  assert.match(result.errors[0], /command: The review-tracker command is invalid/);
  assert.equal(fixture.github.comments.length, 1);
  assert.match(fixture.github.comments[0].body, /Tracking completed with errors/);
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
  return { github, project: new FakeProject(), core: { error() {} } };
}

class FakeGithub {
  constructor(requested) {
    this.comments = [];
    this.issues = new Map();
    this.reviews = new Map();
    this.reviewReads = new Map();
    this.permission = "write";
    this.permissionReads = [];
    this.closing = [{ id: "I_source" }];
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
        create: async (params) => {
          const number = 101 + this.issues.size;
          const issue = { id: 1001 + number, number, state: "open", ...params };
          this.issues.set(number, issue);
          return { data: issue };
        },
        get: async ({ issue_number }) => ({ data: this.issues.get(issue_number) }),
        update: async ({ issue_number, ...changes }) => {
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
    this.status = new Map();
    this.syncs = [];
    this.sourceReviews = [];
  }

  async items() {
    return [sourceItem()];
  }

  async addIssue() {
    return { id: 201 + this.status.size, nodeId: `PVTI_${201 + this.status.size}` };
  }

  async sync(item, source, status = null) {
    this.syncs.push({ item, source, status });
    if (status) this.status.set(item, status);
  }

  async setStatus(item, status) {
    this.status.set(item, status);
  }

  async getItem(item) { return { fields: [{ id: config.statusFieldId, value: { id: this.status.get(item) } }] }; }

  async moveSourceToReady(_source, reviewedAt) {
    this.sourceReviews.push(reviewedAt);
  }

  async find(repository, number) {
    const source = sourceItem();
    return source.repository === repository && source.number === number ? source : null;
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

function reviewer(login) {
  return { login, node_id: `U_${login}` };
}

function submitted(id, user, submittedAt) {
  return { id: Number(id), state: "APPROVED", submitted_at: submittedAt, user };
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
