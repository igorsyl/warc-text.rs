use crate::parser::{Paragraph, Parser};

const MAX_LINK_DENSITY_DEFAULT: f32 = 0.2;
const LENGTH_LOW_DEFAULT: i32 = 70;
const LENGTH_HIGH_DEFAULT: i32 = 200;
const STOPWORDS_LOW_DEFAULT: f32 = 0.30;
const STOPWORDS_HIGH_DEFAULT: f32 = 0.32;
const NO_HEADINGS_DEFAULT: bool = false;
const MAX_HEADING_DISTANCE_DEFAULT: i32 = 200;


pub struct Justext {
    dom_path: String,
    text_nodes: Vec<String>,
    text: String,
    cfclass: String,
    finalclass: String,
    heading: bool,
    bullet: bool,
    word_count: i64,
    linked_char_count: i64,
    tag_count: i64,
    stopword_count: i64,
    stopword_density: f32,
    link_density: f32,
    m_htmlPosition1: i64,
    m_htmlPosition2: i64,
    m_tag: String,
    m_htmlSrc: String,
    m_originalTags: Vec<String>,
    m_htmlParents: Vec<String>,
}

impl Justext {
    pub fn new() -> Justext {
        Justext {
            dom_path: String::new(),
            text_nodes: Vec::new(),
            text: String::new(),
            cfclass: String::new(),
            finalclass: String::new(),
            heading: false,
            bullet: false,
            word_count: 0,
            linked_char_count: 0,
            tag_count: 0,
            stopword_count: 0,
            stopword_density: 0.0,
            link_density: 0.0,
            m_htmlPosition1: 0,
            m_htmlPosition2: 0,
            m_tag: String::new(),
            m_htmlSrc: String::new(),
            m_originalTags: Vec::new(),
            m_htmlParents: Vec::new(),
        }
    }

    pub fn get_content(&self, parser: &mut Parser) -> String {
        let mut out = String::new();
        classify_paragraphs(&mut parser.paragraphs);
        revise_paragraph_classification(&mut parser.paragraphs, MAX_HEADING_DISTANCE_DEFAULT);
        // if self.m_debug {
        //     self.make_debug_output(fsm.get_para(), "test/debugJusText.html", url, encoding);
        // }
        out = output_default(parser.get_para(), true, self.m_cleanEvalFormat);
        if out.is_empty() {
            parser.set_good(false);
        } else {
            parser.set_good(true);
        }
        out
    }
}

fn classify_paragraphs(paragraphs: &mut Vec<Paragraph>) {
    for paragraph in paragraphs.iter_mut() {
        let length = paragraph.text.len();
        let stopword_count = paragraph.text.split(|c| c.is_whitespace()).count() - 1;
        let words: Vec<String> = paragraph.text.split(|c| c.is_whitespace()).map(|s| s.to_string()).collect();
        let stopword_density = stopword_count as f32 / words.len() as f32;
        let link_density = paragraph.linked_char_count as f32 / length as f32;
        paragraph.stopword_count = stopword_count as i64;
        paragraph.stopword_density = stopword_density;
        paragraph.link_density = link_density;

        paragraph.heading = false;
        paragraph.bullet = false;
        if !m_no_headings && search_pattern("(^h\\d|\\.h\\d)", &paragraph.dom_path) {
            paragraph.heading = true;
        } else if search_pattern("(^li|\\.li)", &paragraph.dom_path) {
            paragraph.bullet = true;
        }

        if link_density > m_max_link_density {
            paragraph.cfclass = "bad";
        } else if paragraph.text.contains("\u{a9}") || paragraph.text.contains("&copy") {
            paragraph.cfclass = "bad";
        } else if search_pattern("(^select|\\.select)", &paragraph.dom_path) {
            paragraph.cfclass = "bad";
        } else {
            if length < m_length_low {
                if paragraph.linked_char_count > 0 {
                    paragraph.cfclass = "bad";
                } else {
                    paragraph.cfclass = "short";
                }
            } else {
                if stopword_density >= m_stopwords_high {
                    if length > m_length_high {
                        paragraph.cfclass = "good";
                    } else {
                        paragraph.cfclass = "neargood";
                    }
                } else if stopword_density >= m_stopwords_low {
                    paragraph.cfclass = "neargood";
                } else {
                    paragraph.cfclass = "bad";
                }
            }
        }
    }
}

fn _get_neighbour(i: usize, paragraphs: &[Paragraph], ignore_neargood: bool, inc: usize, boundary: usize) -> String {
    let mut i = i + inc;
    while i != boundary {
        let c = &paragraphs[i].finalclass;
        if c == "good" || c == "bad" {
            return c.to_string();
        }
        if c == "neargood" && !ignore_neargood {
            return c.to_string();
        }
        i += inc;
    }
    "bad".to_string()
}

fn get_prev_neighbour(i: usize, paragraphs: &[Paragraph], ignore_neargood: bool) -> String {
    // Return the class of the paragraph at the top end of the short/neargood
    // paragraphs block. If ignore_neargood is true, than only 'bad' or 'good'
    // can be returned, otherwise 'neargood' can be returned, too.
    _get_neighbour(i, paragraphs, ignore_neargood, -1, -1)
}

