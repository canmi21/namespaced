# Namespaced

A modern, API-driven configuration registry for the modern developer. Namespaced provides a simple, scalable way to manage configuration data through a RESTful API, backed by a file-based storage system using Pathmap. It's designed for developers who need a lightweight, configurable registry for applications, services, or microservices.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub Repo](https://img.shields.io/badge/GitHub-Repo-blue.svg)](https://github.com/canmi21/namespaced)

## Features

- **API-Driven Management**: Create, read, update, and delete configuration values via HTTP endpoints.
- **Project-Based Organization**: Organize configurations into projects, each with its own base path for storage.
- **Dynamic Configuration**: Automatically reloads configuration changes without restarting the server.
- **Thread-Safe Operations**: Uses DashMap for concurrent access to ensure reliability in multi-threaded environments.
- **File Watcher**: Monitors the config file for changes and applies them in real-time.
- **Docker Support**: Easy deployment with Docker and Docker Compose.
- **Error Handling**: Robust error responses with appropriate HTTP status codes.
- **Listing and Existence Checks**: API endpoints for listing namespaces, paths, and checking existence of keys.

## Installation

### Prerequisites

- Rust (edition 2024 or later)
- Cargo (Rust's package manager)
- Docker (optional, for containerized deployment)

### Building from Source

1. Clone the repository:
   ```
   git clone https://github.com/canmi21/namespaced.git
   cd namespaced
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. Run the binary:
   ```
   cargo run --release
   ```

The server will start on port 19950 by default (configurable via the `PORT` environment variable).

### Using Docker

1. Build the Docker image:
   ```
   docker build -t namespaced .
   ```

2. Run the container:
   ```
   docker run -p 19950:19950 -v /opt/namespaced:/opt/namespaced namespaced
   ```

Alternatively, use Docker Compose for a more managed setup:

```
docker-compose up -d
```

This will start the service with persistent volumes and automatic restarts.

## Usage

Once running, the server exposes a REST API for managing projects and configurations. All operations are performed over HTTP.

### Environment Variables

- `PORT`: The port to listen on (default: 19950).
- `LOG_LEVEL`: Logging level (info, debug, warn, error; default: info).

Configuration is stored in `/opt/namespaced/pathmap.json` by default. This file maps project names to their base storage paths.

Example config:
```json
{
  "example_project": "/opt/ns/example"
}
```

## API Documentation

The API is divided into admin routes (for managing projects) and main API routes (for configuration data).

### Admin Routes

These routes manage projects in the configuration registry. They are prefixed with `/_namespaced`.

- **GET /_namespaced/projects**: List all projects.
  - Response: JSON array of projects.

- **POST /_namespaced/projects**: Create a new project.
  - Body: JSON `{ "name": "project_name", "path": "/base/path" }`
  - Response: 201 Created on success.

- **PUT /_namespaced/projects/{project}**: Update a project's path.
  - Body: JSON `{ "path": "/new/base/path" }`
  - Response: 200 OK on success.

- **DELETE /_namespaced/projects/{project}**: Delete a project.
  - Response: 204 No Content on success.

### Main API Routes

These routes handle configuration data within projects.

- **GET /ls/{project}**: List all namespaces in a project.
  - Response: JSON array of namespace strings.

- **GET /ls/{project}/{path}**: List contents (groups and values) at a specific path.
  - Response: JSON `{ "groups": ["group1"], "values": ["value1"] }`.

- **GET /exists/{project}/{path}**: Check if a path or key exists.
  - Response: 200 OK if exists, 404 Not Found otherwise.

- **GET /namespaced/{project}/{path}**: Retrieve a value at the path.
  - Response: JSON value.

- **POST /namespaced/{project}/{path}**: Set a new value (fails if exists).
  - Body: JSON value.
  - Response: 201 Created on success.

- **PUT /namespaced/{project}/{path}**: Overwrite a value (creates if not exists).
  - Body: JSON value.
  - Response: 200 OK on success.

- **DELETE /namespaced/{project}/{path}**: Delete a value or path.
  - Response: 204 No Content on success.

### Error Responses

Errors are returned as JSON with an `"error"` field, e.g.:
```json
{ "error": "Project 'unknown' not found" }
```
Common status codes: 400 Bad Request, 404 Not Found, 409 Conflict, 500 Internal Server Error.

## Configuration File Watching

The server watches `/opt/namespaced/pathmap.json` for changes. Any modifications (e.g., adding/removing projects) are applied automatically without downtime.

## Dependencies

- axum: Web framework
- dashmap: Concurrent hash map
- dotenvy: Environment variable loading
- fancy-log: Logging utility
- http: HTTP types
- lazy-motd: MOTD display
- notify: File watching
- pathmap: Core storage library
- serde & serde_json: Serialization
- thiserror: Error handling
- tokio: Async runtime

See `Cargo.toml` for versions.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

1. Fork the repository.
2. Create a feature branch.
3. Commit your changes.
4. Push to the branch.
5. Open a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.