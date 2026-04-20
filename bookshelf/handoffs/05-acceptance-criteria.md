# Acceptance Criteria

The bookshelf feature is accepted only when all items below are satisfied.

## Functional Criteria

- one top-level build produces one multi-book site
- one top-level serve workflow exists for the multi-book site
- the site root opens the `Bookshelf` page
- the `Bookshelf` page is owned by the configured root book
- the `Bookshelf` page is not presented as a selectable content book
- each shelf item links to the corresponding content-book root page
- each shelf item includes a short description
- every content-book page exposes a visible `Bookshelf` return control
- content-book pages keep sidebar navigation scoped to the active book only
- direct links into a page activate the correct book context automatically
- previous and next never cross book boundaries
- breadcrumbs render as `Book / Page`
- search remains site-wide and shows the owning book label

## Authoring Criteria

- `bookshelf.toml` is the single human-owned site config
- each book keeps one canonical `SUMMARY.md`
- the `Bookshelf` page is generated in memory, not authored as a canonical summary entry
- the implementation does not require authors to duplicate non-root book entries into the root book's summary

## Architecture Criteria

- the final site is rendered directly from the bookshelf-owned site model
- the implementation does not depend on HTML post-processing over stock mdBook output as its primary architecture
- the implementation does not rely on copying the full docs tree into a temporary workspace as its primary architecture

## Verification Criteria

- the small self-contained example passes all bookshelf-specific validation
- the repo-scale scenario passes all bookshelf-specific validation
- the new bookshelf tests described in [04-test-plan.md](./04-test-plan.md) are implemented and passing
- mdBook's stock tests still pass where applicable
