//! Tests for the shared assurance intake path (FR-006).
//!
//! These follow this repository's own binding idiom: a `// Trace:` comment above
//! each `#[test]`, which is what Quire's census reads. They invoke the gates
//! rather than reimplementing them, because a test that recomputes what a gate
//! computes is a second implementation that can agree with itself while both are
//! wrong.
//!
//! A missing prerequisite is a failure here, never a skip. A gate that stands
//! down when its dependency is absent reports the same green as one that ran.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use serde::Deserialize;
use serde_json::Value;
use serde_yaml_ng::Value as YamlValue;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The interpreter `make assurance-env` builds. Its absence is an error.
fn assurance_python() -> PathBuf {
    let path = std::env::var_os("ASSURANCE_PYTHON")
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join(".venv-assurance/bin/python"));
    assert!(
        path.is_file(),
        "the pinned assurance interpreter is missing at {}. Run `make assurance-env`. \
         This is a failure and not a skip: a gate that stands down when its dependency \
         is absent reports the same green as one that ran.",
        path.display()
    );
    path
}

fn run(program: &Path, arguments: &[&str]) -> (i32, String, String) {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(root())
        .output()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", program.display()));
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn json_gate(program: &Path, arguments: &[&str]) -> Value {
    let (code, stdout, stderr) = run(program, arguments);
    assert_eq!(code, 0, "{arguments:?} exited {code}\n{stdout}\n{stderr}");
    serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("{arguments:?} did not emit JSON: {error}\n{stdout}"))
}

fn head_revision() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root())
        .output()
        .expect("git rev-parse failed");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn workflow_run_scripts(workflow: &str) -> Result<Vec<String>, String> {
    let document: YamlValue = serde_yaml_ng::from_str(workflow)
        .map_err(|error| format!("invalid workflow YAML: {error}"))?;
    let mut scripts = Vec::new();
    let key = |name: &str| YamlValue::String(name.to_owned());
    let jobs = document
        .as_mapping()
        .and_then(|root| root.get(key("jobs")))
        .and_then(YamlValue::as_mapping)
        .ok_or_else(|| "workflow has no jobs mapping".to_owned())?;
    for (job_name, job) in jobs {
        let job = job
            .as_mapping()
            .ok_or_else(|| format!("workflow job {job_name:?} is not a mapping"))?;
        let Some(steps) = job.get(key("steps")) else {
            continue;
        };
        let steps = steps
            .as_sequence()
            .ok_or_else(|| format!("workflow job {job_name:?} steps are not a sequence"))?;
        for (index, step) in steps.iter().enumerate() {
            let step = step.as_mapping().ok_or_else(|| {
                format!("workflow job {job_name:?} step {index} is not a mapping")
            })?;
            let Some(run) = step.get(key("run")) else {
                continue;
            };
            scripts.push(
                run.as_str()
                    .ok_or_else(|| {
                        format!(
                            "workflow job {job_name:?} step {index} run value is not a scalar string"
                        )
                    })?
                    .to_owned(),
            );
        }
    }
    Ok(scripts)
}

fn workflow_trigger_names(workflow: &str) -> Result<Vec<String>, String> {
    let document: YamlValue = serde_yaml_ng::from_str(workflow)
        .map_err(|error| format!("invalid workflow YAML: {error}"))?;
    let root = document
        .as_mapping()
        .ok_or_else(|| "workflow document is not a mapping".to_owned())?;
    let on = root
        .get(YamlValue::String("on".to_owned()))
        .ok_or_else(|| "workflow has no on key".to_owned())?;
    match on {
        YamlValue::String(trigger) => Ok(vec![trigger.clone()]),
        YamlValue::Sequence(triggers) => triggers
            .iter()
            .map(|trigger| {
                trigger
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| "workflow on sequence contains a non-string trigger".to_owned())
            })
            .collect(),
        YamlValue::Mapping(triggers) => triggers
            .keys()
            .map(|trigger| {
                trigger
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| "workflow on mapping contains a non-string trigger".to_owned())
            })
            .collect(),
        _ => Err("workflow on value is not a trigger, sequence, or mapping".to_owned()),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ShellToken {
    Word(String),
    Boundary,
}

fn shell_tokens(script: &str) -> Result<Vec<ShellToken>, String> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut word_started = false;
    let mut quote = None;
    let mut escaped = false;
    let mut characters = script.chars().peekable();
    let flush = |tokens: &mut Vec<ShellToken>, word: &mut String, word_started: &mut bool| {
        if *word_started {
            tokens.push(ShellToken::Word(std::mem::take(word)));
            *word_started = false;
        }
    };
    // GitHub evaluates workflow expressions before the generated script
    // reaches the shell. Shell comments, quotes, and backslash escaping
    // therefore cannot make `${{ ... }}` literal at this boundary.
    if script.contains("${{") {
        return Err(format!(
            "non-literal workflow expression is unsupported: {script:?}"
        ));
    }
    while let Some(character) = characters.next() {
        if escaped {
            word_started = true;
            if character != '\n' {
                word.push(character);
            }
            escaped = false;
            continue;
        }
        if quote == Some('\'') {
            if character == '\'' {
                quote = None;
            } else {
                word_started = true;
                word.push(character);
            }
            continue;
        }
        if quote == Some('"') {
            match character {
                '"' => quote = None,
                '\\' => escaped = true,
                '$' | '`' => {
                    return Err(format!(
                        "non-literal shell expansion is unsupported: {script:?}"
                    ));
                }
                _ => {
                    word_started = true;
                    word.push(character);
                }
            }
            continue;
        }
        match character {
            '\'' | '"' => {
                word_started = true;
                quote = Some(character);
            }
            '\\' => {
                word_started = true;
                escaped = true;
            }
            '$' | '`' => {
                return Err(format!(
                    "non-literal shell expansion is unsupported: {script:?}"
                ));
            }
            ' ' | '\t' | '\r' => flush(&mut tokens, &mut word, &mut word_started),
            '#' if !word_started => {
                for comment_character in characters.by_ref() {
                    if comment_character == '\n' {
                        if !matches!(tokens.last(), Some(ShellToken::Boundary)) {
                            tokens.push(ShellToken::Boundary);
                        }
                        break;
                    }
                }
            }
            '<' | '>' => {
                return Err(format!(
                    "non-literal shell redirection is unsupported: {script:?}"
                ));
            }
            '\n' | ';' | '|' | '&' => {
                flush(&mut tokens, &mut word, &mut word_started);
                if !matches!(tokens.last(), Some(ShellToken::Boundary)) {
                    tokens.push(ShellToken::Boundary);
                }
                if matches!(character, '|' | '&') && characters.peek() == Some(&character) {
                    characters.next();
                }
            }
            '(' | '{' if !word_started => {
                if !matches!(tokens.last(), Some(ShellToken::Boundary)) {
                    tokens.push(ShellToken::Boundary);
                }
            }
            ')' | '}' => {
                flush(&mut tokens, &mut word, &mut word_started);
                if !matches!(tokens.last(), Some(ShellToken::Boundary)) {
                    tokens.push(ShellToken::Boundary);
                }
            }
            _ => {
                word_started = true;
                word.push(character);
            }
        }
    }
    if escaped || quote.is_some() {
        return Err(format!(
            "shell script has an unterminated token near {word:?}"
        ));
    }
    flush(&mut tokens, &mut word, &mut word_started);
    Ok(tokens)
}

const NPM_INSTALL_ALIASES: &[&str] = &[
    "install", "add", "i", "in", "ins", "inst", "insta", "instal", "isnt", "isnta", "isntal",
    "isntall",
];

fn is_npm_install_alias(word: &str) -> bool {
    NPM_INSTALL_ALIASES.contains(&word)
}

fn is_shell_interpreter(word: &str) -> bool {
    matches!(word.rsplit('/').next(), Some("sh" | "bash"))
}

fn is_npm_executable(word: &str) -> bool {
    word.rsplit('/').next() == Some("npm")
}

