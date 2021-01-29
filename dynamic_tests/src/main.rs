#![feature(decl_macro)]

use pages::*;
use std::thread;
use std::time::Duration;
use url::Url;
use web_archive::blocking;
use web_archive::parsing::Resource;

mod pages;

fn server() -> rocket::Rocket {
    rocket::ignite().mount(
        "/",
        rocket::routes![index, style, rust_logo, js, blog, ferris],
    )
}

fn main() {
    let _server_handle = thread::spawn(|| server().launch());

    // Allow time to make sure that the server has started up properly
    thread::sleep(Duration::from_secs(1));
    println!("Server launched!");

    // Start running the tests
    test_index();
    test_blog();

    println!("Success! All dynamic tests have passed");
}

fn test_index() {
    let u = "http://localhost:8000/";

    let a = blocking::archive(u).unwrap();

    assert_eq!(a.content, index());

    assert_eq!(
        a.resource_map
            .get(&Url::parse("http://localhost:8000/style.css").unwrap())
            .unwrap(),
        &Resource::Css(style().to_string())
    );
    println!("[PASS] Index page with CSS");
}

fn test_blog() {
    let u = "http://localhost:8000/pages/blog.html";

    let a = blocking::archive(u).unwrap();

    assert_eq!(a.content, blog());
    assert_eq!(a.resource_map.len(), 4);

    assert_eq!(
        a.resource_map
            .get(&Url::parse("http://localhost:8000/style.css").unwrap())
            .unwrap(),
        &Resource::Css(style().to_string())
    );
    println!("[PASS] Blog page with multiple resources");
}
