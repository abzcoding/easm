# EASM Integration Tests

This directory contains integration tests for the EASM project, including API and frontend tests.

## Prerequisites

Before running integration tests, ensure you have:

1. A running PostgreSQL database with test data
2. The API server running on port 8080
3. A WebDriver server for frontend tests (ChromeDriver or Firefox WebDriver) running on port 4444

## Setting Up Test Environment

### Database

Make sure your database is prepared with test data. You can use the migrations in the project:

```sh
sqlx database create
sqlx migrate run
```

### Running the API Server

Start the API server for testing:

```sh
cd crates/api
cargo run
```

### Running the Frontend

In a separate terminal:

```sh
cd crates/frontend
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' trunk serve --port 8080
```

### WebDriver for Frontend Tests

For frontend tests, you need a WebDriver. You can use ChromeDriver or WebDriver for Firefox:

#### ChromeDriver

```sh
# Install ChromeDriver (macOS)
brew install --cask chromedriver

# Start ChromeDriver
chromedriver --port=4444
```

#### Firefox WebDriver (GeckoDriver)

```sh
# Install GeckoDriver (macOS)
brew install geckodriver

# Start GeckoDriver
geckodriver --port 4444
```

## Running the Tests

### Run All Tests

To run all integration tests:

```sh
cd tests
cargo test
```

### Run Specific Test Suite

To run only the API integration tests:

```sh
cd tests
cargo test --test api_integration_test
```

To run only the frontend integration tests:

```sh
cd tests
cargo test --test frontend_integration_test
```

### Test Isolation

The integration tests are designed to create and clean up after themselves. However, if tests fail unexpectedly, you might need to manually clean up test data.

## Troubleshooting

### Common Issues

1. **Connection Refused**: Make sure API server is running on port 8080
2. **WebDriver Connection Failed**: Check if WebDriver is running on port 4444
3. **Database Errors**: Verify database connection string and schema

### Debugging Tests

To see more detailed output during test runs:

```sh
RUST_LOG=debug cargo test -- --nocapture
```

## Adding New Tests

When adding new integration tests:

1. Create a new test file in the tests directory
2. Add the test to `Cargo.toml` using the `[[test]]` syntax
3. Ensure the test cleans up after itself to avoid affecting other tests 