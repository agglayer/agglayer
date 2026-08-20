"use strict";

const assert = require("node:assert/strict");
const test = require("node:test");

let askModel;
let selectSource;
test.before(async () => ({ askModel, selectSource } = await import("../src/source.mjs")));

const config = {
  owner: "agglayer", repo: "agglayer", repository: "agglayer/agglayer",
  projectOwner: "agglayer", botLogin: "github-actions[bot]", maxPromptBytes: 3_500_000,
};

test("one validated closing relationship bypasses Claude", async () => {
  const item = candidate();
  const source = await selectSource({
    github: fakeGithub([{ id: item.issueId }]), project: { items: async () => [item] },
    config, pull: pull(), pullComments: [], anthropic: null, allowModel: true,
  });
  assert.equal(source.via, "closing");
  assert.equal(source.issueId, item.issueId);
});

test("closing relationships are paginated before deciding uniqueness", async () => {
  const first = candidate(), second = candidate({ issueId: "I_2", number: 12 });
  let calls = 0;
  const github = {
    graphql: async (_query, { cursor }) => {
      calls += 1;
      return { repository: { pullRequest: { closingIssuesReferences: cursor ? {
        nodes: [{ id: second.issueId }], pageInfo: { hasNextPage: false, endCursor: null },
      } : {
        nodes: [{ id: "not-in-project" }], pageInfo: { hasNextPage: true, endCursor: "page-2" },
      } } } };
    },
  };
  const source = await selectSource({
    github, project: { items: async () => [first, second] }, config,
    pull: pull(), pullComments: [], anthropic: null, allowModel: false,
  });
  assert.equal(calls, 2);
  assert.equal(source.issueId, second.issueId);
});

test("eligible closing relationships on different pages remain ambiguous", async () => {
  const first = candidate(), second = candidate({ issueId: "I_2", number: 12 });
  const github = {
    graphql: async (_query, { cursor }) => ({ repository: { pullRequest: { closingIssuesReferences: cursor ? {
      nodes: [{ id: second.issueId }], pageInfo: { hasNextPage: false, endCursor: null },
    } : {
      nodes: [{ id: first.issueId }], pageInfo: { hasNextPage: true, endCursor: "page-2" },
    } } } }),
  };
  const source = await selectSource({
    github, project: { items: async () => [first, second] }, config,
    pull: pull(), pullComments: [], anthropic: null, allowModel: false,
  });
  assert.equal(source, null);
});

test("Claude receives every assigned issue, Notes, discussion, and available code", async () => {
  let request;
  const anthropic = { messages: { create: async (body) => {
    request = body;
    return response({ issueId: "I_2" });
  } } };
  const first = candidate();
  const second = candidate({ issueId: "I_2", item: 12, itemNode: "PVTI_12", number: 12, state: "closed", status: "Done" });
  const project = {
    client: {
      rest: { issues: { listComments: "comments" } },
      paginate: async () => [{ body: "full issue conversation" }],
    },
    items: async () => [first, second],
  };
  const source = await selectSource({
    github: fakeGithub([]), project, anthropic, config, pull: pull(),
    pullComments: [
      { user: { login: "author" }, body: "full PR discussion" },
      { user: { login: config.botLogin }, body: "PRIVATE_TRACKER_STATE <!-- review-tracker-state:abc -->" },
      { user: { login: config.botLogin }, body: "PRIVATE_EMERGENCY <!-- review-tracker-emergency -->" },
    ],
    allowModel: true,
  });

  assert.equal(source.issueId, "I_2");
  assert.equal(request.model, "claude-sonnet-5");
  assert.deepEqual(request.thinking, { type: "adaptive" });
  assert.equal(request.output_config.effort, "medium");
  assert.equal(request.temperature, undefined);
  const prompt = request.messages[0].content;
  for (const content of ["I_1", "I_2", "private planning context", "full issue conversation", "full PR discussion", "diff --git a/old b/new"]) {
    assert.match(prompt, new RegExp(content.replaceAll("/", "\\/")));
  }
  assert.doesNotMatch(prompt, /PRIVATE_TRACKER_STATE|PRIVATE_EMERGENCY/);
});

test("no assigned candidates locks a null result without needing a key", async () => {
  const item = candidate();
  item.assignees = [{ login: "someone-else", node_id: "U_other" }];
  const source = await selectSource({
    github: fakeGithub([]), project: { items: async () => [item] }, config,
    pull: pull(), pullComments: [], anthropic: null, allowModel: true,
  });
  assert.deepEqual(source, { none: true, via: "model-none" });
});

