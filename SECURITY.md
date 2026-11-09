# Security policy

## Supported versions

Only the latest 0.1.x release receives fixes.

## Reporting a vulnerability

Please do not open a public issue. Report it privately through GitHub:
[Report a vulnerability](https://github.com/robmeijerink/picqer-rs/security/advisories/new).

Include the version of the crate and, if you can, a few lines of code that reproduce the problem.
Do not send real API keys, webhook secrets or customer data.

## What is in scope

This crate handles API keys and webhook secrets, so these are worth reporting:

- a key or secret that ends up in a log, an error or `Debug` output;
- a flaw in the verification of webhook signatures;
- a request that goes to another host than intended, or one that follows a redirect;
- a value that passes validation but should not, or that changes silently on the way to the API;
- a panic or unbounded memory use caused by a crafted response.

## What is not in scope

- Problems in the Picqer API or platform itself. Report those to Picqer.
- Behaviour that is documented: errors can contain a part of a response body, which may hold
  personal data, and a response has no size limit. The copy of the API key inside a request header
  is held by the HTTP client and is not wiped from memory.
