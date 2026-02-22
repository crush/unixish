fn main() {
	if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
		let mut resource = winres::WindowsResource::new();
		resource.set_icon("assets/unixish.ico");
		resource.compile().expect("icon");
	}
}
