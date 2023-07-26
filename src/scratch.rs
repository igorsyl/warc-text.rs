// Failed at trying to disable libxml error messages
// unsafe {
// ctx = libxml::bindings::
// xmlSchemaSetValidErrors
// let c_string = std::ffi::CString::new("").unwrap();
// let ctxt = libxml::bindings::xmlSchemaNewParserCtxt(c_string.as_ptr());
// let schema = libxml::bindings::xmlSchemaParse(ctxt);
// libxml::bindings::xmlSchemaSetValidErrors(
//     libxml::bindings::xmlSchemaNewValidCtxt(schema),
//     libxml::bindings::xmlSchemaValidityErrorFunc::default(),
//     libxml::bindings::xmlSchemaValidityWarningFunc::default(),
//     std::ptr::null_mut(),
// );
// libxml::bindings::xmlSchemaSetParserErrors(
//     ctxt,
//     libxml::bindings::xmlSchemaValidityErrorFunc::default(),
//     libxml::bindings::xmlSchemaValidityWarningFunc::default(),
//     std::ptr::null_mut(),
// );
// libxml::bindings::xmlSchemaSetParserStructuredErrors(
//     libxml::bindings::xmlSchemaNewParserCtxt(c_string.as_ptr()),
//     libxml::bindings::xmlStructuredErrorFunc::default(),
//     std::ptr::null_mut(),
// );
// }

// let warc_file = std::fs::File::open(&warc_local_path).unwrap();
// let mut file_reader = std::io::BufReader::with_capacity(1_048_576, warc_file);
// let gzip_stream = gzip::GzipReader::new(file_reader);
// let gzip_reader = std::io::BufReader::new(gzip_stream);
// let warc_reader = warc::WarcReader::new(gzip_reader);
// let warc_reader = warc::WarcReader::from_path_gzip(warc_path).unwrap();
// let mut archive_records = ArchiveIterator::new(warc_file);

// let pb = indicatif::ProgressBar::new(warc_file_len);
//with_draw_target(None, indicatif::ProgressDrawTarget::stdout

// fn read_archive_records<R>(warc_reader: WarcReader<R>) -> impl Iterator<Item=String> where R: BufRead{
// fn read_archive_records(warc_file: std::fs::File) -> impl Iterator<Item=String> {
// struct ArchiveIterator<'a> {
//     warc_reader: warc::WarcReader<BufReader<libflate::gzip::MultiDecoder<BufReader<File>>>>,
//     streaming_iter : Option<StreamingIter<'a, BufReader<libflate::gzip::MultiDecoder<BufReader<File>>>>>,
// }

// impl<'a> ArchiveIterator<'a> {
//     fn new(warc_file: File) -> ArchiveIterator<'a> {
//         // let file = File::open(&path).unwrap();
//         let file_reader = std::io::BufReader::with_capacity(1_048_576, warc_file);
//         let gzip_stream = libflate::gzip::MultiDecoder::new(file_reader).unwrap();
//         let gzip_reader = std::io::BufReader::new(gzip_stream);
//         let mut warc_reader = WarcReader::new(gzip_reader);
//         // let gzip_stream = gzip::GzipReader::new(std::io::BufReader::with_capacity(1 * 1_048_576, warc_file));
//         // let warc_reader = warc::WarcReader::new(BufReader::new(gzip_stream));
//         // let warc_reader = warc::WarcReader::from_path_gzip(warc_path).unwrap();
// 		// let streaming_iter = warc_reader.stream_records();
//
//         let mut archive_iterator = ArchiveIterator {
//             warc_reader,
//             streaming_iter: None,
//         };
//         archive_iterator.streaming_iter = Some(archive_iterator.warc_reader.stream_records());
//         archive_iterator
//     }
// }

// impl Iterator for ArchiveIterator<'_> {
// 	type Item = String;
//

// #[derive(Debug)]
// struct Chunk {
//     name: &'static str,
//     value: String,
// }

// fn parse_content(doc: &libxml::tree::Document) -> anyhow::Result<String> {
// let mut chunks : Vec<Chunk> = vec![];

// while let Some(node) = queue.pop() {
//     for child in node.get_child_nodes() {
//         queue.push(child);
//     }
//
//     let tag_name_string = node.get_name();
//     let tag_name_str = tag_name_string.as_str();
//     match tag_name_str {
//         "script" | "style" => continue,
//         "img" if let Some(src) = node.get_attribute("src") => {
//             chunks.push(Chunk {
//                 name: "img",
//                 value: src,
//             });
//         }
//         "video" => {
//             for source in node.get_child_nodes() {
//                 match source.get_name().as_str() {
//                     "source" if let Some(src) = source.get_attribute("src") => {
//                         chunks.push(Chunk {
//                             name: "video",
//                             value: src,
//                         });
//                     }
//                     _ => {}
//                 }
//             }
//         }
//         _ => {
//             if node.is_text_node() {
//                 let content = &node.get_content();
//                 let content = content.trim().clone();
//                 if !content.is_empty() {
//                     chunks.push(Chunk {
//                         name: "text",
//                         value: content.to_string()
//                     });
//                 }
//             }
//         }
//     }
// }
//
// fn get_threshold_value(chunks: &Vec<Chunk>) -> usize {
//     let mut chunk_lengths: Vec<usize> = chunks.iter().filter(|chunk| chunk.name == "text").map(|chunk| chunk.value.len()).collect();
//
//     chunk_lengths.sort_by(|a, b| b.cmp(a));
//
//     let total_length: usize = chunk_lengths.iter().sum();
//
//     if total_length < 2 {
//         return total_length;
//     }
//
//     let mut cumm_length = 0;
//     let mut index = 0;
//
//     while index < chunk_lengths.len() - 1 && (cumm_length as f32 / total_length as f32) < 0.5 {
//         cumm_length += chunk_lengths[index];
//         index += 1;
//     }
//
//     let mut threshold_value = chunk_lengths[index];
//     threshold_value = threshold_value.max(100);
//     threshold_value
// }
//
// fn content(chunks: &[Chunk], threshold: usize) -> String {
//     let mut extract = String::new();
//
//     let mut text_count = 0;
//     for chunk in chunks {
//         match chunk.name {
//             "text" if chunk.value.len() >= threshold => {
//                 extract.push_str(&format!(" {} ", chunk.value));
//                 text_count += 1;
//             },
//             "img" => {
//                 if text_count > 0 {
//                     extract.push_str(&format!(" <IMAGE>{}</IMAGE> ", chunk.value))
//                 }
//             },
//             "video" => {
//                 if text_count > 0 {
//                     extract.push_str(&format!(" <VIDEO>{}</VIDEO> ", chunk.value))
//                 }
//             },
//             "audio" => {
//                 if text_count > 0 {
//                     extract.push_str(&format!(" <AUDIO>{}</AUDIO> ", chunk.value))
//                 }
//             },
//             _ => {}
//         }
//     }
//     extract
// }
