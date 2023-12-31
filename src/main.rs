#![feature(if_let_guard)]

mod justtext;
mod tools;
mod parser;

use clap::Parser;

use std::any::Any;
use std::fs::File;
use std::io::{BufRead, Write};
use anyhow::{anyhow, Context};
use rayon::prelude::*;
use crate::justtext::Justext;

const CC_REMOTE_PATH: &str = "https://data.commoncrawl.org";
const LOCAL_BASE_PATH: &str = "/Users/igor/Downloads/magic";
const WARCS_REL_PATH: &str = "crawl-data/CC-MAIN-2023-23/warc.paths.gz";

#[derive(clap_derive::Parser)]
struct Cli {
    #[clap(short, long, default_value_t = false)]
    debug: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let n_warc_paths = match args.debug {
        true => 1,
        false => 10,
    };

    let pbm = indicatif::MultiProgress::with_draw_target(indicatif::ProgressDrawTarget::stdout());
    let pbm = std::sync::Arc::new(std::sync::Mutex::new(pbm));
    // let pb1 = pbm.add(indicatif::ProgressBar::with_draw_target(None, indicatif::ProgressDrawTarget::stderr()));

    // retrieve warcs.path.gz
    let warcs_filename = WARCS_REL_PATH.split('/').last().unwrap();
    let warcs_remote_path = format!("{}/{}", CC_REMOTE_PATH, WARCS_REL_PATH);
    let warcs_local_path = format!("{}/warcs/{}", LOCAL_BASE_PATH, warcs_filename);
    retrieve_file(&warcs_remote_path, &warcs_local_path, &pbm)?;

    // extract list of warc files
    let warc_path_iterator = read_warc_paths(&warcs_local_path)?;
    let mut warc_paths: Vec<String> = warc_path_iterator.take(n_warc_paths).collect();

    // retrieve warc files
    warc_paths.clone().into_par_iter().for_each(|warc_path| {
        retrieve_warc_file(&warc_path, &pbm).unwrap();
    });

    // extract warc files
    warc_paths.clone().into_par_iter().for_each(|warc_path| {
        extract_warc_file(&warc_path, &pbm).unwrap();
    });

    Ok(())
}

fn get_progress_style() -> indicatif::ProgressStyle {
    indicatif::ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] {msg} {bytes}/{total_bytes} {bytes_per_sec} {eta} [{wide_bar:.cyan/blue}]").unwrap()
}

#[derive(Debug)]
struct ExtractResult {
    content: String,
    // content_annotated: String,
}

fn extract_content(http_response: &str) -> anyhow::Result<ExtractResult> {
    let http_body_start_index = http_response.find("\r\n\r\n").ok_or(anyhow::anyhow!("no newline found"))?;
    let http_body = &http_response[http_body_start_index + 4..];

    let mut parser_options = libxml::parser::ParserOptions::default();
    parser_options.no_blanks = true;
    parser_options.no_net = true;
    let xml_parser = libxml::parser::Parser::default_html();
    let document = xml_parser.parse_string_with_options(http_body, parser_options)?;

    let mut paragraph_parser = parser::Parser::new();
    paragraph_parser.walk_tree(&document)?;
    let mut jt = Justext::new();
    let content = jt.get_content(&mut paragraph_parser);

    Ok(ExtractResult {
        content,
        // annotated_html: annotated.to_string(),
    })
}

fn extract_warc_file(warc_path: &str, pbm: &std::sync::Arc<std::sync::Mutex<indicatif::MultiProgress>>) -> anyhow::Result<()> {
    let write_extracted = true;
    let write_annotated = false;

    let warc_filename = warc_path.split('/').last().unwrap();
    let warc_local_path = format!("{}/warcs/{}", LOCAL_BASE_PATH, warc_filename);

    let warc_file = File::open(&warc_local_path)?;
    let pb = pbm.lock().unwrap().add(indicatif::ProgressBar::with_draw_target(None, indicatif::ProgressDrawTarget::stdout()));
    pb.set_style(get_progress_style());
    pb.set_message(format!(" {}", warc_filename).clone());
    pb.set_length(warc_file.metadata()?.len());

    let warc_file = pb.wrap_read(warc_file);

    let file_reader = std::io::BufReader::with_capacity(1_048_576, warc_file);
    let gzip_stream = libflate::gzip::MultiDecoder::new(file_reader)?;
    let gzip_reader = std::io::BufReader::new(gzip_stream);
    let warc_reader = warc::WarcReader::new(gzip_reader);

    let mut extract_file = std::fs::File::create(format!("{}/extract/{}_extract", LOCAL_BASE_PATH, warc_filename.split('.').next().unwrap()))?;
    // let mut extracted_file = libflate::gzip::Encoder::new(&mut extracted_file).unwrap();

    // let mut extracted_file = pb.wrap_write(extracted_file);
    // let mut annotated_file = std::fs::File::create(format!("{}/annotated.html", LOCAL_BASE_PATH)).unwrap();
    iter_contents(warc_reader, pbm, |parse_result| {
        match parse_result {
            Ok(extract_result) => {
                // pb2.inc(1);
                // println!("record {}", result.text_record.len());
                let extract_content = &extract_result.content;
                if write_extracted && !extract_content.is_empty() {
                    // let extract_content = &extract_result.content.replace("\n", " ");
                    extract_file.write_all(format!("{}\t", extract_content.len()).as_bytes()).unwrap();
                    extract_file.write_all(extract_content.as_bytes()).unwrap();
                    extract_file.write_all(b"\n").unwrap();
                }
                if write_annotated {
                    // annotated_file
                    //     .write_all(result.annotated_html.as_bytes())
                    //     .unwrap();
                    // annotated_file.write_all(b"<hr>").unwrap();
                }
            }
            Err(e) => {}
        }
    });

    Ok(())
}

