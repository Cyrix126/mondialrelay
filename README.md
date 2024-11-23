# MondialRelay-rs
## Status of developmenent
**Work in Progress**  
Version 0.1.0 is in a working state, but not yet tested in production.
## About
mondialrelay-rs is a repository to install an API on your server that will interact with Mondial Relay API to allow your API receiving orders to create a shipment and the worker to print the label for an order.
## Objective
To have a modular API not tied to any CMS which is fast and efficient.
Do not fetch the data of relays, let the client do it. The APU retrieve only the id of the relay/locker.
## Features
- create shipment
- store order_id/label url/date
- return tracking id
- provide label url from order id
## Installation
Working installation on most Linux distribution, but not using opt/ or systemd.
```
git clone https://github.com/Cyrix126/mondialrelay
cargo build --release
sudo cp -r api/config /etc/mondialrelay
sudo cp target/release/mondialrelay-api-server /usr/bin
```
You can also create a init system service, like systemd or dinit.
## Usage
After installing setting your config file, you can start mondialrelay-server-api with DEBUG log the first time.
```
RUST_LOG=debug mondialrelay-server-api
```
## Example
## Bug Reporting
Create an issue on github
## Contributing
## Security
## Documentation
## License
GNU GPL v3
