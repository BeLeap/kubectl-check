# kubectl-check

A CLI tool to prompt users for confirmation before running potentially unsafe kubectl commands.

## Installation

- **Download the binary from releases**

  Head over to the [releases page](https://github.com/beleap/kubectl-check/releases) to download the binary.
- **Using homebrew**

  ```bash
  brew install beleap/tap/kubectl-check
  ```

## Tips

- **Set alias**

  ```bash
  alias k="kubectl check"
  ```
- **Configure unsafe commands**

  can be configured with `KUBECTL_CHECK_UNSAFE` environment variable  with comma delimited string.
  - Default unsafe commands can be found [here](https://github.com/BeLeap/kubectl-check/blob/main/src/main.rs#L63-L66).