fn get_next_neighbour(i: usize, paragraphs: &mut [Paragraph], ignore_neargood: bool) -> String {
    //Return the class of the paragraph at the bottom end of the short/neargood
    //paragraphs block. If ignore_neargood is True, than only 'bad' or 'good'
    //can be returned, otherwise 'neargood' can be returned, too.
    _get_neighbour(i, paragraphs, ignore_neargood, 1, paragraphs.len())
}

fn revise_paragraph_classification(paragraphs: &mut [Paragraph], max_heading_distance: i32) {
    // Context-sensitive paragraph classification. Assumes that classify_pragraphs
    // has already been called.
    // Copy classes
    for paragraph in paragraphs.iter_mut() {
        paragraph.finalclass = paragraph.cfclass;
    }

    // Good headings
    let mut j = 0;
    let mut distance = 0;
    for (i, paragraph) in paragraphs.iter().enumerate() {
        if !paragraph.heading || paragraph.finalclass != "short" {
            continue;
        }
        j = i + 1;
        distance = 0;
        while j < paragraphs.len() && distance <= max_heading_distance {
            if &paragraphs[j].finalclass == "good" {
                paragraphs[i].finalclass = "neargood";
                break;
            }
            distance += paragraphs[j].text.len() as i32;
            j += 1;
        }
    }

    // Classify short
    let mut new_classes: Vec<&str> = vec![];
    new_classes.resize(paragraphs.len(), "");
    for i in 0..paragraphs.len() {
        if &paragraphs[i].finalclass != "short" {
            continue;
        }
        let prev_neighbour = get_prev_neighbour(i, paragraphs, true);
        let next_neighbour = get_next_neighbour(i, paragraphs, true);
        if prev_neighbour == "good" && next_neighbour == "good" {
            new_classes[i] = "good";
        } else if prev_neighbour == "bad" && next_neighbour == "bad" {
            new_classes[i] = "bad";
        } else if (prev_neighbour == "bad" && get_prev_neighbour(i, paragraphs, false) == "neargood") || (next_neighbour == "bad" && get_next_neighbour(i, paragraphs, false) == "neargood") {
            new_classes[i] = "good";
        } else {
            new_classes[i] = "bad";
        }
    }

    for (i, paragraph) in paragraphs.iter_mut().enumerate() {
        if new_classes[i].len() != 0 {
            paragraph.finalclass = new_classes[i].clone();
        }
    }

    // Revise neargood
    for i in 0..paragraphs.len() {
        if &paragraphs[i].finalclass != "neargood" {
            continue;
        }
        let prev_neighbour = get_prev_neighbour(i, paragraphs, true);
        let next_neighbour = get_next_neighbour(i, paragraphs, true);
        if prev_neighbour == "bad" && next_neighbour == "bad" {
            paragraphs[i].finalclass = "bad";
        } else {
            paragraphs[i].finalclass = "good";
        }
    }

    // More good headings
    for (i, paragraph) in paragraphs.iter_mut().enumerate() {
        if !paragraph.heading || paragraph.finalclass != "bad" || paragraph.cfclass == "bad" {
            continue;
        }
        j = i + 1;
        distance = 0;
        while j < paragraphs.len() && distance <= max_heading_distance {
            if &paragraphs[j].finalclass == "good" {
                paragraphs[i].finalclass = "good";
                break;
            }
            distance += paragraphs[j].text.len() as i32;
            j += 1;
        }
    }
}

fn wrap_text(str: &str, len: usize) -> String {
    let mut r = str.to_string();
    let mut line_width = 0;
    let mut last_space = 0;
    let mut next_s = 0;
    let mut next_n = 0;
    for mut i in 0..r.len() {
        next_s = r[i..].find(' ').unwrap_or(r.len() - i);
        if r[next_s..next_s + 1] == "\n" {
            line_width = 0;
            last_space = next_s;
            i += next_s + 1;
            continue;
        }

        line_width += next_s - last_space + 1;

        if line_width > len {
            r.insert_str(last_space + 1, "\n");
            line_width = next_s - last_space + 1;
        }

        i += next_s + 1;
        last_space = next_s;
    }
    r
}

fn output_default(paragraphs: Vec<paragraph>, no_boilerplate: bool, full: bool) -> String {
    let mut out = String::new();
    for paragraph in paragraphs {
        let mut tag = String::new();
        if paragraph.finalclass == "good" {
            if paragraph.heading {
                tag = "h".to_string();
            } else if paragraph.bullet {
                tag = "l".to_string();
            } else {
                tag = "p".to_string();
            }
            if !full {
                out.push_str(&paragraph.text);
                out.push_str("\n");
            }
        } else {
            if no_boilerplate {
                continue;
            } else {
                tag = "b".to_string();
            }
        }
        ReplaceAtoB(&mut paragraph.text, "&nbsp;", " ");
        ReplaceAtoB(&mut paragraph.text, "&quot;", "\"");
        ReplaceAtoB(&mut paragraph.text, "&gt;", ">");
        ReplaceAtoB(&mut paragraph.text, "&lt;", "<");
        if full {
            let tmp = wrapText(&paragraph.text, 80);
            if paragraph.m_tag == "pre" {
                let re = regex::Regex::new(r"\n>?[ \t]?\n>?").unwrap();
                out.push_str(&re.replace_all(&tmp, "\n\n<" + &tag + ">").to_string());
            } else {
                out.push_str("<" + &tag + ">");
                out.push_str(&tmp);
                out.push_str("\n\n");
            }
        }
    }
    out
}
