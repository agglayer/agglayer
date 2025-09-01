# Contributing to Agglayer

**🎉 Thank you for considering contributing to Agglayer!**
We welcome contributions of all kinds, from code improvements to documentation fixes. This guide will help you understand how we collaborate and the conventions we follow.

-----

## 🚀 Getting Started

1. **Fork the repository** to your own GitHub account.
2. **Create a new branch** for your changes:
   ```bash
   git checkout -b my-username/my-feature
   ```
3. **Make your changes,** following our coding conventions displayed in the code around your changes.
4. **Write or update tests** if applicable.
5. **Open a Pull Request** when ready for feedback or review.

-----

## 🔍 Pull Request Guidelines

We follow these practices to keep contributions smooth for everyone:

### 1. Commits

- You need to **sign your commits with OpenPGP** for CI to pass.
- The PR title must **start with `docs:`, `chore:`, `fix:` or `feat:`** for the most frequently-used options.
  We follow [conventional commit guidelines](https://www.conventionalcommits.org/en/v1.0.0/).
- The PR description should **use the BREAKING-CHANGE and CONFIG-CHANGE keywords** as per the explanations in the PR template.

### 2. PR Reviews

- We **aim to acknowledge PRs within 1 business day ⏱️.**
- If we cannot review within that time, we will **leave a comment indicating the expected review timeline.**
- Reviews are collaborative: maintainers and contributors work together to make the PR merge-ready.

### 3. Force-Push Policy

- **Draft PRs:**
  You may **force-push freely** while your PR is in draft mode.
  This keeps your branch tidy before requesting review.

- **Ready-for-Review PRs:**
  Once you mark your PR as **ready for review, do not force-push anymore.**
  - Force-pushing after requesting review makes it harder for reviewers to track changes.
  - Instead, **add one commit per change** requested during review.
  - When you want to pull in new changes from `main` or `master`, **use a merge commit.**

- **Squash & Merge:**
  Even with multiple commits during review, the repository history stays clean because we **always squash-merge PRs.**

-----

## ✅ Tips for a Smooth Contribution

- Keep PRs **focused and small** when possible.
- Write **clear commit messages.**
- If your PR is large or experimental, **start with a draft PR** to get early feedback.
- Respect our **force-push policy** to make life easier for reviewers.

-----

## 📬 Communication

- Use GitHub comments for discussions about the PR itself.
- For bigger design discussions, consider opening an issue first before a PR.

-----

## 🏁 Merging

- PRs are merged via **Squash and Merge.**
- The final commit message will contain the title and description of the PR, hence these should be clean upon merging.

-----

With these guidelines, we aim to keep the review process **fast, clear, and contributor-friendly.**
Thank you for helping make Agglayer better! 💜