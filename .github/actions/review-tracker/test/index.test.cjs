"use strict";

const assert = require("node:assert/strict");
const test = require("node:test");

let resolveEvent;
let runAction;
test.before(async () => ({ resolveEvent, runAction } = await import("../src/tracker.mjs")));

test("missing setup credentials are reported on the PR", async () => {
  const comments = [];
  const failures = [];
  const github = {
    rest: { issues: { createComment: async ({ body }) => comments.push(body) } },
  };
  const result = await runAction({
    getOctokit: () => github,
    Anthropic: class {},
    context: {
      eventName: "pull_request_target",
      repo: { owner: "agglayer", repo: "agglayer" },
      payload: { repository: { id: 775930816 } },
    },
    core: {
      error() {}, setSecret() {}, setOutput() {},
      setFailed: (message) => failures.push(message),
    },
    inputs: {
      mode: "process", "github-token": "github-token", "projects-token": "",
      "claude-api-key": "", "pr-number": "9", "review-id": "",
    },
  });
  assert.equal(result.errors.length, 1);
  assert.match(comments[0], /Missing required projects-token/);
  assert.match(comments[0], /\/review-tracker reconcile/);
  assert.equal(failures.length, 1);
});

test("asynchronous resolver failures stay inside the diagnostic boundary", async () => {
  const failures = [];
  const result = await runAction({
    getOctokit: () => ({ rest: { repos: { getCollaboratorPermissionLevel: async () => {
      throw Object.assign(new Error("permission lookup failed"), { status: 503 });
    } } } }),
    Anthropic: class {},
    context: {
      eventName: "issue_comment",
      repo: { owner: "agglayer", repo: "agglayer" },
      payload: {
        comment: { body: "/review-tracker reconcile", user: { login: "maintainer" } },
        issue: { number: 9 },
      },
    },
    core: {
      getInput: () => "", error() {}, setOutput() {},
      setFailed: (message) => failures.push(message),
    },
    inputs: { mode: "resolve", "github-token": "github-token" },
  });
  assert.match(result.errors[0], /permission lookup failed.*HTTP 503/);
  assert.equal(failures.length, 1);
});

test("the command resolver admits only users with repository write access", async () => {
  const warnings = [];
  const fixture = (login, permission) => ({
    github: { rest: { repos: { getCollaboratorPermissionLevel: async () => ({ data: { permission } }) } } },
    context: {
      eventName: "issue_comment", repo: { owner: "agglayer", repo: "agglayer" },
      payload: { comment: { body: "/review-tracker infer", user: { login } }, issue: { number: 9 } },
    },
    core: { setFailed() {}, setOutput() {}, warning: (message) => warnings.push(message) },
  });
  assert.deepEqual(await resolveEvent(fixture("maintainer", "write")), { pr: 9, review: "", action: "" });
  assert.equal(await resolveEvent(fixture("reader", "read")), null);
  assert.match(warnings[0], /Ignoring review-tracker command from @reader/);
});

test("the powerless review relay resolves a fully bound fork review", async () => {
  const outputs = {};
  const fixture = reviewRelayFixture();
  const result = await resolveEvent({
    ...fixture,
    core: { setFailed() {}, setOutput: (name, value) => { outputs[name] = value; } },
  });
  assert.deepEqual(result, { pr: 9, review: "501", action: "submitted" });
  assert.deepEqual(outputs, { pr: "9", review: "501", action: "submitted" });
});

test("the powerless lifecycle relay resolves without relying on privileged event delivery", async () => {
  const outputs = {}, fixture = reviewRelayFixture();
  Object.assign(fixture.context.payload.workflow_run, {
    event: "pull_request_target",
    display_title: "Review tracker signal #9 action review_requested review 0",
  });
  const result = await resolveEvent({
    ...fixture,
    core: { setFailed() {}, setOutput: (name, value) => { outputs[name] = value; } },
  });
  assert.deepEqual(result, { pr: 9, review: "", action: "review_requested" });
  assert.deepEqual(outputs, { pr: "9", review: "", action: "review_requested" });
});

