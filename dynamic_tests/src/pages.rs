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

#[get("/style.css")]
pub fn style() -> &'static str {
    r#"body {
		background-color: blue;
	}"#
}
