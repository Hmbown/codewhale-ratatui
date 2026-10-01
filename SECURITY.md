# Security

## Reporting a vulnerability

Please do not file public issues for suspected vulnerabilities. Use GitHub's
private vulnerability reporting for this repository
(**Security → Report a vulnerability** on
[github.com/Hmbown/codewhale-ratatui](https://github.com/Hmbown/codewhale-ratatui)).
If that option is not available, open an issue that says only that you have a
security report and how to reach you, without details of the problem.

Include the crate version (or commit), Rust version, terminal and operating
system, and the steps to reproduce.

This crate is pre-1.0 and maintained on a best-effort basis. Reports are read
and acknowledged as time allows; there is no guaranteed response time or
bounty. Only the current `main` branch is supported.

## Scope

The crate paints text to a terminal and probes the terminal's background color
(OSC 11). It makes no network connections and reads no files at runtime.
Reports about terminal-escape handling, such as untrusted text reaching the
terminal unescaped through a component, are in scope.
