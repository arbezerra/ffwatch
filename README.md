# ffwatch

ffwatch is a simple tool to watch a directory for changes and run ffmpeg when a change is detected.

Files found in the directory that have an allowed extension will be enqueued and processed by ffmpeg.

## Installation

```bash
cargo install ffwatch
```

## Usage

Best used with docker. Create a `docker-compose.yml` file with the following content:

```yaml
services:
  ffwatch:
    image: arbezerra/ffwatch:latest
    volumes:
      - ./data:/data
    devices:
      - /dev/dri:/dev/dri
    command:
      [
        "-c:v",
        "libx264",
        "-crf",
        "20",
        "-preset",
        "ultrafast",
        "-c:a",
        "aac",
        "-b:a",
        "128k",
      ]
```

Then run:

```bash
docker compose up
```

## You can customize with environment variables:

| Variable           | Description                                    | Default             |
| ------------------ | ---------------------------------------------- | ------------------- |
| WATCH_DIR          | Directory to watch                             | /data/watch         |
| COMPLETE_DIR       | Directory to move completed files              | /data/complete      |
| TRANSCODING_DIR    | Directory to store currently transcoding files | /data/transcoding   |
| ALLOWED_EXTENSIONS | Comma separated list of allowed extensions     | mkv,mp4,avi,mov,flv |
| HWACCEL            | FFmpeg hardware acceleration                   | auto                |
| PUID               | User ID of the resulting files                 | 1000                |
| PGID               | Group ID of the resulting files                | 1000                |
