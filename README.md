# klebs-general

Reusable Rust packages, procedural macros, and nested workspaces for batch
processing, workspace maintenance, dataflow, language modeling, geography,
and general utilities.

## Repository layout

This repository is a collection of packages and nested Cargo workspaces, not a
single repository-root Cargo workspace. There is no root `Cargo.toml`. Select the
manifest for the package or workspace being inspected; do not assume a command
run at the repository root resolves every project.

| Workspace | Declared members | Main areas |
| --- | ---: | --- |
| [aqua](aqua/Cargo.toml) | 2 | The aqua-resistor and aqua-3p members. |
| [batch-mode](batch-mode/Cargo.toml) | 30 | Batch clients, request construction, execution, reconciliation, schemas, token expansion, and text-to-speech. |
| [credit-card](credit-card/Cargo.toml) | 1 | The credit-card member and its workspace dependency declarations. |
| [hydro2](hydro2/Cargo.toml) | 10 | Operators, network wiring, asynchronous scheduling, mocks, and network performance. |
| [workspacer](workspacer/Cargo.toml) | 71 | Cargo manifest and source analysis, workspace maintenance, testing, publishing, dependency management, and command-line tools. |

The catalog below records directory paths and declared membership, not a claim
that every entry builds independently or that every test has passed. Directory
names need not equal Cargo package names: for example, `music/Cargo.toml` declares
`illtunes`. The retired `gpt-batch-scribe` directory is not a buildable package.

## Root directory catalog

All 81 root directories in the inspected Git tree are indexed here. Open a
directory for its manifest, source, documentation, or other retained material.
Nested workspace members are listed separately below.

| Directory | Directory |
| --- | --- |
| [`abbreviation-trait`](abbreviation-trait/) | [`africa`](africa/) |
| [`ai-descriptor`](ai-descriptor/) | [`ai-descriptor-derive`](ai-descriptor-derive/) |
| [`ai-descriptor-trait`](ai-descriptor-trait/) | [`ai-generate-readme`](ai-generate-readme/) |
| [`aqua`](aqua/) | [`asia`](asia/) |
| [`async-try-from`](async-try-from/) | [`australia-oceania-antarctica`](australia-oceania-antarctica/) |
| [`backoff-macro`](backoff-macro/) | [`basic-shorthand`](basic-shorthand/) |
| [`batch-mode`](batch-mode/) | [`birthday-struct`](birthday-struct/) |
| [`build-readme-generation-queries`](build-readme-generation-queries/) | [`central-america`](central-america/) |
| [`city-and-street`](city-and-street/) | [`count-invert`](count-invert/) |
| [`country`](country/) | [`crate-activity`](crate-activity/) |
| [`credit-card`](credit-card/) | [`disable-macro`](disable-macro/) |
| [`error-tree`](error-tree/) | [`europe`](europe/) |
| [`export-magic`](export-magic/) | [`file-downloader`](file-downloader/) |
| [`file-downloader-derive`](file-downloader-derive/) | [`find-matching-bracket`](find-matching-bracket/) |
| [`form-of-joke-humor`](form-of-joke-humor/) | [`form-of-wordplay`](form-of-wordplay/) |
| [`fs-shorthand`](fs-shorthand/) | [`gather-all-code-from-crates`](gather-all-code-from-crates/) |
| [`gpt-batch-executor`](gpt-batch-executor/) | [`gpt-batch-scribe`](gpt-batch-scribe/) |
| [`hydro2`](hydro2/) | [`io-shorthand`](io-shorthand/) |
| [`json-misunderstanding`](json-misunderstanding/) | [`json-repair`](json-repair/) |
| [`language-distribution`](language-distribution/) | [`language-enum`](language-enum/) |
| [`lyrical-meter`](lyrical-meter/) | [`month-and-season`](month-and-season/) |
| [`music`](music/) | [`name-and-title`](name-and-title/) |
| [`named-item`](named-item/) | [`named-item-derive`](named-item-derive/) |
| [`north-america`](north-america/) | [`ogp-license-text`](ogp-license-text/) |
| [`osmpbf-file-downloader-derive`](osmpbf-file-downloader-derive/) | [`osx-wallpaper-cycler`](osx-wallpaper-cycler/) |
| [`pbx`](pbx/) | [`pdf-to-images`](pdf-to-images/) |
| [`plural-derive`](plural-derive/) | [`plural-trait`](plural-trait/) |
| [`postal-code`](postal-code/) | [`rand-construct`](rand-construct/) |
| [`random-constructible`](random-constructible/) | [`random-constructible-derive`](random-constructible-derive/) |
| [`rck`](rck/) | [`regime`](regime/) |
| [`renew-traits`](renew-traits/) | [`resume-generator`](resume-generator/) |
| [`rhyme-type`](rhyme-type/) | [`scan-crate-for-typedefs`](scan-crate-for-typedefs/) |
| [`simple-address`](simple-address/) | [`south-america`](south-america/) |
| [`sppl-license-text`](sppl-license-text/) | [`static-or-heap-string`](static-or-heap-string/) |
| [`str-shorthand`](str-shorthand/) | [`structured-language-form`](structured-language-form/) |
| [`traced-test`](traced-test/) | [`tracing-setup`](tracing-setup/) |
| [`u`](u/) | [`unsafe-erased`](unsafe-erased/) |
| [`usa`](usa/) | [`variant-builder-macro`](variant-builder-macro/) |
| [`workspacer`](workspacer/) | [`world-region`](world-region/) |
| [`world-region-db`](world-region-db/) | [`world-region-repl`](world-region-repl/) |
| [`zodiac-sign`](zodiac-sign/) |  |