fn is_shell_assignment(word: &str) -> bool {
    let Some((name, _)) = word.split_once('=') else {
        return false;
    };
    let mut characters = name.chars();
    matches!(characters.next(), Some(first) if first == '_' || first.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn command_executable_index(words: &[&str]) -> Option<usize> {
    let mut index = 0;
    while let Some(word) = words.get(index).copied() {
        if is_shell_assignment(word) {
            index += 1;
        } else {
            break;
        }
    }
    if words.get(index).copied() == Some("env") {
        index += 1;
        while let Some(word) = words.get(index).copied() {
            if matches!(
                word,
                "-u" | "--unset" | "-C" | "--chdir" | "-S" | "--split-string"
            ) {
                index += 2;
            } else if word.starts_with('-') || is_shell_assignment(word) {
                index += 1;
            } else {
                break;
            }
        }
    }
    words.get(index).map(|_| index)
}

fn is_shell_command_option(word: &str) -> bool {
    word.starts_with('-') && !word.starts_with("--") && word[1..].contains('c')
}

fn scan_ix_flow_packages(
    script: &str,
    depth: usize,
    packages: &mut Vec<String>,
) -> Result<(), String> {
    if depth > 8 {
        return Err("nested shell command depth exceeds 8".to_owned());
    }
    let tokens = shell_tokens(script)?;
    for command in tokens.split(|token| *token == ShellToken::Boundary) {
        let words: Vec<&str> = command
            .iter()
            .filter_map(|token| match token {
                ShellToken::Word(word) => Some(word.as_str()),
                ShellToken::Boundary => None,
            })
            .collect();
        let Some(executable_index) = command_executable_index(&words) else {
            continue;
        };
        let executable = words[executable_index];
        if is_shell_interpreter(executable) {
            let Some(command_option) = words[executable_index + 1..]
                .iter()
                .position(|word| is_shell_command_option(word))
                .map(|offset| executable_index + 1 + offset)
            else {
                continue;
            };
            let nested = words[command_option + 1..]
                .iter()
                .find(|word| **word != "--")
                .ok_or_else(|| {
                    "shell -c option has no statically classifiable script".to_owned()
                })?;
            scan_ix_flow_packages(nested, depth + 1, packages)?;
        }
        if is_npm_executable(executable) {
            let Some((install_offset, _)) = words[executable_index + 1..]
                .iter()
                .enumerate()
                .find(|(_, word)| is_npm_install_alias(word))
            else {
                continue;
            };
            let install_index = executable_index + 1 + install_offset;
            for argument in &words[install_index + 1..] {
                if *argument != "--"
                    && !argument.starts_with('-')
                    && argument.to_ascii_lowercase().contains("ix-flow")
                {
                    packages.push((*argument).to_owned());
                }
            }
        }
    }
    Ok(())
}

fn ix_flow_package_tokens(workflow: &str) -> Result<Vec<String>, String> {
    let mut packages = Vec::new();
    for script in workflow_run_scripts(workflow)? {
        scan_ix_flow_packages(&script, 0, &mut packages)?;
    }
    Ok(packages)
}

// Trace: TC-036, NFR-003-AC-5
#[test]
fn hosted_ci_uses_the_released_scoped_ix_flow_package_and_stays_manual_only() {
    let workflow_path = root().join(".github/workflows/ci.yml");
    let workflow = fs::read_to_string(&workflow_path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", workflow_path.display()));

    assert_eq!(
        workflow_trigger_names(&workflow).expect("classify hosted triggers"),
        ["workflow_dispatch"],
        "hosted CI must retain workflow_dispatch as its only trigger"
    );
    let trigger_metadata = concat!(
        "name: trigger probe\n",
        "on:\n",
        "  workflow_dispatch:\n",
        "permissions: read-all\n",
        "jobs:\n",
        "  probe:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - run: echo safe\n",
    );
    assert_eq!(
        workflow_trigger_names(trigger_metadata).expect("classify triggers around metadata"),
        ["workflow_dispatch"],
        "valid top-level metadata changed the semantic trigger set"
    );
    let automatic = workflow.replacen(
        "  workflow_dispatch:\n",
        "  workflow_dispatch:\n  push:\n",
        1,
    );
    assert_ne!(
        workflow_trigger_names(&automatic).expect("classify automatic trigger mutation"),
        ["workflow_dispatch"],
        "an automatic trigger was accepted"
    );

    // Scan every package token in the workflow rather than recognizing one npm
    // command spelling. A later `npm i -g` install is just as capable of
    // replacing the executable as the current `npm install --global` form.
    let ix_flow_packages = ix_flow_package_tokens(&workflow).expect("classify hosted workflow");
    assert_eq!(
        ix_flow_packages,
        ["@agent-ix/ix-flow@0.0.4".to_owned()],
        "hosted CI must install the released scoped package exactly once"
    );

    let output = Command::new("ix-flow")
        .arg("--version")
        .current_dir(root())
        .output()
        .expect("the exact ix-flow executable is absent from PATH");
    assert!(
        output.status.success(),
        "ix-flow --version failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "0.0.4",
        "the local gate must exercise the same released version installed by hosted CI"
    );
}

// Trace: TC-036, NFR-003-AC-5
#[test]
fn yaml_comments_do_not_add_packages_but_executable_alias_installs_do() {
    let one_install_with_comments = concat!(
        "name: probe\n",
        "on: workflow_dispatch\n",
        "jobs:\n",
        "  probe:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - run: |\n",
        "          npm install --global '@agent-ix/ix-flow@0.0.4' ",
        "# npm i -g '@agent-ix/ix-flow@9.9.9'\n",
        "          # ix-flow@comment-only\n",
    );
    assert_eq!(
        ix_flow_package_tokens(one_install_with_comments).unwrap(),
        ["@agent-ix/ix-flow@0.0.4".to_owned()]
    );

    let executable_alias = one_install_with_comments.replace(
        "          # ix-flow@comment-only",
        "          npm i -g ix-flow@npm:@agent-ix/ix-flow@9.9.9\n          # ix-flow@comment-only",
    );
    assert_eq!(
        ix_flow_package_tokens(&executable_alias).unwrap(),
        [
            "@agent-ix/ix-flow@0.0.4".to_owned(),
            "ix-flow@npm:@agent-ix/ix-flow@9.9.9".to_owned()
        ]
    );

    let word_internal_hash = concat!(
        "on: workflow_dispatch\n",
        "jobs:\n",
        "  probe:\n",
        "    steps:\n",
        "      - \"r\\u0075n\": |\n",
        "          echo marker#not-a-comment; npm add -g github:agent-ix/ix-flow#v9.9.9\n",
    );
    assert_eq!(
        ix_flow_package_tokens(word_internal_hash).unwrap(),
        ["github:agent-ix/ix-flow#v9.9.9".to_owned()],
        "a word-internal shell hash hid an executable npm-add package"
    );

    let inert_metadata = concat!(
        "on: workflow_dispatch\n",
        "defaults:\n",
        "  run:\n",
        "    shell: bash\n",
        "jobs:\n",
        "  probe:\n",
        "    steps:\n",
        "      - name: |\n",
        "          Bob's inert metadata\n",
        "          run: npm add ix-flow@metadata-only\n",
        "        run : echo safe\n",
    );
    assert!(
        ix_flow_package_tokens(inert_metadata).unwrap().is_empty(),
        "plain-scalar metadata became executable because it contains an apostrophe"
    );

    let nested_shell = one_install_with_comments.replace(
        "          # ix-flow@comment-only",
        "          bash -c 'npm in -g ix-flow@npm:@agent-ix/ix-flow@9.9.9'\n          # ix-flow@comment-only",
    );
    assert_eq!(
        ix_flow_package_tokens(&nested_shell).unwrap(),
        [
            "@agent-ix/ix-flow@0.0.4".to_owned(),
            "ix-flow@npm:@agent-ix/ix-flow@9.9.9".to_owned()
        ],
        "a nested shell invocation hid an alternate npm alias install"
    );

    let grouped_path_install = one_install_with_comments.replace(
        "          # ix-flow@comment-only",
        "          ( /usr/bin/npm in -g github:agent-ix/ix-flow#grouped )\n          # ix-flow@comment-only",
    );
    assert_eq!(
        ix_flow_package_tokens(&grouped_path_install).unwrap(),
        [
            "@agent-ix/ix-flow@0.0.4".to_owned(),
            "github:agent-ix/ix-flow#grouped".to_owned()
        ],
        "a grouped path-qualified npm command hid an alternate install"
    );

    let redirected_path_install = one_install_with_comments.replace(
        "          # ix-flow@comment-only",
        "          >/tmp/reviewer-log /usr/bin/npm add -g github:agent-ix/ix-flow#redirected-attached\n          > /tmp/reviewer-log-2 /usr/bin/npm in -g github:agent-ix/ix-flow#redirected-separate\n          2>&1 /usr/bin/npm inst -g github:agent-ix/ix-flow#redirected-fd\n          2>&1> /dev/null /usr/bin/npm insta -g github:agent-ix/ix-flow#redirected-chained\n          # ix-flow@comment-only",
    );
    let redirected_error = ix_flow_package_tokens(&redirected_path_install).unwrap_err();
    assert!(
        redirected_error.contains("non-literal shell redirection")
            && redirected_error.contains("github:agent-ix/ix-flow#redirected-attached")
            && redirected_error.contains("github:agent-ix/ix-flow#redirected-separate")
            && redirected_error.contains("github:agent-ix/ix-flow#redirected-fd")
            && redirected_error.contains("github:agent-ix/ix-flow#redirected-chained"),
        "an unquoted shell redirection was partially scanned: {redirected_error}"
    );

    let command_substitution = one_install_with_comments.replace(
        "          # ix-flow@comment-only",
        "          printf '%s\\n' \"$(2>&1> /dev/null /usr/bin/npm add -g github:agent-ix/ix-flow#substitution)\"\n          # ix-flow@comment-only",
    );
    let substitution_error = ix_flow_package_tokens(&command_substitution).unwrap_err();
    assert!(
        substitution_error.contains("non-literal shell expansion")
            && substitution_error.contains("github:agent-ix/ix-flow#substitution"),
        "a double-quoted command substitution hid an executable npm command: {substitution_error}"
    );

    let backtick_substitution = one_install_with_comments.replace(
        "          # ix-flow@comment-only",
        "          printf '%s\\n' `/usr/bin/npm add -g github:agent-ix/ix-flow#backtick`\n          # ix-flow@comment-only",
    );
    let backtick_error = ix_flow_package_tokens(&backtick_substitution).unwrap_err();
    assert!(
        backtick_error.contains("non-literal shell expansion")
            && backtick_error.contains("github:agent-ix/ix-flow#backtick"),
        "a backtick command substitution hid an executable npm command: {backtick_error}"
    );

    let inert_substitution_spellings = one_install_with_comments.replace(
        "          # ix-flow@comment-only",
        "          printf '%s\\n' '$(npm add github:agent-ix/ix-flow#single-quoted)' '`npm add github:agent-ix/ix-flow#single-backtick`' \"\\$(npm add github:agent-ix/ix-flow#escaped-dollar)\" \"\\`npm add github:agent-ix/ix-flow#escaped-backtick\\`\"\n          # ix-flow@comment-only",
    );
    assert_eq!(
        ix_flow_package_tokens(&inert_substitution_spellings).unwrap(),
        ["@agent-ix/ix-flow@0.0.4".to_owned()],
        "quoted or escaped substitution spellings became executable"
    );

    let workflow_expression = one_install_with_comments.replace(
        "          # ix-flow@comment-only",
        "          printf '%s\\n' '${{ inputs.script }}'\n          # ix-flow@comment-only",
    );
    let expression_error = ix_flow_package_tokens(&workflow_expression).unwrap_err();
    assert!(
        expression_error.contains("non-literal workflow expression"),
        "single shell quotes hid a GitHub workflow expression: {expression_error}"
    );

    for (label, script) in [
        (
            "shell-escaped",
            "          printf '%s\\n' \\${{ inputs.script }}",
        ),
        (
            "double-quoted shell-escaped",
            "          printf '%s\\n' \"\\${{ inputs.script }}\"",
        ),
    ] {
        let escaped_expression = one_install_with_comments.replace(
            "          # ix-flow@comment-only",
            &format!("{script}\n          # ix-flow@comment-only"),
        );
        let error = ix_flow_package_tokens(&escaped_expression).unwrap_err();
        assert!(
            error.contains("non-literal workflow expression"),
            "{label} GitHub workflow expression stayed green: {error}"
        );
    }

    let comment_expression = one_install_with_comments.replace(
        "          # ix-flow@comment-only",
        "          # ${{ inputs.script }}",
    );
    let comment_error = ix_flow_package_tokens(&comment_expression).unwrap_err();
    assert!(
        comment_error.contains("non-literal workflow expression"),
        "a GitHub workflow expression in a shell comment stayed green: {comment_error}"
    );

    let unquoted_variable = one_install_with_comments.replace(
        "          # ix-flow@comment-only",
        "          printf '%s\\n' $IX_FLOW_INSTALL\n          # ix-flow@comment-only",
    );
    let variable_error = ix_flow_package_tokens(&unquoted_variable).unwrap_err();
    assert!(
        variable_error.contains("non-literal shell expansion"),
        "the unquoted shell-variable guard was not independently exercised: {variable_error}"
    );

    let preceding_shell = one_install_with_comments.replace(
        "          # ix-flow@comment-only",
        "          bash --version; /usr/bin/npm add -g github:agent-ix/ix-flow#after-shell\n          # ix-flow@comment-only",
    );
    assert_eq!(
        ix_flow_package_tokens(&preceding_shell).unwrap(),
        [
            "@agent-ix/ix-flow@0.0.4".to_owned(),
            "github:agent-ix/ix-flow#after-shell".to_owned()
        ],
        "a non--c shell invocation suppressed later commands"
    );

    let long_shell_option = one_install_with_comments.replace(
        "          # ix-flow@comment-only",
        "          bash --norc -c 'npm in -g github:agent-ix/ix-flow#nested-long'\n          # ix-flow@comment-only",
    );
    assert_eq!(
        ix_flow_package_tokens(&long_shell_option).unwrap(),
        [
            "@agent-ix/ix-flow@0.0.4".to_owned(),
            "github:agent-ix/ix-flow#nested-long".to_owned()
        ],
        "a long shell option obscured the actual -c script"
    );

    let inert_arguments = one_install_with_comments.replace(
        "          # ix-flow@comment-only",
        "          printf '%s\\n' npm add github:agent-ix/ix-flow#inert bash -c npm in ix-flow@9.9.9\n          # ix-flow@comment-only",
    );
    assert_eq!(
        ix_flow_package_tokens(&inert_arguments).unwrap(),
        ["@agent-ix/ix-flow@0.0.4".to_owned()],
        "npm- or shell-shaped inert command arguments entered the executable population"
    );

    for alias in NPM_INSTALL_ALIASES {
        let aliased = one_install_with_comments.replacen(
            "npm install --global",
            &format!("npm {alias} --global"),
            1,
        );
        assert_eq!(
            ix_flow_package_tokens(&aliased).unwrap(),
            ["@agent-ix/ix-flow@0.0.4".to_owned()],
            "documented npm install alias {alias:?} changed the package population"
        );
    }
}

// Trace: TC-036, NFR-003-AC-5
#[test]
fn yaml_comment_scan_preserves_hashes_inside_quoted_tokens() {
    let source = "on: workflow_dispatch\njobs:\n  probe:\n    steps:\n      - run: |\n          npm install 'ix-flow@single#kept' \
            \"ix-flow@double#kept\" ix-flow@plain#kept\n\
          # npm install ix-flow@comment\n";

    assert_eq!(
        ix_flow_package_tokens(source).unwrap(),
        [
            "ix-flow@single#kept".to_owned(),
            "ix-flow@double#kept".to_owned(),
            "ix-flow@plain#kept".to_owned()
        ]
    );
}

fn deleted_names_in<'a>(
    _inputs: &AssuranceInputsGuard,
    path: &Path,
    names: &'a [&'a str],
) -> Vec<&'a str> {
    // A file that cannot be read has not been scanned. Read bytes so a source
    // with a valid non-UTF-8 encoding cannot disappear from the census merely
    // because Rust strings require UTF-8.
    let source = fs::read(path)
        .unwrap_or_else(|error| panic!("the census could not read {}: {error}", path.display()));
    names
        .iter()
        .copied()
        .filter(|name| {
            let needle = name.as_bytes();
            source.windows(needle.len()).any(|window| window == needle)
        })
        .collect()
}

