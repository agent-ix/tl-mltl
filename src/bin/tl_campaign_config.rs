//! Build explicit Quoin machine bindings for the source-bound TL campaign.
//! This is a trial configuration adapter, never a measurement producer.
//! Trace: FR-055-AC-1, TC-197.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Machine {
    schema: String,
    sources: BTreeMap<String, String>,
    tools: BTreeMap<String, Tool>,
    contracts: Contracts,
    environment: BTreeMap<String, String>,
    #[serde(default)]
    member_environments: BTreeMap<String, BTreeMap<String, String>>,
    timestamp: String,
    toolchains: BTreeMap<String, String>,
    #[serde(default)]
    member_toolchains: BTreeMap<String, BTreeMap<String, String>>,
    source_remotes: BTreeMap<String, String>,
}

fn validate_toolchains(toolchains: &BTreeMap<String, String>) -> Result<(), String> {
    if toolchains.is_empty() {
        return Err("machine toolchains must name at least one of node, rust, python".into());
    }
    for (language, identity) in toolchains {
        if !matches!(language.as_str(), "node" | "rust" | "python") {
            return Err(format!(
                "machine toolchains.{language} is unsupported; use node, rust, or python"
            ));
        }
        if identity.trim().is_empty() {
            return Err(format!("machine toolchains.{language} has no identity"));
        }
    }
    Ok(())
}

fn member_toolchains<'a>(
    defaults: &'a BTreeMap<String, String>,
    overrides: &'a BTreeMap<String, BTreeMap<String, String>>,
    member: &str,
) -> Result<&'a BTreeMap<String, String>, String> {
    let selected = overrides.get(member).unwrap_or(defaults);
    validate_toolchains(selected).map_err(|error| format!("{member}: {error}"))?;
    Ok(selected)
}

fn member_environment(
    defaults: &BTreeMap<String, String>,
    overrides: &BTreeMap<String, BTreeMap<String, String>>,
    member: &str,
) -> BTreeMap<String, String> {
    let mut selected = defaults.clone();
    if let Some(override_values) = overrides.get(member) {
        selected.extend(override_values.clone());
    }
    selected
}

fn observed_version(executable: &Path, argument: &str) -> Result<String, String> {
    let output = Command::new(executable)
        .arg(argument)
        .output()
        .map_err(|error| format!("{} {argument}: {error}", executable.display()))?;
    if !output.status.success() {
        return Err(format!("{} {argument} failed", executable.display()));
    }
    String::from_utf8(output.stdout).map_err(|error| error.to_string())
}