## Nested workspace members

These lists come from the respective workspace manifests' `members` arrays.
They are sorted for navigation; the manifests remain the source of membership.

<details>
<summary>aqua: 2 declared members</summary>

[Workspace manifest](aqua/Cargo.toml)

| Directory | Directory |
| --- | --- |
| [`aqua-3p`](aqua/aqua-3p/) | [`aqua-resistor`](aqua/aqua-resistor/) |

</details>

<details>
<summary>batch-mode: 30 declared members</summary>

[Workspace manifest](batch-mode/Cargo.toml)

| Directory | Directory |
| --- | --- |
| [`agent-coordinate`](batch-mode/agent-coordinate/) | [`ai-json-template`](batch-mode/ai-json-template/) |
| [`ai-json-template-derive`](batch-mode/ai-json-template-derive/) | [`batch-mode`](batch-mode/batch-mode/) |
| [`batch-mode-3p`](batch-mode/batch-mode-3p/) | [`batch-mode-batch-client`](batch-mode/batch-mode-batch-client/) |
| [`batch-mode-batch-error`](batch-mode/batch-mode-batch-error/) | [`batch-mode-batch-executor`](batch-mode/batch-mode-batch-executor/) |
| [`batch-mode-batch-index`](batch-mode/batch-mode-batch-index/) | [`batch-mode-batch-metadata`](batch-mode/batch-mode-batch-metadata/) |
| [`batch-mode-batch-reconciliation`](batch-mode/batch-mode-batch-reconciliation/) | [`batch-mode-batch-schema`](batch-mode/batch-mode-batch-schema/) |
| [`batch-mode-batch-scribe`](batch-mode/batch-mode-batch-scribe/) | [`batch-mode-batch-triple`](batch-mode/batch-mode-batch-triple/) |
| [`batch-mode-batch-workflow`](batch-mode/batch-mode-batch-workflow/) | [`batch-mode-batch-workspace`](batch-mode/batch-mode-batch-workspace/) |
| [`batch-mode-batch-workspace-interface`](batch-mode/batch-mode-batch-workspace-interface/) | [`batch-mode-json`](batch-mode/batch-mode-json/) |
| [`batch-mode-process-response`](batch-mode/batch-mode-process-response/) | [`batch-mode-token-expansion-axis-derive`](batch-mode/batch-mode-token-expansion-axis-derive/) |
| [`batch-mode-token-expansion-step`](batch-mode/batch-mode-token-expansion-step/) | [`batch-mode-token-expansion-traits`](batch-mode/batch-mode-token-expansion-traits/) |
| [`batch-mode-tts`](batch-mode/batch-mode-tts/) | [`camel-case-token-with-comment`](batch-mode/camel-case-token-with-comment/) |
| [`instructed-language-model-at-coordinate`](batch-mode/instructed-language-model-at-coordinate/) | [`language-model-batch-workflow-derive`](batch-mode/language-model-batch-workflow-derive/) |
| [`language-model-token-expander`](batch-mode/language-model-token-expander/) | [`language-model-type`](batch-mode/language-model-type/) |
| [`save-load-derive`](batch-mode/save-load-derive/) | [`save-load-traits`](batch-mode/save-load-traits/) |