fn git_files(root: &Path, arguments: &[&str]) -> Vec<String> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(root)
        .output()
        .expect("git ls-files failed");
    assert!(
        output.status.success(),
        "git ls-files {arguments:?} exited non-zero; the census cannot enumerate \
         the repository and reporting it clean would be vacuous: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .split('\0')
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

#[derive(Deserialize)]
struct ReviewFrontmatter {
    id: String,
}

fn review_id(contents: &str, path: &str) -> String {
    let mut documents = serde_yaml_ng::Deserializer::from_str(contents);
    let frontmatter = documents
        .next()
        .unwrap_or_else(|| panic!("tracked review {path} has no YAML frontmatter"));
    ReviewFrontmatter::deserialize(frontmatter)
        .unwrap_or_else(|error| {
            panic!("tracked review {path} has invalid YAML frontmatter: {error}")
        })
        .id
}

fn duplicate_review_ids(
    reviews: impl IntoIterator<Item = (String, String)>,
) -> BTreeMap<String, Vec<String>> {
    let mut paths_by_id = BTreeMap::<String, Vec<String>>::new();
    for (path, contents) in reviews {
        paths_by_id
            .entry(review_id(&contents, &path))
            .or_default()
            .push(path);
    }
    assert!(
        !paths_by_id.is_empty(),
        "tracked SpecReview census is empty; uniqueness would be vacuous"
    );
    paths_by_id
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .collect()
}

// Trace: TC-033, NFR-002-AC-5
#[test]
fn every_tracked_spec_review_id_is_unique() {
    let reviews = git_files(&root(), &["ls-files", "-z", "spec/reviews"])
        .into_iter()
        .map(|path| {
            let contents = fs::read_to_string(root().join(&path))
                .unwrap_or_else(|error| panic!("could not read tracked review {path}: {error}"));
            (path, contents)
        });
    let duplicates = duplicate_review_ids(reviews);
    assert!(
        duplicates.is_empty(),
        "duplicate tracked SpecReview ids: {duplicates:?}"
    );
}

// Trace: TC-033, NFR-002-AC-5
#[test]
fn quoted_review_identity_collides_with_its_plain_yaml_value() {
    let duplicates = duplicate_review_ids([
        ("plain.md".to_owned(), "---\nid: SR-091\n---\n".to_owned()),
        (
            "quoted.md".to_owned(),
            "---\nid: \"SR-091\"\n---\n".to_owned(),
        ),
    ]);
    assert_eq!(
        duplicates.get("SR-091"),
        Some(&vec!["plain.md".to_owned(), "quoted.md".to_owned()])
    );
}

// Trace: TC-033, NFR-002-AC-5
#[test]
#[should_panic(expected = "tracked SpecReview census is empty")]
fn review_identity_census_refuses_an_empty_set() {
    let _ = duplicate_review_ids(std::iter::empty::<(String, String)>());
}

fn census_paths<F>(root: &Path, denied: F) -> (Vec<String>, Vec<String>)
where
    F: Fn(&str) -> bool,
{
    let tracked_all = git_files(root, &["ls-files", "-z"]);
    let tracked: Vec<String> = tracked_all
        .iter()
        .filter(|entry| !denied(entry))
        .cloned()
        .collect();
    (tracked_all, tracked)
}

fn scanned_paths(root: &Path, tracked: &[String]) -> BTreeSet<String> {
    let mut scanned: BTreeSet<String> = tracked.iter().cloned().collect();
    // A path reported by `--others` cannot also be one of the tracked paths in
    // the exact deny set. Applying `denied` here created a second, uncontrolled
    // exemption site: one line could name an untracked reintroduction before the
    // file existed and make it invisible forever.
    for entry in git_files(root, &["ls-files", "-z", "--others", "--exclude-standard"]) {
        scanned.insert(entry);
    }
    scanned
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CensusExemption {
    ExactDeclaration,
    HistoricalProse,
}

fn census_exemption(relative: &str) -> Option<CensusExemption> {
    if matches!(
        relative,
        "tests/shared_assurance.rs" | "assurance/change-assurance.json"
    ) {
        return Some(CensusExemption::ExactDeclaration);
    }
    if relative.ends_with(".md")
        && (relative.starts_with("spec/reviews/") || relative.starts_with("spec/plans/"))
    {
        return Some(CensusExemption::HistoricalProse);
    }
    None
}

fn path_has_legacy_compat(path: &str) -> bool {
    path.split('/')
        .any(|component| component.contains("legacy-compat"))
}

fn proof_has_legacy_compat(proof: &Value) -> bool {
    proof["proof_id"]
        .as_str()
        .is_some_and(|proof_id| proof_id.contains("legacy-compat"))
}

fn census_matches<'a>(
    inputs: &AssuranceInputsGuard,
    root: &Path,
    path: &Path,
    names: &'a [&'a str],
) -> Vec<&'a str> {
    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    if census_exemption(&relative).is_some() {
        Vec::new()
    } else {
        deleted_names_in(inputs, path, names)
    }
}

fn area_cardinalities(paths: impl IntoIterator<Item = impl AsRef<str>>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for path in paths {
        let path = path.as_ref();
        let area = path
            .split_once('/')
            .map_or("<root>", |(head, _)| head)
            .to_owned();
        *counts.entry(area).or_insert(0) += 1;
    }
    counts
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else {
        "non-string panic".to_owned()
    }
}

fn digest_of(path: &Path) -> String {
    let output = Command::new("sha256sum")
        .arg(path)
        .output()
        .expect("sha256sum failed");
    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .expect("sha256sum output")
        .to_owned()
}

/// The chain is expensive and several tests read it. It runs once per test
/// binary, and every reader sees the same run rather than a different one.
static CHAIN: OnceLock<Value> = OnceLock::new();

mod shared_inputs {
    use std::sync::{Mutex, MutexGuard};

    static LOCK: Mutex<()> = Mutex::new(());

    pub(super) struct Guard {
        _lock: MutexGuard<'static, ()>,
    }

    pub(super) fn lock() -> Guard {
        let lock = LOCK.lock().unwrap_or_else(|poisoned| {
            panic!(
                "shared assurance inputs may have been left mutated by a panicking test; \
                 re-run `make assurance-inputs`: {poisoned}"
            )
        });
        Guard { _lock: lock }
    }
}

type AssuranceInputsGuard = shared_inputs::Guard;

fn assurance_inputs_guard() -> AssuranceInputsGuard {
    shared_inputs::lock()
}

fn shared_pins_report(_inputs: &AssuranceInputsGuard) -> Value {
    let python = assurance_python();
    json_gate(&python, &["scripts/check_shared_pins.py", "--json"])
}

fn chain_report(_inputs: &AssuranceInputsGuard) -> &'static Value {
    CHAIN.get_or_init(|| {
        // The chain runs under the system interpreter: it only shells out to
        // quoin and never imports engineering-assurance.
        let revision = head_revision();
        let (code, stdout, stderr) = run(
            Path::new("python3"),
            &[
                "scripts/assurance_chain.py",
                "--candidate-revision",
                &revision,
                "--json",
            ],
        );
        assert_eq!(code, 0, "the assurance chain exited {code}\n{stderr}");
        serde_json::from_str(&stdout).expect("the assurance chain did not emit JSON")
    })
}

// Trace: TC-030, FR-007-AC-6
#[test]
fn contextual_native_result_reaches_existing_quoin_intake() {
    let inputs = assurance_inputs_guard();
    let rows = fs::read_to_string(root().join("target/assurance/reference-conformance.jsonl"))
        .expect("reference-conformance producer output is absent; run make assurance-inputs");
    let row = rows
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("producer JSONL row"))
        .find(|row| row["symbol"] == "short-trace-future-v1/closed")
        .expect("contextual producer row");
    let native = &row["detail"]["contextualNative"];
    assert_eq!(native["schemaVersion"], "tl-mltl.evaluation/v2");
    assert_eq!(
        native["requirementContext"]["requirement_id"],
        "agent-ix/tl-mltl/FR-007"
    );
    assert!(native["signalCatalogSha256"].as_str().is_some());
    assert!(native["requestSha256"].as_str().is_some());
    assert!(native["resultSha256"].as_str().is_some());

    // The existing chain seals the exact producer file digest; it reads this
    // already-produced row and does not invoke the producer itself.
    let report = chain_report(&inputs);
    assert_eq!(
        report["attested_results"]["PROOF-reference-conformance"],
        "passed"
    );
}

// Trace: TC-018, FR-006-AC-1, NFR-003-AC-1
#[test]
fn every_shared_pin_is_classified_by_the_packaged_matrix() {
    let inputs = assurance_inputs_guard();
    let report = shared_pins_report(&inputs);
    let python = assurance_python();

    let components = report["components"].as_array().expect("components array");
    assert_eq!(
        components.len(),
        4,
        "the matrix pins four components; this run classified {}",
        components.len()
    );
    for component in components {
        assert_eq!(
            component["verdict"], "compatible",
            "{} is {} ({})",
            component["component"], component["verdict"], component["reason"]
        );
    }
    assert_eq!(report["accepted"], true);
    assert!(report["artifact_mismatches"].as_array().unwrap().is_empty());
    assert!(report["mirror_references"].as_array().unwrap().is_empty());
    assert!(
        report["upstream_pin_mismatches"]
            .as_array()
            .unwrap()
            .is_empty(),
        "the tl-syntax pins disagree across the files that name them: {}",
        report["upstream_pin_mismatches"]
    );

    // Acceptance is reported and never gated on: the pinned release records
    // `pending_human_acceptance` and ships no predicate for it
    // (agent-ix/engineering-assurance#20). Reading an absent field as approval,
    // in either direction, is the mistake this asserts against.
    assert_eq!(report["acceptance_recorded_here"], false);
    assert!(report["acceptance_state"].is_string());

    // The mirror check must be seen to refuse. Without this it is indistinguishable
    // from a check that matches nothing.
    let (code, stdout, stderr) = run(
        &python,
        &[
            "-c",
            "import json,sys;sys.path.insert(0,'scripts');\
             import check_shared_pins as m;\
             pins=json.load(open('assurance/pins.json'));\
             pins['engineering_assurance']['requirement']+=' --registry=https://npm.ix/';\
             print(json.dumps(m.mirror_references(pins)))",
        ],
    );
    assert_eq!(code, 0, "the mirror probe failed: {stderr}");
    let offenders: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        !offenders.is_empty(),
        "a mirror registry reference was not detected; the check matches nothing"
    );

    // The two tl-syntax revisions are two facts. The compiled pin moved onto
    // main; the retained corpus basis did not. Collapsing them is refused, and
    // the refusal is exercised rather than assumed.
    let (code, stdout, stderr) = run(
        &python,
        &[
            "-c",
            "import json,sys;sys.path.insert(0,'scripts');\
             import check_shared_pins as m;\
             pins=json.load(open('assurance/pins.json'));\
             pins['upstream_dependency']['corpus_basis']=\
             pins['upstream_dependency']['compiled_revision'];\
             print(json.dumps(m.upstream_pin_mismatches(pins)))",
        ],
    );
    assert_eq!(code, 0, "the collapsed-revision probe failed: {stderr}");
    let problems: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        problems.iter().any(|item| item.contains("same string")),
        "collapsing the compiled revision and the corpus basis was not detected: {problems:?}"
    );

    // The current-facing prose is part of the dependency identity, not merely
    // an author-maintained explanation. Each stale compiled-pin spelling is
    // independently refused by the same guard that checks Cargo and the wire
    // constant.
    let scratch = std::env::temp_dir().join(format!(
        "tl-mltl-stale-current-pin-probe-{}",
        std::process::id()
    ));
    match fs::remove_dir_all(&scratch) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to clear stale current-pin probe: {error}"),
    }
    fs::create_dir_all(&scratch).unwrap();
    for name in [
        "README.md",
        "corpus/README.md",
        "assurance/change-assurance.json",
    ] {
        let source = root().join(name);
        let candidate = scratch.join(name);
        fs::create_dir_all(candidate.parent().unwrap()).unwrap();
        fs::copy(&source, &candidate).unwrap();
        let stale = fs::read_to_string(&candidate)
            .unwrap()
            .replace(
                "5b1c13440e54d5a851df2d33cc88944135574bc6",
                "8dc18eec5af227f484170362c9e8894b8531a27d",
            )
            .replace("5b1c1344", "8dc18eec");
        fs::write(&candidate, stale).unwrap();
        let (code, stdout, stderr) = run(
            &python,
            &[
                "-c",
                "import json,sys;from pathlib import Path;sys.path.insert(0,'scripts');import check_shared_pins as m;m.ROOT=Path(sys.argv[1]);pins=json.load(open('assurance/pins.json'));print(json.dumps(m.upstream_pin_mismatches(pins)))",
                scratch.to_str().unwrap(),
            ],
        );
        assert_eq!(code, 0, "the stale-current-pin probe failed: {stderr}");
        let problems: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
        assert!(
            problems.iter().any(|problem| problem.starts_with(name)),
            "the pin guard did not reject a stale compiled revision in {name}: {:?}",
            problems
        );
    }
    let corpus_readme = scratch.join("corpus/README.md");
    fs::copy(root().join("corpus/README.md"), &corpus_readme).unwrap();
    let corpus_basis = fs::read_to_string(&corpus_readme)
        .unwrap()
        .replace("740182f1", "6ad7499f");
    fs::write(&corpus_readme, corpus_basis).unwrap();
    let (code, stdout, stderr) = run(
        &python,
        &[
            "-c",
            "import json,sys;from pathlib import Path;sys.path.insert(0,'scripts');import check_shared_pins as m;m.ROOT=Path(sys.argv[1]);pins=json.load(open('assurance/pins.json'));print(json.dumps(m.upstream_pin_mismatches(pins)))",
            scratch.to_str().unwrap(),
        ],
    );
    assert_eq!(code, 0, "the corpus-basis probe failed: {stderr}");
    let problems: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        problems
            .iter()
            .any(|problem| problem == "corpus/README.md: does not name the expected revision"),
        "the pin guard did not reject a corpus README that conflates the retained basis with the compiled revision: {problems:?}"
    );
    match fs::remove_dir_all(&scratch) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to remove stale current-pin probe: {error}"),
    }
}

