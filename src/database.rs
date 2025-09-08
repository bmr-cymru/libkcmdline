use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};
use crate::parameter::{Parameter, ParameterProcessor};
use crate::catalog::VersionInfo;
use crate::error::DatabaseError;

pub trait ParameterSource {
    fn list_parameters(&self) -> Result<Vec<String>, DatabaseError>;
    fn get_parameter_definition(&self, name: &str) -> Result<Option<ParameterDefinitionRaw>, DatabaseError>;
    fn get_parameter_versions(&self, name: &str) -> Result<Option<ParameterVersionsRaw>, DatabaseError>;
    fn get_subparameters(&self, parent: &str) -> Result<Vec<String>, DatabaseError>;
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParameterDefinitionRaw {
    pub name: String,
    pub processor: String,
    pub description: String,
    pub deprecated: Option<bool>,
    pub selectors: Option<Vec<String>>,
    pub syntax: SyntaxDefinitionRaw,
    pub distributions: Option<HashMap<String, DistributionSupportRaw>>,
    pub examples: Option<ExamplesRaw>,
    pub documentation: Option<DocumentationLinksRaw>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SyntaxDefinitionRaw {
    #[serde(rename = "type")]
    pub validator_type: String,
    pub format: String,
    #[serde(flatten)]
    pub config: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VendorVersionRaw {
    pub introduced: Option<String>,
    pub commit: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComponentVersionRaw {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionInfoRaw {
    pub introduced: Option<String>,
    pub commit: Option<String>,
    pub last_modified: Option<String>,
    pub last_modified_commit: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParameterVersionsRaw {
    pub mainline: Option<VersionInfoRaw>,
    pub vendors: Option<HashMap<String, HashMap<String, VendorVersionRaw>>>,
    pub components: Option<HashMap<String, ComponentVersionRaw>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DistributionSupportRaw {
    pub min_version: Option<String>,
    pub max_version: Option<String>,
    pub component_version: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamplesRaw {
    pub valid: Vec<String>,
    pub invalid: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationLinksRaw {
    pub kernel_org: Option<String>,
    pub man_pages: Vec<String>,
    pub distribution_docs: HashMap<String, String>,
}


// Other Raw types follow similar pattern...

pub struct DatabaseLoader {
    sources: Vec<Box<dyn ParameterSource>>,
    cache: Option<LoadedDatabase>,
}

pub struct LoadedDatabase {
    parameters: HashMap<String, Parameter>,
    subparameter_index: HashMap<String, Vec<String>>,
    processor_index: HashMap<ParameterProcessor, Vec<String>>,
}

impl DatabaseLoader {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            cache: None,
        }
    }

    pub fn with_embedded(mut self) -> Self {
        self.sources.push(Box::new(EmbeddedDatabase::new()));
        self
    }

    pub fn with_filesystem<P: AsRef<Path>>(mut self, path: P) -> Result<Self, DatabaseError> {
        let fs_source = FilesystemDatabase::new(path)?;
        self.sources.push(Box::new(fs_source));
        Ok(self)
    }

    pub fn load(&mut self) -> Result<&LoadedDatabase, DatabaseError> {
        if self.cache.is_none() {
            self.cache = Some(self.build_database()?);
        }
        Ok(self.cache.as_ref().unwrap())
    }

    pub fn reload(&mut self) -> Result<&LoadedDatabase, DatabaseError> {
        self.cache = None;
        self.load()
    }

    fn build_database(&self) -> Result<LoadedDatabase, DatabaseError> {
        let mut all_param_names = std::collections::HashSet::new();
        let mut parameters = HashMap::new();
        let mut subparameter_index = HashMap::new();

        // Collect all parameter names from all sources
        for source in &self.sources {
            for name in source.list_parameters()? {
                all_param_names.insert(name);
            }
        }

        // Load each parameter, with later sources overriding earlier ones
        for param_name in all_param_names {
            if let Some(parameter) = self.load_parameter(&param_name)? {
                // Build subparameter relationships
                if let Some(parent) = self.get_parent_parameter(&param_name) {
                    subparameter_index
                        .entry(parent)
                        .or_insert_with(Vec::new)
                        .push(param_name.clone());
                }

                parameters.insert(param_name, parameter);
            }
        }

        // Build processor index
        let mut processor_index: HashMap<ParameterProcessor, Vec<String>> = HashMap::new();
        for (name, param) in &parameters {
            processor_index
                .entry(param.processor.clone())
                .or_insert_with(Vec::new)
                .push(name.clone());
        }

        Ok(LoadedDatabase {
            parameters,
            subparameter_index,
            processor_index,
        })
    }

    fn load_parameter(&self, name: &str) -> Result<Option<Parameter>, DatabaseError> {
        let mut definition: Option<ParameterDefinitionRaw> = None;
        let mut versions: Option<ParameterVersionsRaw> = None;

        // Load from sources in order, later sources override earlier ones
        for source in &self.sources {
            if let Some(def) = source.get_parameter_definition(name)? {
                definition = Some(def);
            }
            if let Some(ver) = source.get_parameter_versions(name)? {
                versions = Some(ver);
            }
        }

        if let Some(def) = definition {
            let parameter = self.convert_raw_parameter(def, versions)?;
            Ok(Some(parameter))
        } else {
            Ok(None)
        }
    }

    fn convert_raw_parameter(
        &self,
        raw_def: ParameterDefinitionRaw,
        _raw_versions: Option<ParameterVersionsRaw>,
    ) -> Result<Parameter, DatabaseError> {
        let processor = self.parse_processor(&raw_def.processor)?;

        Ok(Parameter {
            name: raw_def.name,
            processor,
            description: raw_def.description,
            deprecated: raw_def.deprecated.unwrap_or(false),
            selectors: raw_def.selectors.unwrap_or_default(),
            syntax: crate::parameter::SyntaxDefinition {
                validator_type: raw_def.syntax.validator_type,
                format: raw_def.syntax.format,
                config: raw_def.syntax.config,
            },
            distributions: self.convert_distributions(raw_def.distributions)?,
            examples: self.convert_examples(raw_def.examples)?,
            documentation: self.convert_documentation(raw_def.documentation)?,
        })
    }

    fn parse_processor(&self, processor_str: &str) -> Result<ParameterProcessor, DatabaseError> {
        match processor_str {
            "kernel" => Ok(ParameterProcessor::Kernel),
            "systemd" => Ok(ParameterProcessor::Systemd { min_version: "219".to_string() }),
            "dracut" => Ok(ParameterProcessor::Dracut { min_version: "011".to_string() }),
            "initramfs-tools" => Ok(ParameterProcessor::InitramfsTools),
            "plymouth" => Ok(ParameterProcessor::Plymouth),
            "grub" => Ok(ParameterProcessor::Grub),
            _ => Err(DatabaseError::FormatError(format!("Unknown processor: {}", processor_str))),
        }
    }

    fn get_parent_parameter(&self, param_name: &str) -> Option<String> {
        // Extract parent from path like "pci/resource_alignment" -> "pci"
        param_name.rfind('/').map(|idx| param_name[..idx].to_string())
    }

    // Helper methods for converting raw types to final types...
    fn convert_distributions(&self, _raw: Option<HashMap<String, DistributionSupportRaw>>) -> Result<HashMap<String, crate::parameter::DistributionSupport>, DatabaseError> {
        // Implementation
        todo!()
    }

    fn convert_examples(&self, _raw: Option<ExamplesRaw>) -> Result<crate::parameter::Examples, DatabaseError> {
        // Implementation
        todo!()
    }

    fn convert_documentation(&self, _raw: Option<DocumentationLinksRaw>) -> Result<Option<crate::parameter::DocumentationLinks>, DatabaseError> {
        // Implementation
        todo!()
    }
}

impl LoadedDatabase {
    pub fn get_parameter(&self, name: &str) -> Option<&Parameter> {
        self.parameters.get(name)
    }

    pub fn get_subparameters(&self, parent: &str) -> Vec<&Parameter> {
        self.subparameter_index
            .get(parent)
            .map(|children| {
                children.iter()
                    .filter_map(|name| self.parameters.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn parameters_by_processor(&self, processor: &ParameterProcessor) -> Vec<&Parameter> {
        self.processor_index
            .get(processor)
            .map(|names| {
                names.iter()
                    .filter_map(|name| self.parameters.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn all_parameters(&self) -> impl Iterator<Item = &Parameter> {
        self.parameters.values()
    }
}

// Embedded database source (uses build.rs generated data)
pub struct EmbeddedDatabase {
    // This would be populated by build.rs
}

impl EmbeddedDatabase {
    pub fn new() -> Self {
        Self {}
    }
}

impl ParameterSource for EmbeddedDatabase {
    fn list_parameters(&self) -> Result<Vec<String>, DatabaseError> {
        // Return compiled parameter list
        Ok(include!(concat!(env!("OUT_DIR"), "/parameter_names.rs")))
    }

    fn get_parameter_definition(&self, name: &str) -> Result<Option<ParameterDefinitionRaw>, DatabaseError> {
        // Load from embedded data
        let definitions: HashMap<&str, &str> = include!(concat!(env!("OUT_DIR"), "/compiled_db.rs"));

        if let Some(toml_str) = definitions.get(name) {
            let def: ParameterDefinitionRaw = toml::from_str(toml_str)
                .map_err(|e| DatabaseError::FormatError(format!("Parse error for {}: {}", name, e)))?;
            Ok(Some(def))
        } else {
            Ok(None)
        }
    }

    fn get_parameter_versions(&self, _name: &str) -> Result<Option<ParameterVersionsRaw>, DatabaseError> {
        // Similar to definitions but for versions
        todo!()
    }

    fn get_subparameters(&self, _parent: &str) -> Result<Vec<String>, DatabaseError> {
        // Return embedded subparameter list
        todo!()
    }
}

// Filesystem database source
pub struct FilesystemDatabase {
    root_path: PathBuf,
}

impl FilesystemDatabase {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, DatabaseError> {
        let root_path = path.as_ref().to_path_buf();
        if !root_path.exists() {
            return Err(DatabaseError::LoadError(format!("Database path does not exist: {:?}", root_path)));
        }
        Ok(Self { root_path })
    }

    fn get_parameter_path(&self, name: &str) -> PathBuf {
        self.root_path.join("parameters").join(name)
    }
}

impl ParameterSource for FilesystemDatabase {
    fn list_parameters(&self) -> Result<Vec<String>, DatabaseError> {
        let mut parameters = Vec::new();
        let params_dir = self.root_path.join("parameters");

        if params_dir.exists() {
            self.collect_parameters_recursive(&params_dir, "", &mut parameters)?;
        }

        Ok(parameters)
    }

    fn get_parameter_definition(&self, name: &str) -> Result<Option<ParameterDefinitionRaw>, DatabaseError> {
        let def_path = self.get_parameter_path(name).join("definition.toml");

        if def_path.exists() {
            let content = fs::read_to_string(&def_path)
                .map_err(|e| DatabaseError::LoadError(format!("Failed to read {:?}: {}", def_path, e)))?;

            let def: ParameterDefinitionRaw = toml::from_str(&content)
                .map_err(|e| DatabaseError::FormatError(format!("Parse error in {:?}: {}", def_path, e)))?;

            Ok(Some(def))
        } else {
            Ok(None)
        }
    }

    fn get_parameter_versions(&self, name: &str) -> Result<Option<ParameterVersionsRaw>, DatabaseError> {
        let versions_path = self.get_parameter_path(name).join("versions.toml");

        if versions_path.exists() {
            let content = fs::read_to_string(&versions_path)
                .map_err(|e| DatabaseError::LoadError(format!("Failed to read {:?}: {}", versions_path, e)))?;

            let versions: ParameterVersionsRaw = toml::from_str(&content)
                .map_err(|e| DatabaseError::FormatError(format!("Parse error in {:?}: {}", versions_path, e)))?;

            Ok(Some(versions))
        } else {
            Ok(None)
        }
    }

    fn get_subparameters(&self, parent: &str) -> Result<Vec<String>, DatabaseError> {
        let parent_path = self.get_parameter_path(parent);
        let mut subparams = Vec::new();

        if parent_path.exists() {
            for entry in fs::read_dir(&parent_path)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        let full_name = if parent.is_empty() {
                            name.to_string()
                        } else {
                            format!("{}/{}", parent, name)
                        };
                        subparams.push(full_name);
                    }
                }
            }
        }

        Ok(subparams)
    }
}

impl FilesystemDatabase {
    fn collect_parameters_recursive(
        &self,
        dir: &Path,
        prefix: &str,
        parameters: &mut Vec<String>,
    ) -> Result<(), DatabaseError> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    let full_name = if prefix.is_empty() {
                        name.to_string()
                    } else {
                        format!("{}/{}", prefix, name)
                    };

                    // Check if this directory has a definition.toml
                    if path.join("definition.toml").exists() {
                        parameters.push(full_name.clone());
                    }

                    // Recurse into subdirectories
                    self.collect_parameters_recursive(&path, &full_name, parameters)?;
                }
            }
        }
        Ok(())
    }
}