test("the powerless review relay rejects every repository, PR, and review mismatch", async () => {
  for (const mutate of [
    (fixture) => { fixture.context.payload.workflow_run.path = ".github/workflows/other.yml"; },
    (fixture) => { fixture.context.payload.workflow_run.event = "push"; },
    (fixture) => { fixture.context.payload.workflow_run.display_title = "renamed signal 9 501"; },
    (fixture) => { fixture.context.payload.workflow_run.repository.id = 999; },
    (fixture) => { fixture.context.payload.workflow_run.head_branch = "other-branch"; },
    (fixture) => { fixture.pull.base.repo.id = 999; },
    (fixture) => { fixture.pull.head.ref = "other-branch"; },
    (fixture) => { fixture.submitted.id = 999; },
    (fixture) => { fixture.submitted.submitted_at = null; },
    (fixture) => { fixture.submitted.state = "PENDING"; },
  ]) {
    const fixture = reviewRelayFixture();
    mutate(fixture);
    const failures = [];
    const outputs = {};
    const result = await resolveEvent({
      ...fixture,
      core: {
        setOutput: (name, value) => { outputs[name] = value; },
        setFailed: (message) => failures.push(message),
      },
    });
    assert.equal(result, null);
    assert.equal(failures.length, 1);
    assert.deepEqual(outputs, {});
  }
});

test("the Anthropic client refuses redirects", async () => {
  let options;
  const comments = [];
  const result = await runAction({
    getOctokit: () => ({ rest: { issues: { createComment: async ({ body }) => comments.push(body) } } }),
    Anthropic: class { constructor(value) { options = value; throw new Error("constructor stop"); } },
    context: {
      repo: { owner: "agglayer", repo: "agglayer" },
      payload: { repository: { id: 775930816 } },
    },
    core: { error() {}, setSecret() {}, setFailed() {} },
    inputs: {
      mode: "process", "github-token": "github-token", "projects-token": "projects-token",
      "claude-api-key": "claude-key", "pr-number": "9", "review-id": "",
    },
  });
  assert.deepEqual(options.fetchOptions, { redirect: "error" });
  assert.equal(options.baseURL, "https://api.anthropic.com");
  assert.equal(result.errors.length, 1);
  assert.match(comments[0], /constructor stop/);
});

test("failure to post the emergency comment retains safe GitHub diagnostics", async () => {
  const errors = [];
  await runAction({
    getOctokit: () => ({ rest: { issues: { createComment: async () => {
      throw { status: 403, response: { headers: { "x-github-request-id": "REQ_safe" } } };
    } } } }),
    Anthropic: class { constructor() { throw new Error("constructor stop"); } },
    context: { repo: { owner: "agglayer", repo: "agglayer" }, payload: { repository: { id: 775930816 } } },
    core: { error: (message) => errors.push(message), setSecret() {}, setFailed() {} },
    inputs: {
      mode: "process", "github-token": "github-token", "projects-token": "projects-token",
      "claude-api-key": "claude-key", "pr-number": "9", "review-id": "",
    },
  });
  assert.match(errors.at(-1), /Could not report.*HTTP 403.*Request REQ_safe/);
});

function reviewRelayFixture() {
  const pull = {
    base: { repo: { id: 775930816 } },
    head: { repo: { id: 1234 }, ref: "feature" },
  };
  const submitted = {
    id: 501, submitted_at: "2026-01-01T00:00:00Z", state: "APPROVED",
  };
  return {
    pull,
    submitted,
    github: { rest: { pulls: {
      get: async () => ({ data: pull }),
      getReview: async () => ({ data: submitted }),
    } } },
    context: {
      eventName: "workflow_run",
      repo: { owner: "agglayer", repo: "agglayer" },
      payload: {
        repository: { id: 775930816 },
        workflow_run: {
          path: ".github/workflows/pr-review-tracker-signal.yml@refs/heads/main",
          event: "pull_request_review",
          repository: { id: 775930816 },
          display_title: "Review tracker signal #9 action submitted review 501",
          head_repository: { id: 775930816 },
          head_branch: "feature",
        },
      },
    },
  };
}
