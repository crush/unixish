#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
	pub x: i32,
	pub y: i32,
	pub width: i32,
	pub height: i32,
}

pub fn almost(screen: Rect, width: f64, height: f64) -> Rect {
	let xgap = ((1.0 - width).max(0.0) * f64::from(screen.width) / 2.0).round() as i32;
	let ygap = ((1.0 - height).max(0.0) * f64::from(screen.height) / 2.0).round() as i32;
	let gap = ((xgap + ygap) as f64 / 2.0).round() as i32;
	let maxx = (screen.width - 1) / 2;
	let maxy = (screen.height - 1) / 2;
	let gap = gap.clamp(0, maxx.min(maxy));
	let width = (screen.width - (gap * 2)).max(1);
	let height = (screen.height - (gap * 2)).max(1);
	let x = screen.x + gap;
	let y = screen.y + gap;
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
	let left = screen.width / 2;
	let width = screen.width - left;
	Rect {
		x: screen.x + left,
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
	let top = screen.height / 2;
	let height = screen.height - top;
	Rect {
		x: screen.x,
		y: screen.y + top,
		width: screen.width,
		height,
	}
}
