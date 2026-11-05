const HEADERS = Object.freeze({
  accept: "application/vnd.github+json",
  "x-github-api-version": "2026-03-10",
});
const GET_PARENT = "GET /repos/{owner}/{repo}/issues/{issue_number}/parent";
const GET_ISSUE = "GET /repos/{owner}/{repo}/issues/{issue_number}";
const ADD_SUB_ISSUE = "POST /repos/{owner}/{repo}/issues/{issue_number}/sub_issues";
const REMOVE_SUB_ISSUE = "DELETE /repos/{owner}/{repo}/issues/{issue_number}/sub_issue";

export class Hierarchy {
  constructor(client, config) { Object.assign(this, { client, config }); }
  async parent(child, knownParents = []) {
    const issue = currentChild(child, this.config);
    if (!Array.isArray(knownParents)) throw new Error("The known parent list is invalid.");
    const known = knownParents.map((parent) => ownedParent(parent, this.config));
    let response;
    try {
      response = await this.client.request(GET_PARENT, params(issue));
    } catch (error) {
      if (error?.status !== 404) throw new ParentReadError(error);
      return this.confirmMissingParent(issue, known);
    }
    const current = normalizeIssue(response.data);
    ownedParent(current, this.config);
    return current;
  }
  async confirmMissingParent(issue, known) {
    let visible;
    try {
      const { data } = await this.client.request(GET_ISSUE, params(issue));
      visible = normalizeIssue(data);
    } catch (error) { throw new ParentReadError(error); }
    if (!sameChild(visible, issue)) throw new ParentReadError(new Error("The visible child issue does not match."));
    for (const expected of known) {
      try {
        const { data } = await this.client.request(GET_ISSUE, params(expected));
        const current = normalizeIssue(data);
        ownedParent(current, this.config);
        if (!sameIssue(current, expected)) throw new Error("The visible parent issue does not match.");
      } catch (error) {
        throw new ParentReadError(Object.assign(new Error("The child has no visible parent, but a recorded " +
          "parent could not be verified; a trusted /review-tracker unmanage command relinquishes it."),
        { cause: error, status: error?.status }));
      }
    }
    return null;
  }
  async attach(parent, child, replaceParent, authenticate) {
    const target = ownedParent(parent, this.config), subIssue = currentChild(child, this.config);
    if (sameIssue(target, subIssue) || sameRoute(target, subIssue)) throw new Error("An issue cannot be its own parent.");
    if (replaceParent !== false) throw new Error("Replacing a parent directly is forbidden.");
    let live, confirmed;
    try {
      live = await this.liveParent(target); confirmed = await authenticatedChild(authenticate, subIssue, this.config);
      if (sameIssue(live, confirmed) || sameRoute(live, confirmed)) throw new Error("An issue cannot be its own parent.");
    } catch (error) { throw new AttachPreflightError(error); }
    try {
      await this.client.request(ADD_SUB_ISSUE, params(live, {
        sub_issue_id: confirmed.id, replace_parent: replaceParent,
      }));
    } catch (error) {
      if (error?.status !== 422) throw error;
      let current;
      try { current = await this.parent(confirmed); } catch { /* Preserve the original add failure. */ }
      if (!sameIssue(current, target)) throw error;
    }
  }
  async detach(parent, child, authenticate) {
    const target = ownedParent(parent, this.config), subIssue = currentChild(child, this.config);
    const live = await this.liveParent(target), confirmed = await authenticatedChild(authenticate, subIssue, this.config);
    await this.client.request(REMOVE_SUB_ISSUE, params(live, { sub_issue_id: confirmed.id }));
  }
  async liveParent(target) {
    let response;
    try { response = await this.client.request(GET_ISSUE, params(target)); }
    catch (error) { throw error?.status === 404 ? error : new ParentReadError(error); }
    const live = ownedParent(normalizeIssue(response.data), this.config);
    if (!sameIssue(live, target)) throw new Error("The live parent issue does not match the selected source.");
    return live;
  }
}
export class ParentReadError extends Error {
  constructor(error) {
    super(String(error?.message ?? error ?? "The parent lookup failed."), { cause: error });
    this.name = "ParentReadError"; if (error?.status) this.status = error.status;
    if (error?.requestId) this.requestId = error.requestId;
  }
}
export class AttachPreflightError extends ParentReadError {
  constructor(error) { super(error); this.name = "AttachPreflightError"; }
}
export function sameIssue(left, right) {
  return validNodeId(left?.issueId) && validNodeId(right?.issueId) && left.issueId === right.issueId;
}
function sameRoute(left, right) {
  return left.repository.toLowerCase() === right.repository.toLowerCase() && left.number === right.number;
}
function sameChild(left, right) {
  return left.id === right.id && sameIssue(left, right) && sameRoute(left, right);
}
async function authenticatedChild(authenticate, expected, config) {
  if (typeof authenticate !== "function") throw new Error("The child authentication callback is missing.");
  const current = currentChild(await authenticate(), config);
  if (!sameChild(current, expected)) throw new Error("The authenticated child issue changed before the parent mutation.");
  return current;
}
function currentChild(child, config) {
  const configured = splitRepository(config?.repository, "configured repository");
  if (configured[0].toLowerCase() !== segment(config?.projectOwner, "project owner").toLowerCase())
    throw new Error("The configured repository is outside the configured project owner.");
  const repository = splitRepository(child?.repository, "child repository");
  if (child.repository !== config.repository) throw new Error("The child issue is outside the configured repository.");
  return {
    id: positive(child.id, "child issue ID"), issueId: nodeId(child.issueId, "child node ID"),
    owner: repository[0], repo: repository[1], repository: child.repository, number: positive(child.number, "child issue number"),
  };
}
function ownedParent(parent, config) {
  const owner = segment(config?.projectOwner, "project owner"), repository = splitRepository(parent?.repository, "parent repository");
  if (repository[0].toLowerCase() !== owner.toLowerCase())
    throw new Error("The parent issue is outside the configured project owner.");
  return {
    issueId: nodeId(parent?.issueId, "parent node ID"), owner: repository[0], repo: repository[1],
    repository: repository.join("/"), number: positive(parent?.number, "parent issue number"),
  };
}
function normalizeIssue(issue) {
  return {
    id: positive(issue?.id, "returned issue ID"), issueId: nodeId(issue?.node_id, "returned issue node ID"),
    repository: repositoryUrl(issue?.repository_url), number: positive(issue?.number, "returned issue number"),
  };
}
export function repositoryUrl(value) {
  let url;
  try { url = new URL(value); } catch { /* Report one stable validation error below. */ }
  const match = /\/repos\/([^/]+)\/([^/]+)$/.exec(url?.pathname ?? "");
  if (!url || url.protocol !== "https:" || url.username || url.password || url.search || url.hash || !match)
    throw new Error("The returned issue repository URL is invalid.");
  return [segment(match[1], "returned issue owner"), segment(match[2], "returned issue repository")].join("/");
}
function params(issue, extra = {}) {
  return { owner: issue.owner, repo: issue.repo, issue_number: issue.number, headers: HEADERS, ...extra };
}
function splitRepository(value, name) {
  const parts = typeof value === "string" ? value.split("/") : [];
  if (parts.length !== 2) throw new Error(`The ${name} is invalid.`);
  return [segment(parts[0], `${name} owner`), segment(parts[1], `${name} name`)];
}
function segment(value, name) {
  if (typeof value !== "string" || value.length > 100 || !/^[A-Za-z0-9_.-]+$/.test(value) || value === "." || value === "..")
    throw new Error(`The ${name} is invalid.`);
  return value;
}
function positive(value, name) {
  if (!Number.isSafeInteger(value) || value <= 0) throw new Error(`The ${name} must be a positive safe integer.`);
  return value;
}
function nodeId(value, name) {
  if (!validNodeId(value)) throw new Error(`The ${name} is invalid.`);
  return value;
}
function validNodeId(value) {
  return typeof value === "string" && value.length > 0 && value.length <= 256 && !/[\s\x00-\x1f\x7f]/.test(value);
}
