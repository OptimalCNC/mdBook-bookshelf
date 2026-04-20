use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::reader_context::ReaderContextModel;
use crate::site_model::{SiteModel, SiteRoot};

#[derive(Debug)]
pub enum BuildRenderManifestError {
    RootEntryMismatch {
        expected_page_id: String,
        actual_page_id: String,
    },
    SourcePathOutsideConfig {
        source_path: PathBuf,
        config_dir: PathBuf,
    },
    OutputPathCollision {
        output_path: PathBuf,
        first_page: String,
        second_page: String,
    },
}

impl fmt::Display for BuildRenderManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootEntryMismatch {
                expected_page_id,
                actual_page_id,
            } => write!(
                f,
                "site root entry points to `{}` instead of the synthetic Bookshelf page `{}`",
                actual_page_id, expected_page_id
            ),
            Self::SourcePathOutsideConfig {
                source_path,
                config_dir,
            } => write!(
                f,
                "authored source path {} is not under the bookshelf config directory {}",
                source_path.display(),
                config_dir.display()
            ),
            Self::OutputPathCollision {
                output_path,
                first_page,
                second_page,
            } => write!(
                f,
                "rendered output path {} is claimed by both {} and {}",
                output_path.display(),
                first_page,
                second_page
            ),
        }
    }
}

impl std::error::Error for BuildRenderManifestError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderManifest {
    pub entries: Vec<RenderedPageManifestEntry>,
}

impl RenderManifest {
    pub fn authored_entry_for_source_path(
        &self,
        source_path: impl AsRef<Path>,
    ) -> Option<&RenderedPageManifestEntry> {
        let normalized = source_path.as_ref().canonicalize().ok()?;
        self.entries.iter().find(|entry| {
            matches!(
                &entry.identity,
                RenderedPageIdentity::AuthoredPage { source_path } if source_path == &normalized
            )
        })
    }

    pub fn entry_for_page_id(&self, page_id: &str) -> Option<&RenderedPageManifestEntry> {
        self.entries.iter().find(|entry| {
            matches!(
                &entry.identity,
                RenderedPageIdentity::SyntheticPage { page_id: entry_page_id } if entry_page_id == page_id
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedPageManifestEntry {
    pub identity: RenderedPageIdentity,
    pub title: String,
    pub route_path: PathBuf,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderedPageIdentity {
    SyntheticPage { page_id: String },
    AuthoredPage { source_path: PathBuf },
}

pub fn build_render_manifest(
    site_model: &SiteModel,
    reader_context: &ReaderContextModel,
) -> Result<RenderManifest, BuildRenderManifestError> {
    let expected_page_id = site_model.bookshelf_page.page_id.clone();
    let actual_page_id = match &site_model.root_entry {
        SiteRoot::BookshelfPage { page_id } => page_id.clone(),
    };
    if actual_page_id != expected_page_id {
        return Err(BuildRenderManifestError::RootEntryMismatch {
            expected_page_id,
            actual_page_id,
        });
    }

    let config_dir = site_model
        .config_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let mut entries = Vec::with_capacity(reader_context.authored_page_contexts.len() + 1);
    let mut claimed_outputs: HashMap<PathBuf, String> = HashMap::new();

    let synthetic_entry = RenderedPageManifestEntry {
        identity: RenderedPageIdentity::SyntheticPage {
            page_id: reader_context.bookshelf_page_context.page_id.clone(),
        },
        title: reader_context.bookshelf_page_context.title.clone(),
        route_path: PathBuf::new(),
        output_path: PathBuf::from("index.html"),
    };
    claim_output_path(&mut claimed_outputs, &synthetic_entry)?;
    entries.push(synthetic_entry);

    for context in &reader_context.authored_page_contexts {
        let relative_source = context.source_path.strip_prefix(&config_dir).map_err(|_| {
            BuildRenderManifestError::SourcePathOutsideConfig {
                source_path: context.source_path.clone(),
                config_dir: config_dir.clone(),
            }
        })?;
        let output_path = rendered_output_path(relative_source);
        let entry = RenderedPageManifestEntry {
            identity: RenderedPageIdentity::AuthoredPage {
                source_path: context.source_path.clone(),
            },
            title: context.page_title.clone(),
            route_path: output_path.clone(),
            output_path,
        };
        claim_output_path(&mut claimed_outputs, &entry)?;
        entries.push(entry);
    }

    Ok(RenderManifest { entries })
}

fn rendered_output_path(relative_source_path: &Path) -> PathBuf {
    let mut output_path = relative_source_path.to_path_buf();
    output_path.set_extension("html");
    output_path
}

fn claim_output_path(
    claimed_outputs: &mut HashMap<PathBuf, String>,
    entry: &RenderedPageManifestEntry,
) -> Result<(), BuildRenderManifestError> {
    let page_label = match &entry.identity {
        RenderedPageIdentity::SyntheticPage { page_id } => format!("synthetic page `{}`", page_id),
        RenderedPageIdentity::AuthoredPage { source_path } => {
            format!("authored page {}", source_path.display())
        }
    };

    if let Some(existing) = claimed_outputs.insert(entry.output_path.clone(), page_label.clone()) {
        return Err(BuildRenderManifestError::OutputPathCollision {
            output_path: entry.output_path.clone(),
            first_page: existing,
            second_page: page_label,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::rendered_output_path;
    use std::path::{Path, PathBuf};

    #[test]
    fn replaces_markdown_extensions_with_html_outputs() {
        assert_eq!(
            rendered_output_path(Path::new("docs/index.md")),
            PathBuf::from("docs/index.html")
        );
        assert_eq!(
            rendered_output_path(Path::new("modules/parser/docs/grammar.markdown")),
            PathBuf::from("modules/parser/docs/grammar.html")
        );
    }
}
