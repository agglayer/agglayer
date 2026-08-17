"use strict";

const assert = require("node:assert/strict");
const test = require("node:test");
let Project;
let normalizeItem;
test.before(async () => ({ Project, normalizeItem } = await import("../src/project.mjs")));

const config = {
  projectOwner: "agglayer",
  projectNumber: 47,
  statusFieldId: 1,
  statusFieldNodeId: "F_status",
  estimateFieldId: 2,
  notesFieldId: 3,
  copyFieldIds: [4, 5, 6, 7, 8],
  contextFieldIds: [1, 4, 5, 6, 7, 8, 2, 3],
  readyOptionId: "ready",
  inReviewOptionId: "in-review",
  blockedOptionId: "blocked",
};

test("sync batches the five selected fields, Estimate, and Status without Notes", async () => {
  const client = fakeClient();
  client.item.fields = [
    field(4, "component"),
    field(5, "mission"),
    field(7, "risk"),
    { id: 3, name: "Notes", value: "private planning" },
  ];
  const project = new Project(client, config);
  await project.sync(22, { item: 11 }, config.readyOptionId);

  const patch = client.requests.find(([route]) => route.startsWith("PATCH"))[1];
  assert.deepEqual(patch.fields, [
    { id: 4, value: "component" },
    { id: 5, value: "mission" },
    { id: 6, value: null },
    { id: 7, value: "risk" },
    { id: 8, value: null },
    { id: 2, value: 0 },
    { id: 1, value: "ready" },
  ]);
  assert.equal(patch.fields.some(({ id }) => id === config.notesFieldId), false);
  assert.equal(patch.headers["x-github-api-version"], "2026-03-10");
});

test("no source clears copy fields while preserving the Notes omission", async () => {
  const client = fakeClient();
  const project = new Project(client, config);
  await project.sync(22, null);
  const fields = client.requests.at(-1)[1].fields;
  assert.deepEqual(fields.slice(0, 5), config.copyFieldIds.map((id) => ({ id, value: null })));
  assert.deepEqual(fields.at(-1), { id: config.estimateFieldId, value: 0 });
});

test("addIssue accepts the current direct REST response", async () => {
  const client = fakeClient();
  client.postResult = { id: 22, node_id: "PVTI_22" };
  const project = new Project(client, config);
  assert.deepEqual(await project.addIssue(1234), { id: 22, nodeId: "PVTI_22" });
  assert.deepEqual(client.requests.at(-1)[1].id, 1234);
});

test("a review moves only an older In Review or Blocked source status", async () => {
  for (const [optionId, updatedAt, expected] of [
    ["in-review", "2026-01-01T00:00:00Z", true],
    ["blocked", "2026-01-03T00:00:00Z", false],
    ["ready", "2026-01-01T00:00:00Z", false],
  ]) {
    const client = fakeClient();
    client.status = { optionId, updatedAt, field: { id: "F_status" } };
    const project = new Project(client, config);
    assert.equal(
      await project.moveSourceToReady(
        { item: 10, itemNode: "PVTI_10" },
        "2026-01-02T00:00:00Z",
      ),
      expected,
    );
    assert.equal(
      client.requests.some(([route]) => route.startsWith("PATCH")),
      expected,
    );
  }
});

test("normalization recognizes archived and generated items", () => {
  const item = normalizeItem({
    id: 1,
    node_id: "PVTI_1",
    archived_at: "2026-01-01T00:00:00Z",
    fields: [],
    content: {
      node_id: "I_1",
      number: 9,
      repository: { full_name: "agglayer/agglayer" },
      user: { login: "github-actions[bot]" },
      body: "<!-- review-tracker-task:abc -->",
    },
  });
  assert.equal(item.archived, true);
  assert.equal(item.generated, true);
});

test("a user-authored marker cannot hide a source issue", () => {
  const item = normalizeItem({ id: 1, node_id: "PVTI_1", fields: [], content: {
    node_id: "I_1", number: 9, repository: { full_name: "agglayer/agglayer" },
    user: { login: "author" }, body: "<!-- review-tracker-task:spoofed -->",
  } });
  assert.equal(item.generated, false);
});

function fakeClient() {
  return {
    requests: [],
    item: {
      id: 11,
      node_id: "PVTI_11",
      archived_at: null,
      fields: [],
      content: {
        node_id: "I_11",
        number: 11,
        repository: { full_name: "agglayer/agglayer" },
        body: "source",
        assignees: [],
      },
    },
    status: null,
    postResult: {},
    async paginate() {
      return [];
    },
    async request(route, params) {
      this.requests.push([route, params]);
      return {
        data: route.startsWith("GET")
          ? this.item
          : route.startsWith("POST")
            ? this.postResult
            : {},
      };
    },
    async graphql() {
      return { node: { fieldValues: { nodes: [this.status] } } };
    },
  };
}

function field(id, value) {
  return { id, value: { id: value } };
}
