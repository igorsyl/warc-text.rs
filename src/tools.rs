/**
str: input
leftAndRight: left ('l'), right('r') or both ('b'), def: 'b'
punctuation: remove punctuation as well, def. false
 */
fn trim(str: &str, left_and_right: char, punctuation: bool) -> String {
    let mut out = str.to_string();
    let re = if left_and_right == 'l' {
        if punctuation {
            regex::Regex::new(r"^[[:punct:]\s]+").unwrap()
        } else {
            regex::Regex::new(r"^\s+").unwrap()
        }
    } else if left_and_right == 'r' {
        if punctuation {
            regex::Regex::new(r"[[:punct:]\s]+$").unwrap()
        } else {
            regex::Regex::new(r"\s+$").unwrap()
        }
    } else {
        if punctuation {
            regex::Regex::new(r"^[[:punct:]\s]+|[[:punct:]\s]+$").unwrap()
        } else {
            regex::Regex::new(r"^\s+|\s+$").unwrap()
        }
    };
    re.replace_all(&mut out, "").to_string()
}

// fn detect_code_page(html: &str) -> String {
//     let mut out = String::new();
//     let mut opt = pcre2::options().caseless(true);
//
//     let r1 = pcre2::compile(
// 		r#"meta\s+http-equiv=['"]?content-type['"]?\s+content=['"]?[^'"]*charset=([^'"]+)"#,
// 		opt,
// 	).unwrap();
//     let r2 = pcre2::compile(
// 		r#"meta\s+content=['"]?[^'"]*charset=([^'"]+)['"]?\s+http-equiv=['"]?content-type['"]?"#,
// 		opt,
// 	).unwrap();
//     let r3 = pcre2::compile(
// 		r#"meta\s+http-equiv=['"]?charset['"]?\s+content=['"]?([^'"]+)"#,
// 		opt,
// 	).unwrap();
//     let r4 = pcre2::compile(
// 		r#"meta\s+content=['"]?([^'"]+)['"]?\s+http-equiv=['"]?charset['"]?"#,
// 		opt,
// 	).unwrap();
//     let r5 = pcre2::compile(
// 		r#"meta\s+charset=['"]?([^'"]+)"#,
// 		opt,
// 	).unwrap();
//
//     if r1.partial_match(html, &mut out).is_some() {
//         return out.clone();
//     } else if r2.partial_match(html, &mut out).is_some() {
//         return out.clone();
//     } else if r3.partial_match(html, &mut out).is_some() {
//         return out.clone();
//     } else if r4.partial_match(html, &mut out).is_some() {
//         return out.clone();
//     } else if r5.partial_match(html, &mut out).is_some() {
//         return out.clone();
//     }
//
//     String::new()
// }
