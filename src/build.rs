use crate::catalog::{build_input_catalog, InputBook};
use anyhow::{Context, Result};
use mdbook_driver::{config::Config, MDBook};
use mdbook_summary::parse_summary;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub fn build_bookshelf(config_path: impl AsRef<Path>, dest_dir: Option<PathBuf>) -> Result<()> {
    let config_path = config_path.as_ref();
    let catalog = build_input_catalog(config_path)?;
    let projected_config = project_mdbook_config(config_path)?;
    let site_dest_dir = resolve_site_dest_dir(&catalog.config_dir, &projected_config, dest_dir)?;
    let config_root = catalog.config_dir.clone();

    for book in &catalog.books {
        build_catalog_book(book, &projected_config, &config_root, &site_dest_dir)?;
    }

    write_site_root_index(&catalog, &projected_config, &site_dest_dir)?;

    Ok(())
}

pub fn project_mdbook_config(config_path: impl AsRef<Path>) -> Result<Config> {
    let config_path = config_path.as_ref();
    let raw = fs::read_to_string(config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;
    let mut toml_root: toml::Table = toml::from_str(&raw)
        .with_context(|| format!("failed to parse TOML in {}", config_path.display()))?;
    toml_root.remove("bookshelf");

    let projected = toml::to_string(&toml_root).with_context(|| {
        format!(
            "failed to serialize projected mdBook config from {}",
            config_path.display()
        )
    })?;

    Config::from_str(&projected).with_context(|| {
        format!(
            "failed to parse projected mdBook config from {} after removing [bookshelf]",
            config_path.display()
        )
    })
}

fn build_catalog_book(
    book: &InputBook,
    projected_config: &Config,
    config_root: &Path,
    site_dest_dir: &Path,
) -> Result<()> {
    let summary_text = fs::read_to_string(&book.summary_abs).with_context(|| {
        format!(
            "book '{}' failed to read canonical summary at {}",
            book.id,
            book.summary_abs.display()
        )
    })?;
    let summary = parse_summary(&summary_text).with_context(|| {
        format!(
            "book '{}' failed to parse canonical summary at {}",
            book.id,
            book.summary_abs.display()
        )
    })?;

    let mut config = projected_config.clone();
    config.book.src = book.book_src_rel.clone();
    config.build.build_dir = site_dest_dir.join("books").join(&book.id);

    let mdbook = MDBook::load_with_config_and_summary(config_root.to_path_buf(), config, summary)
        .with_context(|| {
        format!(
            "book '{}' failed to load mdbook from root {} and source {}",
            book.id,
            config_root.display(),
            book.book_src_abs.display()
        )
    })?;

    let html_build_dir = mdbook.build_dir_for("html");
    mdbook.build().with_context(|| {
        format!(
            "book '{}' failed to build mdbook output at {}",
            book.id,
            html_build_dir.display()
        )
    })
}

fn resolve_site_dest_dir(
    config_dir: &Path,
    projected_config: &Config,
    dest_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("failed to determine current working directory")?;

    match dest_dir {
        Some(dest_dir) => Ok(make_absolute(&cwd, dest_dir)),
        None => {
            let config_dir = make_absolute(&cwd, config_dir.to_path_buf());
            Ok(make_absolute(
                &config_dir,
                projected_config.build.build_dir.clone(),
            ))
        }
    }
}

fn make_absolute(base: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

fn write_site_root_index(
    catalog: &crate::catalog::InputCatalog,
    projected_config: &Config,
    site_dest_dir: &Path,
) -> Result<()> {
    fs::create_dir_all(site_dest_dir).with_context(|| {
        format!(
            "failed to create site output directory {}",
            site_dest_dir.display()
        )
    })?;

    let index_path = site_dest_dir.join("index.html");
    fs::write(
        &index_path,
        render_site_root_index(catalog, projected_config),
    )
    .with_context(|| {
        format!(
            "failed to write synthetic bookshelf chooser page at {}",
            index_path.display()
        )
    })
}

fn render_site_root_index(
    catalog: &crate::catalog::InputCatalog,
    projected_config: &Config,
) -> String {
    let mut cards = String::new();
    for book in &catalog.books {
        let description = book
            .description
            .as_deref()
            .map(|description| {
                format!(
                    "<p class=\"bookshelf-card__description\">{}</p>",
                    escape_html(description)
                )
            })
            .unwrap_or_default();

        // The root book still enters its stock mdBook content root, not the chooser itself.
        let href = format!("books/{}/index.html", escape_html_attr(&book.id));
        cards.push_str(&format!(
            "<li class=\"bookshelf-card\"><a class=\"bookshelf-card__link\" href=\"{href}\"><h2>{}</h2>{description}</a></li>",
            escape_html(&book.title),
        ));
    }

    let lang = projected_config.book.language.as_deref().unwrap_or("en");
    let site_title = projected_config
        .book
        .title
        .as_deref()
        .unwrap_or("Bookshelf");
    let site_title = site_title.trim();
    let site_title = if site_title.is_empty() {
        "Bookshelf"
    } else {
        site_title
    };

    format!(
        "<!DOCTYPE html>\n\
<html lang=\"{}\">\n\
<head>\n\
  <meta charset=\"utf-8\">\n\
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
  <title>{} | Bookshelf</title>\n\
  <style>\n\
    :root {{ color-scheme: light; }}\n\
    * {{ box-sizing: border-box; }}\n\
    body {{ margin: 0; font-family: sans-serif; background: #f5f6f8; color: #1f2933; }}\n\
    main {{ max-width: 64rem; margin: 0 auto; padding: 3rem 1.5rem 4rem; }}\n\
    header {{ margin-bottom: 2rem; }}\n\
    .bookshelf-site-title {{ margin: 0 0 0.5rem; font-size: 0.95rem; letter-spacing: 0.08em; text-transform: uppercase; color: #52606d; }}\n\
    h1 {{ margin: 0 0 0.75rem; font-size: clamp(2rem, 4vw, 3rem); }}\n\
    .bookshelf-intro {{ margin: 0; max-width: 42rem; line-height: 1.6; color: #52606d; }}\n\
    .bookshelf-list {{ list-style: none; margin: 0; padding: 0; display: grid; gap: 1rem; }}\n\
    .bookshelf-card__link {{ display: block; padding: 1.25rem 1.5rem; border: 1px solid #d9e2ec; border-radius: 0.9rem; background: #fff; color: inherit; text-decoration: none; box-shadow: 0 10px 30px rgba(15, 23, 42, 0.06); }}\n\
    .bookshelf-card__link:hover, .bookshelf-card__link:focus-visible {{ border-color: #486581; transform: translateY(-1px); }}\n\
    .bookshelf-card__link h2 {{ margin: 0; font-size: 1.2rem; }}\n\
    .bookshelf-card__description {{ margin: 0.75rem 0 0; line-height: 1.6; color: #52606d; }}\n\
  </style>\n\
</head>\n\
<body>\n\
  <main>\n\
    <header>\n\
      <p class=\"bookshelf-site-title\">{}</p>\n\
      <h1>Bookshelf</h1>\n\
      <p class=\"bookshelf-intro\">Choose a book to enter its stock mdBook root page.</p>\n\
    </header>\n\
    <ul class=\"bookshelf-list\">{cards}</ul>\n\
  </main>\n\
</body>\n\
</html>\n",
        escape_html_attr(lang),
        escape_html(site_title),
        escape_html(site_title),
    )
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_html_attr(text: &str) -> String {
    escape_html(text).replace('"', "&quot;")
}