// Trace: TC-019, FR-006-AC-2, NFR-003-AC-1, SUITE-004, SUITE-005, SUITE-006
#[test]
fn the_chain_reaches_quoin_without_quoin_or_quire_executing_a_producer() {
    let inputs = assurance_inputs_guard();
    let report = chain_report(&inputs);
    assert_eq!(report["matched"], true, "{report:#}");

    for group in ["scenarios", "controls", "adapter_probes"] {
        let items = report[group]
            .as_array()
            .unwrap_or_else(|| panic!("{group}"));
        assert!(!items.is_empty(), "{group} is empty");
        for item in items {
            assert_eq!(
                item["matched"], true,
                "{group} entry did not match: {item:#}"
            );
        }
    }

    // Every attested result is read out of the producer's bytes. Asserting the
    // values here means a chain that reverted to sealing a literal "passed"
    // would still have to agree with what the producers actually wrote — and a
    // chain that attested `inconclusive`, `not_computed` or `unavailable` while
    // still matching every byte-identity scenario would fail here rather than
    // reporting exit 0.
    let attested = report["attested_results"]
        .as_object()
        .expect("attested_results");
    assert_eq!(
        attested.len(),
        6,
        "six proof obligations are declared; {} were attested",
        attested.len()
    );
    for (proof, result) in attested {
        assert_eq!(result, "passed", "{proof} was attested {result}");
    }

    // The adapter transcribes one named protocol and refuses another, rather than
    // guessing. A verdict recovered from an unrecognised stream is a verdict
    // recovered from nothing.
    let probes = report["adapter_probes"].as_array().unwrap();
    for required in [
        "refuses-a-foreign-protocol",
        "refuses-an-unnamed-outcome",
        "refuses-an-empty-stream",
        "accepts-the-real-run",
    ] {
        assert!(
            probes.iter().any(|probe| probe["probe"] == required),
            "adapter probe {required} is missing"
        );
    }
}

/// Write an executable shim for each name that records every invocation.
///
/// The log is the point. A shim that is never consulted and a producer that is
/// never run look identical from the outside, so the shims write down every call
/// and the test reads the file rather than assuming.
///
/// A version query is answered rather than refused, and deliberately so. Asking
/// a tool its version is an observation — it is what the compatibility matrix's
/// own `observe` column does — and it is not the thing this test forbids. What
/// is forbidden is asking a tool to build, compile, test, evaluate, or replay
/// anything. Every such invocation is logged and the log must be empty.
///
/// `--version` is matched anywhere in the argv, not just in `$1`, because the
/// MSRV attestation observes `rustup run 1.75.0 cargo --version`: its declared
/// command runs cargo through the pinned toolchain, so the version sealed into
/// the attestation has to come from that toolchain rather than from ambient
/// cargo. That is still a version observation. Anything without a version flag
/// — `cargo build`, `cargo run`, `rustup run … check` — is logged and fails the
/// test, which is what keeps it able to fail.
///
/// `provenance` is answered for the same reason: it is how the driver observes
/// Quire's version, because `quire provenance` reports the CLI and engine
/// identity as JSON and `quire --version` reports only the CLI. Answering it
/// lets `quire` be shimmed at all, which matters — `quire coverage` is a
/// producer and would otherwise be invisible to this test. That gap was real
/// until an injected `quire coverage` in the driver went undetected here.
fn producer_shims(_inputs: &AssuranceInputsGuard, directory: &Path, names: &[&str]) -> PathBuf {
    // Removed and recreated, not merely topped up. `target/` survives between
    // runs, so a shim written by an earlier version of this test would still be
    // on the shimmed PATH and would silently change what is being measured —
    // which is exactly what happened while this test was being written.
    match fs::remove_dir_all(directory) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to clear stale producer shims: {error}"),
    }
    fs::create_dir_all(directory).unwrap();
    let log = directory.join("invocations.log");
    for name in names {
        let path = directory.join(name);
        fs::write(
            &path,
            format!(
                "#!/bin/sh\n\
                 for argument in \"$@\"; do\n\
                 case \"$argument\" in\n\
                 --version|-V) echo \"{name} 9.9.9 (shim)\"; exit 0 ;;\n\
                 provenance) echo '{{\"cli\":{{\"version\":\"9.9.9\"}},\
                 \"engine\":{{\"version\":\"9.9.9\"}}}}'; exit 0 ;;\n\
                 esac\n\
                 done\n\
                 echo \"$0 $@\" >> {}\n\
                 exit 97\n",
                log.display()
            ),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
    log
}

fn run_chain_with_path(_inputs: &AssuranceInputsGuard, shims: &Path) -> std::process::Output {
    let inherited = std::env::var("PATH").unwrap_or_default();
    let revision = head_revision();
    Command::new("python3")
        .args([
            "scripts/assurance_chain.py",
            "--candidate-revision",
            &revision,
        ])
        .current_dir(root())
        .env("PATH", format!("{}:{inherited}", shims.display()))
        .output()
        .expect("failed to run the assurance chain")
}

fn assert_probe_store_isolated(_inputs: &AssuranceInputsGuard, scratch_target: &Path, probe: &str) {
    let scratch_store = fs::canonicalize(scratch_target.join("assurance-store"))
        .unwrap_or_else(|error| panic!("{probe} did not create its Quoin store: {error}"));
    let unresolved_real_store = fs::canonicalize(root().join("target"))
        .expect("canonical repository target directory")
        .join("assurance-store");
    let real_store = match fs::canonicalize(&unresolved_real_store) {
        Ok(path) => path,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => unresolved_real_store,
        Err(error) => panic!("could not resolve the repository Quoin store: {error}"),
    };
    assert!(
        !scratch_store.starts_with(&real_store),
        "{probe} placed its Quoin store in the repository store: {scratch_store:?}"
    );
}

// Trace: TC-019, FR-006-AC-2, NFR-003-AC-2
#[test]
fn the_chain_never_executes_a_producer_and_the_probe_can_prove_it() {
    let inputs = assurance_inputs_guard();
    // Two runs, because one proves nothing.
    //
    // Run A replaces every producer — cargo, rustup, rustc, quire, and the
    // external monitor and its compiler — with a stub that logs and fails. The
    // chain must finish, and the log must be empty: not one producer was
    // invoked.
    //
    // `quire` is deliberately NOT in this list, and the reason is worth stating
    // because the obvious reading is that it was forgotten.
    //
    // `quire coverage` is a producer of one of the seven inputs, so a PATH shim
    // looks like the right instrument. It is not: `quoin evidence record`
    // invokes `quire coverage` itself, inside the store it is writing to. That
    // is Quoin using the static exporter, which is exactly what the architecture
    // says Quire is for, and a PATH shim cannot tell it apart from the driver
    // regenerating its own input. Shimming `quire` makes run A fail on a clean
    // tree — measured, not assumed.
    //
    // The property is instead tested directly, and more strongly, by run C
    // below: every declared input is moved aside in turn and the driver is
    // required to refuse rather than recreate it. That covers `quire coverage`
    // and the other six producers by name.
    //
    // Run B is the control. It stubs `quoin`, which the chain is supposed to run,
    // and requires the chain to fail and the log to be non-empty. Without it, an
    // empty log in run A would be equally consistent with PATH never being
    // consulted at all.
    let producers = root().join("target/producer-shims");
    let producer_log = producer_shims(
        &inputs,
        &producers,
        &["cargo", "rustup", "rustc", "r2u2", "c2po"],
    );
    let output = run_chain_with_path(&inputs, &producers);
    let logged = fs::read_to_string(&producer_log).unwrap_or_default();
    assert!(
        output.status.success(),
        "the assurance chain failed with producers stubbed, which means it ran one:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        logged.trim().is_empty(),
        "the assurance driver asked a producer to do work, not just to name its version:\n{logged}"
    );

    let tools = root().join("target/tool-shims");
    let tool_log = producer_shims(&inputs, &tools, &["quoin"]);
    let control = run_chain_with_path(&inputs, &tools);
    let tool_logged = fs::read_to_string(&tool_log).unwrap_or_default();
    assert!(
        !tool_logged.trim().is_empty(),
        "stubbing quoin produced no invocation, so PATH is not being consulted by \
         the subprocess and the run above proves nothing"
    );
    assert!(
        !control.status.success(),
        "the chain succeeded with quoin stubbed out, so it is not actually using it"
    );

    // Run C. Every declared input, one at a time: move it aside, run the driver,
    // and require it to refuse with exit 2 naming the target that writes the
    // file. A driver that can produce its own inputs can produce a green run out
    // of nothing, and this is the direct measurement of that — it covers `quire
    // coverage`, which cannot be PATH-shimmed for the reason given above, and it
    // covers the other five producers by name rather than by absence of evidence.
    let assurance = root().join("target/assurance");
    let inputs = [
        "reference-conformance.jsonl",
        "r2u2-differential.jsonl",
        "cli-conformance.jsonl",
        "test-census.json",
        "quire-static-export.json",
        "msrv.jsonl",
    ];
    for name in inputs {
        let present = assurance.join(name);
        assert!(
            present.is_file(),
            "{name} is absent before the probe even starts; run `make assurance-inputs`"
        );
        let stashed = assurance.join(format!("{name}.stashed-by-test"));
        fs::rename(&present, &stashed).unwrap();
        let revision = head_revision();
        let output = Command::new("python3")
            .args([
                "scripts/assurance_chain.py",
                "--candidate-revision",
                &revision,
            ])
            .current_dir(root())
            .output()
            .expect("failed to run the assurance chain");
        fs::rename(&stashed, &present).unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        assert_eq!(
            output.status.code(),
            Some(2),
            "with {name} absent the driver exited {:?}; a driver that carries on \
             without a producer's output is a driver that can report a result nobody \
             produced\n{stderr}",
            output.status.code()
        );
        assert!(
            stderr.contains("make assurance-inputs"),
            "the refusal for an absent {name} did not name the target that writes \
             it: {stderr}"
        );
        assert!(
            present.is_file(),
            "the driver recreated {name} instead of refusing; it is a producer"
        );
    }
}

// Trace: TC-019, FR-006-AC-2, NFR-003-AC-1
#[test]
fn an_unobservable_tool_version_is_refused_rather_than_defaulted() {
    let inputs = assurance_inputs_guard();
    // A sealed attestation names the version of the tool that produced the bytes.
    // A version nobody measured, filled in with a plausible-looking default, is
    // worse than an absent one: a reader cannot tell it apart from a real
    // observation. The driver raises rather than defaulting, and that raise is
    // on a branch the honest path never takes — measured, not assumed: a
    // mutation inserting `observed = "0.0.0"` before the raise was applied and
    // no gate detected it, because on a working toolchain the branch is dead.
    //
    // So the branch is made live. `rustup` is replaced by a stub that fails for
    // every argument including `--version`, which is how the MSRV proof's cargo
    // version is observed. The chain must refuse with exit 2 and say why.
    let directory = root().join("target/unobservable-version-shim");
    match fs::remove_dir_all(&directory) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to clear stale unobservable-version shims: {error}"),
    }
    fs::create_dir_all(&directory).unwrap();
    let shim = directory.join("rustup");
    fs::write(&shim, "#!/bin/sh\nexit 1\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&shim, fs::Permissions::from_mode(0o755)).unwrap();
    }

    let output = run_chain_with_path(&inputs, &directory);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(2),
        "the chain did not refuse an unobservable tool version; it exited {:?} \
         and would have sealed an attestation naming a version nobody \
         measured\n{stderr}",
        output.status.code()
    );
    assert!(
        stderr.contains("could not be observed"),
        "the refusal did not name the cause: {stderr}"
    );
}

