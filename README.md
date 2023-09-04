# Rust Chat Server

This project serves as a learning exercise in [Rust](https://www.rust-lang.org/), [Tokio](https://tokio.rs/), [Channels](https://tokio.rs/tokio/tutorial/channels), and TUI (Terminal User Interface) programming. It features a room-based chat server with a Terminal User Interface (TUI), utilizing technologies such as Tokio, Ratatui, and a Redux-inspired architecture.

![TUI Demo](./tui/docs/tui.gif)

**Note**: This project is not suitable for production use. It's designed strictly for educational purposes.

## Setup Instructions

To get the project up and running, follow these steps:

1. Clone the repository: `git clone git@github.com:Yengas/rust-chat-server.git`
2. Make sure you have [Rust and Cargo](https://www.rust-lang.org/tools/install) installed.
3. Change to the project directory: `cd rust-chat-server`
4. Start the server: `cargo run --bin server`
5. Launch one or more TUI instances: `cargo run --bin tui`

## Project Overview

The project utilizes Rust Workspaces to divide itself into three sub-projects, each with its own README that details the concepts and architecture. Below is a brief overview:

- [comms](./comms/): This sub-project houses a library crate that provides Events and Commands used for server-client communication. It also offers client/server socket utilities, enabled via feature flags, to assist in serializing and deserializing events and commands.
- [server](./server/): Built on the [Tokio Runtime](https://tokio.rs/) and using [Tokio Channels](https://tokio.rs/tokio/tutorial/channels), this sub-project implements a single-instance chat server that manages room states and user participation.
- [tui](./tui/): Leveraging [Ratatui](https://github.com/ratatui-org/ratatui), this sub-project implements a terminal-based user interface. Users can connect to a chat server, join rooms, and send/receive messages. The code follows a Redux-inspired structure to separate state management from TUI rendering.

## License

The project is distributed under the [MIT License](./LICENSE).
