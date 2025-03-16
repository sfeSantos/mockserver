# Mock Server (Rust)

## Overview
This project is a **dynamic mock server** written in Rust. It reads a YAML configuration file to define API endpoints, supported HTTP methods, and response files. The main goal is to provide a simple, file-based mock API server that supports:

- **GET requests**: Returns JSON responses from predefined files.
- **POST & PUT requests**: Saves incoming request bodies as JSON files.
- **DELETE requests**: Responds with HTTP `204 No Content`.
- **Local file storage only**: All response files are stored in the `responses/` directory.

## Why This Project?
Many developers need quick and flexible mock servers to simulate backend APIs during frontend or integration testing. This project provides a **lightweight and fast** alternative to heavy solutions like JSON Server or WireMock.

## Features
- 🚀 **Fast**: Uses `warp` and `tokio` for high-performance asynchronous processing.
- 📜 **Easy Configuration**: Define endpoints via a simple `config.yaml` file.
- 💾 **File-Based Storage**: Store and retrieve JSON responses without a database.
- 🔄 **Dynamic API Handling**: Automatically updates responses with `POST`/`PUT`.
- 🛠 **Unit-Tested**: Includes tests for configuration loading and request handling.
- 🔐 **Authorization**: Supports mocking of **Basic Authentication** and **Bearer Token Authentication**:
    - **Basic Auth**: Validates username and password based on the configuration.
    - **Bearer Token**: Validates tokens and their claims, ensuring that the token matches expected values and claims (e.g., roles, permissions).

## Installation
### Prerequisites
- Install [Rust](https://www.rust-lang.org/tools/install)

### Clone and Build
```sh
git clone https://github.com/your-repo/mockserver.git
cd mockserver
cargo build --release
```

## Configuration
```yaml
/api/user:
  method: GET
  file: user_response.json
  authentication:
    basic:
      user: 'admin'
      password: 'secret'

/api/order:
  method: POST
  file: order_data.json
  status_code: 202 #custom Http status code
  authentication:
    bearer:
      token: 'valid_token'
      claims:
        role: 'admin'
```
This means:
- `GET /api/user` &rarr; Returns `response/user_reponse.json`
- `POST /api/order` &rarr; Returns `response/order_data.json`

## Running the server
```sh
cargo run
```
Server starts on http://localhost:8080

## Example Usage
### Retrieve a mock Response

```sh
curl http://localhost:8080/api/user
```

### Save/Update Data
```sh
curl -X POST http://localhost:8080/api/order -d '{"item": "Laptop"}' -H "Content-Type: application/json"
```

### Delete Data
```sh
curl -X DELETE http://localhost:8080/api/order
```

### Running Testss
```sh
cargo test
```

## Contributions
Contributions are welcome! Feel free to submit issues or pull requests.

## License
This project is licensed under the **MIT License** - see the full details at [MIT License](https://mit-license.org/).