// Trace: TC-019, FR-006-AC-2, NFR-003-AC-2
#[test]
fn the_driver_refuses_to_start_any_child_that_is_not_quoin_or_a_version_probe() {
    let inputs = assurance_inputs_guard();
    // The PATH-shim runs cannot establish this and an adversarial review showed
    // exactly why: `quoin evidence record` runs `quire coverage` itself, so
    // `quire` cannot be shimmed; and a driver that ran `quire coverage` and
    // discarded the output left no shim invocation, no recreated input file and
    // no trace of any kind. The isolation test passed with that injection in.
    //
    // So the boundary is enforced inside the driver by an audit hook, and here
    // it is exercised: a copy of the driver with the review's exact injection
    // must exit 2 and name the argv it refused.
    let report = chain_report(&inputs);
    let children = report["child_processes"]
        .as_array()
        .expect("child_processes");
    assert!(
        !children.is_empty(),
        "the driver recorded no child processes at all, so this test is measuring \
         nothing: {report:#}"
    );
    for child in children {
        let command = child.as_str().unwrap_or("");
        let permitted = command.starts_with("quoin ")
            || command == "quoin"
            || command.starts_with("quire provenance")
            || command.contains(" --version")
            || command.contains(" -V");
        assert!(
            permitted,
            "the driver started `{command}`, which is neither the pinned Quoin CLI \
             nor a version observation"
        );
    }

    let scratch = root().join("target/execution-boundary-probe");
    // A failed probe retains its scratch for diagnosis. The next run must
    // remove that exact owned tree or fail with the cleanup cause, rather than
    // turning stale links into a misleading EEXIST later in setup.
    match fs::remove_dir_all(&scratch) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            panic!("failed to clear the previous execution-boundary scratch: {error}")
        }
    }
    fs::create_dir_all(scratch.join("scripts")).unwrap();
    let driver = fs::read_to_string(root().join("scripts/assurance_chain.py")).unwrap();
    let marker = "def run_chain(candidate_revision: str, workspace: Path) -> dict[str, Any]:\n";
    assert!(driver.contains(marker), "the driver's entry point moved");
    let mutated = driver.replacen(
        marker,
        &format!(
            "{marker}    subprocess.run([\"quire\", \"coverage\", \"--scope\", \".\", \
             \"--json\"], cwd=ROOT, check=False, capture_output=True)\n"
        ),
        1,
    );
    assert_ne!(mutated, driver, "the injection did not apply");
    fs::write(scratch.join("scripts/assurance_chain.py"), &mutated).unwrap();
    for entry in fs::read_dir(root()).expect("repository root") {
        let path = entry.expect("directory entry").path();
        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_owned();
        if name == "scripts" || name == ".git" || name == "target" {
            continue;
        }
        std::os::unix::fs::symlink(&path, scratch.join(&name))
            .unwrap_or_else(|error| panic!("failed to link {name} into the probe: {error}"));
    }
    let scratch_target = scratch.join("target");
    // Check ownership before creating anything under target. If `target` drops
    // out of the skip set, this is the first reactor and no self-referential
    // link can be created through the repository target.
    match fs::symlink_metadata(&scratch_target) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Ok(_) => panic!(
            "the execution-boundary probe must own target/ rather than inherit a root-entry link"
        ),
        Err(error) => panic!("could not establish scratch target ownership: {error}"),
    }
    fs::create_dir_all(&scratch_target).expect("create isolated probe target");
    std::os::unix::fs::symlink(
        root().join("target/assurance"),
        scratch_target.join("assurance"),
    )
    .expect("share assurance inputs with the isolated probe");

    let revision = head_revision();
    let output = Command::new("python3")
        .args([
            "scripts/assurance_chain.py",
            "--candidate-revision",
            &revision,
        ])
        .current_dir(&scratch)
        .output()
        .expect("failed to run the injected chain");
    assert_probe_store_isolated(&inputs, &scratch_target, "the execution-boundary probe");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(2),
        "a driver that ran `quire coverage` was not refused; it exited {:?}\n{stderr}",
        output.status.code()
    );
    assert!(
        stderr.contains("not permitted to start"),
        "the refusal did not name the cause: {stderr}"
    );
    assert!(
        stderr.contains("quire"),
        "the refusal did not name the argv it refused: {stderr}"
    );

    // Restore the original driver in the same isolated fixture. Exit 0 proves
    // the injected child, rather than scratch construction, caused the refusal.
    fs::write(scratch.join("scripts/assurance_chain.py"), &driver).unwrap();
    let bypassed = Command::new("python3")
        .args([
            "scripts/assurance_chain.py",
            "--candidate-revision",
            &revision,
        ])
        .current_dir(&scratch)
        .output()
        .expect("failed to run the unmutated chain in the execution-boundary scratch");
    assert_eq!(
        bypassed.status.code(),
        Some(0),
        "the execution-boundary scratch is invalid without the injected child:\n{}\n{}",
        String::from_utf8_lossy(&bypassed.stdout),
        String::from_utf8_lossy(&bypassed.stderr)
    );
    // `remove_dir_all` does not follow directory symlinks, but explicitly
    // unlinking every shared input keeps that safety boundary visible and stops
    // a future walk-and-delete replacement reaching repository inputs.
    fs::remove_file(scratch_target.join("assurance")).expect("unlink shared assurance inputs");
    for entry in fs::read_dir(&scratch).expect("read execution-boundary scratch") {
        let path = entry.expect("scratch entry").path();
        if fs::symlink_metadata(&path)
            .expect("scratch entry metadata")
            .file_type()
            .is_symlink()
        {
            fs::remove_file(path).expect("unlink execution-boundary repository input");
        }
    }
    fs::remove_dir_all(&scratch).expect("remove execution-boundary scratch");
}

// Trace: TC-020, FR-006-AC-3, SUITE-003
#[test]
fn the_sealed_records_impact_snapshot_is_the_quire_export() {
    let inputs = assurance_inputs_guard();
    let report = chain_report(&inputs);
    let export = root().join(report["quire_export"].as_str().expect("quire_export"));
    let bytes =
        fs::read(&export).unwrap_or_else(|error| panic!("{} is absent: {error}", export.display()));

    assert_eq!(
        report["impact_snapshot_digest"],
        digest_of(&export),
        "the sealed record's impact snapshot does not name the Quire export it claims"
    );
    // An empty object has a digest too. The snapshot is only worth its content,
    // so the export is required to actually carry the coverage facts the record
    // claims it snapshotted, and to name every requirement this repository has.
    let parsed: Value = serde_json::from_slice(&bytes).expect("the Quire export is JSON");
    let text = String::from_utf8_lossy(&bytes);
    for requirement in [
        "FR-001", "FR-002", "FR-003", "FR-004", "FR-005", "FR-006", "FR-007", "FR-008", "FR-009",
        "FR-010", "FR-011", "FR-012", "FR-013", "FR-014", "FR-015", "FR-016", "FR-017", "NFR-001",
        "NFR-002", "NFR-003", "NFR-004", "NFR-005", "StR-001", "StR-002", "StR-003",
    ] {
        assert!(
            text.contains(requirement),
            "the Quire export does not mention {requirement}; it is not a coverage \
             export of this repository"
        );
    }
    assert!(
        parsed.is_object() && !parsed.as_object().unwrap().is_empty(),
        "the Quire export is not a populated document"
    );

    // The measured coverage, pinned. `derive_result` refuses an export that
    // measured nothing or carries a status lie, but the figures themselves are
    // asserted too: an export reporting different totals has to move a number in
    // this file rather than only a threshold the driver applies.
    let totals = &parsed["totals"];
    // 186 is every row Quire counts from `spec/`: 104 acceptance criteria and
    // 82 test-matrix rows. Issues #47/#48 added the backed W/M rows; issue #38
    // contributes 32 planned M4 rows, and issue #39 contributes 55 planned M5
    // rows. The suite registry is not counted because spec-artifacts-process
    // 737987b (quire-rs#363) declares evidence registries reference-only.
    assert_eq!(
        totals["total"], 186,
        "the declared-row population changed: {totals}. It is 104 acceptance \
         criteria + 82 test-matrix rows; suite-registry rows are reference-only."
    );
    assert_eq!(
        totals["backed"], 99,
        "backed-row count changed: {totals}. The 87 M4/M5 rows are planned and \
         intentionally unbacked: 32 M4 rows plus 55 M5 rows. If this count moved, \
         update the campaigns deliberately rather than adjusting the assertion."
    );
    // With suite rows out of the coverage totals, the totals no longer notice a
    // suite binding disappearing, so the registry's own claim is checked
    // directly: every suite except SUITE-001 and SUITE-002 is named on a
    // compiled test's trace line, and those two are named on none.
    let registry = fs::read_to_string(root().join("spec/evidence/suites.md"))
        .expect("read the suite registry");
    let registered: BTreeSet<&str> = registry
        .lines()
        .filter_map(|line| line.strip_prefix("| SUITE-"))
        .filter_map(|rest| rest.split_whitespace().next())
        .collect();
    let mut bound = BTreeSet::new();
    for entry in fs::read_dir(root().join("tests")).expect("list tests") {
        let path = entry.expect("tests entry").path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let source = fs::read_to_string(&path).expect("read test source");
        for trace in source
            .lines()
            .filter_map(|line| line.trim().strip_prefix("// Trace:"))
        {
            for id in trace.split(',').map(str::trim) {
                if let Some(suite) = id.strip_prefix("SUITE-") {
                    bound.insert(suite.to_owned());
                }
            }
        }
    }
    let expected_bound: BTreeSet<String> = registered
        .iter()
        .filter(|suite| !matches!(**suite, "001" | "002"))
        .map(|suite| (*suite).to_owned())
        .collect();
    assert_eq!(
        registered.len(),
        8,
        "the suite registry population changed: {registered:?}"
    );
    assert_eq!(
        bound, expected_bound,
        "suite bindings disagree with spec/evidence/suites.md: six rows are bound \
         by a test running that suite's command and SUITE-001/SUITE-002 by none"
    );
    assert!(
        parsed["status_lies"].as_array().unwrap().is_empty(),
        "Quire reported a row whose declared status disagrees with its evidence: {}",
        parsed["status_lies"]
    );

    // And the chain must have read it as such rather than as a not-computed run.
    assert_eq!(
        report["attested_results"]["PROOF-quire-static-export"], "passed",
        "the Quire export was attested as {}",
        report["attested_results"]["PROOF-quire-static-export"]
    );
}

