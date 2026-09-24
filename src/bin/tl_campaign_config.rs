//! Build explicit Quoin machine bindings for the source-bound TL campaign.
//! This is a trial configuration adapter, never a measurement producer.
//! Trace: FR-055-AC-1, TC-197.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
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
    timestamp: String,
    toolchains: BTreeMap<String, String>,
    source_remotes: BTreeMap<String, String>,
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

fn validate_plan(repo: &Path, member: &Member) -> Result<(), String> {
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
                let value = machine
                    .environment
                    .get(&entry.name)
                    .ok_or(format!("missing explicit host environment {}", entry.name))?;
                environment.insert(entry.name.clone(), value.clone());
            }
            "runtime" if entry.value.starts_with("source:") => {}
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
    let mut members = BTreeMap::new();
    for member in definition.members {
        validate_plan(repo, &member)?;
        let procedure = load_procedure(repo, &member)?;
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
            "producer":binding(&procedure,&machine,&revisions)?,
            "checker":binding(checker,&machine,&revisions)?,
            "inputs":selected,
            "timestamp":machine.timestamp,
            "toolchains":machine.toolchains,
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
    let definition_path = PathBuf::from(&args[2]);
    let repo = definition_path
        .parent()
        .and_then(Path::parent)
        .ok_or("definition must be under campaign/")?;
    let definition: Definition =
        serde_json::from_slice(&fs::read(&definition_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let machine: Machine = serde_json::from_slice(&fs::read(&args[4]).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
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
    use super::{selected_inputs, Member, Procedure};

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
