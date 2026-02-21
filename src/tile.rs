#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
	pub x: i32,
	pub y: i32,
	pub width: i32,
	pub height: i32,
}

pub fn almost(screen: Rect, width: f64, height: f64) -> Rect {
	let width = (f64::from(screen.width) * width).round() as i32;
	let height = (f64::from(screen.height) * height).round() as i32;
	let x = screen.x + (screen.width - width) / 2;
	let y = screen.y + (screen.height - height) / 2;
	Rect { x, y, width, height }
}

pub fn center(screen: Rect, window: Rect) -> Rect {
	let x = screen.x + (screen.width - window.width) / 2;
	let y = screen.y + (screen.height - window.height) / 2;
	Rect {
		x,
		y,
		width: window.width,
		height: window.height,
	}
}

pub fn left(screen: Rect) -> Rect {
	Rect {
		x: screen.x,
		y: screen.y,
		width: screen.width / 2,
		height: screen.height,
	}
}

pub fn right(screen: Rect) -> Rect {
	let width = screen.width / 2;
	Rect {
		x: screen.x + width,
		y: screen.y,
		width,
		height: screen.height,
	}
}

pub fn top(screen: Rect) -> Rect {
	Rect {
		x: screen.x,
		y: screen.y,
		width: screen.width,
		height: screen.height / 2,
	}
}

pub fn bottom(screen: Rect) -> Rect {
	let height = screen.height / 2;
	Rect {
		x: screen.x,
		y: screen.y + height,
		width: screen.width,
		height,
	}
}
