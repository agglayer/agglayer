"use strict";

const assert = require("node:assert/strict");
const test = require("node:test");
let COMMANDS;
let emptyState;
let escapeMarkdown;
let loadState;
let renderComment;
let taskMarker;
test.before(async () => ({
  COMMANDS, emptyState, escapeMarkdown, loadState, renderComment, taskMarker,
} = await import("../src/tracker.mjs")));

const config = {
  repositoryId: "775930816",
  repository: "agglayer/agglayer",
  botLogin: "github-actions[bot]",
  serverUrl: "https://github.com",
  projectsToken: "projects-secret",
};

test("canonical state round-trips identifiers and repeats every command", () => {
  const state = emptyState(config, 42);
  state.source = {
    via: "model",
    issueId: "I_source",
    item: 10,
    itemNode: "PVTI_source",
    repository: "agglayer/agglayer",
    number: 7,
  };
  state.tasks.U_alice = {
    login: "alice",
    issue: 99,
    item: 11,
    closedByPr: false,
  };
  state.reviews.push("123");
  config.runUrl = "https://github.com/run/1";
  const body = renderComment(config, state);
  const loaded = loadState(
    [{ id: 1, user: { login: config.botLogin }, body }],
    config,
    42,
  );

  assert.deepEqual(loaded.state, state);
  assert.match(body, /@alice: \[#99\]/);
  assert.match(body, /review-tracker-state:[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+/);
  for (const command of COMMANDS) assert.match(body, new RegExp(escape(command)));
  assert.doesNotMatch(body, /private issue title|private notes|model prompt/);
});

test("unsigned bot state resets visibly instead of trusting the marker", () => {
  const bad = `<!-- review-tracker-state:${Buffer.from('{"v":99}').toString("base64url")} -->`;
  const loaded = loadState(
    [{ id: 1, user: { login: config.botLogin }, body: bad }],
    config,
    42,
  );
  assert.deepEqual(loaded.state, emptyState(config, 42));
  assert.match(loaded.warnings[0], /State was reset/);
});

test("signed state rejects payload tampering", () => {
  const body = renderComment(config, emptyState(config, 42)).replace(
    /(review-tracker-state:)([A-Za-z0-9_-])/,
    (_, prefix, first) =>
    `${prefix}${first === "A" ? "B" : "A"}`,
  );
  const loaded = loadState([{ id: 1, user: { login: config.botLogin }, body }], config, 42);
  assert.deepEqual(loaded.state, emptyState(config, 42));
});

test("task markers contain only routing identifiers", () => {
  const marker = taskMarker(config, 42, "U_alice");
  assert.match(marker, /review-tracker-task/);
  assert.match(marker, /review-tracker-task:[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+/);
  assert.doesNotMatch(marker, /title|Notes|alice/);
});

test("diagnostics are flattened and escaped for Markdown", () => {
  assert.equal(escapeMarkdown("failure\n## forged heading"), "failure \\#\\# forged heading");
});

test("processed review state is bounded before signing", () => {
  const state = emptyState(config, 42);
  state.reviews = Array.from({ length: 2_001 }, (_, index) => String(index));
  assert.throws(() => renderComment(config, state), /processed-review list is too large/);
});

function escape(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
