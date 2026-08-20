const API_VERSION = "2026-03-10";
const STATUS_QUERY = `query ReviewTrackerStatus($item: ID!) {
  node(id: $item) { ... on ProjectV2Item { fieldValues(first: 100) { nodes {
    ... on ProjectV2ItemFieldSingleSelectValue { optionId updatedAt field { ... on ProjectV2SingleSelectField { id } } } } } } } }`;
export class Project {
  constructor(client, config) {
    Object.assign(this, { client, config });
    this.headers = { "x-github-api-version": API_VERSION };
  }
  params(extra = {}) {
    return { org: this.config.projectOwner, project_number: this.config.projectNumber, headers: this.headers, ...extra };
  }
  async items(refresh = false) {
    if (!refresh && this.cachedItems) return this.cachedItems;
    const data = await this.client.paginate("GET /orgs/{org}/projectsV2/{project_number}/items", this.params({
      fields: this.config.contextFieldIds.join(","), per_page: 100,
    }));
    this.cachedItems = data.filter((item) => item.content_type === "Issue" && item.content).map(normalizeItem);
    return this.cachedItems;
  }
  async ensureIssue(issue) {
    const existing = (await this.items(true)).find((item) => item.issueId === issue.node_id);
    if (existing) return { id: existing.item, nodeId: existing.itemNode };
    try { return await this.addIssue(issue.id); }
    catch (error) {
      try {
        const raced = (await this.items(true)).find((item) => item.issueId === issue.node_id);
        if (raced) return { id: raced.item, nodeId: raced.itemNode };
      } catch { /* Preserve the original add failure. */ }
      throw error;
    }
  }
  async getItem(id) {
    const { data } = await this.client.request("GET /orgs/{org}/projectsV2/{project_number}/items/{item_id}", this.params({
      item_id: id, fields: this.config.contextFieldIds.join(","),
    }));
    return normalizeItem(data);
  }
  async addIssue(issueId) {
    const { data } = await this.client.request(
      "POST /orgs/{org}/projectsV2/{project_number}/items", this.params({ type: "Issue", id: issueId }),
    );
    const item = data?.value ?? data;
    if (!Number.isSafeInteger(item?.id) || !item.node_id) throw new Error("GitHub did not return the new Project item IDs.");
    delete this.cachedItems;
    return { id: item.id, nodeId: item.node_id };
  }
  async sync(itemId, source, status = null) {
    const sourceItem = source?.item ? await this.getItem(source.item) : null;
    const sourceFields = new Map((sourceItem?.fields ?? []).map((field) => [field.id, field]));
    const fields = this.config.copyFieldIds.map((id) => ({ id, value: sourceFields.get(id)?.value?.id ?? null }));
    fields.push({ id: this.config.estimateFieldId, value: 0 });
    if (status) fields.push({ id: this.config.statusFieldId, value: status });
    await this.update(itemId, fields);
  }
  setStatus(itemId, optionId) { return this.update(itemId, [{ id: this.config.statusFieldId, value: optionId }]); }
  async moveSourceToReady(source, reviewedAt) {
    const data = await this.client.graphql(STATUS_QUERY, { item: source.itemNode });
    const status = data.node?.fieldValues?.nodes?.find((value) => value?.field?.id === this.config.statusFieldNodeId);
    const changed = Date.parse(status?.updatedAt), reviewed = Date.parse(reviewedAt);
    if (![this.config.inReviewOptionId, this.config.blockedOptionId].includes(status?.optionId) ||
      !Number.isFinite(changed) || !Number.isFinite(reviewed) || changed > reviewed) return false;
    await this.setStatus(source.item, this.config.readyOptionId);
    return true;
  }
  update(itemId, fields) {
    return this.client.request("PATCH /orgs/{org}/projectsV2/{project_number}/items/{item_id}",
      this.params({ item_id: itemId, fields }));
  }
  async find(repository, number) {
    if (repository.split("/")[0]?.toLowerCase() !== this.config.projectOwner.toLowerCase()) return null;
    return (await this.items()).find((item) => !item.archived && !item.generated &&
      item.repository?.toLowerCase() === repository.toLowerCase() && item.number === number);
  }
}
export function normalizeItem(item) {
  const issue = item.content;
  return {
    item: item.id, itemNode: item.node_id, archived: Boolean(item.archived_at),
    generated: issue.user?.login === "github-actions[bot]" && String(issue.body ?? "").includes("<!-- review-tracker-task:"),
    issueId: issue.node_id, repository: issue.repository?.full_name, number: issue.number,
    assignees: issue.assignees ?? [], fields: item.fields ?? [], content: issue,
  };
}
