# Phish Set List Cloudflare Worker

This is caching layer for the phish net API that serves set list data for Phish shows. It is built with Rust and WebAssembly for optimal performance. It uses d1 to cache the data and runs on cloudflare workers.

This is meant to solve the problem of user oriented queries for set list data. The phish net API is great for getting all the data or individual shows, but it is not great for getting multiple shows at once. This worker solves that problem by allowing users to query multiple shows at once, filling in gaps from the phish net API as it goes.

## Features

- Serves Phish set list data a d1 database
- Allows filtering of setlists by one or multiple show IDs
- Built with Rust and WebAssembly for optimal performance

## Usage

The worker responds to HTTP GET requests and accepts optional `showid` query parameters for filtering.

### Endpoints

- `/`: Returns a 400, just hit the phish net API???
- `/?showid=XXXXXXXX`: Returns setlist data for a specific show ID
- `/?showid=XXXXXXXX&showid=YYYYYYYY`: Returns setlist data for multiple show IDs

### Examples

1. Get a 400 error:
   ```
   GET /
   ```

2. Get setlist for a specific show:
   ```
   GET /?showid=1252683584
   ```

3. Get setlists for multiple shows:
   ```
   GET /?showid=1252683584&showid=1234567890
   ```

## Response Format

The API returns JSON data in the following format:

```json
[
  {
    "showid": "1252683584",
    "showdate": "1997-11-22",
    "permalink": "https://phish.net/setlists/phish-november-22-1997-hampton-coliseum-hampton-va-usa.html",
    "venue": "Hampton Coliseum",
    "city": "Hampton",
    "state": "VA",
    "country": "USA",
    "song": "Mike's Song",
    // ... other fields
  },
  // ... other songs in the setlist
]
```

## Development

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [wrangler](https://developers.cloudflare.com/workers/wrangler/install-and-update/)

### Local Development

1. Clone the repository:
   ```
   git clone https://github.com/paradise-runner/phish-multi-setlist.git
   cd phish-multi-setlist
   ```

2. Install dependencies:
   ```
   cargo build
   ```

3. Run the worker locally:
   ```
   wrangler dev
   ```

### Deployment

Deploy the worker to Cloudflare:

```
wrangler publish
```

## License

[GPL v3 License](LICENSE)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.