// Trace: TC-022, FR-006-AC-5, NFR-003-AC-3
#[test]
fn all_twelve_verification_outcomes_are_demonstrated_and_paired_with_controls() {
    let inputs = assurance_inputs_guard();
    // The twelve states this migration must keep distinguishable, and the gate
    // that owns each. A state nobody demonstrates is a state nobody would notice
    // the loss of.
    //
    // `malformed` is owned by the shared temporal corpus: three of its eight
    // fixtures are malformed by design. `unsupported` is owned by the R2U2
    // exchange, whose manifest declares one case outside the adapter's profile.
    // Both are this repository's own domain behaviour rather than a compatibility
    // artefact.
    // Each state is bound to the NAMED case that owns it, not merely to the
    // set of state strings the run happened to emit. An adversarial review
    // deleted the only real `suspect` demonstration and relabelled an unrelated
    // probe `suspect`; every gate stayed green, because `states_demonstrated`
    // is built from a free-text label the author types next to the assertion.
    // Requiring a named owner means a relabelled bystander no longer stands in
    // for a demonstration that was removed.
    const REQUIRED: [(&str, &str); 12] = [
        ("pass", "retain-producer-output"),
        ("fail", "attested-failed"),
        ("unavailable", "attested-unavailable"),
        (
            "unsupported",
            "declared-unsupported-case-is-reported-unsupported",
        ),
        ("inconclusive", "declared-unknowns-are-carried-not-dropped"),
        ("not-computed", "attested-not_computed"),
        ("malformed", "malformed-formula-is-reported-as-malformed"),
        ("partial", "receipt-reports-the-absent-human-decision"),
        ("stale", "stale-candidate-binding"),
        ("suspect", "audit-reports-a-suspect-link"),
        ("vacuous", "audit-reports-a-vacuous-run"),
        ("tampered", "refuse-an-edited-receipt"),
    ];

    let report = chain_report(&inputs);

    // Only MEASURED outcomes count. The chain's `states_demonstrated` is built
    // from cases that ran and matched — never from a free-text label typed next
    // to an assertion, which would let a state stop being demonstrated while
    // this test stayed green. That is the exact failure mode this test exists to
    // rule out.
    let demonstrated: BTreeSet<String> = report["states_demonstrated"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();

    let missing: Vec<&str> = REQUIRED
        .iter()
        .filter(|(state, _)| !demonstrated.contains(*state))
        .map(|(state, _)| *state)
        .collect();
    assert!(
        missing.is_empty(),
        "these verification outcomes were never demonstrated: {missing:?}; \
         demonstrated: {demonstrated:?}"
    );

    // And each one is demonstrated by the case that is supposed to demonstrate
    // it. Without this, deleting a demonstration and relabelling any other case
    // with its state keeps the set complete and the gate green.
    let mut owners: BTreeSet<(String, String)> = BTreeSet::new();
    for group in ["scenarios", "adapter_probes"] {
        for item in report[group].as_array().unwrap() {
            if item["matched"] != serde_json::Value::Bool(true) {
                continue;
            }
            let name = item
                .get("scenario")
                .or_else(|| item.get("probe"))
                .and_then(|value| value.as_str())
                .unwrap_or("");
            if let Some(state) = item["state"].as_str() {
                owners.insert((state.to_owned(), name.to_owned()));
            }
        }
    }
    for (state, owner) in REQUIRED {
        assert!(
            owners.contains(&(state.to_owned(), owner.to_owned())),
            "the state `{state}` is not demonstrated by `{owner}`, which is the case \
             that owns it. A state carried by some other case is a label, not a \
             demonstration. Observed owners: {owners:?}"
        );
    }

    // The two states Quoin's audit produces are additionally required to carry
    // the finding that produced them, so the label cannot outlive the finding.
    let probes = report["adapter_probes"].as_array().unwrap();
    for (probe_name, kind) in [
        ("audit-reports-a-suspect-link", "suspect-link"),
        ("audit-reports-a-vacuous-run", "vacuous-evidence"),
    ] {
        let probe = probes
            .iter()
            .find(|item| item["probe"] == probe_name)
            .unwrap_or_else(|| panic!("the probe {probe_name} did not run"));
        let count = probe["detail"][kind].as_u64().unwrap_or(0);
        assert!(
            count > 0,
            "{probe_name} reports no `{kind}` finding, so the state it claims to \
             demonstrate was not produced by anything: {probe:#}"
        );
    }

    // Every negative names the positive control that proves the step it refuses
    // is a step that works.
    let controls = report["controls"].as_array().unwrap();
    assert!(!controls.is_empty(), "no positive controls were run");
    let negatives: BTreeSet<&str> = controls
        .iter()
        .map(|control| control["pairs_with"].as_str().unwrap())
        .collect();
    for required in [
        "retained-bytes-changed-after-sealing",
        "refuse-an-edited-receipt",
        "stale-candidate-binding",
        "attested-failed",
        "malformed-formula-is-reported-as-malformed",
        "declared-unsupported-case-is-reported-unsupported",
        "differential-comparison-is-not-a-boolean",
    ] {
        assert!(
            negatives.contains(required),
            "the negative {required} has no positive control"
        );
    }
}

// Trace: TC-023, FR-006-AC-6, StR-002-VC-1, StR-002-VC-2, SUITE-006
#[test]
fn the_r2u2_differential_is_a_comparison_and_never_a_boolean() {
    let inputs = assurance_inputs_guard();
    let report = chain_report(&inputs);

    // The counts come from the two corpus manifests, so a producer that stopped
    // reporting a state cannot also move the number it is checked against.
    let declared_malformed = report["declared_malformed_fixtures"].as_u64().unwrap();
    let declared_unsupported = report["declared_unsupported_cases"].as_u64().unwrap();
    let declared_supported = report["declared_supported_cases"].as_u64().unwrap();
    assert_eq!(
        declared_malformed, 3,
        "the shared corpus manifest declares {declared_malformed} invalid fixtures; if a \
         fixture was added or removed this expectation should move deliberately"
    );
    assert_eq!(
        declared_unsupported, 1,
        "the R2U2 corpus manifest declares {declared_unsupported} unsupported cases"
    );
    assert_eq!(
        declared_supported, 8,
        "the R2U2 corpus manifest declares {declared_supported} comparable cases"
    );
    assert_eq!(
        report["malformed_rows"].as_u64().unwrap(),
        declared_malformed
    );
    assert_eq!(
        report["unsupported_rows"].as_u64().unwrap(),
        declared_unsupported
    );

    // Three comparison classifications and four external-monitor states, all
    // observed. This is the assertion that stops the differential collapsing
    // into a bit: a producer that only ever emitted `agreement` would satisfy
    // every count above and fail here.
    let comparisons: BTreeSet<&str> = report["observed_comparisons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert_eq!(
        comparisons,
        BTreeSet::from(["agreement", "mismatch", "non_conclusive"]),
        "the differential reported {comparisons:?}; agreement, mismatch and \
         non-conclusive are three answers and all three have to have been observed"
    );
    let external: BTreeSet<&str> = report["observed_external_states"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert_eq!(
        external,
        BTreeSet::from(["conclusive", "pending", "tool_error", "unsupported"]),
        "the differential observed {external:?}; pending, unsupported and tool_error \
         are three different reasons to be non-conclusive and none may be folded away"
    );

    // The facts the chain asserts, each named, so that dropping any one of them
    // is visible here rather than only inside the driver.
    let scenarios = report["scenarios"].as_array().unwrap();
    for required in [
        "malformed-formula-is-reported-as-malformed",
        "malformed-does-not-fail-its-proof",
        "declared-unsupported-case-is-reported-unsupported",
        "differential-comparison-is-not-a-boolean",
        "external-monitor-states-stay-separate",
        "every-supported-case-was-compared",
        "differential-states-survive-into-retained-bytes",
    ] {
        let found = scenarios
            .iter()
            .find(|item| item["scenario"] == required)
            .unwrap_or_else(|| panic!("the scenario {required} did not run"));
        assert_eq!(
            found["matched"], true,
            "{required} did not match: {found:#}"
        );
    }

    // Neither state is a failure: the proofs they belong to are attested `passed`.
    for proof in ["PROOF-reference-conformance", "PROOF-r2u2-differential"] {
        assert_eq!(
            report["attested_results"][proof], "passed",
            "{proof} was attested {}",
            report["attested_results"][proof]
        );
    }

    // And they are not silent passes either: the producer's own rows say
    // `unsupported`, and the adapter carries that word alongside Quoin's
    // three-valued entry outcome rather than discarding it.
    let (code, stdout, stderr) = run(
        Path::new("python3"),
        &[
            "scripts/assurance_chain.py",
            "--adapt",
            "target/assurance/r2u2-differential.jsonl",
        ],
    );
    assert_eq!(code, 0, "the adapter refused the real stream: {stderr}");
    let adapted: Value = serde_json::from_str(&stdout).expect("the adapter emits JSON");
    let carried = adapted["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["domainOutcome"] == "unsupported")
        .count() as u64;
    assert_eq!(
        carried, declared_unsupported,
        "the adapter dropped the unsupported domain outcome; Quoin's entry vocabulary \
         is three-valued, so the twelve-state word has to survive alongside it"
    );
}

// Trace: TC-017, NFR-002-AC-3, SUITE-008
#[test]
fn every_requirement_tagged_test_is_a_test_cargo_compiles_and_runs() {
    // Deliberately unguarded: this census touches neither `target/assurance`
    // nor `requirements-assurance.txt`.
    let report = json_gate(
        Path::new("python3"),
        &["scripts/rust_test_census.py", "--json"],
    );
    assert_eq!(report["matched"], true, "{report:#}");
    let tagged = report["tagged"].as_array().unwrap();
    let compiled = report["compiled"].as_array().unwrap();
    assert!(
        report["ignored"].as_array().unwrap().is_empty(),
        "a requirement-tagged Rust test is ignored: {}",
        report["ignored"]
    );
    // A census over an empty tagged set compares nothing and passes. The floor
    // is asserted so the gate cannot become vacuous by deletion.
    assert!(
        tagged.len() >= 20,
        "the requirement-tagged test set is unexpectedly small ({}); a census over \
         too few tests asserts almost nothing",
        tagged.len()
    );
    assert_eq!(
        tagged.len(),
        compiled.len(),
        "tagged and compiled test sets differ in size"
    );

    // And the census must be seen to refuse. Without this the comparison is
    // indistinguishable from one that always agrees with itself.
    let (code, stdout, stderr) = run(
        Path::new("python3"),
        &[
            "-c",
            "import sys,json;sys.path.insert(0,'scripts');\
             import rust_test_census as m;\
             print(json.dumps(sorted(m.listed_test_names('a::b: test\\nc::d: test\\n')[0])))",
        ],
    );
    assert_eq!(code, 0, "the census parser probe failed: {stderr}");
    let parsed: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(
        parsed,
        vec!["b".to_owned(), "d".to_owned()],
        "the census does not parse cargo's own --list output; it would compare an \
         empty observed set against an empty expected set and agree"
    );
}

// Trace: TC-024, FR-006-AC-7, NFR-003-AC-1
#[test]
fn no_local_evidence_framework_remains() {
    let root = root();
    const DELETED_REFERENCES: [&str; 10] = [
        "check-failure-propagation",
        "check-tool-identities",
        "ci-for-evidence",
        "verify-evidence",
        "evidence-tool",
        "legacy_evidence_view",
        "compat-view",
        "COMPAT_RESULT",
        "tl-mltl-evidence-input-v1.schema.json",
        "tl-mltl-evidence-manifest-v1.schema.json",
    ];
    // The expected sets come from the change declaration rather than an
    // adjacent copy of the implementation literals. This keeps the authorial
    // statement and the executable census separately reviewable.
    let declaration: Value = serde_json::from_slice(
        &fs::read(root.join("assurance/change-assurance.json"))
            .expect("read change-assurance declaration for census controls"),
    )
    .expect("change-assurance declaration is JSON");
    let expected_deleted_references: Vec<&str> = declaration["census_controls"]
        ["deleted_reference_needles"]
        .as_array()
        .expect("census_controls.deleted_reference_needles is an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("every deleted-reference needle is a string")
        })
        .collect();
    assert_eq!(
        DELETED_REFERENCES.as_slice(),
        expected_deleted_references,
        "the executable deleted-reference needles differ from the change declaration"
    );

    // The generic machinery is gone, by name.
    for removed in [
        "scripts/build_evidence_envelope.py",
        "scripts/collect_evidence.sh",
        "scripts/finalize_collection.py",
        "scripts/verify_evidence.sh",
        "scripts/evidence_profile.py",
        "scripts/check_failure_propagation.py",
        "scripts/parameter_identity.py",
        "scripts/run_policy_tests.py",
        "scripts/tool_identity.py",
        "scripts/validate_json_schema.py",
        "scripts/test_evidence_tool.py",
        "scripts/test_failure_propagation.py",
        "scripts/test_tool_identity.py",
        "tools.lock",
        "tests/evidence_contract.rs",
        // Released for the pre-stable phase by the owner decision recorded in
        // agent-ix/engineering-assurance#7 and deleted under
        // agent-ix/tl-mltl#16. The reader, its fixtures, the two frozen schemas
        // and the retained records themselves go together: a tree that still
        // holds any one of them has not made the deletion it claims to have.
        "evidence",
        "schemas",
        "scripts/legacy_evidence_view.py",
        "tests/fixtures/legacy-compat",
    ] {
        assert!(
            !root.join(removed).exists(),
            "{removed} is still present; the generic evidence machinery was not removed"
        );
    }

    // Enumerate the repository by Git identity, not by an extension allow-list.
    // The scan covers tracked and untracked-not-ignored paths; population and
    // area controls are tracked-only so local scratch files cannot pad them.
    let denied = |path: &str| matches!(path, "Cargo.lock" | "LICENSE-APACHE" | "LICENSE-MIT");
    for included in [
        "GNUmakefile",
        "makefile",
        "compat.mk",
        ".github/workflows/probe.yaml",
        "scripts/LICENSE_scanner.py",
    ] {
        assert!(
            !denied(included),
            "the exact deny predicate widened to hide {included}"
        );
    }
    let (tracked_all, tracked) = census_paths(&root, denied);
    assert!(
        !path_has_legacy_compat("spec/current/review.md"),
        "the clean path fixture was classified as legacy compatibility"
    );
    assert!(
        path_has_legacy_compat("tests/fixtures/legacy-compat-v2/input.json"),
        "the hostile path fixture was not classified as legacy compatibility"
    );
    let proof_obligations = declaration["record"]["definition"]["proof_obligations"]
        .as_array()
        .expect("record.definition.proof_obligations is an array");
    assert!(
        !proof_obligations.is_empty(),
        "the proof-obligation census must not pass over an empty declaration"
    );
    assert!(
        !proof_obligations.iter().any(proof_has_legacy_compat),
        "a renamed legacy-compatibility proof obligation remains in the declaration"
    );
    let proof_fixture = serde_json::json!([
        {"proof_id": "PROOF-current"},
        {"proof_id": "PROOF-legacy-compat-v2"}
    ]);
    let proof_fixture = proof_fixture.as_array().expect("proof fixture is an array");
    assert!(
        !proof_has_legacy_compat(&proof_fixture[0]),
        "the clean proof fixture was classified as legacy compatibility"
    );
    assert!(
        proof_has_legacy_compat(&proof_fixture[1]),
        "the hostile proof fixture was not classified as legacy compatibility"
    );
    let observed_denied: BTreeSet<String> = tracked_all
        .iter()
        .filter(|entry| denied(entry))
        .cloned()
        .collect();
    let expected_denied: BTreeSet<String> = declaration["census_controls"]["denied_paths"]
        .as_array()
        .expect("census_controls.denied_paths is an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("every census denied path is a string")
                .to_owned()
        })
        .collect();
    assert_eq!(
        observed_denied, expected_denied,
        "the census deny-list no longer excludes exactly the three named lock or licence files"
    );

    let observed_areas = area_cardinalities(&tracked);
    let expected_areas: BTreeMap<String, usize> = [
        ("<root>", 13),
        (".agent", 1),
        (".github", 2),
        ("assurance", 3),
        // Issue #48 adds the 20-file retained tl-syntax future-operator corpus.
        ("corpus", 45),
        ("examples", 3),
        // PLAN-006 is the machine-readable owner/dependency manifest for the
        // issue #38 campaign: overview, index, log, and eight tasks.
        ("plan", 11),
        ("scripts", 5),
        // Current main plus M4's author and independent review sets contribute
        // 113 spec-area paths. Issue #39 adds 30 M5 requirements, measurements,
        // matrix, plan, campaign, and author-review artifacts; independent M5
        // review adds SR-062 through SR-069.
        ("spec", 151),
        // Context-bound wire decoding adds src/context.rs; the #57-shaped
        // fixture adds tests/contextual.rs. TC-030 itself extends an existing
        // shared-assurance test file.
        ("src", 8),
        // Issue #47 adds tests/future_parity.rs, the W/M parity controls; issue
        // #48 adds tests/future_interop.rs, the W/M export and loss controls.
        ("tests", 19),
    ]
    .into_iter()
    .map(|(area, count)| (area.to_owned(), count))
    .collect();
    assert_eq!(
        observed_areas, expected_areas,
        "the tracked per-area population changed; a new, missing, or moved path must be \
         classified deliberately"
    );

    // A total-only equality is blind to a compensating cross-area swap. Drive
    // that exact mutation against the cardinality helper so this control is
    // known to distinguish it while the total remains unchanged.
    let mut compensated = tracked.clone();
    let removed = compensated
        .iter()
        .position(|path| path.starts_with("tests/"))
        .expect("tracked test path for compensating-swap control");
    compensated.remove(removed);
    compensated.push("spec/compensating-swap-control.md".to_owned());
    assert_eq!(
        compensated.len(),
        tracked.len(),
        "the compensating-swap control did not preserve the total population"
    );
    assert_ne!(
        area_cardinalities(&compensated),
        expected_areas,
        "a cross-area file swap preserved both the total and the per-area control"
    );

    // Current main, the retained future/W-M corpus, M4 specifications and both
    // M4 review sets, PLAN-006, the 30 M5 authoring paths, and SR-062 through
    // SR-069 bring the reviewed population to 261 tracked paths.
    // Check it before taking the shared-input lock: ordinary reviewed source
    // growth must report its own census error without poisoning a mutex whose
    // recovery message is specifically about interrupted input mutation.
    let inspected = tracked.len();
    assert_eq!(
        inspected, 261,
        "the source census population changed from the reviewed 261 tracked files \
         ({inspected} observed); review the census scope and update this control deliberately"
    );

    // The byte census reads every tracked non-exempt file, including
    // `requirements-assurance.txt`; serialize that access with the probe that
    // temporarily rewrites the same shared input. Passing the private token to
    // the byte-scanning helpers makes this acquisition compile-time load-bearing.
    let inputs = assurance_inputs_guard();
    let scanned = scanned_paths(&root, &tracked);
    assert!(
        !scanned.iter().any(|path| path_has_legacy_compat(path)),
        "a renamed legacy-compatibility fixture path remains in the repository"
    );

    // Retained controls use the same enumeration, exemption and byte-scanning
    // functions as the real census. The fixture is its own Git repository, so a
    // preferred `GNUmakefile` can be exercised without changing which makefile a
    // concurrent command in this checkout selects.
    let fixture = std::env::temp_dir().join(format!(
        "tl-mltl-removal-census-fixture-{}",
        std::process::id()
    ));
    match fs::remove_dir_all(&fixture) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to clear the previous census fixture: {error}"),
    }
    let template = std::env::temp_dir().join(format!(
        "tl-mltl-removal-census-template-{}",
        std::process::id()
    ));
    match fs::remove_dir_all(&template) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to clear the previous census template: {error}"),
    }
    fs::create_dir_all(template.join("info")).expect("create hostile Git template control");
    fs::write(template.join("info/exclude"), "GNUmakefile\n")
        .expect("write hostile Git template exclude control");
    fs::create_dir_all(fixture.join(".github/workflows")).expect("create census fixture");
    let initialized = Command::new("git")
        .args(["init", "--quiet", "--template="])
        .env("GIT_TEMPLATE_DIR", &template)
        .current_dir(&fixture)
        .status()
        .expect("initialize census fixture repository");
    assert!(
        initialized.success(),
        "could not initialize census fixture repository"
    );
    assert!(
        !fixture.join(".git/info/exclude").exists(),
        "the empty-template override inherited a hostile Git template exclude"
    );
    let make_probe = fixture.join("GNUmakefile");
    fs::write(&make_probe, ".PHONY: compat-view\n").expect("write GNUmakefile census control");
    fs::write(
        fixture.join(".github/workflows/probe.yaml"),
        "name: census-control\n",
    )
    .expect("write yaml census control");

    // Stage `core.excludesFile` at repository-local scope, exercising the same
    // `--exclude-standard` path a developer-global setting would take. The
    // first enumeration must miss GNUmakefile for that reason; the second must
    // see it after this fixture's local configuration selects /dev/null.
    let fixture_excludes = fixture.join("fixture-global-excludes");
    fs::write(&fixture_excludes, "GNUmakefile\n").expect("write fixture excludes control");
    let inherited_excludes = Command::new("git")
        .args(["config", "core.excludesFile"])
        .arg(&fixture_excludes)
        .current_dir(&fixture)
        .status()
        .expect("stage census fixture core.excludesFile");
    assert!(
        inherited_excludes.success(),
        "could not stage the census fixture core.excludesFile"
    );
    let (_, fixture_tracked) = census_paths(&fixture, |_| false);
    let excluded_scanned = scanned_paths(&fixture, &fixture_tracked);
    assert!(
        !excluded_scanned.contains("GNUmakefile"),
        "the staged core.excludesFile did not hide GNUmakefile: {excluded_scanned:?}"
    );
    let isolated_excludes = Command::new("git")
        .args(["config", "core.excludesFile", "/dev/null"])
        .current_dir(&fixture)
        .status()
        .expect("isolate census fixture from global Git excludes");
    assert!(
        isolated_excludes.success(),
        "could not isolate the census fixture from global Git excludes"
    );
    let (_, fixture_tracked) = census_paths(&fixture, |_| false);
    let fixture_scanned = scanned_paths(&fixture, &fixture_tracked);
    let make_matches = census_matches(&inputs, &fixture, &make_probe, &DELETED_REFERENCES);

    let byte_probe = fixture.join("all-deleted-names.bin");
    let mut probe_bytes = expected_deleted_references.join("\n").into_bytes();
    probe_bytes.push(0xff);
    fs::write(&byte_probe, probe_bytes).expect("write raw-byte census control");
    let byte_matches = census_matches(&inputs, &fixture, &byte_probe, &DELETED_REFERENCES);

    let missing = fixture.join("cannot-be-read.py");
    let unreadable = std::panic::catch_unwind(|| {
        let _ = census_matches(&inputs, &fixture, &missing, &DELETED_REFERENCES);
    })
    .expect_err("an unreadable census path did not fail closed");
    let unreadable = panic_message(unreadable);

    let non_repository =
        std::env::temp_dir().join(format!("tl-mltl-census-nonrepo-{}", std::process::id()));
    match fs::remove_dir_all(&non_repository) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to clear the non-repository control: {error}"),
    }
    fs::create_dir_all(&non_repository).expect("create non-repository control");
    let unenumerable = std::panic::catch_unwind(|| {
        let _ = git_files(&non_repository, &["ls-files", "-z"]);
    })
    .expect_err("an unenumerable repository did not fail closed");
    let unenumerable = panic_message(unenumerable);
    fs::remove_dir_all(&non_repository).expect("remove non-repository control");
    fs::remove_dir_all(&fixture).expect("remove census fixture repository");
    fs::remove_dir_all(&template).expect("remove hostile Git template control");

    assert!(
        fixture_scanned.contains("GNUmakefile")
            && fixture_scanned.contains(".github/workflows/probe.yaml"),
        "the real Git enumeration omitted an extensionless makefile or .yaml workflow: \
         {fixture_scanned:?}"
    );
    assert_eq!(
        make_matches,
        vec!["compat-view"],
        "a preferred GNUmakefile can restore the deleted target without the census naming it"
    );
    assert_eq!(
        byte_matches, expected_deleted_references,
        "the raw-byte census did not exercise every deleted reference"
    );
    assert!(
        unreadable.contains("the census could not read")
            && unreadable.contains("cannot-be-read.py"),
        "the unreadable-file control failed for the wrong reason: {unreadable}"
    );
    assert!(
        unenumerable.contains("the census cannot enumerate the repository"),
        "the enumeration control failed for the wrong reason: {unenumerable}"
    );

    // The two exemption classes are narrow and tested through the same helper
    // the real scan calls. A `.py` under tasks or a `.md` under `src/reviews`
    // must never inherit the historical-prose exemption.
    assert_eq!(
        census_exemption("tests/shared_assurance.rs"),
        Some(CensusExemption::ExactDeclaration)
    );
    assert_eq!(
        census_exemption("assurance/change-assurance.json"),
        Some(CensusExemption::ExactDeclaration)
    );
    assert_eq!(
        census_exemption("spec/reviews/SR-011.md"),
        Some(CensusExemption::HistoricalProse)
    );
    assert_eq!(
        census_exemption("spec/plans/PLAN-002/tasks/Task-004.md"),
        Some(CensusExemption::HistoricalProse)
    );
    for not_exempt in [
        "spec/plans/PLAN-002/tasks/legacy_evidence_view.py",
        "src/reviews/reader.md",
        "spec/reviews/reader.rs",
        "Makefile",
    ] {
        assert_eq!(
            census_exemption(not_exempt),
            None,
            "the census exemption widened to hide {not_exempt}"
        );
    }

    let observed_exact_exemptions: BTreeSet<String> = scanned
        .iter()
        .filter(|relative| census_exemption(relative) == Some(CensusExemption::ExactDeclaration))
        .cloned()
        .collect();
    let expected_exact_exemptions: BTreeSet<String> = [
        "assurance/change-assurance.json",
        "tests/shared_assurance.rs",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    assert_eq!(
        observed_exact_exemptions, expected_exact_exemptions,
        "the exact census exemption set widened or lost one of its declarations"
    );

    let sources: Vec<PathBuf> = scanned.iter().map(|entry| root.join(entry)).collect();
    for path in &sources {
        let deleted_names = census_matches(&inputs, &root, path, &DELETED_REFERENCES);
        assert!(
            deleted_names.is_empty(),
            "{} references {}, which was deleted with the retained evidence",
            path.display(),
            deleted_names.join(", ")
        );
    }
    drop(inputs);

    // The Makefile is orchestration, not a trust root. Pin the disclosure text,
    // reject the live special targets it warns about, and require one literal
    // `ci` declaration. This is deliberately not a replacement for issue #14's
    // full execution-control qualification work.
    let makefile = fs::read_to_string(root.join("Makefile")).unwrap();
    assert!(
        makefile.contains("Adding a single `.IGNORE:` line to this file makes all 10 report")
            && makefile.contains("success and `make ci` exits 0. Nothing here notices."),
        "the Makefile no longer states the measured execution-control limitation"
    );
    for line in makefile.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        for forbidden in [
            ".IGNORE:",
            ".SILENT:",
            ".ONESHELL:",
            "SHELL",
            ".SHELLFLAGS",
            "MAKEFLAGS",
        ] {
            assert!(
                !trimmed.starts_with(forbidden),
                "the Makefile activates `{forbidden}` even though its disclosure says the \
                 execution-control class is unpoliced: {trimmed}"
            );
        }
    }
    let ci_declarations = makefile
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !trimmed.starts_with('#') && (trimmed.starts_with("ci:") || trimmed.starts_with("ci::"))
        })
        .count();
    assert_eq!(
        ci_declarations, 1,
        "the Makefile must carry exactly one literal ci declaration"
    );

    // And `ci` still names the work it claims to run. Dropping a prerequisite
    // from a composite removes a whole enforcement layer while every remaining
    // check stays green, which is a false green nothing else here would catch.
    //
    // The graph is expanded with `make -n` and read for the COMMANDS, not for
    // target names. `cargo test`, `cargo clippy`, `cargo fmt` and the MSRV
    // toolchain invocation are named specifically: checking only for the
    // producer scripts would prove nothing, because `assurance-inputs` supplies
    // those whether or not `test` is still a prerequisite of `ci`.
    let expansion = Command::new("make")
        .args(["-n", "ci"])
        .current_dir(&root)
        .output()
        .expect("make -n ci failed");
    let graph = String::from_utf8_lossy(&expansion.stdout).into_owned();
    assert!(
        expansion.status.success(),
        "make -n ci did not expand: {}",
        String::from_utf8_lossy(&expansion.stderr)
    );
    for required in [
        "cargo test --all-targets --all-features",
        "cargo clippy --all-targets --all-features",
        "cargo fmt --all -- --check",
        "cargo deny check licenses",
        // Split, because `$(CARGO)` expands to an absolute rustup path when
        // `make` runs under `cargo test` — which is exactly the environment
        // this assertion runs in.
        "rustup run 1.75.0",
        "cargo check --locked --all-targets --all-features",
        "scripts/assurance_chain.py",
        "scripts/check_shared_pins.py",
        "scripts/rust_test_census.py",
        "quire validate",
        "quire coverage",
        "check_unsafe_comments.sh",
        "sha256sum --check",
        "example reference_conformance",
        "example r2u2_differential",
        "example cli_conformance",
        // The build line that keeps `cli_conformance` from replaying a stale
        // binary. The producer cannot check this for itself without becoming a
        // builder, so the graph is what holds it.
        "build --quiet --bin tl-mltl",
        "RUSTDOCFLAGS=-Dwarnings",
    ] {
        assert!(
            graph.contains(required),
            "`make ci` no longer runs `{required}`; a composite that loses a \
             prerequisite loses an enforcement layer while everything else stays \
             green. Expansion was:\n{graph}"
        );
    }
}

