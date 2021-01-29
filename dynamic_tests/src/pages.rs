use rocket::get;

#[get("/")]
pub fn index() -> &'static str {
    r#"<html>
		<head>
			<link rel="stylesheet" href="style.css" />
		</head>
		<body>
		</body>
	</html>"#
}

#[get("/pages/blog.html")]
pub fn blog() -> &'static str {
    r#"
	<html>
		<head>
			<title>Blog!!</title>
			<link rel="stylesheet" href="/style.css" />
			<script language="javascript" src="../scripts/1.js"></script>
		</head>
		<body>
			<div>
				<p>
					<img src="/images/rust-logo-blk.svg" />
				</p>
				<div>
					<div>
						<img src="../images/rustacean-flat-happy.png" />
						<p>
							<img src="notfound.jpg" />
						</p>
					</div>
				</div>
			</div>
		</body>
	</html>
	"#
}

#[get("/style.css")]
pub fn style() -> &'static str {
    r#"body {
		background-color: blue;
	}"#
}

#[get("/images/rust-logo-blk.svg")]
pub fn rust_logo() -> &'static str {
    include_str!("../resources/rust-logo-blk.svg")
}

#[get("/images/rustacean-flat-happy.png")]
pub fn ferris() -> &'static [u8] {
    include_bytes!("../resources/rustacean-flat-happy.png")
}

#[get("/scripts/1.js")]
pub fn js() -> &'static str {
    r#"function js_function() {
		console.log("Here is some javascript!");
	}
	"#
}
