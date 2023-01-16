// TODO: When there is better lighting / macro support, put these in the appropriate files

pub enum LightingEffectType {
    /// Static "DIY" lighting with an RGB value for each key
    Static = 0,
    /// Lighting effect with 1 or more frames of lighting
    Dynamic = 3,
}

pub enum LightingEffectColorType {
    /// Single solid color
    Monochrome = 0,
    /// Color which changes through the color spectrum
    RGB = 1,
    /// Color which changes through the color spectrum, and visually "breathes"
    Breathing = 2,
}

/// Defines how a macro should be repeated when pressing a key bound to a macro
pub enum MacroRepeatType {
    /// Repeat the macro X number of times after the key is pressed (subsequent key presses are ignored until the
    /// macro has completely finished - there doesn't appear to be any way to stop the macro once it has started,
    /// and the key must be released and pressed again to start the macro again after it has finished)
    RepeatXTimes = 1,
    /// Release key to stop the macro (repeats the macro until the key is released (even if the macro is partially complete))
    ReleaseKeyToStop = 2,
    /// Press key a second time to stop the macro (repeats the macro until the key is pressed again)
    PressKeyAgainToStop = 3,
}

/// This is different compared to DriverValue
pub enum MacroKeyType {
    Key = 1,
    Mouse = 2,
}

pub enum MacroKeyState {
    Down = 1,
    Up = 2,
}
