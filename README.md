# web-archive

Library for archiving a web page along with its linked resources (images,
css, js) for local use.

## Example

```toml
web-archive = "0.1.0"
```

```rust
use web_archive::{archive, blocking};

// Build a collection of linked resources attached to the page

// async API
let archive = archive("http://example.com").await.unwrap();

// blocking API
let archive = blocking::archive("http://example.com").unwrap();


// Embed the resources into the HTML
let page = archive.embed_resources();

println!("{}", page);
```
