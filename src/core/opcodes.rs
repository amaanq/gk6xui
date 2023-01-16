pub enum OpCodes {
    /// Information about the keyboard
    Info = 0x01,
    /// Restarts the keyboard (can reboot in special modes such as "CDBoot")
    RestartKeyboard = 0x03,
    Unk04 = 0x04, // Some diagnostics stuff? Or maybe something related to key input? Or macros ("KeyPress")?
    /// Set the active layer (base / driver / 1 / 2 / 3)
    SetLayer = 0x0B,
    /// Ping / keep alive
    Ping = 0x0C,

    /// Using macros when in the "driver" layer
    DriverMacro = 0x15,
    /// Set "driver" layer key values
    DriverLayerSetKeyValues = 0x16,
    /// Set "driver" layer config values (seem to be hard coded in the application)
    DriverLayerSetConfig = 0x17,
    /// The keyboard sends this packet to enable macros/shortcut/keypress lighting in the "driver" layer (the PC doesn't have to send request)
    DriverKeyCallback = 0x18,
    /// Updates the lighting in real time when in the "driver" layer
    DriverLayerUpdateRealtimeLighting = 0x1A,

    /// Resets a type of data (keys, lights, etc) for a layer
    LayerResetDataType = 0x21,
    LayerSetKeyValues = 0x22,
    Unk23KbData = 0x23, // Likely a keyboard data set (see KeyboardLayerDataType)
    Unk24KbDataLighting = 0x24, // Some lighting related data (see KeyboardLayerDataType)
    LayerSetMacros = 0x25,
    /// Sets the lighting effects which should play when pressing keys ("Press Light")
    LayerSetKeyPressLightingEffect = 0x26,
    LayerSetLightValues = 0x27,
    /// Function key values
    LayerFnSetKeyValues = 0x31,
}
