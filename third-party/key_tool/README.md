# transfer zksync key to halo2 key

The key in zkSync use the pairing_ce library and encode/decode using uncompressed data. 
The key in Halo2 use the pairing library and encode/decode using compressed data.

The pairing library does not provide encoding for uncompressed data, so a tool is needed to convert zkSync key to Halo2 key

## Features

- Converts zkSync key to Halo2 key

## Usage

To run the tool, use the following command:

```sh
cargo run -- --k=22

replace 22 with the appropriate value you want to convert.