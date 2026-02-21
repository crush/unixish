use anyhow::{Result, anyhow};
use windows::Win32::UI::Input::KeyboardAndMouse::*;

#[derive(Debug, Clone, Copy)]
pub struct Chord {
    pub mods: HOT_KEY_MODIFIERS,
    pub key: u32,
}

pub fn parse(text: &str) -> Result<Chord> {
    let mut mods = HOT_KEY_MODIFIERS(0);
    let mut key = None;
    for raw in text.split('+') {
        let part = raw.trim().to_lowercase();
        match part.as_str() {
            "ctrl" | "control" => mods |= MOD_CONTROL,
            "shift" => mods |= MOD_SHIFT,
            "alt" => mods |= MOD_ALT,
            "win" | "super" => mods |= MOD_WIN,
            "left" => key = Some(VK_LEFT.0 as u32),
            "right" => key = Some(VK_RIGHT.0 as u32),
            "up" => key = Some(VK_UP.0 as u32),
            "down" => key = Some(VK_DOWN.0 as u32),
            "=" | "plus" => key = Some(VK_OEM_PLUS.0 as u32),
            "-" | "minus" => key = Some(VK_OEM_MINUS.0 as u32),
            _ => {
                if part.len() == 1 {
                    let byte = part.as_bytes()[0].to_ascii_uppercase();
                    if byte.is_ascii_alphanumeric() {
                        key = Some(byte as u32);
                    }
                }
            }
        }
    }
    match key {
        Some(key) => Ok(Chord { mods, key }),
        None => Err(anyhow!("key")),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parsealpha() {
        let chord = parse("ctrl+shift+x").expect("chord");
        assert_eq!(chord.key, b'X' as u32);
        assert_eq!(chord.mods.0 & MOD_CONTROL.0, MOD_CONTROL.0);
        assert_eq!(chord.mods.0 & MOD_SHIFT.0, MOD_SHIFT.0);
    }

    #[test]
    fn parsearrow() {
        let chord = parse("shift+left").expect("chord");
        assert_eq!(chord.key, VK_LEFT.0 as u32);
        assert_eq!(chord.mods.0 & MOD_SHIFT.0, MOD_SHIFT.0);
    }

    #[test]
    fn parseinvalid() {
        assert!(parse("ctrl+shift").is_err());
    }
}
