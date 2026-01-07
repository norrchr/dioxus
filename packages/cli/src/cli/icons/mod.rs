use anyhow::Context;
use clap::{Parser, Subcommand};
use git2::{FetchOptions, Repository, build::RepoBuilder};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
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
        todo!()
    }

    async fn remove_icons(args: IconArgs) -> Result<()> {
        todo!()
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
    /// Check if the supplied name is the built-it default registry name
    pub fn is_default_name(name: &str) -> bool {
        name == "iconify"
    }

    /// Resolve the path to the icon registry, cloning the remote registry if needed
    pub fn resolve(name: Option<&str>, config: &DioxusConfig) -> Result<PathBuf> {
        let (name, registry) = if let Some(name) = name.filter(|n| !Self::is_default_name(n))  {
            (name, config.icons.registeries.get(name).cloned())
        } else if let Some(default) = config.icons.default.as_deref().filter(|d| !Self::is_default_name(d)) {
            (default, config.icons.registeries.get(default).cloned())
        } else {
            ("iconify", Some(config.icons.registeries.get("iconify").cloned().unwrap_or_default()))
        };

        if let Some(registry) = registry {
            let path = registry.path.unwrap_or_else(|| Workspace::icon_registry_cache_dir().join(name));

            if !path.exists() {
                if let Some(git) = &registry.git {
                    println!("Cloning icon registry from git '{}'", git);
                    Self::clone(&git, registry.rev.as_deref(), registry.depth, &path)?;
                    return Ok(path)
                } else {
                    return Err(anyhow::anyhow!("Icon registry path for '{}' does not exist at: {}", name, path.display()))
                }
            } else {
                return Ok(path)
            }
        }

        return Err(anyhow::anyhow!("Could not resolve icon registry named '{}'", name))
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