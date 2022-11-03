lazy_static::lazy_static! {
pub static ref T: std::collections::HashMap<&'static str, &'static str> =
    [
        ("desk_tip", "Your desktop can be accessed with this ID and password."),
        ("connecting_status", "Connecting to the HopToDesk network..."),
		("connecting_status_short", "Connecting..."),
        ("not_ready_status", "Not ready. Please check your connection"),
		("not_ready_status_short", "Not ready"),
        ("install_tip", "For best performance, complete a full install."),
        ("config_acc", "Enable the \"Accessibility\" permission to use screen share."),
        ("config_screen", "Enable \"Screen Recording\" permissions to use screen share."),
        ("agreement_tip", "By starting the installation, you accept the license agreement."),
        ("not_close_tcp_tip", "Don't close this window while you are using the tunnel"),
        ("Auto Login", "Auto Login (Only valid if you set \"Lock after session end\")"),
        ("whitelist_tip", "Only whitelisted IP can access me"),
        ("whitelist_sep", "Seperated by comma, semicolon, spaces or new line"),
        ("Wrong credentials", "Wrong username or password"),
        ("invalid_http", "must start with http:// or https://"),
        ("install_daemon_tip", "For starting on boot, you need to install system service."),
        ("android_input_permission_tip1", "In order for a remote device to control your Android device via mouse or touch, you need to allow HopToDesk to use the \"Accessibility\" service."),
        ("android_input_permission_tip2", "Please go to the next system settings page, find and enter [Installed Services], turn on [HopToDesk Input] service."),
        ("android_new_connection_tip", "New control request has been received, which wants to control your current device."),
        ("android_service_will_start_tip", "Turning on \"Screen Capture\" will automatically start the service, allowing other devices to request a connection to your device."),
        ("android_stop_service_tip", "Closing the service will automatically close all established connections."),
        ("android_version_audio_tip", "The current Android version does not support audio capture, please upgrade to Android 10 or higher."),
        ("android_start_service_tip", "Tap [Start Screen Share] to allow screen sharing."),
		("minimize_to_tray", "Minimize to Tray when closing main window"),
        ("android_open_battery_optimizations_tip", "If you want to disable this feature, please go to the next HopToDesk application settings page, find and enter [Battery], Uncheck [Unrestricted]"),
        ("remote_restarting_tip", "Remote device is restarting, please close this message box and reconnect with permanent password after a while"),
        ("Are you sure to close the connection?", "Are you sure you want to close the connection?"),
        ("elevation_prompt", "Running software without privilege elevation may cause problems when remote users operate certain windows."),
        ("uac_warning", "Temporarily denied access due to elevation request, please wait for the remote user to accept the UAC dialog. To avoid this problem, it is recommended to install the software on the remote device or run it with administrator privileges."),
        ("elevated_foreground_window_warning", "Temporarily unable to use the mouse and keyboard, because the current window of the remote desktop requires higher privilege to operate, you can request the remote user to minimize the current window. To avoid this problem, it is recommended to install the software on the remote device or run it with administrator privileges."),
    ].iter().cloned().collect();
}