</details>

<details>
<summary>credit-card: 1 declared members</summary>

[Workspace manifest](credit-card/Cargo.toml)

| Directory | Directory |
| --- | --- |
| [`credit-card`](credit-card/credit-card/) |  |

</details>

<details>
<summary>hydro2: 10 declared members</summary>

[Workspace manifest](hydro2/Cargo.toml)

| Directory | Directory |
| --- | --- |
| [`hydro2`](hydro2/hydro2/) | [`hydro2-3p`](hydro2/hydro2-3p/) |
| [`hydro2-async-scheduler`](hydro2/hydro2-async-scheduler/) | [`hydro2-basic-operators`](hydro2/hydro2-basic-operators/) |
| [`hydro2-mock`](hydro2/hydro2-mock/) | [`hydro2-network`](hydro2/hydro2-network/) |
| [`hydro2-network-performance`](hydro2/hydro2-network-performance/) | [`hydro2-network-wire-derive`](hydro2/hydro2-network-wire-derive/) |
| [`hydro2-operator`](hydro2/hydro2-operator/) | [`hydro2-operator-derive`](hydro2/hydro2-operator-derive/) |

</details>

<details>
<summary>workspacer: 71 declared members</summary>

[Workspace manifest](workspacer/Cargo.toml)

| Directory | Directory |
| --- | --- |
| [`generate-report-trait`](workspacer/generate-report-trait/) | [`get-file-size`](workspacer/get-file-size/) |
| [`lightweight-command-runner`](workspacer/lightweight-command-runner/) | [`sync-run-async`](workspacer/sync-run-async/) |
| [`workspacer`](workspacer/workspacer/) | [`workspacer-3p`](workspacer/workspacer-3p/) |
| [`workspacer-add-internal-dep`](workspacer/workspacer-add-internal-dep/) | [`workspacer-add-new-crate-to-workspace`](workspacer/workspacer-add-new-crate-to-workspace/) |
| [`workspacer-analysis`](workspacer/workspacer-analysis/) | [`workspacer-bump`](workspacer/workspacer-bump/) |
| [`workspacer-check-crates-io`](workspacer/workspacer-check-crates-io/) | [`workspacer-check-private`](workspacer/workspacer-check-private/) |
| [`workspacer-check-publish-ready`](workspacer/workspacer-check-publish-ready/) | [`workspacer-cleanup`](workspacer/workspacer-cleanup/) |
| [`workspacer-cli`](workspacer/workspacer-cli/) | [`workspacer-config`](workspacer/workspacer-config/) |
| [`workspacer-consolidate`](workspacer/workspacer-consolidate/) | [`workspacer-crate`](workspacer/workspacer-crate/) |
| [`workspacer-crate-interface`](workspacer/workspacer-crate-interface/) | [`workspacer-crate-mock`](workspacer/workspacer-crate-mock/) |
| [`workspacer-cratesio-mock`](workspacer/workspacer-cratesio-mock/) | [`workspacer-detect-circular-deps`](workspacer/workspacer-detect-circular-deps/) |
| [`workspacer-docs`](workspacer/workspacer-docs/) | [`workspacer-document`](workspacer/workspacer-document/) |
| [`workspacer-either`](workspacer/workspacer-either/) | [`workspacer-ensure-git-clean`](workspacer/workspacer-ensure-git-clean/) |
| [`workspacer-errors`](workspacer/workspacer-errors/) | [`workspacer-export`](workspacer/workspacer-export/) |
| [`workspacer-file-filter`](workspacer/workspacer-file-filter/) | [`workspacer-format-imports`](workspacer/workspacer-format-imports/) |
| [`workspacer-git`](workspacer/workspacer-git/) | [`workspacer-git-commit`](workspacer/workspacer-git-commit/) |
| [`workspacer-info`](workspacer/workspacer-info/) | [`workspacer-interface`](workspacer/workspacer-interface/) |
| [`workspacer-linting`](workspacer/workspacer-linting/) | [`workspacer-lock`](workspacer/workspacer-lock/) |
| [`workspacer-lossless-file`](workspacer/workspacer-lossless-file/) | [`workspacer-metadata`](workspacer/workspacer-metadata/) |
| [`workspacer-mock`](workspacer/workspacer-mock/) | [`workspacer-name-all-files`](workspacer/workspacer-name-all-files/) |
| [`workspacer-organize`](workspacer/workspacer-organize/) | [`workspacer-pin`](workspacer/workspacer-pin/) |
| [`workspacer-prune-bad-category-slugs`](workspacer/workspacer-prune-bad-category-slugs/) | [`workspacer-publish`](workspacer/workspacer-publish/) |
| [`workspacer-readme-writer`](workspacer/workspacer-readme-writer/) | [`workspacer-rebuild-or-test`](workspacer/workspacer-rebuild-or-test/) |
| [`workspacer-register`](workspacer/workspacer-register/) | [`workspacer-register-crate-files`](workspacer/workspacer-register-crate-files/) |
| [`workspacer-register-internal-crate-in-prefix-group`](workspacer/workspacer-register-internal-crate-in-prefix-group/) | [`workspacer-run`](workspacer/workspacer-run/) |
| [`workspacer-scan-for-prefix-groups`](workspacer/workspacer-scan-for-prefix-groups/) | [`workspacer-show`](workspacer/workspacer-show/) |
| [`workspacer-show-dependency-tree`](workspacer/workspacer-show-dependency-tree/) | [`workspacer-syntax`](workspacer/workspacer-syntax/) |
| [`workspacer-test-coverage`](workspacer/workspacer-test-coverage/) | [`workspacer-toml`](workspacer/workspacer-toml/) |
| [`workspacer-toml-interface`](workspacer/workspacer-toml-interface/) | [`workspacer-toml-mock`](workspacer/workspacer-toml-mock/) |
| [`workspacer-topo`](workspacer/workspacer-topo/) | [`workspacer-tree`](workspacer/workspacer-tree/) |
| [`workspacer-upgrade-function-tracing`](workspacer/workspacer-upgrade-function-tracing/) | [`workspacer-upgrade-test`](workspacer/workspacer-upgrade-test/) |
| [`workspacer-upgrade-test-suite-tracing`](workspacer/workspacer-upgrade-test-suite-tracing/) | [`workspacer-upgrade-test-suites`](workspacer/workspacer-upgrade-test-suites/) |
| [`workspacer-upgrade-test-tracing`](workspacer/workspacer-upgrade-test-tracing/) | [`workspacer-upgrade-tracing`](workspacer/workspacer-upgrade-tracing/) |
| [`workspacer-validate`](workspacer/workspacer-validate/) | [`workspacer-watch-and-reload`](workspacer/workspacer-watch-and-reload/) |
| [`workspacer-workspace`](workspacer/workspacer-workspace/) | [`workspacer-workspace-interface`](workspacer/workspacer-workspace-interface/) |
| [`workspacer-workspace-mock`](workspacer/workspacer-workspace-mock/) |  |

