import * as core from "@actions/core";
import { context, getOctokit } from "@actions/github";
import Anthropic from "@anthropic-ai/sdk";
import { runAction } from "./tracker.mjs";
runAction({ core, context, getOctokit, Anthropic }).catch((error) => {
  core.error("The review tracker failed outside its normal error handler."); core.setFailed("The review tracker failed unexpectedly.");
});
