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


## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
