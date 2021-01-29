#![feature(decl_macro)]

use bytes::Bytes;
use pages::*;
use std::thread;
use std::time::Duration;
use tokio_test::block_on;
use url::Url;
use web_archive::parsing::Resource;
use web_archive::{archive, blocking};

mod pages;

fn server() -> rocket::Rocket {
    rocket::ignite().mount(
        "/",
        rocket::routes![
            blog,
            err_500,
            ferris,
            index,
            js,
            page_with_500_resource,
            rust_logo,
            style,
        ],
    )
}

#[derive(Copy, Clone, Debug)]
enum Mode {
    Async,
    Blocking,
}

fn main() {
    let _server_handle = thread::spawn(|| server().launch());

    // Allow time to make sure that the server has started up properly
    thread::sleep(Duration::from_secs(1));
    println!("Server launched!");

    // Start running the tests
    let test_cases = [test_index, test_blog, test_500];

    let mut results: Vec<(Mode, &'static str)> =
        Vec::with_capacity(2 * test_cases.len());
    for mode in [Mode::Async, Mode::Blocking].iter() {
        for test_case in test_cases.iter() {
            results.push((*mode, test_case(&mode)));
        }
    }

    for (mode, result) in results {
        println!("[PASS-{:?}] {}", mode, result);
    }

    println!("Success! All dynamic tests have passed");
}

fn test_index(mode: &Mode) -> &'static str {
    let u = "http://localhost:8000/";

    let a = match mode {
        Mode::Blocking => blocking::archive(u).unwrap(),
        Mode::Async => block_on(archive(u)).unwrap(),
    };

    assert_eq!(a.content, index());

    assert_eq!(
        a.resource_map
            .get(&Url::parse("http://localhost:8000/style.css").unwrap())
            .unwrap(),
        &Resource::Css(style().to_string())
    );
    "Index page with CSS"
}

fn test_blog(mode: &Mode) -> &'static str {
    let u = "http://localhost:8000/pages/blog.html";

    let a = match mode {
        Mode::Blocking => blocking::archive(u).unwrap(),
        Mode::Async => block_on(archive(u)).unwrap(),
    };

    assert_eq!(a.content, blog());
    assert_eq!(a.resource_map.len(), 4);

    assert_eq!(
        a.resource_map
            .get(&Url::parse("http://localhost:8000/style.css").unwrap())
            .unwrap(),
        &Resource::Css(style().to_string())
    );
    assert_eq!(
        a.resource_map
            .get(&Url::parse("http://localhost:8000/scripts/1.js").unwrap())
            .unwrap(),
        &Resource::Javascript(js().to_string())
    );
    assert_eq!(
        a.resource_map
            .get(
                &Url::parse("http://localhost:8000/images/rust-logo-blk.svg")
                    .unwrap()
            )
            .unwrap(),
        &Resource::Image(Bytes::copy_from_slice(rust_logo()))
    );
    assert_eq!(
        a.resource_map
            .get(
                &Url::parse(
                    "http://localhost:8000/images/rustacean-flat-happy.png"
                )
                .unwrap()
            )
            .unwrap(),
        &Resource::Image(Bytes::copy_from_slice(ferris()))
    );
    assert!(a
        .resource_map
        .get(&Url::parse("http://localhost:8000/pages/notfound.jpg").unwrap())
        .is_none(),);

    "Blog page with multiple resources"
}

fn test_500(mode: &Mode) -> &'static str {
    let u = "http://localhost:8000/500.jpg";
    let a = match mode {
        Mode::Blocking => blocking::archive(u).unwrap(),
        Mode::Async => block_on(archive(u)).unwrap(),
    };

    assert!(a.resource_map.is_empty());

    let u = "http://localhost:8000/500.html";
    let a = match mode {
        Mode::Blocking => blocking::archive(u).unwrap(),
        Mode::Async => block_on(archive(u)).unwrap(),
    };

    assert_eq!(a.content, page_with_500_resource().to_string());
    assert_eq!(a.resource_map.len(), 1);

    assert_eq!(
        a.resource_map
            .get(&Url::parse("http://localhost:8000/style.css").unwrap())
            .unwrap(),
        &Resource::Css(style().to_string())
    );

    "Endpoints returning Internal Server Errors"
}
