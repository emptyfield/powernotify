# powernotify

A lightweight, configurable power notification daemon for Linux. `powernotify` listens to UPower events via D-Bus and triggers desktop notifications and custom commands based on your battery status.

## Features

- **Event-Driven:** Reacts instantly to power state changes.
- **Highly Configurable:** Use a simple TOML file to define your own rules.
- **Custom Notifications:** Control the summary, body, icon, and urgency of notifications.
- **Actionable Alerts:** Add buttons to your notifications to run commands like enabling power-saving mode or suspending the system.
- **Command Execution:** Automatically run any shell command (`cmd`) when an event is triggered.
- **Low Resource Usage:** Written in Rust with an async runtime for minimal system impact.

## Installation

### Prerequisites

You must have the `upower` service running on your system, which is standard on most modern Linux desktops.

### From Source

1. Clone the repository:

    ```sh
    git clone https://github.com/emptyfield/powernotify.git
    cd powernotify
    ```

2. Build the release binary:

    ```sh
    cargo build --release
    ```

3. Copy the binary to a location in your `PATH`:

    ```sh
    sudo cp target/release/powernotify /usr/local/bin/
    ```

## Usage

`powernotify` is designed to be run as a background service. By default, it will look for a configuration file at `~/.config/powernotify/config.toml`.

### Running the Daemon

To run `powernotify` as a background daemon, simply execute it:

```sh
powernotify
```

For it to be useful, you should start it with your desktop session.

### Command-Line Options

You can customize the behavior of `powernotify` with the following command-line arguments:

```
powernotify [OPTIONS]
```

- `-c, --config <FILE>`  
    Sets a custom configuration file path.  
    **Default:** `~/.config/powernotify/config.toml`

- `-t, --test <EVENT>`  
    Triggers a test notification for a specific event from your config file without starting the daemon. This is useful for checking your configuration.
    Example Events: `ac`, `bat`, `20`, `5`

- `-h, --help`  
    Prints help information.

- `-V, --version`  
    Prints the version information.

### Examples

Run with a custom configuration file:

```sh
powernotify --config ~/dotfiles/powernotify.toml
```

Test the notification for the "20%" battery level event:

```sh
powernotify --test 20
```

Test the "AC connected" event using the short flag:

```sh
powernotify -t ac
```

## Configuration

Create your configuration file at `~/.config/powernotify/config.toml`.

The configuration is based on event triggers, which are defined as TOML tables.

### Event Triggers

There are two types of triggers:

1. **State Triggers:**
    - `[ac]`: Fired when the AC adapter is **plugged in**.
    - `[bat]`: Fired when the AC adapter is **unplugged** and the system switches to battery power.

2. **Percentage Triggers:**
    - `[percentage]`: (e.g., `[20]`, `[10]`, `[5]`) Fired when the battery level **drops to or below** the specified percentage while on battery power. The trigger will only fire once per percentage level until the device is charged above that level again.

---

### Example Configuration

Here is a complete example `config.toml` that demonstrates the features.

```toml
# ~/.config/powernotify/config.toml

# Triggered when AC power is connected.
[ac]
summary = "AC Power Connected"
body = "The battery is now charging."
icon = "battery-full-charging"
# Example cmd: set screen brightness to a comfortable level for AC power.
# cmd = "brightnessctl s 90%"


# Triggered when AC power is disconnected and the system switches to battery.
[bat]
summary = "Switched to Battery Power"
body = "AC adapter was unplugged."
icon = "battery-good"
# Example cmd: dim the screen slightly to save power.
cmd = "brightnessctl s 50%"


# First warning: Triggered when the battery drops to 20% or lower.
[20]
summary = "Low Battery Warning"
body = "Battery at 20%. Consider plugging in your AC adapter."
icon = "battery-low"
urgency = "normal"
# Play a warning sound. There is no separate parameter for notification sound yet.
cmd = "ffplay -autoexit -nodisp /usr/share/sounds/freedesktop/stereo/dialog-warning.oga"
[20.actions]
"Dim Screen (20%)" = "brightnessctl s 20%"


# Critical warning: Triggered when the battery drops to 5% or lower.
[5]
summary = "Critical Battery Level!"
body = "Battery at 5%. The system will suspend soon if not charged."
icon = "battery-empty"
urgency = "critical"
cmd = "ffplay -autoexit -nodisp /usr/share/sounds/freedesktop/stereo/dialog-error.oga"
# Add actions to the critical notification for quick power saving.
[5.actions]
"Enable Extreme Power Save" = "brightnessctl s 10%"
"Hibernate" = "systemctl hibernate"
"Suspend" = "systemctl suspend"
```

## Dependencies

`powernotify` relies on the following components:

- A running D-Bus session bus.
- The `upower` daemon (`upowerd`) to provide power information.
- A desktop notification server (e.g., `dunst`, `mako`, or the one provided by your DE).

## Contributing

Contributions are welcome! Feel free to open an issue to report a bug or suggest a feature, or open a pull request with your improvements.