// Trace: TC-022, FR-006-AC-5, NFR-003-AC-3
#[test]
fn a_control_naming_a_scenario_that_does_not_exist_is_refused() {
    let _inputs = assurance_inputs_guard();
    // NFR-003-AC-3 claims this guard is checked. The driver has it; nothing
    // exercised it, which is the same shape of gap the guard itself is there to
    // catch.
    //
    // The driver is copied and one `pairs_with` — and only that one — is
    // renamed. Renaming the scenario as well would leave the pairing consistent
    // and prove nothing.
    let scratch = root().join("target/dangling-probe");
    match fs::remove_dir_all(&scratch) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to clear the previous dangling-probe scratch: {error}"),
    }
    fs::create_dir_all(scratch.join("scripts")).unwrap();
    let driver = fs::read_to_string(root().join("scripts/assurance_chain.py")).unwrap();

    let control_marker =
        "        \"verify-accepts-an-unedited-receipt\",\n        \"refuse-an-edited-receipt\",";
    assert!(
        driver.contains(control_marker),
        "the control this probe renames is no longer present in the driver"
    );
    let mutated = driver.replacen(
        control_marker,
        "        \"verify-accepts-an-unedited-receipt\",\n        \"refuse-an-edited-receipt-typo\",",
        1,
    );
    assert_ne!(mutated, driver, "the mutation did not apply");
    fs::write(scratch.join("scripts/assurance_chain.py"), &mutated).unwrap();

    // Everything else the driver reads comes from the real tree. Every root
    // entry except `scripts` and `target` is symlinked, rather than an enumerated list, so
    // that a driver which starts reading a new directory does not turn this
    // probe into one that fails for an unrelated reason. The scratch owns
    // `target/assurance-store` and shares only the already-produced assurance
    // inputs; symlinking all of `target` coupled this probe to the real store.
    let scratch_target = scratch.join("target");
    for entry in fs::read_dir(root()).expect("repository root") {
        let path = entry.expect("directory entry").path();
        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_owned();
        if name == "scripts" || name == ".git" || name == "target" {
            continue;
        }
        std::os::unix::fs::symlink(&path, scratch.join(&name))
            .unwrap_or_else(|error| panic!("failed to link {name} into the probe: {error}"));
    }
    match fs::symlink_metadata(&scratch_target) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Ok(_) => panic!(
            "the dangling-scenario probe must own target/ rather than inherit a root-entry link"
        ),
        Err(error) => panic!("could not establish scratch target ownership: {error}"),
    }
    fs::create_dir_all(&scratch_target).expect("create isolated probe target");
    std::os::unix::fs::symlink(
        root().join("target/assurance"),
        scratch_target.join("assurance"),
    )
    .expect("share assurance inputs with the isolated probe");
    let revision = head_revision();
    let output = Command::new("python3")
        .args([
            "scripts/assurance_chain.py",
            "--candidate-revision",
            &revision,
        ])
        .current_dir(&scratch)
        .output()
        .expect("failed to run the mutated chain");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(2),
        "a control naming a non-existent scenario was not refused\n{stderr}"
    );
    assert!(
        stderr.contains("name a scenario that does not exist"),
        "the refusal did not name the cause: {stderr}"
    );
    assert_probe_store_isolated(&_inputs, &scratch_target, "the dangling-scenario probe");

    fs::write(scratch.join("scripts/assurance_chain.py"), &driver).unwrap();
    let bypassed = Command::new("python3")
        .args([
            "scripts/assurance_chain.py",
            "--candidate-revision",
            &revision,
        ])
        .current_dir(&scratch)
        .output()
        .expect("failed to run the unmutated chain in the isolated scratch");
    assert_eq!(
        bypassed.status.code(),
        Some(0),
        "the isolated scratch is not a valid environment for the unmutated chain:\n{}\n{}",
        String::from_utf8_lossy(&bypassed.stdout),
        String::from_utf8_lossy(&bypassed.stderr)
    );

    // `remove_dir_all` does not follow directory symlinks, but explicitly
    // unlinking every shared input keeps that safety boundary visible and stops
    // a future walk-and-delete replacement reaching repository inputs.
    fs::remove_file(scratch_target.join("assurance")).expect("unlink shared assurance inputs");
    for entry in fs::read_dir(&scratch).expect("read dangling-probe scratch") {
        let path = entry.expect("scratch entry").path();
        if fs::symlink_metadata(&path)
            .expect("scratch entry metadata")
            .file_type()
            .is_symlink()
        {
            fs::remove_file(path).expect("unlink dangling-probe repository input");
        }
    }
    fs::remove_dir_all(&scratch).expect("remove dangling-probe scratch");
}

