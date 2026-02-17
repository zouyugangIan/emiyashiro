//! 统一文本输入系统
//!
//! 提供强大的键盘输入处理和文本验证功能

use bevy::prelude::*;
use std::collections::HashSet;

/// 文本输入状态资源
#[derive(Resource, Default)]
pub struct TextInputState {
    pub current_text: String,
    pub is_active: bool,
    pub cursor_position: usize,
    pub max_length: usize,
    pub allowed_chars: HashSet<char>,
}

impl TextInputState {
    pub fn new(max_length: usize) -> Self {
        let mut allowed_chars = HashSet::new();

        // 添加字母
        for c in 'A'..='Z' {
            allowed_chars.insert(c);
        }

        // 添加数字
        for c in '0'..='9' {
            allowed_chars.insert(c);
        }

        // 添加特殊字符
        allowed_chars.insert('_'); // 空格替换为下划线
        allowed_chars.insert('-'); // 连字符

        Self {
            current_text: String::new(),
            is_active: false,
            cursor_position: 0,
            max_length,
            allowed_chars,
        }
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.current_text.clear();
        self.cursor_position = 0;
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn is_valid_char(&self, c: char) -> bool {
        self.allowed_chars.contains(&c.to_ascii_uppercase())
    }

    pub fn add_char(&mut self, c: char) -> bool {
        if self.current_text.len() < self.max_length && self.is_valid_char(c) {
            let uppercase_char = c.to_ascii_uppercase();
            self.current_text.push(uppercase_char);
            self.cursor_position = self.current_text.len();
            true
        } else {
            false
        }
    }

    pub fn remove_char(&mut self) -> bool {
        if !self.current_text.is_empty() {
            self.current_text.pop();
            self.cursor_position = self.current_text.len();
            true
        } else {
            false
        }
    }
}
/// 键盘输入处理器资源
#[derive(Resource, Default)]
pub struct KeyboardInputHandler {
    pub last_input_time: f32,
}

impl KeyboardInputHandler {
    pub fn new() -> Self {
        Self {
            last_input_time: 0.0,
        }
    }
}

/// 输入验证器
pub struct InputValidator {
    pub max_length: usize,
    pub allowed_chars: HashSet<char>,
    pub reserved_names: HashSet<String>,
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl InputValidator {
    pub fn new() -> Self {
        let mut allowed_chars = HashSet::new();

        // 添加字母
        for c in 'A'..='Z' {
            allowed_chars.insert(c);
        }

        // 添加数字
        for c in '0'..='9' {
            allowed_chars.insert(c);
        }

        // 添加特殊字符
        allowed_chars.insert('_');
        allowed_chars.insert('-');

        let mut reserved_names = HashSet::new();
        reserved_names.insert("CON".to_string());
        reserved_names.insert("PRN".to_string());
        reserved_names.insert("AUX".to_string());
        reserved_names.insert("NUL".to_string());

        Self {
            max_length: 25,
            allowed_chars,
            reserved_names,
        }
    }

    pub fn validate_save_name(&self, name: &str) -> Result<String, ValidationError> {
        if name.is_empty() {
            return Ok("DefaultSave".to_string());
        }

        if name.len() > self.max_length {
            return Err(ValidationError::TooLong);
        }

        for c in name.chars() {
            if !self.allowed_chars.contains(&c.to_ascii_uppercase()) {
                return Err(ValidationError::InvalidCharacters);
            }
        }

        if self.reserved_names.contains(&name.to_uppercase()) {
            return Err(ValidationError::ReservedName);
        }

        Ok(name.to_string())
    }

    pub fn sanitize_input(&self, input: &str) -> String {
        input
            .chars()
            .filter(|c| self.is_valid_char(*c))
            .take(self.max_length)
            .map(|c| c.to_ascii_uppercase())
            .collect()
    }

    pub fn is_valid_char(&self, c: char) -> bool {
        self.allowed_chars.contains(&c.to_ascii_uppercase())
    }
}

#[derive(Debug)]
pub enum ValidationError {
    TooLong,
    InvalidCharacters,
    ReservedName,
    Empty,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::TooLong => write!(f, "Name too long (max 25 characters)"),
            ValidationError::InvalidCharacters => {
                write!(f, "Invalid characters (use A-Z, 0-9, _, -)")
            }
            ValidationError::ReservedName => write!(f, "Reserved name not allowed"),
            ValidationError::Empty => write!(f, "Name cannot be empty"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// 处理键盘输入的系统
pub fn handle_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_input_state: ResMut<TextInputState>,
    time: Res<Time>,
    mut keyboard_handler: ResMut<KeyboardInputHandler>,
) {
    if !text_input_state.is_active {
        return;
    }

    keyboard_handler.last_input_time += time.delta_secs();

    // 处理特殊键
    if keyboard_input.just_pressed(KeyCode::Backspace) {
        text_input_state.remove_char();
        crate::debug_log!(
            "🔤 Backspace pressed, current text: '{}'",
            text_input_state.current_text
        );
    }

    if keyboard_input.just_pressed(KeyCode::Enter) {
        crate::debug_log!(
            "🔤 Enter pressed, confirming input: '{}'",
            text_input_state.current_text
        );
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        crate::debug_log!("🔤 Escape pressed, canceling input");
        text_input_state.deactivate();
        return;
    }

    // 处理字符输入
    for key in keyboard_input.get_just_pressed() {
        if let Some(character) = map_keycode_to_char(key) {
            if text_input_state.add_char(character) {
                crate::debug_log!(
                    "🔤 Added character '{}', current text: '{}'",
                    character,
                    text_input_state.current_text
                );
            } else {
                crate::debug_log!(
                    "🔤 Failed to add character '{}' (max length or invalid)",
                    character
                );
            }
        }
    }
}

/// 将键码映射到字符
fn map_keycode_to_char(keycode: &KeyCode) -> Option<char> {
    match keycode {
        // 字母键
        KeyCode::KeyA => Some('A'),
        KeyCode::KeyB => Some('B'),
        KeyCode::KeyC => Some('C'),
        KeyCode::KeyD => Some('D'),
        KeyCode::KeyE => Some('E'),
        KeyCode::KeyF => Some('F'),
        KeyCode::KeyG => Some('G'),
        KeyCode::KeyH => Some('H'),
        KeyCode::KeyI => Some('I'),
        KeyCode::KeyJ => Some('J'),
        KeyCode::KeyK => Some('K'),
        KeyCode::KeyL => Some('L'),
        KeyCode::KeyM => Some('M'),
        KeyCode::KeyN => Some('N'),
        KeyCode::KeyO => Some('O'),
        KeyCode::KeyP => Some('P'),
        KeyCode::KeyQ => Some('Q'),
        KeyCode::KeyR => Some('R'),
        KeyCode::KeyS => Some('S'),
        KeyCode::KeyT => Some('T'),
        KeyCode::KeyU => Some('U'),
        KeyCode::KeyV => Some('V'),
        KeyCode::KeyW => Some('W'),
        KeyCode::KeyX => Some('X'),
        KeyCode::KeyY => Some('Y'),
        KeyCode::KeyZ => Some('Z'),
        // 数字键
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        // 特殊字符
        KeyCode::Space => Some('_'), // 空格转换为下划线
        KeyCode::Minus => Some('-'), // 连字符
        _ => None,
    }
}
