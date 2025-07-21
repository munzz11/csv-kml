use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;

use csv::Reader;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::collections::HashSet;
use std::collections::BTreeMap;

fn parse_timestamp(ts: &str) -> Option<i64> {
    // Try parsing as integer seconds since epoch
    if let Ok(secs) = ts.parse::<i64>() {
        return Some(secs);
    }
    // Try parsing as float seconds since epoch
    if let Ok(secs) = ts.parse::<f64>() {
        return Some(secs as i64);
    }
    // Try parsing as ISO8601 string
    if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
        return Some(dt.timestamp());
    }
    // Try parsing as naive datetime (e.g., 2024-07-21 12:34:56)
    if let Ok(dt) = NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S") {
        return Some(dt.timestamp());
    }
    None
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.csv> <output.kml>", args[0]);
        std::process::exit(1);
    }
    let input_path = &args[1];
    let output_path = &args[2];

    let mut rdr = Reader::from_path(input_path)?;
    let mut kml_buf = Vec::new();
    let mut writer = Writer::new(&mut kml_buf);

    // Write KML header
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    writer.write_event(Event::Start(BytesStart::new("kml")))?;
    writer.write_event(Event::Start(BytesStart::new("Document")))?;

    let mut last_written_ts: Option<i64> = None;
    let mut per_sec: BTreeMap<i64, (i64, String)> = BTreeMap::new();
    for result in rdr.records() {
        let record = result?;
        let timestamp = record.get(0).unwrap_or("");
        let latitude = record.get(1).unwrap_or("");
        let longitude = record.get(2).unwrap_or("");

        let ts = match parse_timestamp(timestamp) {
            Some(t) => t,
            None => continue, // skip if timestamp can't be parsed
        };
        let sec = ts;
        let coord = format!("{},{},0", longitude, latitude);
        // If this second is not present, or this point is closer to the second, keep it
        per_sec.entry(sec).and_modify(|entry| {
            let (prev_ts, prev_coord) = entry;
            if (ts - sec).abs() < (*prev_ts - sec).abs() {
                *prev_ts = ts;
                *prev_coord = coord.clone();
            }
        }).or_insert((ts, coord));
    }

    let mut coords_by_ts: Vec<(i64, String)> = per_sec.into_iter().map(|(_, v)| v).collect();
    if !coords_by_ts.is_empty() {
        coords_by_ts.sort_by_key(|(ts, _)| *ts);
        let coords: Vec<String> = coords_by_ts.iter().map(|(_, c)| c.clone()).collect();
        // Write a LineString path
        let placemark = BytesStart::new("Placemark");
        writer.write_event(Event::Start(placemark))?;
        // Add a visible red line style
        writer.write_event(Event::Start(BytesStart::new("Style")))?;
        writer.write_event(Event::Start(BytesStart::new("LineStyle")))?;
        writer.write_event(Event::Start(BytesStart::new("color")))?;
        writer.write_event(Event::Text(BytesText::new("ff0000ff")))?; // Red, aabbggrr
        writer.write_event(Event::End(BytesEnd::new("color")))?;
        writer.write_event(Event::Start(BytesStart::new("width")))?;
        writer.write_event(Event::Text(BytesText::new("4")))?;
        writer.write_event(Event::End(BytesEnd::new("width")))?;
        writer.write_event(Event::End(BytesEnd::new("LineStyle")))?;
        writer.write_event(Event::End(BytesEnd::new("Style")))?;
        let linestring = BytesStart::new("LineString");
        writer.write_event(Event::Start(linestring))?;
        // Add altitudeMode
        writer.write_event(Event::Start(BytesStart::new("altitudeMode")))?;
        writer.write_event(Event::Text(BytesText::new("clampToGround")))?;
        writer.write_event(Event::End(BytesEnd::new("altitudeMode")))?;
        let coordinates = BytesStart::new("coordinates");
        writer.write_event(Event::Start(coordinates))?;
        let coord_text = coords.join(" ");
        writer.write_event(Event::Text(BytesText::new(&coord_text)))?;
        writer.write_event(Event::End(BytesEnd::new("coordinates")))?;
        writer.write_event(Event::End(BytesEnd::new("LineString")))?;
        writer.write_event(Event::End(BytesEnd::new("Placemark")))?;
    }

    // Write KML footer
    writer.write_event(Event::End(BytesEnd::new("Document")))?;
    writer.write_event(Event::End(BytesEnd::new("kml")))?;

    let mut file = File::create(output_path)?;
    file.write_all(&kml_buf)?;
    println!("KML file written to {}", output_path);
    Ok(())
}