fn validate_rust_binding(
    member: &str,
    procedure: &Procedure,
    toolchains: &BTreeMap<String, String>,
    environment: &BTreeMap<String, String>,
    tools: &BTreeMap<String, Tool>,
) -> Result<(), String> {
    if procedure.producer_name != "cargo" && !procedure.producer_name.starts_with("cargo-") {
        return Ok(());
    }
    for name in ["RUSTC", "PATH"] {
        if !procedure.environment.iter().any(|entry| {
            entry.name == name && entry.kind == "runtime" && entry.value == format!("host:{name}")
        }) {
            return Err(format!(
                "{member}: Cargo-family procedure must bind host:{name}"
            ));
        }
    }
    let identity = toolchains.get("rust").ok_or(format!(
        "{member}: Rust producer has no rust toolchain identity"
    ))?;
    let rustc = Path::new(
        environment
            .get("RUSTC")
            .ok_or(format!("{member}: Rust producer has no RUSTC"))?,
    );
    if !rustc.is_absolute() {
        return Err(format!("{member}: RUSTC must be absolute"));
    }
    let path = environment
        .get("PATH")
        .ok_or(format!("{member}: Rust producer has no PATH"))?;
    let resolved = std::env::split_paths(path)
        .map(|directory| directory.join("rustc"))
        .find(|candidate| candidate.is_file())
        .ok_or(format!("{member}: PATH does not resolve rustc"))?;
    if resolved.canonicalize().map_err(|error| error.to_string())?
        != rustc.canonicalize().map_err(|error| error.to_string())?
    {
        return Err(format!(
            "{member}: PATH resolves a different rustc than RUSTC"
        ));
    }
    let verbose = observed_version(rustc, "-vV")?;
    let field = |name: &str| {
        verbose
            .lines()
            .find_map(|line| line.strip_prefix(name))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or(format!("{member}: RUSTC -vV has no {name}"))
    };
    let release = field("release:")?;
    let host = field("host:")?;
    if identity != &format!("rustc {release} ({host})") {
        return Err(format!(
            "{member}: rust toolchain identity differs from observed RUSTC release/host"
        ));
    }
    if procedure.producer_name == "cargo" {
        if procedure.producer_version != release {
            return Err(format!(
                "{member}: Cargo procedure version differs from RUSTC release"
            ));
        }
        let key = format!("cargo@{}", procedure.producer_version);
        let cargo = tools
            .get(&key)
            .ok_or(format!("{member}: missing machine tool {key}"))?;
        let observed = observed_version(Path::new(&cargo.executable), "--version")?;
        if observed.split_whitespace().nth(1) != Some(procedure.producer_version.as_str()) {
            return Err(format!(
                "{member}: Cargo executable version differs from procedure"
            ));
        }
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Tool {
    executable: String,
    digest: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Contracts {
    caller: Value,
    containment: Value,
    response_protocol: Value,
    response_adapter: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Definition {
    source_graph: Vec<Source>,
    members: Vec<Member>,
}

#[derive(Deserialize)]
struct Source {
    repository: String,
    revision: String,
    digest: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Closure {
    schema: String,
    campaign_definition: String,
    sources: Vec<ClosureSource>,
    equal_harness: Vec<Harness>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClosureSource {
    repository: String,
    revision: String,
    tree_digest: String,
    cargo_lock_digest: Option<String>,
    cargo_manifest_digest: Option<String>,
    tl_git_dependencies: Vec<LockedDependency>,
}

#[derive(Deserialize)]
struct LockedDependency {
    source: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Harness {
    repository: String,
    baseline_revision: String,
    candidate_revision: String,
    identical_files: Vec<HarnessFile>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HarnessFile {
    path: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Member {
    name: String,
    plan_id: String,
    definition_version: String,
    checker_procedure: Option<Procedure>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Procedure {
    producer_name: String,
    producer_version: String,
    source_repository: String,
    #[serde(default)]
    environment: Vec<ProcedureEnvironment>,
    #[serde(default)]
    inputs: Vec<Artifact>,
    #[serde(default)]
    outputs: Vec<Artifact>,
    #[serde(default)]
    output_trees: Vec<Artifact>,
    response_protocol: String,
    response_adapter: String,
    response_adapter_version: String,
    timeout_millis: u64,
}

#[derive(Deserialize)]
struct ProcedureEnvironment {
    name: String,
    kind: String,
    value: String,
}

#[derive(Deserialize)]
struct Artifact {
    role: String,
    required: bool,
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value[key]
        .as_str()
        .ok_or_else(|| format!("missing contract {key}"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn git_output(checkout: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(checkout)
        .args(args)
        .output()
        .map_err(|error| format!("git {}: {error}", checkout.display()))?;
    if !output.status.success() {
        return Err(format!("git {} {:?} failed", checkout.display(), args));
    }
    Ok(output.stdout)
}

fn verify_checkout(checkout: &Path, source: &Source) -> Result<(), String> {
    let head = git_output(checkout, &["rev-parse", "HEAD"])?;
    if head.strip_suffix(b"\n") != Some(source.revision.as_bytes()) {
        return Err(format!("{} checkout revision changed", source.repository));
    }
    if !git_output(
        checkout,
        &["status", "--porcelain", "--untracked-files=all"],
    )?
    .is_empty()
    {
        return Err(format!("{} checkout is dirty", source.repository));
    }
    let tree = git_output(
        checkout,
        &["ls-tree", "-r", "-z", "--full-tree", &source.revision],
    )?;
    if sha256(&tree) != source.digest {
        return Err(format!("{} tree digest changed", source.repository));
    }
    Ok(())
}

fn verify_closure(
    definition: &Definition,
    machine: &Machine,
    closure: &Closure,
) -> Result<(), String> {
    if closure.schema != "tl-mltl.campaign-source-closure/v1"
        || closure.campaign_definition != "campaign/stage1-campaign-definition.json"
        || closure.sources.len() != definition.source_graph.len()
        || closure.equal_harness.len() != 3
    {
        return Err("source closure shape differs from CampaignDefinition".into());
    }
    let expected = definition
        .source_graph
        .iter()
        .map(|source| {
            (
                source.repository.as_str(),
                (source.revision.as_str(), source.digest.as_str()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if expected.len() != closure.sources.len() {
        return Err("duplicate source graph or closure alias".into());
    }
    let mut seen_sources = BTreeSet::new();
    for source in &closure.sources {
        if !seen_sources.insert(&source.repository) {
            return Err(format!("duplicate closure source {}", source.repository));
        }
        if expected.get(source.repository.as_str())
            != Some(&(source.revision.as_str(), source.tree_digest.as_str()))
        {
            return Err(format!(
                "{} closure identity differs from source graph",
                source.repository
            ));
        }
        let checkout = Path::new(
            machine
                .sources
                .get(&source.repository)
                .ok_or(format!("missing checkout {}", source.repository))?,
        );
        if !checkout.is_absolute() {
            return Err(format!("{} checkout must be absolute", source.repository));
        }
        verify_checkout(
            checkout,
            &Source {
                repository: source.repository.clone(),
                revision: source.revision.clone(),
                digest: source.tree_digest.clone(),
            },
        )?;
        if let Some(digest) = &source.cargo_manifest_digest {
            let manifest = fs::read(checkout.join("Cargo.toml"))
                .map_err(|error| format!("{} Cargo.toml: {error}", source.repository))?;
            if sha256(&manifest) != *digest {
                return Err(format!("{} Cargo.toml changed", source.repository));
            }
        } else if source.repository != "r2u2" {
            return Err(format!("{} has no Cargo.toml digest", source.repository));
        }
        if let Some(digest) = &source.cargo_lock_digest {
            let lock = fs::read(checkout.join("Cargo.lock"))
                .map_err(|error| format!("{} Cargo.lock: {error}", source.repository))?;
            if sha256(&lock) != *digest {
                return Err(format!("{} Cargo.lock changed", source.repository));
            }
            let text = String::from_utf8(lock).map_err(|error| error.to_string())?;
            let mut actual = text
                .lines()
                .filter_map(|line| {
                    line.trim()
                        .strip_prefix("source = \"git+https://github.com/agent-ix/tl-")
                        .and_then(|tail| tail.strip_suffix('"'))
                        .map(|tail| format!("git+https://github.com/agent-ix/tl-{tail}"))
                })
                .collect::<Vec<_>>();
            let mut declared = source
                .tl_git_dependencies
                .iter()
                .map(|dep| dep.source.clone())
                .collect::<Vec<_>>();
            actual.sort();
            declared.sort();
            if actual != declared {
                return Err(format!(
                    "{} TL lock dependencies changed",
                    source.repository
                ));
            }
        } else if source.repository != "r2u2" || !source.tl_git_dependencies.is_empty() {
            return Err(format!(
                "{} has unreviewed absent Cargo.lock",
                source.repository
            ));
        }
    }
    if seen_sources.len() != expected.len() {
        return Err("source closure is incomplete".into());
    }
    let mut seen_harnesses = BTreeSet::new();
    for harness in &closure.equal_harness {
        if !seen_harnesses.insert(&harness.repository) {
            return Err(format!("duplicate harness {}", harness.repository));
        }
        let candidate = expected
            .get(harness.repository.as_str())
            .ok_or(format!("missing candidate {}", harness.repository))?;
        let baseline_name = format!("{}-baseline", harness.repository);
        let baseline = expected
            .get(baseline_name.as_str())
            .ok_or(format!("missing baseline {baseline_name}"))?;
        if candidate.0 != harness.candidate_revision
            || baseline.0 != harness.baseline_revision
            || harness.identical_files.is_empty()
        {
            return Err(format!("{} harness revision mismatch", harness.repository));
        }
        let mut seen_files = BTreeSet::new();
        for file in &harness.identical_files {
            if !seen_files.insert(&file.path) {
                return Err(format!(
                    "{} duplicate harness file {}",
                    harness.repository, file.path
                ));
            }
            for repo_name in [&harness.repository, &baseline_name] {
                let checkout = Path::new(
                    machine
                        .sources
                        .get(repo_name)
                        .ok_or(format!("missing checkout {repo_name}"))?,
                );
                let bytes = fs::read(checkout.join(&file.path))
                    .map_err(|error| format!("{repo_name}/{}: {error}", file.path))?;
                if sha256(&bytes) != file.sha256 {
                    return Err(format!("{repo_name}/{} harness bytes changed", file.path));
                }
            }
        }
    }
    Ok(())
}

fn procedure_path(member: &Member) -> Result<PathBuf, String> {
    let (group, suffix) = member.name.split_once('.').ok_or("invalid member name")?;
    let slug = if group == "V10" {
        suffix.replace('.', "-")
    } else {
        suffix.replace('_', "-")
    };
    Ok(PathBuf::from(format!(
        "campaign/procedures/{}-{slug}.json",
        group.to_lowercase()
    )))
}

fn load_procedure(repo: &Path, member: &Member) -> Result<Procedure, String> {
    let path = repo.join(procedure_path(member)?);
    let bytes = fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))
}

fn validate_plan(repo: &Path, member: &Member) -> Result<PathBuf, String> {
    let directory = repo.join("spec/assurance");
    let mut matching = fs::read_dir(&directory)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with(&format!("{}-", member.plan_id)) && name.ends_with(".md")
                })
        });
    let path = matching
        .next()
        .ok_or(format!("plan {} is absent", member.plan_id))?;
    if matching.next().is_some() {
        return Err(format!("plan {} is ambiguous", member.plan_id));
    }
    let contents = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let expected_path = procedure_path(member)?.to_string_lossy().to_string();
    if !contents
        .lines()
        .any(|line| line == format!("execution_procedure: {expected_path}"))
        || !contents
            .lines()
            .any(|line| line == format!("definition_version: {}", member.definition_version))
    {
        return Err(format!(
            "{} does not bind procedure and version",
            path.display()
        ));
    }
    Ok(path)
}

fn verify_control_file(control: &Path, measured: &Path, relative: &Path) -> Result<(), String> {
    let control_path = control.join(relative);
    let measured_path = measured.join(relative);
    let control_bytes =
        fs::read(&control_path).map_err(|error| format!("{}: {error}", control_path.display()))?;
    let measured_bytes = fs::read(&measured_path)
        .map_err(|error| format!("{}: {error}", measured_path.display()))?;
    if control_bytes != measured_bytes {
        return Err(format!(
            "control and measured tl-mltl differ at {}",
            relative.display()
        ));
    }
    Ok(())
}

fn checker_module_paths(root: &Path) -> Result<BTreeSet<PathBuf>, String> {
    let mut pending = vec![PathBuf::from("src/bin/tl_campaign_check")];
    let mut files = BTreeSet::new();
    while let Some(relative) = pending.pop() {
        for entry in fs::read_dir(root.join(&relative))
            .map_err(|error| format!("{}: {error}", root.join(&relative).display()))?
        {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = relative.join(entry.file_name());
            let kind = entry.file_type().map_err(|error| error.to_string())?;
            if kind.is_dir() {
                pending.push(path);
            } else if kind.is_file() {
                files.insert(path);
            } else {
                return Err(format!(
                    "checker source is not a regular file: {}",
                    path.display()
                ));
            }
        }
    }
    if files.is_empty() {
        return Err("checker module inventory is empty".into());
    }
    Ok(files)
}

fn verify_checker_sources(control: &Path, measured: &Path) -> Result<(), String> {
    verify_control_file(control, measured, Path::new("src/bin/tl_campaign_check.rs"))?;
    let control_modules = checker_module_paths(control)?;
    if control_modules != checker_module_paths(measured)? {
        return Err("control and measured checker module inventories differ".into());
    }
    for module in control_modules {
        verify_control_file(control, measured, &module)?;
    }
    Ok(())
}

fn output_path(role: &str) -> Result<&'static str, String> {
    match role {
        "crash" => Ok("crash.bin"),
        "coverage" => Ok("coverage.json"),
        "manifest" => Ok("v10-manifest.json"),
        "binary" => Ok("v10-compiled.bin"),
        "example" => Ok(".quoin-target/debug/examples/fuzz_campaign"),
        "verdict" => Ok(".quoin-campaign/domain-verdict.json"),
        "mutants" => Ok("mutants.out"),
        "criterion" => Ok(".quoin-target/criterion"),
        "inputs" => Ok("v10"),
        _ => Err(format!("unreviewed output role {role}")),
    }
}

fn binding(
    procedure: &Procedure,
    machine: &Machine,
    revisions: &BTreeMap<String, String>,
    selected_environment: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let tool_key = format!("{}@{}", procedure.producer_name, procedure.producer_version);
    let tool = machine
        .tools
        .get(&tool_key)
        .ok_or(format!("missing machine tool {tool_key}"))?;
    if !Path::new(&tool.executable).is_absolute() || tool.digest.len() != 64 {
        return Err(format!("invalid executable identity for {tool_key}"));
    }
    let metadata = fs::symlink_metadata(&tool.executable).map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 1_073_741_824 {
        return Err(format!(
            "{tool_key} must name a bounded regular executable file"
        ));
    }
    let executable_bytes = fs::read(&tool.executable).map_err(|error| error.to_string())?;
    if format!("{:x}", Sha256::digest(&executable_bytes)) != tool.digest {
        return Err(format!("{tool_key} executable digest changed"));
    }
    let revision = revisions
        .get(&procedure.source_repository)
        .ok_or(format!("source {} absent", procedure.source_repository))?;
    if text(&machine.contracts.response_protocol, "kind")? != procedure.response_protocol
        || text(&machine.contracts.response_adapter, "kind")? != procedure.response_adapter
        || text(&machine.contracts.response_adapter, "version")?
            != procedure.response_adapter_version
    {
        return Err("machine response contract differs from authored procedure".into());
    }
    let mut environment = BTreeMap::new();
    for entry in &procedure.environment {
        match entry.kind.as_str() {
            "literal" => {
                environment.insert(entry.name.clone(), entry.value.clone());
            }
            "runtime" if entry.value.starts_with("host:") => {
                let value = selected_environment
                    .get(&entry.name)
                    .ok_or(format!("missing explicit host environment {}", entry.name))?;
                environment.insert(entry.name.clone(), value.clone());
            }
            "runtime" if entry.value.starts_with("source:") => {
                let repository = entry
                    .value
                    .strip_prefix("source:")
                    .and_then(|value| value.strip_suffix(".revision"))
                    .ok_or(format!("unknown source selector {}", entry.value))?;
                let revision = revisions
                    .get(repository)
                    .ok_or(format!("missing source revision {repository}"))?;
                environment.insert(entry.name.clone(), revision.clone());
            }
            _ => return Err(format!("unsupported environment selector {}", entry.value)),
        }
    }
    let outputs = procedure.outputs.iter().map(|artifact| {
        Ok(json!({"role":artifact.role,"path":output_path(&artifact.role)?,"required":artifact.required}))
    }).collect::<Result<Vec<_>, String>>()?;
    let output_trees = procedure.output_trees.iter().map(|artifact| {
        Ok(json!({"role":artifact.role,"path":output_path(&artifact.role)?,"required":artifact.required}))
    }).collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "producer": {"name":procedure.producer_name,"version":procedure.producer_version,
            "sourceRevision":revision,"executable":tool.executable,"executableDigest":tool.digest},
        "caller":machine.contracts.caller,
        "environment":environment,
        "outputs":outputs,
        "outputTrees":output_trees,
        "stdin":{"kind":"null"},
        "containment":{"profile":"process-group-v1","contract":machine.contracts.containment},
        "cancellation":{"kind":"disabled"},
        "budget":{"timeoutMillis":procedure.timeout_millis,"maxStdoutBytes":8388608,
            "maxStderrBytes":8388608,"maxInputBytes":1073741824,"maxOutputArtifacts":4096,
            "maxOutputBytes":1073741824,"maxDescendants":4096,"maxConcurrency":1},
        "responseProtocol":machine.contracts.response_protocol,
        "responseAdapter":machine.contracts.response_adapter,
        "exitCodes":{"kind":"any"}
    }))
}

fn selected_inputs(member: &Member, procedure: &Procedure) -> Result<Vec<Value>, String> {
    let mut result = Vec::new();
    for artifact in &procedure.inputs {
        let role = artifact.role.as_str();
        let (source, path, executable) = if member.name == "V8.parse_default" && role == "example" {
            (
                json!({"kind":"dependency","member":"V8.parse_example_prep","index":1,"artifactRole":"example"}),
                ".quoin-target/debug/examples/fuzz_campaign".to_owned(),
                true,
            )
        } else if let Some(case) = member.name.strip_prefix("V10.compile.") {
            let extension = if role == "spec" {
                "c2po"
            } else if role == "map" {
                "map"
            } else {
                "csv"
            };
            let source = if matches!(case, "bounded" | "past" | "unsafe-since") {
                let path = match (case, role) {
                    ("bounded", "spec") => "corpus/r2u2-v4.2/formulas.c2po",
                    ("bounded", "map") => "corpus/r2u2-v4.2/signals.map",
                    ("past", "spec") => "corpus/past-c2po-v1/target-4.2/past.c2po",
                    ("past", "trace") => "corpus/past-c2po-v1/target-4.2/trace.csv",
                    ("unsafe-since", "spec") => "corpus/past-c2po-v1/target-4.2/unsafe-since.c2po",
                    ("unsafe-since", "trace") => "corpus/past-c2po-v1/target-4.2/unsafe-since.csv",
                    _ => return Err(format!("unreviewed V10 source input {case}/{role}")),
                };
                json!({"kind":"source_file","repository":"tl-mltl","path":path})
            } else {
                json!({"kind":"dependency","member":"V10.inputs","index":1,
                    "artifactRole":format!("inputs/{case}.{extension}")})
            };
            (source, format!("v10/input.{extension}"), false)
        } else if let Some(case) = member.name.strip_prefix("V10.monitor.") {
            if role == "binary" {
                (
                    json!({"kind":"dependency","member":format!("V10.compile.{case}"),"index":1,
                    "artifactRole":"binary"}),
                    "v10/compiled.bin".into(),
                    false,
                )
            } else {
                let source = if matches!(case, "bounded" | "past" | "unsafe-since") {
                    let path = match case {
                        "bounded" => "corpus/r2u2-v4.2/trace.csv",
                        "past" => "corpus/past-c2po-v1/target-4.2/trace.csv",
                        _ => "corpus/past-c2po-v1/target-4.2/unsafe-since.csv",
                    };
                    json!({"kind":"source_file","repository":"tl-mltl","path":path})
                } else {
                    json!({"kind":"dependency","member":"V10.inputs","index":1,
                        "artifactRole":format!("inputs/{case}.csv")})
                };
                (source, "v10/trace.csv".into(), false)
            }
        } else {
            return Err(format!("unreviewed input {role} for {}", member.name));
        };
        result.push(json!({"role":role,"path":path,"executable":executable,"source":source}));
    }
    Ok(result)
}

fn build(repo: &Path, definition: Definition, machine: Machine) -> Result<Value, String> {
    if machine.schema != "tl-mltl.campaign-machine/v1" {
        return Err("wrong TL machine schema".into());
    }
    if !machine.toolchains.is_empty() {
        validate_toolchains(&machine.toolchains)?;
    }
    let member_names = definition
        .members
        .iter()
        .map(|member| member.name.as_str())
        .collect::<BTreeSet<_>>();
    for name in machine.member_toolchains.keys() {
        if !member_names.contains(name.as_str()) {
            return Err(format!(
                "machine toolchain override names unknown member {name}"
            ));
        }
    }
    for name in machine.member_environments.keys() {
        if !member_names.contains(name.as_str()) {
            return Err(format!(
                "machine environment override names unknown member {name}"
            ));
        }
    }
    let revisions = definition
        .source_graph
        .iter()
        .map(|source| (source.repository.clone(), source.revision.clone()))
        .collect::<BTreeMap<_, _>>();
    if revisions.len() != definition.source_graph.len()
        || machine.sources.keys().collect::<Vec<_>>() != revisions.keys().collect::<Vec<_>>()
    {
        return Err("machine source aliases differ from CampaignDefinition".into());
    }
    let measured = Path::new(
        machine
            .sources
            .get("tl-mltl")
            .ok_or("missing tl-mltl source")?,
    );
    let control = repo.canonicalize().map_err(|error| error.to_string())?;
    let measured_canonical = measured.canonicalize().map_err(|error| error.to_string())?;
    if control == measured_canonical {
        return Err("control checkout must differ from measured tl-mltl checkout".into());
    }
    let mut source_paths = BTreeSet::new();
    for (alias, path) in &machine.sources {
        let canonical = Path::new(path)
            .canonicalize()
            .map_err(|error| error.to_string())?;
        if !source_paths.insert(canonical) {
            return Err(format!("{alias} reuses another source checkout"));
        }
    }
    verify_checker_sources(repo, measured)?;
    let mut members = BTreeMap::new();
    for member in definition.members {
        let toolchains = member_toolchains(
            &machine.toolchains,
            &machine.member_toolchains,
            &member.name,
        )?;
        let environment = member_environment(
            &machine.environment,
            &machine.member_environments,
            &member.name,
        );
        let plan_path = validate_plan(repo, &member)?;
        let plan_relative = plan_path
            .strip_prefix(repo)
            .map_err(|error| error.to_string())?;
        verify_control_file(repo, measured, plan_relative)?;
        verify_control_file(repo, measured, &procedure_path(&member)?)?;
        let procedure = load_procedure(repo, &member)?;
        let producer_binding = binding(&procedure, &machine, &revisions, &environment)?;
        validate_rust_binding(
            &member.name,
            &procedure,
            toolchains,
            &environment,
            &machine.tools,
        )?;
        let checker = member
            .checker_procedure
            .as_ref()
            .ok_or(format!("{} has no checker", member.name))?;
        let environment_sources = procedure
            .environment
            .iter()
            .filter_map(|entry| {
                entry.value.strip_prefix("source:").map(|value| {
                    let repo = value
                        .strip_suffix(".revision")
                        .ok_or(format!("unknown source selector {value}"))?;
                    Ok((
                        entry.name.clone(),
                        json!({"kind":"source_revision","repository":repo}),
                    ))
                })
            })
            .collect::<Result<BTreeMap<_, _>, String>>()?;
        let selected = selected_inputs(&member, &procedure)?;
        let row = json!({
            "producer":producer_binding,
            "checker":binding(checker,&machine,&revisions,&environment)?,
            "inputs":selected,
            "timestamp":machine.timestamp,
            "toolchains":toolchains,
            "sourceRemotes":machine.source_remotes,
            "environmentSources":environment_sources
        });
        if members.insert(member.name.clone(), row).is_some() {
            return Err(format!("duplicate member {}", member.name));
        }
    }
    Ok(json!({"schema":"quoin.campaign-run-config/v1","sources":machine.sources,"members":members}))
}

fn run() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 7
        || args[1] != "--definition"
        || args[3] != "--machine"
        || args[5] != "--output"
    {
        return Err(
            "usage: tl_campaign_config --definition FILE --machine FILE --output FILE".into(),
        );
    }
    let definition_path = PathBuf::from(&args[2])
        .canonicalize()
        .map_err(|error| format!("definition {}: {error}", args[2]))?;
    let repo = definition_path
        .parent()
        .and_then(Path::parent)
        .ok_or("definition must be under campaign/")?;
    let definition: Definition =
        serde_json::from_slice(&fs::read(&definition_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let machine: Machine = serde_json::from_slice(&fs::read(&args[4]).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let closure: Closure = serde_json::from_slice(
        &fs::read(repo.join("campaign/source-closure.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    verify_closure(&definition, &machine, &closure)?;
    let config = build(repo, definition, machine)?;
    fs::write(
        &args[6],
        serde_json::to_vec_pretty(&config).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("tl_campaign_config: {error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::{
        binding, member_environment, member_toolchains, selected_inputs, sha256,
        validate_rust_binding, validate_toolchains, verify_checker_sources, verify_control_file,
        Contracts, Machine, Member, Procedure, Tool,
    };

    // Trace: FR-055-AC-1, TC-197
    #[test]
    fn v9_source_revision_environment_is_declared_for_quoin_replacement() {
        let procedure: Procedure = serde_json::from_str(include_str!(
            "../../campaign/procedures/v9-mltl-pair1-candidate.json"
        ))
        .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let executable = directory.path().join("cargo");
        std::fs::write(&executable, b"test executable").unwrap();
        let tool = Tool {
            executable: executable.to_str().unwrap().to_owned(),
            digest: sha256(b"test executable"),
        };
        let machine = Machine {
            schema: "tl-mltl.campaign-machine/v1".into(),
            sources: BTreeMap::new(),
            tools: BTreeMap::from([(
                format!("{}@{}", procedure.producer_name, procedure.producer_version),
                tool,
            )]),
            contracts: Contracts {
                caller: json!({"kind":"test.caller"}),
                containment: json!({"kind":"test.containment"}),
                response_protocol: json!({"kind":procedure.response_protocol}),
                response_adapter: json!({"kind":procedure.response_adapter,
                    "version":procedure.response_adapter_version}),
            },
            environment: procedure
                .environment
                .iter()
                .filter(|entry| entry.value.starts_with("host:"))
                .map(|entry| (entry.name.clone(), "/tmp".into()))
                .collect(),
            member_environments: BTreeMap::new(),
            timestamp: String::new(),
            toolchains: BTreeMap::new(),
            member_toolchains: BTreeMap::new(),
            source_remotes: BTreeMap::new(),
        };
        let revision = "a".repeat(40);
        let revisions = BTreeMap::from([("tl-mltl".to_owned(), revision.clone())]);
        let selected = binding(&procedure, &machine, &revisions, &machine.environment).unwrap();
        assert_eq!(selected["environment"]["TL_MLTL_SOURCE_REVISION"], revision);
        assert_eq!(selected["environment"]["TL_MLTL_SOURCE_STATE"], "clean");
    }

    // Trace: FR-055-AC-1, TC-197
    #[test]
    fn control_and_measured_procedure_bytes_must_match() {
        let control = tempfile::tempdir().unwrap();
        let measured = tempfile::tempdir().unwrap();
        std::fs::write(control.path().join("procedure.json"), b"version-1").unwrap();
        std::fs::write(measured.path().join("procedure.json"), b"version-2").unwrap();
        assert!(verify_control_file(
            control.path(),
            measured.path(),
            std::path::Path::new("procedure.json")
        )
        .is_err());
        std::fs::write(measured.path().join("procedure.json"), b"version-1").unwrap();
        assert!(verify_control_file(
            control.path(),
            measured.path(),
            std::path::Path::new("procedure.json")
        )
        .is_ok());
    }

    // Trace: FR-055-AC-1, TC-197
    #[test]
    fn control_and_measured_checker_module_inventory_and_bytes_must_match() {
        let control = tempfile::tempdir().unwrap();
        let measured = tempfile::tempdir().unwrap();
        for root in [control.path(), measured.path()] {
            let bin = root.join("src/bin");
            std::fs::create_dir_all(bin.join("tl_campaign_check")).unwrap();
            std::fs::write(bin.join("tl_campaign_check.rs"), b"mod replay;").unwrap();
            std::fs::write(bin.join("tl_campaign_check/replay.rs"), b"pub fn run() {}").unwrap();
        }
        assert!(verify_checker_sources(control.path(), measured.path()).is_ok());
        std::fs::write(
            measured.path().join("src/bin/tl_campaign_check/replay.rs"),
            b"pub fn run() { panic!() }",
        )
        .unwrap();
        assert!(verify_checker_sources(control.path(), measured.path()).is_err());
        std::fs::write(
            measured.path().join("src/bin/tl_campaign_check/replay.rs"),
            b"pub fn run() {}",
        )
        .unwrap();
        std::fs::write(
            measured.path().join("src/bin/tl_campaign_check/extra.rs"),
            b"pub fn extra() {}",
        )
        .unwrap();
        assert!(verify_checker_sources(control.path(), measured.path()).is_err());
    }

    // Trace: FR-055-AC-1, TC-197
    #[test]
    fn machine_toolchains_need_a_supported_language_identity() {
        assert!(validate_toolchains(&BTreeMap::new()).is_err());
        assert!(
            validate_toolchains(&BTreeMap::from([("rustc".into(), "rustc 1.98.1".into())]))
                .is_err()
        );
        assert!(validate_toolchains(&BTreeMap::from([("rust".into(), " ".into())])).is_err());
        assert!(validate_toolchains(&BTreeMap::from([
            (
                "rust".into(),
                "rustc 1.98.1 (aarch64-unknown-linux-gnu)".into()
            ),
            ("python".into(), "Python 3.13.11".into()),
        ]))
        .is_ok());
    }

    // Trace: FR-055-AC-1, TC-197
    #[test]
    fn member_toolchain_overrides_bind_the_selected_producer() {
        let defaults = BTreeMap::from([("rust".into(), "rustc 1.98.1".into())]);
        let overrides = BTreeMap::from([(
            "V7.miri".into(),
            BTreeMap::from([("rust".into(), "rustc nightly-2026-08-21".into())]),
        )]);
        assert_eq!(
            member_toolchains(&defaults, &overrides, "V1.oracle").unwrap(),
            &defaults
        );
        assert_eq!(
            member_toolchains(&defaults, &overrides, "V7.miri").unwrap(),
            &overrides["V7.miri"]
        );
        let invalid = BTreeMap::from([("V7.miri".into(), BTreeMap::new())]);
        assert!(member_toolchains(&defaults, &invalid, "V7.miri").is_err());
    }

    // Trace: FR-055-AC-4, TC-200
    #[cfg(unix)]
    #[test]
    fn rust_producer_refuses_contradictory_toolchain_and_environment() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().unwrap();
        let stable = directory.path().join("stable");
        let nightly = directory.path().join("nightly");
        for (path, release) in [(&stable, "1.98.1"), (&nightly, "1.100.0-nightly")] {
            std::fs::create_dir(path).unwrap();
            for (name, response) in [
                (
                    "rustc",
                    format!("release: {release}\nhost: x86_64-unknown-linux-gnu"),
                ),
                ("cargo", format!("cargo {release} (test)")),
            ] {
                let executable = path.join(name);
                std::fs::write(
                    &executable,
                    format!("#!/bin/sh\nprintf '%s\\n' '{response}'\n"),
                )
                .unwrap();
                std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755))
                    .unwrap();
            }
        }
        let stable_procedure: Procedure = serde_json::from_str(include_str!(
            "../../campaign/procedures/v7-embedded-core.json"
        ))
        .unwrap();
        let nightly_procedure: Procedure = serde_json::from_str(include_str!(
            "../../campaign/procedures/v8-parse-example-prep.json"
        ))
        .unwrap();
        let defaults = BTreeMap::from([
            ("PATH".into(), stable.to_string_lossy().into_owned()),
            (
                "RUSTC".into(),
                stable.join("rustc").to_string_lossy().into_owned(),
            ),
        ]);
        let overrides = BTreeMap::from([(
            "V8.parse_example_prep".into(),
            BTreeMap::from([
                ("PATH".into(), nightly.to_string_lossy().into_owned()),
                (
                    "RUSTC".into(),
                    nightly.join("rustc").to_string_lossy().into_owned(),
                ),
            ]),
        )]);
        let stable_environment = member_environment(&defaults, &overrides, "V7.embedded_core");
        let nightly_environment =
            member_environment(&defaults, &overrides, "V8.parse_example_prep");
        let stable_identity = BTreeMap::from([(
            "rust".into(),
            "rustc 1.98.1 (x86_64-unknown-linux-gnu)".into(),
        )]);
        let nightly_identity = BTreeMap::from([(
            "rust".into(),
            "rustc 1.100.0-nightly (x86_64-unknown-linux-gnu)".into(),
        )]);
        let tools = BTreeMap::from([
            (
                "cargo@1.98.1".into(),
                Tool {
                    executable: stable.join("cargo").to_string_lossy().into_owned(),
                    digest: String::new(),
                },
            ),
            (
                "cargo@1.100.0-nightly".into(),
                Tool {
                    executable: nightly.join("cargo").to_string_lossy().into_owned(),
                    digest: String::new(),
                },
            ),
        ]);
        assert!(validate_rust_binding(
            "V7.embedded_core",
            &stable_procedure,
            &stable_identity,
            &stable_environment,
            &tools
        )
        .is_ok());
        assert!(validate_rust_binding(
            "V8.parse_example_prep",
            &nightly_procedure,
            &nightly_identity,
            &nightly_environment,
            &tools
        )
        .is_ok());
        assert!(validate_rust_binding(
            "V7.embedded_core",
            &stable_procedure,
            &nightly_identity,
            &stable_environment,
            &tools
        )
        .is_err());
        assert!(validate_rust_binding(
            "V8.parse_example_prep",
            &nightly_procedure,
            &stable_identity,
            &nightly_environment,
            &tools
        )
        .is_err());
        let mut wrong_path = stable_environment.clone();
        wrong_path.insert("PATH".into(), nightly.to_string_lossy().into_owned());
        assert!(validate_rust_binding(
            "V7.embedded_core",
            &stable_procedure,
            &stable_identity,
            &wrong_path,
            &tools
        )
        .is_err());
        let mut wrong_cargo = stable_procedure;
        wrong_cargo.producer_version = "1.100.0-nightly".into();
        assert!(validate_rust_binding(
            "V7.embedded_core",
            &wrong_cargo,
            &stable_identity,
            &stable_environment,
            &tools
        )
        .is_err());
        let mut missing_binding = nightly_procedure;
        missing_binding
            .environment
            .retain(|entry| entry.name != "RUSTC");
        assert!(validate_rust_binding(
            "V8.parse_example_prep",
            &missing_binding,
            &nightly_identity,
            &nightly_environment,
            &tools
        )
        .is_err());
    }

    // Trace: FR-055-AC-1, TC-197
    #[test]
    fn v10_static_and_generated_inputs_have_distinct_source_bindings() {
        let bounded: Procedure = serde_json::from_str(include_str!(
            "../../campaign/procedures/v10-compile-bounded.json"
        ))
        .unwrap();
        let bounded_member = Member {
            name: "V10.compile.bounded".into(),
            plan_id: "MP-117".into(),
            definition_version: "tl.v10.compile-bounded/v1".into(),
            checker_procedure: None,
        };
        let selected = selected_inputs(&bounded_member, &bounded).unwrap();
        assert_eq!(selected[0]["source"]["kind"], "source_file");
        assert_eq!(
            selected[0]["source"]["path"],
            "corpus/r2u2-v4.2/formulas.c2po"
        );
        assert_eq!(
            selected[1]["source"]["path"],
            "corpus/r2u2-v4.2/signals.map"
        );

        let generated: Procedure = serde_json::from_str(include_str!(
            "../../campaign/procedures/v10-compile-zero-upper-all-true.json"
        ))
        .unwrap();
        let generated_member = Member {
            name: "V10.compile.zero-upper-all-true".into(),
            plan_id: "MP-087".into(),
            definition_version: "tl.v10.compile-zero-upper-all-true/v1".into(),
            checker_procedure: None,
        };
        let selected = selected_inputs(&generated_member, &generated).unwrap();
        assert_eq!(selected[0]["source"]["kind"], "dependency");
        assert_eq!(selected[0]["source"]["member"], "V10.inputs");
        assert_eq!(
            selected[0]["source"]["artifactRole"],
            "inputs/zero-upper-all-true.c2po"
        );
    }
}
