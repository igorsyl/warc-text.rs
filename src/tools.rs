fn replace_a_to_b(str: &mut String, a: &str, b: &str) {
    let mut i = 0;
    while let Some(index) = str[i..].find(a) {
        let start = i + index;
        let end = start + a.len();
        str.replace_range(start..end, b);
        i = end;
    }
}

// fn explode(s: &str, e: &str, ret: &mut std::collections::HashSet<String>) {
//     let mut i_pos = 0;
//     let mut offset = 0;
//     let i_pit = e.len();
//     ret.clear();
//     while let Some(index) = s[offset..].find(e) {
//         let start = offset + index;
//         let end = start + i_pit;
//         ret.insert(s[offset..start].to_string());
//         offset = end;
//     }
//     if offset < s.len() {
//         ret.insert(s[offset..].to_string());
//     }
// }

fn implode(s: &Vec<String>, delim: &str) -> String {
    let mut ret = String::new();
    for (i, item) in s.iter().enumerate() {
        if i > 0 {
            ret.push_str(delim);
        }
        ret.push_str(item);
    }
    ret
}

/**
str: input
leftAndRight: left ('l'), right('r') or both ('b'), def: 'b'
punctuation: remove punctuation as well, def. false
 */ fn trim(str: &str, left_and_right: char, punctuation: bool) -> String {
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


// fn search_pattern(re: &str, str: &str) -> bool {
//     let r = pcrecpp::RE::new(re).unwrap();
//     let i = pcrecpp::StringPiece::from(str);
//     r.PartialMatch(i).unwrap_or(false)
// }

// // TODO: Does not handle "##a", "#" case properly fn split(inn: &str, sep: &mut str) -> Vec<String> {
// fn split(inn: &str, sep: &str) -> Vec<String> {
//     let mut v = Vec::new();
//     let mut s = inn.to_string();
//     let mut sep = sep.to_string();
//     if sep.is_empty() {
//         sep = String::from(",\\.?:;!+\"%/=\\(\\)\\n-");
//     }
//     let re = regex::Regex::new(&format!("([^{}]+)[]{}", sep, sep)).unwrap();
//     while let Some(caps) = re.captures(&s) {
//         let match_str = caps.get(1).unwrap().as_str();
//         v.push(match_str.to_string());
//         s = s[match_str.len() + sep.len()..].to_string();
//     }
//     if !s.is_empty() {
//         v.push(s);
//     }
//     v
// }

fn detect_code_page(html: &str) -> String {
    let mut out = String::new();
    let mut opt = pcre2::options().caseless(true);

    let r1 = pcre2::compile(
		r#"meta\s+http-equiv=['"]?content-type['"]?\s+content=['"]?[^'"]*charset=([^'"]+)"#,
		opt,
	).unwrap();
    let r2 = pcre2::compile(
		r#"meta\s+content=['"]?[^'"]*charset=([^'"]+)['"]?\s+http-equiv=['"]?content-type['"]?"#,
		opt,
	).unwrap();
    let r3 = pcre2::compile(
		r#"meta\s+http-equiv=['"]?charset['"]?\s+content=['"]?([^'"]+)"#,
		opt,
	).unwrap();
    let r4 = pcre2::compile(
		r#"meta\s+content=['"]?([^'"]+)['"]?\s+http-equiv=['"]?charset['"]?"#,
		opt,
	).unwrap();
    let r5 = pcre2::compile(
		r#"meta\s+charset=['"]?([^'"]+)"#,
		opt,
	).unwrap();

    if r1.partial_match(html, &mut out).is_some() {
        return out.clone();
    } else if r2.partial_match(html, &mut out).is_some() {
        return out.clone();
    } else if r3.partial_match(html, &mut out).is_some() {
        return out.clone();
    } else if r4.partial_match(html, &mut out).is_some() {
        return out.clone();
    } else if r5.partial_match(html, &mut out).is_some() {
        return out.clone();
    }

    String::new()
}

// fn to_lower(inn: &str) -> String {
//     let mut out = inn.to_string();
//     out.make_ascii_lowercase();
//     out
// }
