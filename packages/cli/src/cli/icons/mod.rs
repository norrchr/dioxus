use clap::{Parser, Subcommand};
use serde::Serialize;
use std::path::PathBuf;
use crate::{Result, StructuredOutput};

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

    /// The default registry to use for non @registry icon identifiers
    #[clap(short, long, default_value = "iconify")]
    #[serde(default = "iconify")]
    registry: String,

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
}