fn iter_contents<R, F>(mut warc_reader: warc::WarcReader<R>, pbm: &std::sync::Arc<std::sync::Mutex<indicatif::MultiProgress>>, mut f: F)
    where R: BufRead, F: FnMut(anyhow::Result<ExtractResult>) {
    // let mut pbs: Vec<ProgressBar> = vec![];
    // for _ in 0..6 {
    //     let pbi = pb.add(indicatif::ProgressBar::with_draw_target(None, indicatif::ProgressDrawTarget::stderr()));
    //     pbi.set_style(indicatif::ProgressStyle::with_template("{pos}").unwrap());
    //     pbs.push(pbi);
    // }

    let mut warc_streaming_iter = warc_reader.stream_records();
    while let Some(warc_record_streaming_body) = warc_streaming_iter.next_item() {
        // pb1.inc(1);
        match warc_record_streaming_body {
            Ok(warc_response_record_streaming_body) => {
                match warc_response_record_streaming_body.warc_type() {
                    warc::RecordType::Response | warc::RecordType::Continuation => {
                        // pbs[0].inc(1);
                        match warc_response_record_streaming_body.into_buffered() {
                            Ok(warc_response_record_buffered_body) => {
                                // pb4.inc(1);
                                let warc_response_body_bytes = warc_response_record_buffered_body.body();
                                match std::str::from_utf8(warc_response_body_bytes) {
                                    Ok(warc_response_body_str) => {
                                        // pb5.inc(1);
                                        let http_response = warc_response_body_str;
                                        match extract_content(&http_response) {
                                            Ok(extract_result) => {
                                                // pbs[1].inc(1);
												f(Ok(extract_result))
                                            }
                                            Err(e) => f(Err(anyhow!(e)).with_context(|| "extract_content"))
                                        }
                                    }
                                    Err(e) => f(Err(anyhow!(e)).with_context(|| "from_utf8"))
                                }
                            }
                            Err(e) => f(Err(anyhow!(e)).with_context(|| "into_buffered"))
                        }
                    }
                    _ => {}
                    // _ => pb3.inc(1)
                }
            }
            Err(warc::Error::UnexpectedEOB) => {
                println!("UnexpectedEOB");
                break;
            }
            Err(e) => f(Err(anyhow!(e)).with_context(|| "next_item"))
        };
    }
}

fn retrieve_warc_file(warc_path: &str, pbm: &std::sync::Arc<std::sync::Mutex<indicatif::MultiProgress>>) -> anyhow::Result<()> {
    let warc_filename = warc_path.split('/').last().unwrap();
    let warc_remote_path = format!("{}/{}", CC_REMOTE_PATH, warc_path);
    let warc_local_path = format!("{}/warcs/{}", LOCAL_BASE_PATH, warc_filename);
    retrieve_file(&warc_remote_path, &warc_local_path, pbm)
}

fn retrieve_file(remote_path: &str, local_path: &str, pbm: &std::sync::Arc<std::sync::Mutex<indicatif::MultiProgress>>) ->anyhow::Result<()> {
    if std::path::Path::new(&local_path).exists() {
        return Ok(());
    }
    let mut response = reqwest::blocking::get(remote_path)?;

    let pb = pbm.lock().unwrap().add(indicatif::ProgressBar::with_draw_target(None, indicatif::ProgressDrawTarget::stdout()));
    pb.set_style(get_progress_style());

    pb.set_message(remote_path.split('/').last().unwrap().to_string());
    pb.set_length(response.content_length().unwrap());

    let local_path_temp = format!("{}.tmp", local_path);
    let file = std::fs::File::create(&local_path_temp)?;
    let mut file = pb.wrap_read(file);
    std::io::copy(&mut response, &mut file)?;
    std::fs::rename(&local_path_temp, &local_path)?;
    Ok(())
}

fn read_warc_paths(warcs_path: &str) -> anyhow::Result<impl Iterator<Item=String>> {
    let file = std::fs::File::open(warcs_path)?;
    let reader = std::io::BufReader::new(file);
    let decoder = flate2::bufread::GzDecoder::new(reader);
    let reader = std::io::BufReader::new(decoder);
    Ok(std::io::BufReader::lines(reader).map(|s| s.unwrap().to_string()))
}