fn mirror_scan_with_staged_requirement(_inputs: &AssuranceInputsGuard) -> (i32, String, String) {
    let python = assurance_python();
    run(
        &python,
        &[
            "-c",
            "import json,sys,pathlib;sys.path.insert(0,'scripts');\
             import check_shared_pins as m;\
             original=pathlib.Path('requirements-assurance.txt').read_text();\
             pathlib.Path('requirements-assurance.txt').write_text(\
             original+'\\n--registry=https://npm.ix/\\n');\
             pins=json.load(open('assurance/pins.json'));\
             found=m.mirror_references(pins);\
             pathlib.Path('requirements-assurance.txt').write_text(original);\
             print(json.dumps(found))",
        ],
    )
}

// Trace: TC-018, FR-006-AC-1, NFR-003-AC-1, SUITE-009
#[test]
fn the_mirror_scan_refuses_a_registry_reference_in_a_real_file() {
    let inputs = assurance_inputs_guard();
    // The structural branch of `mirror_references` (pins.json) already has a
    // control. The file-scan branch did not: it was never observed to fire, so
    // it was indistinguishable from a loop over files that never match.
    let (code, stdout, stderr) = mirror_scan_with_staged_requirement(&inputs);
    assert_eq!(code, 0, "the mirror file-scan probe failed: {stderr}");
    let offenders: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        offenders
            .iter()
            .any(|entry| entry.starts_with("requirements-assurance.txt:")),
        "a mirror reference written into a scanned FILE was not detected; the \
         file-scan branch matches nothing. Detected: {offenders:?}"
    );

    // And the file must be restored, or this test has dirtied the tree.
    let restored = fs::read_to_string(root().join("requirements-assurance.txt")).unwrap();
    assert!(
        !restored.contains("npm.ix/"),
        "the probe left a mirror reference in requirements-assurance.txt"
    );
}
