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
- 🌍 **CORS Support**: Configured to allow cross-origin requests, making it easier to integrate with frontend applications.
- 🛠 **Unit-Tested**: Includes tests for configuration loading and request handling.
- 🔐 **Authorization**: Supports mocking of **Basic Authentication** and **Bearer Token Authentication**:
    - **Basic Auth**: Validates username and password based on the configuration.
    - **Bearer Token**: Validates tokens and their claims, ensuring that the token matches expected values and claims (e.g., roles, permissions).
- ⚙️ **Custom Configuration**:
  - Load a custom `config.yaml` file using a command-line argument (`--file`).
  - Set the server to listen on a custom port using `--port`.
  - Set a custom response folder location for the mockserver `--responses-folder`
- 📝 Logging: Enables detailed request logging for easier debugging.
- - ⏳ **Configurable Delays**: Simulate slow or rate-limited APIs by defining a `delay` field in `config.yaml`.
- Add an artificial delay (in milliseconds) before responding to requests.
- Useful for testing timeout handling and performance in client applications.
- 🚧 **Rate Limiting**: Control the number of requests allowed per endpoint within a specified time window:
    - **Requests per window**: Define the maximum number of requests allowed in a given time window (in milliseconds).
    - **Separate counters per method**: Rate limits are tracked separately for different HTTP methods (e.g., `GET`, `POST`).
    - **429 Too Many Requests**: Returns a `429` status code when the rate limit is exceeded.


## Installation


### Download and Run
1. **Download the latest release** for **Linux**, **Windows**, or **macOS** from the [Releases Page](https://github.com/sfeSantos/mockserver/releases).
2. Extract the downloaded file.
3. Run the server with:
  - **Linux/macOS**:
    ```sh
    ./mockserver --file config.yaml --port 8080 --responses-folder folder_location
    ```
  - **Windows** (PowerShell) add the extension `.exe` and then run the following line:
    ```sh
    .\mockserver.exe --file config.yaml --port 8080 --responses-folder folder_location
    ```
4. Replace:
  - `config.yaml` with your configuration file.
  - `8080` with the desired port.
  - `folder_location` with the directory for mock responses.

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
