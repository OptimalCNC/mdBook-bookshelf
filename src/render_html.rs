#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootBookLink {
    pub book_id: String,
    pub href: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarItem {
    pub page_id: String,
    pub href: String,
    pub title: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentPageRenderInput {
    pub html_lang: String,
    pub default_theme: String,
    pub page_title: String,
    pub page_id: String,
    pub owning_book_id: String,
    pub breadcrumb: String,
    pub bookshelf_href: String,
    pub sidebar_items: Vec<SidebarItem>,
    pub prev_href: Option<String>,
    pub next_href: Option<String>,
}

pub fn render_root_page(html_lang: &str, default_theme: &str, books: &[RootBookLink]) -> String {
    let mut book_list_items = String::new();
    for book in books {
        book_list_items.push_str(&format!(
            "<li data-book-id=\"{}\"><a href=\"{}\">{}</a></li>",
            escape_html(&book.book_id),
            escape_html(&book.href),
            escape_html(&book.title)
        ));
    }

    format!(
        "<!doctype html><html lang=\"{}\"><head><meta charset=\"utf-8\"><title>Bookshelf</title></head>\
         <body data-mdbook-default-theme=\"{}\"><h1>Bookshelf</h1><ul>{}</ul></body></html>",
        escape_html(html_lang),
        escape_html(default_theme),
        book_list_items
    )
}

pub fn render_content_page(input: &ContentPageRenderInput) -> String {
    let mut sidebar_items_html = String::new();
    for item in &input.sidebar_items {
        sidebar_items_html.push_str(&format!(
            "<li data-page-id=\"{}\"{}><a href=\"{}\">{}</a></li>",
            escape_html(&item.page_id),
            if item.is_active {
                " data-active=\"true\""
            } else {
                ""
            },
            escape_html(&item.href),
            escape_html(&item.title)
        ));
    }

    let prev_html = if let Some(prev) = &input.prev_href {
        format!("<a class=\"prev\" href=\"{}\">Prev</a>", escape_html(prev))
    } else {
        "<span class=\"prev missing\"></span>".to_string()
    };
    let next_html = if let Some(next) = &input.next_href {
        format!("<a class=\"next\" href=\"{}\">Next</a>", escape_html(next))
    } else {
        "<span class=\"next missing\"></span>".to_string()
    };

    format!(
        "<!doctype html><html lang=\"{}\"><head><meta charset=\"utf-8\"><title>{}</title></head>\
         <body data-mdbook-default-theme=\"{}\">\
         <a class=\"bookshelf-return\" href=\"{}\">Bookshelf</a>\
         <aside class=\"sidebar\" data-active-book=\"{}\"><ul>{}</ul></aside>\
         <main><h1>{}</h1><div class=\"breadcrumb\">{}</div>\
         <nav class=\"pager\">{} {}</nav></main></body></html>",
        escape_html(&input.html_lang),
        escape_html(&input.page_title),
        escape_html(&input.default_theme),
        escape_html(&input.bookshelf_href),
        escape_html(&input.owning_book_id),
        sidebar_items_html,
        escape_html(&input.page_title),
        escape_html(&input.breadcrumb),
        prev_html,
        next_html
    )
}

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}
