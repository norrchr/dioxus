use anyhow::Context;
use clap::{Parser, Subcommand};
use git2::{FetchOptions, Repository, build::RepoBuilder};
use serde::{Serialize, Deserialize};
use std::{collections::{HashMap, HashSet}, path::PathBuf};
use crate::{DioxusConfig, Result, StructuredOutput, Workspace};

mod tui;

#[derive(Clone, Debug, Parser)]
#[command(name = "icons")]
pub struct IconCommand {
    #[command(subcommand)]
    pub command: Option<IconSubcommand>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum IconSubcommand {
    /// Add icon(s) to your project
    Add {
        #[clap(flatten)]
        icons: IconArgs,

        /// Overwrite the icon(s) if they already exist in your module_path
        #[clap(long, short, default_value = "false")]
        force: bool,
    },

    /// Remove icon(s)
    Remove {
        #[clap(flatten)]
        icons: IconArgs,
    },
}

#[derive(Clone, Debug, Parser, Serialize)]
pub struct IconArgs {
    /// The icon(s) to add or remove - [@registry:]library:list-of-icons...
    #[clap(required = true)]
    icons: Vec<String>,

    /// The default registry to use
    #[clap(long)]
    registry: Option<String>,

    /// The location of the icons module in your project (default: src/icons)
    #[clap(long)]
    module_path: Option<PathBuf>,
}

impl IconCommand {
    pub async fn run(self) -> Result<StructuredOutput> {
        match self.command {
            Some(IconSubcommand::Add { icons: args, force }) => {
                Self::add_icons(args, force).await?
            },
            Some(IconSubcommand::Remove { icons: args }) => {
                Self::remove_icons(args).await?
            },

            None => {
                // terminal icon browser (tui)
                tui::tui_main()?;
            }
        }

        Ok(StructuredOutput::Success)
    }

    async fn add_icons(args: IconArgs, force: bool) -> Result<()> {        
        let config = Self::resolve_config().await?;
        let mut icons_to_add: HashMap<String, HashMap<String, HashSet<String>>> = HashMap::new();
        let mut icon_count = 0;

        for icon in args.icons {
            // Format: @registry:library:list-of-icons...
            let parsed_icon = Self::parse_icon_identifier(icon)?;
            
            let registry = if let Some(r) = parsed_icon.registry {
                Some(r) 
            } else if let Some(r) = &args.registry {
                Some(r.clone())
            } else {
                None
            };

            let resolved_registry_name = IconRegistry::resolve_valid_name(registry.as_deref(), &config)?;
            icon_count += parsed_icon.icons.len();

            icons_to_add
                .entry(resolved_registry_name)
                .or_insert(HashMap::new())
                .entry(parsed_icon.library)
                .or_insert(HashSet::new())
                .extend(parsed_icon.icons);
        }

        println!("Parsed {} icon(s) across {} registries and {} libraries", icon_count, icons_to_add.len(), icons_to_add.values().map(|v| v.len()).sum::<usize>());

        let mut resolved_registry_paths = HashMap::new();

        // resolve the registry paths
        for registry in icons_to_add.keys() {
            println!("Resolving registry '{}'...", registry);
            resolved_registry_paths.insert(registry, IconRegistry::resolve(Some(registry), &config)?);
        }

        // Todo:
        // - check if provided icon names exist in their respective registries
        // - process icon data from registry
        // - write the icon data out to the managed icon module

        // managed icon module structure
        // src/icons/mod.rs - top level module for all icons (exports registeries)
        // src/icons/registry/mod.rs - top level registry module (exports libraries)
        // src/icons/registry/library/mod.rs - top level library module (exports icons)
        // src/icons/registry/library/name.rs - module per icon

        Ok(())
    }

    async fn remove_icons(args: IconArgs) -> Result<()> {
        todo!()
    }

    fn parse_icon_identifier(input: String) -> Result<IconIdentifier> {
        let mut parts = input.split(':').collect::<Vec<_>>();
        let mut registry = None;
        
        if parts.len() == 3 {
            if let Some(reg) = parts[0].strip_prefix("@") {
                let reg = reg.trim();
                if reg.is_empty() {
                    return Err(anyhow::anyhow!("Empty registry name. Expected [@registry:]library:list-of-icons, got '{}'", input))
                }
                registry = Some(reg.to_string());
            } else {
                return Err(anyhow::anyhow!("Registry must start with '@', got '{}'", parts[0]))
            }

            parts = parts[1..].to_vec();
        }

        if parts.len() == 2 {
            let library = parts[0].trim().to_string();
            let icons = parts[1].split(",").map(|i| i.trim().to_string()).filter(|i| !i.is_empty()).collect::<Vec<_>>();

            if library.is_empty() || icons.is_empty() {
                return Err(anyhow::anyhow!("Empty library and/or icon list. Expected [@registry:]library:list-of-icons, got '{}'", input))
            }

            return Ok(IconIdentifier {
                registry,
                library,
                icons
            })
        }

        Err(anyhow::anyhow!("Invalid icon identifier. Expected [@registry:]library:list-of-icons, got '{}'", input))
    }

