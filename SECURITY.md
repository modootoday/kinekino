# Security policy

kinekino has no released version yet. Fixes land on `main`.

## Reporting a vulnerability

Report it privately through GitHub: **Security → Report a vulnerability** in this repository
(<https://github.com/modootoday/kinekino/security/advisories/new>). Do not open a public issue.

Include what is affected, how to reproduce it, and the commit you tested. You get an answer
in the advisory thread.

## Scope

- The player, the CLI, the plugin host and the plugin SDK in this repository.
- kinekino treats external URLs and web messages as untrusted and never runs a program or shell
  from browser data. A way around that is in scope.
