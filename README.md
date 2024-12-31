# Aserial - A Serial Monitor with Error/Warning Highlighting and Scrollable Interface

**Aserial** is a Rust-based serial monitor that allows you to connect to a serial port and read incoming data. The terminal interface highlights errors and warnings, displays them in a separate section, and provides smooth scrolling for both the main data and the error/warning messages. This project uses the `ratatui` and `crossterm` crates to build a user-friendly terminal interface for monitoring serial data.

## Features
- Connects to a serial port and reads incoming data in real-time.
- Automatically detects and highlights error and warning messages.
- Scrollable terminal interface with two separate sections:
  - **Main Data**: Displays all incoming serial data.
  - **Errors and Warnings**: Displays only error and warning messages, with different color highlights (Red for errors, Yellow for warnings).
- Support for manual and automatic scrolling.
- Exit the program gracefully by pressing the `q` key.

## Installation


   ```sh
   cargo install aserial
   ```
## Usage

### Serial Connection
- The program automatically detects available serial ports and attempts to connect to the first available port. It uses a default baud rate of `115200` and a timeout of `1000ms`.
- The connection will display data in real-time, with automatic detection of error and warning messages.

### Key Bindings
- **`q`**: Quit the program.
- **Arrow Up/Down**: Scroll through the data (both main and error/warning sections).
- **`a`**: Reset to auto-scroll mode.
- **`b`**: Reset error/warning section to auto-scroll.

### Interface
The terminal interface is divided into two sections:
1. **Main Data Section (70% of the screen)**: This section displays all incoming serial data, with each line printed in **green**.
2. **Errors and Warnings Section (30% of the screen)**: This section displays any detected errors or warnings, with:
   - **Red** for errors (e.g., "ERR", "ERROR")
   - **Yellow** for warnings (e.g., "WRN", "WARN")
   
Both sections support scrolling. If the data exceeds the visible area, it will scroll automatically unless you manually scroll with the arrow keys.

## Example Output
The terminal interface will look something like this:

```
---------------------- Serial Monitor ----------------------
| Serial data output (green text):                         |
|                                                          |
| ...                                                      |
| Data message 1                                            |
| Data message 2                                            |
| Data message 3                                            |
| ...                                                      |
------------------------------------------------------------
| Errors and Warnings (red/yellow text):                    |
|                                                          |
| ERR: Something went wrong!                               |
| WRN: Possible issue detected                             |
|                                                          |
------------------------------------------------------------
```

## Contributing

1. Fork the repository.
2. Create a new branch (`git checkout -b feature-branch`).
3. Commit your changes (`git commit -am 'Add feature'`).
4. Push to the branch (`git push origin feature-branch`).
5. Create a new pull request.

## License

This project is licensed under the MIT License
---

