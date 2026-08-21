import { createHmac, timingSafeEqual } from "node:crypto";
import { AttachPreflightError, Hierarchy, ParentReadError, repositoryUrl, sameIssue } from "./hierarchy.mjs";
import { Project } from "./project.mjs";
import { selectSource, sourceFrom } from "./source.mjs";
const DEPLOYMENT = {
  projectOwner: "agglayer", projectNumber: 47,
  statusFieldId: 369829350, statusFieldNodeId: "PVTSSF_lADOCeYRi84BdrR0zhYLJeY",
  readyOptionId: "61e4505c", inReviewOptionId: "df73e18b", blockedOptionId: "13f63909",
  copyFieldIds: [369832156, 369833082, 369835321, 369832813, 369832890],
  estimateFieldId: 369829594, notesFieldId: 374176592,
  legacyTasks: [{ issue: 1747, issueDatabaseId: 5169690048, issueNodeId: "I_kwDOLj_DwM8AAAABNCM1wA",
    pr: 1746, reviewerId: "MDQ6VXNlcjc1ODM3MTQ=" }],
  maxPromptBytes: 3_500_000 };
export const COMMANDS = ["/review-tracker set #123", "/review-tracker set REPOSITORY#123", "/review-tracker set OWNER/REPOSITORY#123", "/review-tracker none", "/review-tracker infer", "/review-tracker reconcile"];
const STATE_MARKER = "review-tracker-state";
const LIFECYCLE_ACTIONS = new Set([
  "opened", "edited", "synchronize", "review_requested", "review_request_removed", "closed", "reopened",
]);
export async function runAction({ core, context, getOctokit, Anthropic, inputs = null }) {
  const input = (name) => inputs?.[name] ?? core.getInput(name);
  let github;
  try {
    github = getOctokit(required(input("github-token"), "github-token"), { userAgent: "agglayer-review-tracker" });
    if (input("mode") === "resolve") return await resolveEvent({ github, context, core });
    if (input("mode") !== "process") throw new Error("mode must be resolve or process.");
  } catch (error) {
    const message = diagnostic(error, "review tracker", []);
    core.error(message); core.setFailed(message);
    return { errors: [message], warnings: [], state: null };
  }
  let pr = Number(input("pr-number")), projectsToken = "", apiKey = "", tracker;
  try {
    pr = positive(input("pr-number"), "pr-number");
    projectsToken = required(input("projects-token"), "projects-token");
    apiKey = input("claude-api-key");
    core.setSecret(projectsToken); if (apiKey) core.setSecret(apiKey);
    const config = makeConfig(context, projectsToken);
    const projectClient = getOctokit(projectsToken, { userAgent: "agglayer-review-tracker" });
    const anthropic = apiKey ? new Anthropic({
      apiKey, baseURL: "https://api.anthropic.com", timeout: 180_000, maxRetries: 2, fetchOptions: { redirect: "error" },
    }) : null;
    tracker = new Tracker({
      github, project: new Project(projectClient, config), hierarchy: new Hierarchy(projectClient, config), anthropic, apiKey, config, context, core,
      reviewId: input("review-id") || null, eventAction: input("event-action") || null,
    });
    const result = await tracker.run(pr);
    if (result.errors.length) core.setFailed(`Review tracking completed with ${result.errors.length} error(s).`);
    return result;
  } catch (error) {
    const message = diagnostic(error, "review tracker", [projectsToken, apiKey]);
    core.error(message);
    if (Number.isSafeInteger(pr) && pr > 0) await reportFatal(github, context.repo, pr, message, core);
    core.setFailed(message);
    return { errors: [message], warnings: [], state: tracker?.state ?? null };
  }
}
export async function resolveEvent({ github, context, core }) {
  let pr, review = "", action = "";
  if (context.eventName === "issue_comment") {
    if (!/^\/review-tracker(?:\s|$)/.test(String(context.payload.comment?.body ?? ""))) return null;
    const login = context.payload.comment?.user?.login;
    if (!login) return fail(core, "The command author is missing.");
    if (!await hasWriteAccess(github, context.repo, login)) {
      core.warning(`Ignoring review-tracker command from @${login}.`); return null;
    }
    pr = context.payload.issue.number;
  } else {
    const run = context.payload.workflow_run;
    const match = /^Review tracker signal #([1-9]\d*) action ([a-z_]+) review (0|[1-9]\d*)$/.exec(run?.display_title ?? "");
    if (
      String(run?.path ?? "").split("@")[0] !== ".github/workflows/pr-review-tracker-signal.yml" ||
      run?.repository?.id !== context.payload.repository?.id || !match
    ) return fail(core, "Review signal metadata failed validation.");
    pr = Number(match[1]); action = match[2]; review = match[3] === "0" ? "" : match[3];
    const lifecycle = run.event === "pull_request_target" && LIFECYCLE_ACTIONS.has(action) && !review;
    const submittedReview = run.event === "pull_request_review" && action === "submitted" && review;
    if (!lifecycle && !submittedReview) return fail(core, "Review signal event failed validation.");
    const { data: pull } = await github.rest.pulls.get({ ...context.repo, pull_number: pr });
    // GitHub reports the BASE repository as a run's head repository for pull_request_review events
    // on fork PRs, so comparing the two would reject every fork review; getReview is the real anchor.
    if (pull.base?.repo?.id !== context.payload.repository?.id || pull.head?.ref !== run.head_branch)
      return fail(core, "Review signal does not match the pull request.");
    if (submittedReview) {
      const { data: submitted } = await github.rest.pulls.getReview({
        ...context.repo, pull_number: pr, review_id: Number(review),
      });
      if (String(submitted.id) !== review || !submitted.submitted_at || submitted.state === "PENDING")
        return fail(core, "Review signal does not match the submitted review.");
    }
  }
  if (!Number.isSafeInteger(pr) || pr <= 0) return fail(core, "Invalid PR number.");
  core.setOutput("pr", String(pr)); core.setOutput("review", review); core.setOutput("action", action);
  return { pr, review, action };
}
export class Tracker {
  constructor(deps) { Object.assign(this, deps, { errors: [], warnings: [] }); }
  async run(prNumber) {
    const { data: pull } = await this.github.rest.pulls.get({ ...this.context.repo, pull_number: prNumber });
    if (String(pull.base?.repo?.id) !== this.config.repositoryId) throw new Error("The target PR does not belong to this repository.");
    this.pull = pull;
    this.comments = await this.github.paginate(this.github.rest.issues.listComments, {
      ...this.context.repo, issue_number: prNumber, per_page: 100 });
    const { state, commentId, warnings } = loadState(this.comments, this.config, prNumber);
    Object.assign(this, { state, commentId }); this.warnings.push(...warnings);
    const event = eventFrom(this.context, this.reviewId, this.eventAction);
    const lifecycleReady = event.kind !== "lifecycle" ||
      await this.capture("lifecycle state", () => this.refreshLifecycleState()) === true;
    let command = null;
    if (event.kind === "command") {
      const commandId = positiveDecimal(this.context.payload.comment?.id, "comment ID");
      if (BigInt(commandId) <= BigInt(this.state.lastCommand))
        this.warnings.push(`Ignored out-of-order command comment ${commandId}.`);
      else {
        this.state.lastCommand = commandId;
        command = await this.capture("command", () => parseCommand(this.context.payload.comment?.body, this.context.repo));
      }
    }
    let sourceChanged = false;
    if (command) sourceChanged = await this.capture("command", () => this.applyCommand(command)) === true;
    if (this.state.taskRecovery !== 1 || command?.kind === "reconcile")
      await this.capture("review task recovery", () => this.recoverTasks());
    const infer = command?.kind === "infer";
    const authorCanInfer = event.kind === "lifecycle" && event.action === "opened" && await this.capture("PR author permission",
      () => hasWriteAccess(this.github, this.context.repo, this.pull.user?.login)) === true;
    const refresh = event.kind === "lifecycle" && ["edited", "synchronize"].includes(event.action) &&
      !String(this.state.source?.via ?? "").startsWith("manual");
    if (lifecycleReady && (!this.state.source || infer || refresh) && this.pull.state !== "closed" &&
      (event.kind !== "command" || command)) {
      const selected = await this.capture("source selection", () => selectSource({
        github: this.github, project: this.project, anthropic: this.anthropic, config: this.config,
        pull: this.pull, pullComments: this.comments, allowModel: authorCanInfer || infer,
      }));
      if (selected) { this.state.source = selected; sourceChanged = true; }
    }
    await this.prepareHierarchy(sourceChanged, command?.kind === "reconcile");
    if (event.kind === "lifecycle" && lifecycleReady) {
      await this.capture("review reconciliation", () => this.reconcileReviews());
      await this.reconcileLifecycle();
    } else if (event.kind === "reviewed")
      await this.capture("review reconciliation", () => this.reconcileReviews(event.reviewId));
    else if (command?.kind === "reconcile") {
      const current = await this.capture("lifecycle state", () => this.refreshLifecycleState()) === true;
      await this.capture("review reconciliation", () => this.reconcileReviews());
      if (current) await this.reconcileLifecycle();
    }
    if (sourceChanged || command?.kind === "reconcile") await this.syncTasks();
    await this.syncHierarchies();
    await this.save();
    return { state: this.state, errors: this.errors, warnings: this.warnings };
  }
  async applyCommand(command) {
    if (command.kind === "set") {
      const item = await this.project.find(command.repository, command.number);
      if (!item) throw Object.assign(new Error("That issue is not an active, non-generated item in Project 47."), { stage: "command" });
      this.state.source = sourceFrom(item, "manual"); return true;
    }
    if (command.kind === "none") { this.state.source = { none: true, via: "manual-none" }; return true; }
    return false;
  }
  async ensureTask(reviewer) {
    if (!reviewer?.node_id || !reviewer.login) throw new Error("GitHub returned an invalid reviewer.");
    let task = this.state.tasks[reviewer.node_id];
    const issueData = { ...this.context.repo,
      title: `Review PR #${this.pull.number}: ${this.pull.title}`.slice(0, 256),
      assignees: [reviewer.login] };
    const updateIssue = async (issue) => (await this.github.rest.issues.update({ ...issueData,
      body: reviewIssueBody(this.config, this.pull, reviewer, issue), issue_number: issue.number, state: "open" })).data;
    let issue, created = false;
    if (!task) {
      issue = (await this.taskIssues()).get(reviewer.node_id)?.issue;
      if (issue) task = recoveredTask(issue, reviewer.login);
      else {
        ({ data: issue } = await this.github.rest.issues.create({ ...issueData,
          body: reviewIssueBody(this.config, this.pull, reviewer) }));
        task = { login: reviewer.login, issue: issue.number, pending: true, hierarchyPending: true,
          closedByPr: false, fulfilled: false };
        created = true;
      }
      this.state.tasks[reviewer.node_id] = task;
      if (created) {
        try {
          issue = await updateIssue(issue);
          this.cachedTaskIssues?.set(reviewer.node_id, { issue, legacy: false });
        } finally { await this.save(); }
      }
      else await this.save();
    }
    if (!created) issue = await updateIssue(issue ?? await this.getIssue(task.issue));
    Object.assign(task, { login: reviewer.login, closedByPr: false, fulfilled: false });
    await this.syncTask(task, issue, this.config.readyOptionId);
  }
  async taskIssues(allowLegacy = false) {
    if (this.cachedTaskIssues) return this.cachedTaskIssues;
    const issues = await this.github.paginate(this.github.rest.issues.listForRepo, {
      ...this.context.repo, state: "all", creator: this.config.botLogin, per_page: 100 });
    this.cachedTaskIssues = new Map();
    for (const issue of issues) {
      if (issue.pull_request || issue.user?.login !== this.config.botLogin ||
        !String(issue.body ?? "").includes("<!-- review-tracker-task:")) continue;
      let marker, legacy = false;
      try { marker = decodeMarker(issue.body, "review-tracker-task", this.config.projectsToken); }
      catch {
        if (!allowLegacy) continue;
        try { marker = decodeMarker(issue.body, "review-tracker-task", null); legacy = true; } catch { continue; }
      }
      if (legacy ? !matchesLegacyTask(this.config, this.pull.number, marker, issue)
        : !boundTaskMarker(marker, this.config, this.pull.number, marker?.reviewerId, issue)) continue;
      const current = this.cachedTaskIssues.get(marker.reviewerId);
      if (!current || (current.legacy && !legacy) || current.legacy === legacy && issue.number < current.issue.number)
        this.cachedTaskIssues.set(marker.reviewerId, { issue, legacy });
      if (current) this.warnings.push(`Multiple review task issues exist for reviewer ${marker.reviewerId}; the authenticated or oldest issue was used.`);
    }
    return this.cachedTaskIssues;
  }
  async recoverTasks() {
    let changed = false;
    for (const [reviewerId, entry] of await this.taskIssues(this.state.taskRecovery !== 1)) {
      let { issue } = entry;
      const assignee = issue.assignees?.find(({ node_id }) => node_id === reviewerId);
      if (!assignee?.login) { this.warnings.push(`Could not recover review task #${issue.number}: its reviewer is no longer assigned.`); continue; }
      if (!this.state.tasks[reviewerId]) {
        this.state.tasks[reviewerId] = recoveredTask(issue, assignee.login); changed = true; await this.save();
      }
      if (entry.legacy) {
        ({ data: issue } = await this.github.rest.issues.update({
          ...this.context.repo, issue_number: issue.number, body: reviewIssueBody(this.config, this.pull, assignee, issue),
        }));
        Object.assign(entry, { issue, legacy: false });
      }
    }
    this.state.taskRecovery = 1;
    if (changed) await this.save();
  }
  async recoverTask(reviewer) {
    const issue = (await this.taskIssues()).get(reviewer.node_id)?.issue;
    if (!issue) return null;
    const task = recoveredTask(issue, reviewer.login);
    this.state.tasks[reviewer.node_id] = task; await this.save();
    return task;
  }
  async syncTask(task, issue = null, status = null) {
    task.pending = true; issue ??= await this.getIssue(task.issue);
    if (!task.item) task.item = (await this.project.ensureIssue(issue)).id;
    await this.mutateTask(task, () => this.project.sync(task.item, this.state.source, status));
    delete task.pending;
  }
  async mutateTask(task, operation) {
    try { return await operation(); }
    catch (error) { task.pending = true; if (error.status === 404 && !error.sourceMissing) delete task.item; throw error; }
  }
  async refreshLifecycleState() {
    const [{ data: pull }, { data: requested }] = await Promise.all([
      this.github.rest.pulls.get({ ...this.context.repo, pull_number: this.pull.number }),
      this.github.rest.pulls.listRequestedReviewers({ ...this.context.repo, pull_number: this.pull.number }),
    ]);
    if (String(pull.base?.repo?.id) !== this.config.repositoryId)
      throw new Error("The refreshed PR does not belong to this repository.");
    this.pull = { ...pull, requested_reviewers: requested.users ?? [], requested_teams: requested.teams ?? [] };
    return true;
  }
  async reconcileLifecycle() {
    if (this.pull.state === "closed") return this.closeTasks();
    const requested = new Map((this.pull.requested_reviewers ?? []).map((reviewer) => [reviewer.node_id, reviewer]));
    if (this.pull.requested_teams?.length) this.warnings.push("Team review requests are not tracked.");
    for (const [reviewerId, reviewer] of requested) await this.capture(`reconcile review task for @${reviewer.login}`, async () => {
      const task = this.state.tasks[reviewerId];
      if (!task) return this.ensureTask(reviewer);
      task.login = reviewer.login;
      if (task.fulfilled) return this.ensureTask(reviewer);
      const issue = await this.getIssue(task.issue);
      if (issue.state === "open") {
        task.closedByPr = false; delete task.reopenStatus;
        if (task.pending || !task.item) await this.ensureTask(reviewer);
        return;
      }
      if (task.closedByPr) return this.reopenTask(task);
      await this.ensureTask(reviewer);
    });
    for (const [reviewerId, task] of Object.entries(this.state.tasks)) if (!requested.has(reviewerId)) {
      if (task.fulfilled) {
        if (task.closedByPr) await this.capture(`reopen review task #${task.issue}`, () => this.reopenTask(task));
        continue;
      }
      await this.capture(`close unrequested review task #${task.issue}`, async () => {
        if ((await this.getIssue(task.issue)).state === "open") await this.setIssue(task.issue, "closed");
        task.closedByPr = false; delete task.reopenStatus;
      });
    }
  }
  async reconcileReviews(exactReviewId = null) {
    const reviews = await this.github.paginate(this.github.rest.pulls.listReviews, {
      ...this.context.repo, pull_number: this.pull.number, per_page: 100,
    });
    const ids = new Set(reviews.filter((review) => review.submitted_at && review.state !== "PENDING")
      .map((review) => String(review.id)));
    if (exactReviewId) ids.add(String(exactReviewId)); let complete = true;
    for (const id of ids) complete = await this.capture(`process review ${id}`, () => this.processReview(id)) === true && complete;
    if (complete) for (const task of Object.values(this.state.tasks)) delete task.replayAfter;
  }
  async processReview(reviewId) {
    const id = String(reviewId);
    const processed = this.state.reviews.includes(id);
    if (processed && !Object.values(this.state.tasks).some((task) => task.replayAfter)) return true;
    if (!processed && this.state.reviews.length >= 2_000) throw new Error("The processed-review list is too large.");
    const { data: review } = await this.github.rest.pulls.getReview({
      ...this.context.repo, pull_number: this.pull.number, review_id: Number(id),
    });
    if (String(review.id) !== id || !review.user?.node_id || !review.submitted_at || review.state === "PENDING") {
      throw new Error("GitHub returned an invalid submitted review.");
    }
    const task = this.state.tasks[review.user.node_id] ?? await this.recoverTask(review.user);
    const replay = task?.replayAfter && Date.parse(review.submitted_at) >= Date.parse(task.replayAfter);
    if (processed && !replay) return true;
    const issue = task && await this.getIssue(task.issue);
    const closedLaterByPr = task?.closedByPr && this.pull.closed_at &&
      Date.parse(review.submitted_at) <= Date.parse(this.pull.closed_at);
    if (task && (issue.state === "open" || closedLaterByPr)) {
      task.fulfilled = true; await this.save(); if (task.pending || !task.item) await this.syncTask(task, issue, this.config.readyOptionId);
      await this.mutateTask(task, () => this.project.setStatus(task.item, this.config.inReviewOptionId));
      if (closedLaterByPr) task.reopenStatus = this.config.inReviewOptionId;
      if (this.state.source?.item) await this.project.moveSourceToReady(this.state.source, review.submitted_at);
      else if (!this.state.source?.none) throw new Error("The source issue is unresolved; correct it and rerun this review workflow.");
    } else this.warnings.push(task
      ? `Review ${id} was submitted by @${review.user.login}, but task #${task.issue} is closed.`
      : `Review ${id} was submitted by @${review.user.login} without a recorded review task.`);
    if (!this.state.reviews.includes(id)) this.state.reviews.push(id);
    return true;
  }
  async closeTasks() {
    for (const task of Object.values(this.state.tasks)) await this.capture(`close review task #${task.issue}`, async () => {
      if ((await this.getIssue(task.issue)).state === "open") {
        delete task.reopenStatus;
        const item = task.item && await this.mutateTask(task, () => this.project.getItem(task.item));
        if (item) task.reopenStatus = item.fields.find((field) => field.id === this.config.statusFieldId)?.value?.id ?? null;
        task.closedByPr = true; await this.save(); await this.setIssue(task.issue, "closed"); }
    });
  }
  async reopenTask(task) {
    const { data: issue } = await this.setIssue(task.issue, "open");
    const status = task.reopenStatus ?? (task.fulfilled ? this.config.inReviewOptionId : this.config.readyOptionId);
    if (task.pending || !task.item) await this.syncTask(task, issue, status);
    else await this.mutateTask(task, () => this.project.setStatus(task.item, status));
    delete task.reopenStatus; task.closedByPr = false;
  }
  async syncTasks() {
    for (const task of Object.values(this.state.tasks)) await this.capture(`sync review task #${task.issue}`,
      () => this.syncTask(task));
  }
  async prepareHierarchy(sourceChanged, reconcile) {
    let changed = sourceChanged;
    for (const task of Object.values(this.state.tasks)) if (sourceChanged || reconcile ||
      typeof task.hierarchyPending !== "boolean") { task.hierarchyPending = true; changed = true; }
    if (changed) await this.save();
  }
  async syncHierarchies() {
    for (const [reviewerId, task] of Object.entries(this.state.tasks)) if (task.hierarchyPending)
      await this.capture(`sync parent for review task #${task.issue}`, () => this.syncHierarchy(reviewerId, task));
  }
  async syncHierarchy(reviewerId, task) {
    if (!this.state.source?.item && !this.state.source?.none) return;
    let child = await this.authenticatedChild(reviewerId, task);
    const target = this.state.source.item ? parentFrom(this.state.source, this.config) : null;
    let managed = task.managedParent ? parentFrom(task.managedParent, this.config) : null;
    let attempted = task.attemptedParent ? parentFrom(task.attemptedParent, this.config) : null;
    if (sameIssue(target, child)) throw new Error(`Review task #${task.issue} cannot be its own parent.`);
    const recovering = task.hierarchyInFlight === true || Boolean(attempted);
    if (!recovering) {
      task.hierarchyInFlight = true;
      try { await this.save(); } catch (error) { delete task.hierarchyInFlight; throw error; }
    }
    let current;
    try { current = await this.hierarchy.parent(child, [managed, attempted].filter(Boolean)); }
    catch (error) {
      if (!recovering && error instanceof ParentReadError) {
        delete task.hierarchyInFlight;
        try { await this.save(); } catch { /* A persisted fence remains the fail-closed fallback. */ }
      }
      throw error;
    }
    if (recovering) {
      delete task.managedParent; delete task.attemptedParent; delete task.hierarchyInFlight;
      if (current) {
        if (sameIssue(current, target)) {
          task.hierarchyPending = false;
          this.warnings.push(`Review task #${task.issue} has the desired parent after an interrupted sync; the relationship was left unmanaged.`);
          await this.save(); return;
        }
        await this.save();
        throw new Error(`Review task #${task.issue} has a parent after an interrupted sync; it was preserved and left unmanaged.`);
      }
      await this.save();
      return this.syncHierarchy(reviewerId, task);
    }
    if (sameIssue(current, target)) {
      if (sameIssue(current, managed) || sameIssue(current, attempted)) task.managedParent = target;
      else {
        delete task.managedParent;
        this.warnings.push(`Review task #${task.issue} already has the desired parent; the existing relationship was left unmanaged.`);
      }
      delete task.attemptedParent;
      task.hierarchyPending = false;
      delete task.hierarchyInFlight;
      await this.save();
      return;
    }
    if (current && !sameIssue(current, managed) && !sameIssue(current, attempted)) {
      delete task.managedParent; delete task.attemptedParent; delete task.hierarchyInFlight; await this.save();
      throw new Error(`Review task #${task.issue} has an unrelated parent; it was preserved.`);
    }
    if (current && sameIssue(current, attempted)) {
      task.managedParent = attempted;
      delete task.attemptedParent; attempted = null;
      await this.save();
    }
    if (current) {
      try { await this.hierarchy.detach(current, child, () => this.authenticatedChild(reviewerId, task)); }
      catch (error) {
        if (error instanceof ParentReadError) {
          delete task.hierarchyInFlight;
          try { await this.save(); } catch { /* A persisted fence remains the fail-closed fallback. */ }
        }
        throw error;
      }
      current = await this.hierarchy.parent(child);
      if (current) {
        if (sameIssue(current, target)) {
          delete task.managedParent;
          this.warnings.push(`Review task #${task.issue} acquired the desired parent concurrently; the relationship was left unmanaged.`);
          delete task.attemptedParent;
          task.hierarchyPending = false;
          delete task.hierarchyInFlight;
          await this.save();
          return;
        }
        delete task.managedParent; delete task.attemptedParent; delete task.hierarchyInFlight; await this.save();
        throw new Error(`Review task #${task.issue} acquired an unrelated parent; it was preserved.`);
      }
    }
    if (target && !current) {
      if (!sameIssue(attempted, target)) { task.attemptedParent = target; await this.save(); }
      try { await this.hierarchy.attach(target, child, false, () => this.authenticatedChild(reviewerId, task)); }
      catch (error) {
        if (error instanceof AttachPreflightError) {
          delete task.managedParent; delete task.attemptedParent; delete task.hierarchyInFlight; await this.save();
        }
        throw error;
      }
    }
    if (target) task.managedParent = target; else delete task.managedParent;
    delete task.attemptedParent;
    task.hierarchyPending = false;
    delete task.hierarchyInFlight;
    await this.save();
  }
  async authenticatedChild(reviewerId, task) {
    let issue, marker;
    try { issue = await this.getIssue(task.issue); }
    catch (error) { throw new ParentReadError(error); }
    try { marker = decodeMarker(issue?.body, "review-tracker-task", this.config.projectsToken); }
    catch {
      // The signed PR state already binds this exact issue and reviewer, so an unsigned
      // production v0/v1 marker can be upgraded without trusting the marker itself.
      try { marker = decodeMarker(issue?.body, "review-tracker-task", null); } catch { /* Validate below. */ }
    }
    if ([0, 1].includes(marker?.v) && baseTaskMarker(marker, this.config, this.pull.number, reviewerId) &&
      issue?.number === task.issue && issue?.user?.login === this.config.botLogin && !issue.pull_request) {
      try {
        ({ data: issue } = await this.github.rest.issues.update({ ...this.context.repo, issue_number: task.issue,
          body: reviewIssueBody(this.config, this.pull, { login: task.login, node_id: reviewerId }, issue) }));
      } catch (error) { throw new ParentReadError(error); }
    }
    return authenticatedChild(this.config, this.pull, reviewerId, task, issue);
  }
  async getIssue(number) { return (await this.github.rest.issues.get({ ...this.context.repo, issue_number: number })).data; }
  setIssue(number, state) { return this.github.rest.issues.update({ ...this.context.repo, issue_number: number, state }); }
  async save() {
    const body = renderComment(this.config, this.state, this.errors, this.warnings);
    if (Buffer.byteLength(body) > 65_000) throw new Error("The tracker comment is too large.");
    if (this.commentId) return this.github.rest.issues.updateComment({ ...this.context.repo, comment_id: this.commentId, body });
    const { data } = await this.github.rest.issues.createComment({ ...this.context.repo, issue_number: this.pull.number, body });
    this.commentId = data.id; }
  async capture(stage, operation) {
    try { return await operation(); }
    catch (error) {
      const message = diagnostic(error, stage, [this.apiKey, this.config.projectsToken]);
      this.errors.push(message); this.core.error(message);
    }
  }
}
function makeConfig(context, projectsToken) {
  const repositoryId = String(context.payload.repository?.id ?? "");
  if (!/^\d+$/.test(repositoryId)) throw new Error("The repository ID is missing.");
  const { owner, repo } = context.repo, serverUrl = context.serverUrl ?? "https://github.com";
  return {
    ...DEPLOYMENT, owner, repo, repository: `${owner}/${repo}`, repositoryId, projectsToken, serverUrl,
    botLogin: "github-actions[bot]", runUrl: `${serverUrl}/${owner}/${repo}/actions/runs/${process.env.GITHUB_RUN_ID}`,
    contextFieldIds: [DEPLOYMENT.statusFieldId, ...DEPLOYMENT.copyFieldIds,
      DEPLOYMENT.estimateFieldId, DEPLOYMENT.notesFieldId],
  };
}
export function emptyState(config, pr) {
  return { v: 1, repositoryId: config.repositoryId, pr, source: null, tasks: {}, reviews: [], lastCommand: "0", taskRecovery: 1 };
}
export function loadState(comments, config, pr) {
  const matches = comments.filter((comment) => comment.user?.login === config.botLogin &&
    String(comment.body ?? "").includes(`<!-- ${STATE_MARKER}:`));
  if (!matches.length) return { commentId: null, state: emptyState(config, pr), warnings: [] };
  const warnings = matches.length > 1 ? ["Multiple tracker comments exist; the oldest one was used."] : [];
  try {
    const state = decodeMarker(matches[0].body, STATE_MARKER, config.projectsToken);
    if (state?.v !== 1 || state.repositoryId !== config.repositoryId || state.pr !== pr || !state.tasks ||
      !Array.isArray(state.reviews) || (state.lastCommand !== undefined && !/^(?:0|[1-9]\d*)$/.test(state.lastCommand)) ||
      (state.taskRecovery !== undefined && state.taskRecovery !== 1))
      throw new Error("The hidden tracker state has an invalid binding or shape.");
    state.lastCommand ??= "0";
    return { commentId: matches[0].id, state, warnings };
  } catch (error) {
    warnings.push(`${error.message} State was reset; manual repair may be required.`);
    return { commentId: matches[0].id, state: emptyState(config, pr), warnings };
  }
}
export function renderComment(config, state, errors = [], warnings = []) {
  if (state.reviews.length > 2_000) throw new Error("The processed-review list is too large.");
  const source = state.source?.item
    ? `[${escapeMarkdown(state.source.repository)}#${state.source.number}](${config.serverUrl}/${state.source.repository}/issues/${state.source.number}) via ${escapeMarkdown(state.source.via)}`
    : state.source?.none ? `None (${escapeMarkdown(state.source.via)})` : "Unresolved";
  const tasks = Object.values(state.tasks).sort((a, b) => a.login.localeCompare(b.login)).map((task) =>
    `- @${escapeMarkdown(task.login)}: [#${task.issue}](${config.serverUrl}/${config.repository}/issues/${task.issue})${task.pending || !task.item ? " (Project sync pending)" : ""}${task.hierarchyPending ? " (parent sync pending)" : ""}`);
  const lines = [
    "## PR review tracker", "", errors.length
      ? `Tracking completed with errors: [workflow run](${config.runUrl}).`
      : `Tracking completed successfully: [workflow run](${config.runUrl}).`,
    "", `**Detected source issue:** ${source}`, "", "Please correct this mapping with a trusted command if it is wrong.",
    "", "**Review issues:**", ...(tasks.length ? tasks : ["- None yet."]),
  ];
  append(lines, "Warnings", warnings); append(lines, "Errors", errors);
  lines.push("", "**Trusted maintainer commands:**", "", "```text", ...COMMANDS, "```", "",
    "Only users with repository write permission can run these commands.", encodeMarker(STATE_MARKER, state, config.projectsToken));
  return lines.join("\n");
}
function append(lines, title, values) {
  if (values.length) lines.push("", `**${title}:**`, ...values.slice(0, 20).map((value) => `- ${escapeMarkdown(value)}`));
}
function encodeMarker(name, value, key = "") {
  const payload = Buffer.from(JSON.stringify(value)).toString("base64url");
  const signature = key ? `.${createHmac("sha256", key).update(payload).digest("base64url")}` : "";
  return `<!-- ${name}:${payload}${signature} -->`;
}
function decodeMarker(body, name, key) {
  const match = String(body ?? "").match(new RegExp(`<!-- ${name}:([A-Za-z0-9_-]+)(?:\\.([A-Za-z0-9_-]+))? -->`));
  if (!match) throw new Error(`The ${name} marker is missing.`);
  if (key === null && match[2]) throw new Error(`The ${name} marker is not legacy.`);
  if (key !== null) {
    const expected = Buffer.from(createHmac("sha256", key).update(match[1]).digest("base64url"));
    const actual = Buffer.from(match[2] ?? "");
    if (actual.length !== expected.length || !timingSafeEqual(actual, expected)) throw new Error(`The ${name} signature is invalid.`);
  }
  return JSON.parse(Buffer.from(match[1], "base64url").toString("utf8"));
}
export function taskMarker(config, pr, reviewerId, issue) {
  if (!Number.isSafeInteger(issue?.id) || issue.id <= 0 || !Number.isSafeInteger(issue?.number) || issue.number <= 0 ||
    typeof issue?.node_id !== "string" || !issue.node_id) throw new Error("The review task identity is invalid.");
  return encodeMarker("review-tracker-task", { v: 2, repositoryId: config.repositoryId, pr, reviewerId,
    issueDatabaseId: issue.id, issueNodeId: issue.node_id, issue: issue.number }, config.projectsToken);
}
function recoveredTask(issue, login) {
  const task = { login, issue: issue.number, pending: true, hierarchyPending: true, closedByPr: false, fulfilled: false };
  if (issue.created_at) task.replayAfter = issue.created_at;
  return task;
}
function authenticatedChild(config, pull, reviewerId, task, issue) {
  let marker, repository;
  try { marker = decodeMarker(issue?.body, "review-tracker-task", config.projectsToken); } catch { /* Report one stable error. */ }
  try { repository = repositoryUrl(issue?.repository_url); } catch { /* Report one stable error. */ }
  if (issue?.pull_request || issue?.user?.login !== config.botLogin || issue?.number !== task.issue ||
    repository?.toLowerCase() !== config.repository.toLowerCase() ||
    !boundTaskMarker(marker, config, pull.number, reviewerId, issue) || !Number.isSafeInteger(issue.id) || issue.id <= 0 ||
    typeof issue.node_id !== "string" || !issue.node_id || issue.node_id.length > 200)
    throw new Error(`Review task #${task.issue} is not an authenticated tracker-owned issue.`);
  return { id: issue.id, issueId: issue.node_id, repository: config.repository, number: issue.number };
}
function parentFrom(source, config) {
  const [owner, repo, extra] = String(source?.repository ?? "").split("/");
  if (extra || owner?.toLowerCase() !== config.projectOwner.toLowerCase() || !validSegment(repo) ||
    typeof source?.issueId !== "string" || !source.issueId || source.issueId.length > 200 ||
    !Number.isSafeInteger(source.number) || source.number <= 0)
    throw new Error("The review-task parent routing state is invalid.");
  return { issueId: source.issueId, repository: `${owner}/${repo}`, number: source.number };
}
function reviewIssueBody(config, pull, reviewer, issue = null) {
  const marker = issue ? taskMarker(config, pull.number, reviewer.node_id, issue) :
    encodeMarker("review-tracker-task", { v: 0, repositoryId: config.repositoryId,
      pr: pull.number, reviewerId: reviewer.node_id }, config.projectsToken);
  return [`Review [${config.repository}#${pull.number}](${pull.html_url}).`, "", `Assigned reviewer: @${reviewer.login}.`, "",
    "This issue is managed by the PR review tracker.", marker].join("\n");
}
function baseTaskMarker(marker, config, pr, reviewerId) {
  return [0, 1, 2].includes(marker?.v) && marker.repositoryId === config.repositoryId && marker.pr === pr &&
    typeof reviewerId === "string" && reviewerId && marker.reviewerId === reviewerId;
}
function boundTaskMarker(marker, config, pr, reviewerId, issue) {
  return marker?.v === 2 && baseTaskMarker(marker, config, pr, reviewerId) &&
    marker.issueDatabaseId === issue?.id && marker.issueNodeId === issue?.node_id && marker.issue === issue?.number;
}
function matchesLegacyTask(config, pr, marker, issue) {
  const expected = (config.legacyTasks ?? []).find((task) => task.issue === issue?.number);
  return marker?.v === 1 && expected?.pr === pr && expected.reviewerId === marker.reviewerId &&
    expected.issueDatabaseId === issue?.id && expected.issueNodeId === issue?.node_id &&
    baseTaskMarker(marker, config, pr, expected.reviewerId);
}
export function parseCommand(body, current = {}) {
  const text = String(body ?? "");
  for (const kind of ["none", "infer", "reconcile"]) if (text === `/review-tracker ${kind}`) return { kind };
  const match = /^\/review-tracker set (?:(?:https:\/\/github\.com\/)?([\w.-]+\/[\w.-]+)(?:\/issues\/|#)|([\w.-]+)?#)([1-9]\d*)$/.exec(text);
  const explicit = match?.[1]?.split("/");
  const repository = explicit ? (explicit.every(validSegment) ? match[1] : null) :
    (match && validSegment(current.owner) && validSegment(match[2] ?? current.repo)
      ? `${current.owner}/${match[2] ?? current.repo}` : null), number = Number(match?.[3]);
  if (!repository || !Number.isSafeInteger(number)) throw new Error("The review-tracker command is invalid.");
  return { kind: "set", repository, number };
}
function eventFrom(context, reviewId, eventAction) {
  if (context.eventName === "workflow_run") return reviewId
    ? { kind: "reviewed", reviewId }
    : { kind: "lifecycle", action: eventAction };
  if (context.eventName === "issue_comment") return { kind: "command" };
  return { kind: "lifecycle", action: eventAction || context.payload.action };
}
export function diagnostic(error, fallbackStage, secrets) {
  let message = String(error?.message ?? error ?? "Unknown error.");
  for (const secret of secrets.filter(Boolean)) message = message.replaceAll(secret, "[secret]");
  const status = error?.status ? ` HTTP ${error.status}.` : "",
    request = error?.requestId && error.requestId !== "unknown" ? ` Request ${error.requestId}.` : "";
  return `${error?.stage ?? fallbackStage}: ${message}${status}${request}`;
}
export function escapeMarkdown(value) { return String(value).replace(/\s+/g, " ").replace(/([\\`*_[\]<>()#!|])/g, "\\$1"); }
function required(value, name) { if (!value) throw new Error(`Missing required ${name}.`); return value; }
async function hasWriteAccess(github, repo, login) {
  if (!login) return false;
  try { return ["admin", "maintain", "write"].includes((await github.rest.repos
    .getCollaboratorPermissionLevel({ ...repo, username: login })).data.permission); }
  catch (error) { if (error.status === 404) return false; throw error; } }
function positive(value, name) {
  const number = Number(value);
  if (!Number.isSafeInteger(number) || number <= 0) throw new Error(`${name} must be a positive integer.`);
  return number; }
function positiveDecimal(value, name) {
  const text = String(value ?? "");
  if (!/^[1-9]\d*$/.test(text)) throw new Error(`${name} must be a positive integer.`);
  return text;
}
function validSegment(value) { return typeof value === "string" && value.length <= 100 &&
  /^[\w.-]+$/.test(value) && value !== "." && value !== ".."; }
function fail(core, message) { core.setFailed(message); return null; }
async function reportFatal(github, repo, pr, message, core) {
  try {
    await github.rest.issues.createComment({ ...repo, issue_number: pr, body: [
      "## PR review tracker", "", `Tracking failed: ${escapeMarkdown(message)}`, "", "**Trusted maintainer commands:**",
      "", "```text", ...COMMANDS, "```", "<!-- review-tracker-emergency -->",
    ].join("\n") });
  } catch (error) {
    const status = Number.isSafeInteger(error?.status) ? ` HTTP ${error.status}.` : "", request = String(error?.response?.headers?.["x-github-request-id"] ?? error?.response?.headers?.get?.("x-github-request-id") ?? error?.requestId ?? "");
    core.error(`Could not report the failure on PR #${pr}.${status}${/^[A-Za-z0-9_-]{1,100}$/.test(request) ? ` Request ${request}.` : ""}`); }
}
