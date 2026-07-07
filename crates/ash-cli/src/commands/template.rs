//! App template commands.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};

use crate::commands::check::{CheckArgs, CheckOutputFormat, check};
use crate::error::{CliError, CliResult};
use crate::templates::{TemplateManifest, validate_template_manifest};

/// Template command arguments.
#[derive(Args, Debug, Clone)]
pub struct TemplateArgs {
    /// Template subcommand.
    #[command(subcommand)]
    pub command: TemplateCommand,
}

/// Template subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum TemplateCommand {
    /// Instantiate a validated template manifest.
    Instantiate(TemplateInstantiateArgs),
}

/// Arguments for `ash template instantiate`.
#[derive(Args, Debug, Clone)]
pub struct TemplateInstantiateArgs {
    /// Path to a JSON template manifest.
    #[arg(long, value_name = "PATH")]
    pub manifest: PathBuf,
    /// Output directory for generated files.
    #[arg(long, value_name = "DIR")]
    pub out: PathBuf,
    /// Template parameter in key=value form. Repeat for multiple parameters.
    #[arg(long = "param", value_name = "KEY=VALUE")]
    pub params: Vec<String>,
    /// Allow replacing existing generated files.
    #[arg(long)]
    pub overwrite: bool,
}

/// Run the template command.
pub fn template(args: &TemplateArgs) -> CliResult<()> {
    match &args.command {
        TemplateCommand::Instantiate(args) => instantiate(args),
    }
}

fn instantiate(args: &TemplateInstantiateArgs) -> CliResult<()> {
    let manifest = load_manifest(&args.manifest)?;
    validate_template_manifest(&manifest)
        .map_err(|err| CliError::general(format!("template manifest validation failed: {err}")))?;

    let params = validate_params(&manifest, &args.params)?;
    write_template_files(&manifest, &params, &args.out, args.overwrite)?;
    run_generated_checks(&manifest, &args.out)?;
    Ok(())
}

fn load_manifest(path: &Path) -> CliResult<TemplateManifest> {
    let text = fs::read_to_string(path)
        .map_err(|err| CliError::io("read template manifest", Some(path.to_path_buf()), err))?;
    serde_json::from_str(&text).map_err(|err| CliError::parse("parse template manifest JSON", err))
}

fn validate_params(
    manifest: &TemplateManifest,
    raw_params: &[String],
) -> CliResult<BTreeMap<String, String>> {
    let declared = manifest
        .parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut values = BTreeMap::new();

    for raw in raw_params {
        let Some((key, value)) = raw.split_once('=') else {
            return Err(CliError::general(format!(
                "template parameter `{raw}` must use key=value form"
            )));
        };
        if !declared.contains(key) {
            return Err(CliError::general(format!(
                "template parameter `{key}` is not declared by manifest"
            )));
        }
        values.insert(key.to_string(), value.to_string());
    }

    for parameter in &manifest.parameters {
        if !values.contains_key(&parameter.name) {
            if let Some(default) = &parameter.default {
                values.insert(parameter.name.clone(), default.clone());
            } else if parameter.required {
                return Err(CliError::general(format!(
                    "required template parameter `{}` was not provided",
                    parameter.name
                )));
            }
        }
    }

    Ok(values)
}

fn write_template_files(
    manifest: &TemplateManifest,
    params: &BTreeMap<String, String>,
    out: &Path,
    overwrite: bool,
) -> CliResult<()> {
    for file in &manifest.files {
        let target = out.join(&file.path);
        if target.exists() && !overwrite {
            return Err(CliError::general(format!(
                "refusing to overwrite existing file {}",
                target.display()
            )));
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                CliError::io(
                    "create template output directory",
                    Some(parent.to_path_buf()),
                    err,
                )
            })?;
        }
        fs::write(&target, substitute_params(&file.content, params))
            .map_err(|err| CliError::io("write template output file", Some(target.clone()), err))?;
        println!("created {}", file.path);
    }
    Ok(())
}

fn substitute_params(content: &str, params: &BTreeMap<String, String>) -> String {
    let mut out = content.to_string();
    for (key, value) in params {
        out = out.replace(&format!("{{{{{key}}}}}"), value);
    }
    out
}

fn run_generated_checks(manifest: &TemplateManifest, out: &Path) -> CliResult<()> {
    for generated_check in &manifest.generated_checks {
        let Some(relative) = generated_check.command.strip_prefix("ash check ") else {
            return Err(CliError::general(format!(
                "unsupported generated check command `{}`",
                generated_check.command
            )));
        };
        let path = out.join(relative.trim());
        let check_args = CheckArgs {
            path: path.display().to_string(),
            all: false,
            strict: false,
            format: CheckOutputFormat::Human,
            policy_check: false,
            proof_fuel: ash_typeck::DEFAULT_PROOF_FUEL,
        };
        check(&check_args)?;
    }
    Ok(())
}