test("Project items outside the configured owner never cause repository requests", async () => {
  const external = candidate(); external.repository = "outside/private";
  let conversations = 0;
  const source = await selectSource({
    github: fakeGithub([{ id: external.issueId }]), project: {
      items: async () => [external],
      client: { paginate: async () => { conversations += 1; return []; } },
    },
    config, pull: pull(), pullComments: [], anthropic: null, allowModel: true,
  });
  assert.deepEqual(source, { none: true, via: "model-none" });
  assert.equal(conversations, 0);
});

test("every assigned candidate is considered above the former count limit", async () => {
  const items = Array.from({ length: 30 }, (_, index) => candidate({
    issueId: `I_${index + 1}`,
    number: index + 1,
  }));
  let request, conversations = 0;
  const source = await selectSource({
    github: fakeGithub([]), project: {
      items: async () => items,
      client: {
        rest: { issues: { listComments: "comments" } },
        paginate: async () => { conversations += 1; return []; },
      },
    },
    config, pull: pull(), pullComments: [], allowModel: true,
    anthropic: { messages: { create: async (body) => {
      request = body; return response({ issueId: "I_30" });
    } } },
  });
  assert.equal(source.issueId, "I_30");
  assert.equal(conversations, 30);
  assert.deepEqual(request.output_config.format.schema.properties.issueId.anyOf,
    [{ type: "string" }, { type: "null" }]);
  assert.equal(request.output_config.format.schema.properties.issueId.anyOf[0].enum, undefined);
  assert.match(request.messages[0].content, /I_1/);
  assert.match(request.messages[0].content, /I_30/);
});

test("aggregate model context remains bounded after loading every candidate", async () => {
  let modelCalls = 0;
  await assert.rejects(selectSource({
    github: fakeGithub([]), project: {
      items: async () => [candidate()],
      client: {
        rest: { issues: { listComments: "comments" } },
        paginate: async () => [{ body: "conversation" }],
      },
    },
    config: { ...config, maxPromptBytes: 1 }, pull: pull(), pullComments: [], allowModel: true,
    anthropic: { messages: { create: async () => { modelCalls += 1; } } },
  }), /above the 1-byte limit/);
  assert.equal(modelCalls, 0);
});

test("structured output cannot select an issue outside the candidate set", async () => {
  const anthropic = { messages: { create: async () => response({ issueId: "I_private" }) } };
  await assert.rejects(askModel(anthropic, ["I_1"], "user"), /outside the candidate set/);
});

test("Anthropic errors expose status and request ID, not reflected content", async () => {
  const anthropic = { messages: { create: async () => {
    throw { status: 401, requestID: "req_safe", message: "PRIVATE_CONTEXT" };
  } } };
  let captured;
  await assert.rejects(askModel(anthropic, ["I_1"], "user"), (error) => {
    captured = error;
    return error.status === 401;
  });
  assert.equal(captured.requestId, "req_safe");
  assert.doesNotMatch(captured.message, /PRIVATE_CONTEXT/);
});

function fakeGithub(closing) {
  return {
    graphql: async () => ({ repository: { pullRequest: { closingIssuesReferences: {
      nodes: closing, pageInfo: { hasNextPage: false, endCursor: null },
    } } } }),
    request: async () => ({ data: "diff --git a/old b/new" }),
  };
}

function candidate(overrides = {}) {
  const issueId = overrides.issueId ?? "I_1";
  return {
    item: overrides.item ?? 11, itemNode: overrides.itemNode ?? "PVTI_11", archived: false, generated: false,
    issueId, repository: "agglayer/agglayer", number: overrides.number ?? 11,
    assignees: [{ login: "author", node_id: "U_author" }],
    fields: [
      { id: 1, name: "Status", value: { name: overrides.status ?? "Ready" } },
      { id: 3, name: "Notes", value: "private planning context" },
    ],
    content: { node_id: issueId, title: `issue ${issueId}`, body: "full issue body", state: overrides.state ?? "open" },
  };
}

function pull() {
  return { number: 9, title: "Implement source issue", body: "full PR body", user: { login: "author", node_id: "U_author" } };
}
function response(selection) {
  return { stop_reason: "end_turn", content: [{ type: "text", text: JSON.stringify(selection) }] };
}
