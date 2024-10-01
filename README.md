# kubectl-check

Ask to proceed with current context and namespace before running unsafe kubectl command.

## Installation

- get binary from [release](https://github.com/beleap/kubectl-check/releases)
- `brew install beleap/tap/kubectl-check`

## Tips

- set alias (`alias k="kubectl check"`).
- unsafe command can be configured with `KUBECTL_CHECK_UNSAFE` environment variable  with comma delimited string.