    /// Load the config
    async fn resolve_config() -> Result<DioxusConfig> {
        let workspace = Workspace::current().await?;

        let crate_package = workspace.find_main_package(None)?;

        Ok(workspace
            .load_dioxus_config(crate_package)?
            .unwrap_or_default())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IconIdentifier {
    pub registry: Option<String>,
    pub library: String,
    pub icons: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IconRegistry {
    /// Url of git repository
    pub git: Option<String>,

    /// Git revision to checkout
    pub rev: Option<String>,

    /// Depth of git clone
    pub depth: Option<i32>,

    /// Path to local registry
    pub path: Option<PathBuf>,
}

impl Default for IconRegistry {
    fn default() -> Self {
        Self {
            git: Some("https://github.com/iconify/icon-sets.git".to_string()),
            rev: None,
            depth: Some(1),
            path: None,
        }
    }
}

impl IconRegistry {
    /// Check if the supplied name is the built-in global registry name
    pub fn is_default_name(name: &str) -> bool {
        name == "iconify"
    }

    /// Resolve the provided name to a registry name, checking if it's a valid entry in the config
    pub fn resolve_valid_name(name: Option<&str>, config: &DioxusConfig) -> Result<String> {
        let name = Self::resolve_name(name, config);

        if Self::is_default_name(&name) || config.icons.registeries.contains_key(&name) {
            return Ok(name.to_string())
        }
        
        Err(anyhow::anyhow!("Icon registry '{}' does not exist in the config", name))
    }

    /// Resolve the provided name to a registry name (does not check if it's a valid entry in the config)
    pub fn resolve_name(name: Option<&str>, config: &DioxusConfig) -> String {
        // If a name is provided, and it's not the global default, use it
        if let Some(name) = name.filter(|n| !Self::is_default_name(n))  {
            return name.to_string()
        }
        
        // If the config has a default, and it's not the global default, use it
        if let Some(default) = config.icons.default.as_deref().filter(|d| !Self::is_default_name(d)) {
            return default.to_string()
        }

        // Otherwise use the global default registry name
        return "iconify".to_string()
    }

    /// Resolve the path to the icon registry, cloning the remote registry if needed
    pub fn resolve(name: Option<&str>, config: &DioxusConfig) -> Result<PathBuf> {
        let name = Self::resolve_name(name, config);
        
        let registry = if Self::is_default_name(&name) {
            Some(config.icons.registeries.get(&name).cloned().unwrap_or_default())
        } else {
            config.icons.registeries.get(&name).cloned()
        };

        if let Some(registry) = registry {
            let path = registry.path.unwrap_or_else(|| Workspace::icon_registry_cache_dir().join(&name));

            if !path.exists() {
                if let Some(git) = &registry.git {
                    println!("Cloning icon registry from git '{}'", git);
                    Self::clone(&git, registry.rev.as_deref(), registry.depth, &path)?;
                    return Ok(path)
                } else {
                    return Err(anyhow::anyhow!("Icon registry path for '{}' does not exist at: {}", &name, path.display()))
                }
            } else {
                return Ok(path)
            }
        }

        Err(anyhow::anyhow!("Icon registry '{}' does not exist in the config", &name))
    }

    /// Clone an icon registry from the given git url to the given destination
    fn clone(git: &str, rev: Option<&str>, depth: Option<i32>, dest: &PathBuf) -> Result<Repository> {
        let mut fetch_options = FetchOptions::new();
        fetch_options.depth(depth.unwrap_or_default());

        let mut repo_builder = RepoBuilder::new();
        repo_builder.fetch_options(fetch_options);

        let repo = repo_builder.clone(&git, dest)?;

        if let Some(rev) = rev {
            Self::checkout_rev(&repo, &git, &rev)?;
        }

        Ok(repo)
    }

    /// Checkout the given rev in the given repo
    fn checkout_rev(repo: &Repository, git: &str, rev: &str) -> Result<()> {
        let (object, reference) = repo
            .revparse_ext(rev)
            .with_context(|| format!("Failed to find revision '{}' in '{}'", rev, git))?;
        repo.checkout_tree(&object, None)?;

        if let Some(gref) = reference {
            if let Some(name) = gref.name() {
                repo.set_head(name)?;
            }
        } else {
            repo.set_head_detached(object.id())?;
        }

        Ok(())
    }
}