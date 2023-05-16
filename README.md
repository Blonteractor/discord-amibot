# Amibot

[![Discord](https://img.shields.io/static/v1?logo=Discord&logoColor=fff&labelColor=5562ea&label=Amibot%20Early%20Access&message=⬅️%20Click%20to%20use%20&color=50524f)](https://discord.gg/T9ZDVTw4mQ)

![Rust](https://img.shields.io/badge/Rust-%23000000.svg?style=flat&logo=rust&logoColor=white)
![MongoDB Logo](https://img.shields.io/badge/-MongoDB-47A248?logo=MongoDB&logoColor=fff)
[![go-amizone Logo](https://img.shields.io/badge/go%20amizone-2D8CFF?logo=go&logoColor=white)](https://github.com/ditsuke/go-amizone)
[![Issues](https://img.shields.io/github/issues/blonteractor/discord-amibot?logo=github)](https://github.com/blonteractor/discord-amibot/issues)
[![License](https://img.shields.io/github/license/blonteractor/discord-amibot)](./LICENSE)

This is a Discord bot that allows users to access their student portal directly in Discord. The bot is written in Rust and uses [go-amizone](https://github.com/ditsuke/go-amizone) as its backend.
This project uses [poise.rs](https://github.com/serenity-rs/poise) as its Discord bot framework.

## Getting Started

1. Clone this repository to your local machine.
2. Install Rust and its package manager, Cargo, if you haven't already.
3. Install the required dependencies by running `cargo build`.
4. Create a Discord bot and obtain its token.
5. Create a `.env` file in the root directory of the project and set the following environment variables:
    - `DISCORD_TOKEN`
    - `DATABASE_URL`
    - `AMIZONE_API_URL`
    - `DEV_ID`
    - `DEV_SERVER_ID` (optional if you compile with the --release flag)
6. Run the bot by running `cargo run`.

## Commands

> **Note**: Bot is in active developement, most commands below dont exist yet

- `/attendance`: Displays the user's attendance for the current semester.
- `/schedule <YYYY-MM-DD>`: Displays the user's schedule for the given date.
- `/exams`: Displays the user's exam schedule for the current semester.
- `/semesters`: Displays a list of past and current semesters.
- `/courses <semester_ref>`: Displays a list of courses for the given semester.
- `/profile`: Displays the user's profile information.
- `/wifi`: Displays the user's registered WiFi MAC addresses.
- `/wifi <ACTION> <MAC>`: Registers/Derigsters a WiFi MAC address for the user.
- `/feedback <rating> <query_rating> <comment>`: Fills the faculty feedback.

## Contributing

Contributions are welcome! If you find a bug or have a feature request, please open an issue. If you'd like to contribute code, please fork the repository and create a pull request.

## License

This project is licensed under the [MIT License](./LICENSE).
