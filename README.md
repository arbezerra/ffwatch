# ffwatch

ffwatch is a simple tool to watch a directory for changes and run ffmpeg when a change is detected.

Files found in the directory that have an allowed extension will be enqueued and processed by ffmpeg.

## Installation

```bash
cargo install ffwatch
```

## Usage

Best used with docker. Look at the `docker-compose.yml` file for an example.

```bash
docker compose up --build
```
