# csv-kml-tool

A simple Rust CLI tool to convert a CSV file with columns `timestamp,latitude,longitude,altitude,source_file` to a KML file containing placemarks for each latitude/longitude pair.

## Usage

```
cargo run --release -- <input.csv> <output.kml>
```

- `<input.csv>`: Path to the input CSV file.
- `<output.kml>`: Path to the output KML file.

## CSV Format

The CSV file should have the following columns (header required):

```
timestamp,latitude,longitude,altitude,source_file
...
```

## Dependencies
- [csv](https://crates.io/crates/csv)
- [quick-xml](https://crates.io/crates/quick-xml)

## Example

```
cargo run --release -- data.csv output.kml
``` 