---
title: Committing
---

Harper follows [conventional commit practices](https://www.conventionalcommits.org/en/v1.0.0/).
Before creating a pull request, please make sure all your commits follow the linked conventions.

Additionally, to minimize the labor required to review your commit, we run a relatively strict suite of formatting and linting programs.
We highly recommend that you run both `just format` and `just precommit` before submitting a pull request.
If those scripts don't work in your environment, we run `just precommit` through GitHub Actions inside of pull requests, so you may make modifications and push until the checks pass.

If this sounds intimidating, don't worry.
We are entirely willing to work with you to make sure your code can make it into Harper, just know it might take a little longer.