</details>

## Batch-scribe migration

The current implementation is
[`batch-mode/batch-mode-batch-scribe`](batch-mode/batch-mode-batch-scribe/), a member
of the batch-mode workspace. Its main request type is
`LanguageModelBatchAPIRequest`. The former [`gpt-batch-scribe`](gpt-batch-scribe/)
directory retains historical material, but no manifest or library source; its
obsolete lockfile does not describe the current workspace.

There are still legacy consumers to review separately:

- [`gpt-batch-executor/Cargo.toml`](gpt-batch-executor/Cargo.toml) has a path dependency
  pointing at the retired directory's missing manifest.
- [`ai-generate-readme/Cargo.toml`](ai-generate-readme/Cargo.toml) and
  [`build-readme-generation-queries/Cargo.toml`](build-readme-generation-queries/Cargo.toml)
  still name the old registry crate. These are not automatically equivalent to
  the replacement API and should not be renamed without integration testing.

## Dependency maintenance

Both [batch-mode](batch-mode/Cargo.toml) and [workspacer](workspacer/Cargo.toml)
pin `async-openai` to `=0.29.0`. Preserve that compatibility boundary until a
separate migration has been built and tested. In particular, the current
[upload implementation](batch-mode/batch-mode-batch-client/src/upload_batch_file.rs)
constructs `CreateFileRequest` without the `expires_after` field added upstream
in [async-openai 0.29.6](https://github.com/64bit/async-openai/blob/async-openai-v0.29.6/async-openai/src/types/file.rs).
Optional fields still have to be accounted for in Rust struct literals. The
[January 13, 2026 change](https://github.com/klebs6/klebs-general/commit/2362ad775fc42554c5c5c5471c4ef2508204b6a0)
introduced the exact pin and recorded that higher versions break.

Pinning `async-openai` does not pin every transitive package to an old version.
Its [0.29.0 manifest](https://github.com/64bit/async-openai/blob/async-openai-v0.29.0/async-openai/Cargo.toml)
permits `reqwest` versions in the `0.12` series starting at `0.12.12`. Assess transport dependencies using the selected workspace's actual
lockfile and feature graph; do not add Quinn as a direct dependency merely to
respond to a match in a retired lockfile.

The repository's `.gitignore` excludes `Cargo.lock`; a remote source inventory
is therefore not a substitute for the local lockfiles used by existing builds.
Keep those lockfiles as evidence before updating dependencies. For example,
inspect the existing batch-mode resolution from the repository root with:

```sh
cargo tree --manifest-path batch-mode/Cargo.toml --locked
```

A missing or out-of-date lockfile makes `--locked` fail; that is not a successful
audit. Audit the actual lockfile for each build root, inspect any necessary
transitive updates, and rerun the relevant checks using the same targets and
features. Removing a historical lockfile does not upgrade installed binaries,
other workspaces, or previously published releases.

## Maintaining this catalog

This catalog was refreshed from the root Git tree and the five workspace
manifests at [`b2f167b7`](https://github.com/klebs6/klebs-general/tree/b2f167b739087424d72b03ebdd63ff77f5e56bb3),
inspected on September 5, 2026. When the repository changes, reconcile the root
directory index with the Git tree and each nested list with that workspace's
`members` array. Keep historical entries labeled rather than presenting them as
current implementations.

This is a navigation inventory, not a generated package-version catalog or a
whole-repository build/security certification. Consult each package's own
manifest and license files for its version, requirements, and license.
