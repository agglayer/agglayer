"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");

const root = path.resolve(__dirname, "../../../..");

test("the lifecycle and submitted-review relay carries exact metadata but has no power", () => {
  const workflow = read(".github/workflows/pr-review-tracker-signal.yml");
  const processor = read(".github/workflows/pr-review-tracker-process.yml");
  assert.match(workflow, /^name: PR Review Tracker Signal$/m);
  assert.ok(workflow.includes('run-name: "Review tracker signal #${{ github.event.pull_request.number }} action ${{ github.event.action }} review ${{ github.event.review.id || 0 }}"'));
  assert.match(processor, /workflows: \[PR Review Tracker Signal\]/);
  assert.match(workflow, /pull_request_target:/);
  assert.match(workflow, /pull_request_review:/);
  assert.match(workflow, /pull_request\.number.*review\.id/);
  assert.match(workflow, /permissions: \{\}/);
  assert.doesNotMatch(workflow, /secrets\.|actions\/checkout|upload-artifact|download-artifact/);
});

test("commands and review signals are validated before either secret is exposed", () => {
  const workflow = read(".github/workflows/pr-review-tracker-process.yml");
  const source = read(".github/actions/review-tracker/src/tracker.mjs");
  for (const event of ["issue_comment:", "workflow_run:"]) assert.match(workflow, new RegExp(event));
  assert.doesNotMatch(workflow, /pull_request_target:/);
  for (const association of ["OWNER", "MEMBER", "COLLABORATOR"]) assert.match(workflow, new RegExp(association));
  assert.match(source, /getCollaboratorPermissionLevel/);
  assert.match(source, /getReview/);
  assert.match(source, /pull\.base\?\.repo\?\.id/);
  assert.doesNotMatch(source, /pull_requests/);
  assert.doesNotMatch(source, /submitted\.commit_id.*run\.head_sha/);
  assert.doesNotMatch(source, /!== run\.head_repository/);
  assert.match(source, /run\.head_branch/);
  assert.match(workflow, /group: review-tracker-project-47-pr-\$\{\{ needs\.resolve\.outputs\.pr \}\}/);
  assert.match(workflow, /cancel-in-progress: false/);
  assert.doesNotMatch(workflow, /queue:/);
  assert.match(workflow, /ref: \$\{\{ github\.sha \}\}/);
  assert.match(workflow, /secrets\.PROJECTS_TOKEN/);
  assert.match(workflow, /secrets\.CLAUDE_API_KEY/);
  assert.match(workflow, /process:[\s\S]*pull-requests: write/);
  assert.doesNotMatch(workflow.slice(0, workflow.indexOf("\n  process:")), /write/);
  assert.doesNotMatch(workflow.slice(0, workflow.indexOf("\n  process:")), /secrets\./);
  assert.doesNotMatch(workflow, /ANTHROPIC_API_KEY|workflow_dispatch|upload-artifact|download-artifact/);
});

test("Project configuration uses rename-stable IDs and explicitly names Notes", () => {
  const source = read(".github/actions/review-tracker/src/tracker.mjs");
  for (const value of [
    "projectNumber: 47", "statusFieldId: 369829350", "PVTSSF_lADOCeYRi84BdrR0zhYLJeY",
    "369832156, 369833082, 369835321, 369832813, 369832890",
    "estimateFieldId: 369829594", "notesFieldId: 374176592", "legacyTasks: \\[{ issue: 1747",
  ]) assert.match(source, new RegExp(value));
});

test("the local action is a bundled Node 24 action with exact official dependencies", () => {
  const action = read(".github/actions/review-tracker/action.yml");
  const manifest = JSON.parse(read(".github/actions/review-tracker/package.json"));
  assert.match(action, /using: node24/);
  assert.match(action, /main: dist\/index\.cjs/);
  assert.equal(manifest.dependencies["@actions/core"], "3.0.1");
  assert.equal(manifest.dependencies["@actions/github"], "9.1.1");
  assert.equal(manifest.dependencies["@anthropic-ai/sdk"], "0.115.0");
  assert.doesNotMatch(action, /npm install|actions\/github-script/);
});

test("every external runtime action is pinned to a commit SHA", () => {
  for (const file of [
    ".github/workflows/pr-review-tracker-process.yml",
    ".github/workflows/pr-review-tracker-signal.yml",
    ".github/workflows/pr-review-tracker-test.yml",
  ]) for (const line of read(file).split("\n")) {
    const match = /^\s*uses:\s*([^./][^@]*)@(\S+)/.exec(line);
    if (match) assert.match(match[2], /^[0-9a-f]{40}$/, `${file}: ${line.trim()}`);
  }
});

test("maintained production code remains at most 1,150 physical lines", () => {
  const files = [
    ...fs.readdirSync(path.join(root, ".github/actions/review-tracker/src"))
      .filter((file) => file.endsWith(".mjs")).map((file) => `.github/actions/review-tracker/src/${file}`),
    ".github/actions/review-tracker/action.yml",
    ".github/actions/review-tracker/package.json",
    ".github/workflows/pr-review-tracker-process.yml",
    ".github/workflows/pr-review-tracker-signal.yml",
  ];
  const lines = files.reduce((total, file) => total + read(file).split("\n").length - 1, 0);
  assert.ok(lines <= 1_150, `maintained production code grew to ${lines} lines`);
});

function read(relativePath) { return fs.readFileSync(path.join(root, relativePath), "utf8"); }
