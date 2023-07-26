use std::collections::HashSet;
use std::fs::{File, read_to_string};
use std::io::{BufRead, Read};
use lazy_static::lazy_static;
use regex::Regex;
use crate::parser::{Paragraph, Parser};

const MAX_LINK_DENSITY_DEFAULT: f32 = 0.2;
const LENGTH_LOW_DEFAULT: i32 = 70;
const LENGTH_HIGH_DEFAULT: i32 = 200;
const STOPWORDS_LOW_DEFAULT: f32 = 0.30;
const STOPWORDS_HIGH_DEFAULT: f32 = 0.32;
const NO_HEADINGS_DEFAULT: bool = false;
const MAX_HEADING_DISTANCE_DEFAULT: i32 = 200;

lazy_static! {
    pub static ref JUSTEXT_RE1 : Regex = Regex::new(r"(^h\\d|\\.h\\d)").unwrap();
    pub static ref JUSTEXT_RE2 : Regex = Regex::new(r"(^li|\\.li)").unwrap();
    pub static ref JUSTEXT_RE3 : Regex = Regex::new(r"(^select|\\.select)").unwrap();
    pub static ref STOPLIST : HashSet<String> = read_to_string("stoplists/English.txt").unwrap().lines().map(|s| s.to_string()).collect();
}

#[derive(Debug, Clone, Default)]
pub struct Justext {
    m_length_low: i64,
    m_length_high: i64,
    m_stopwords_low: f32,
    m_stopwords_high: f32,
    m_max_link_density: f32,
    m_no_headings: bool,
    m_debug: bool,
}

impl Justext {
    pub fn new() -> Justext {
        Justext::default()
    }

    pub fn get_content(&mut self, parser: &mut Parser) -> String {
        self.classify_paragraphs(&mut parser.m_paragraphs);
        self.revise_paragraph_classification(&mut parser.m_paragraphs, MAX_HEADING_DISTANCE_DEFAULT);
        // if self.m_debug {
        //     self.make_debug_output(fsm.get_para(), "test/debugJusText.html", url, encoding);
        // }
        self.output_default(&mut parser.m_paragraphs)
    }

    fn classify_paragraphs(&mut self, paragraphs: &mut Vec<Paragraph>) {
        for paragraph in paragraphs.iter_mut() {
            let length = paragraph.text.len() as i64;
            let stopword_count = paragraph.text.split(|c : char| c.is_whitespace()).filter(|s| STOPLIST.contains(*s)).count();

            let words: Vec<String> = paragraph.text.split(|c : char| c.is_whitespace()).map(|s| s.to_string()).collect();
            let stopword_density = stopword_count as f32 / words.len() as f32;
            let link_density = paragraph.linked_char_count as f32 / length as f32;
            paragraph.stopword_count = stopword_count as i64;
            paragraph.stopword_density = stopword_density;
            paragraph.link_density = link_density;

            paragraph.heading = false;
            paragraph.bullet = false;
            if !self.m_no_headings && JUSTEXT_RE1.find(&paragraph.dom_path).is_some() {
                paragraph.heading = true;
            } else if JUSTEXT_RE2.find(&paragraph.dom_path).is_some() {
                paragraph.bullet = true;
            }

            if link_density > self.m_max_link_density {
                paragraph.cfclass = "bad";
            } else if paragraph.text.contains("\u{a9}") || paragraph.text.contains("&copy") {
                paragraph.cfclass = "bad";
            } else if JUSTEXT_RE3.find(&paragraph.dom_path).is_some() {
                paragraph.cfclass = "bad";
            } else {
                if length < self.m_length_low {
                    if paragraph.linked_char_count > 0 {
                        paragraph.cfclass = "bad";
                    } else {
                        paragraph.cfclass = "short";
                    }
                } else {
                    if stopword_density >= self.m_stopwords_high {
                        if length > self.m_length_high {
                            paragraph.cfclass = "good";
                        } else {
                            paragraph.cfclass = "neargood";
                        }
                    } else if stopword_density >= self.m_stopwords_low {
                        paragraph.cfclass = "neargood";
                    } else {
                        paragraph.cfclass = "bad";
                    }
                }
            }
        }
    }

    fn revise_paragraph_classification(&self, paragraphs: &mut Vec<Paragraph>, max_heading_distance: i32) {
        // Context-sensitive paragraph classification. Assumes that classify_pragraphs
        // has already been called.
        // Copy classes
        for paragraph in paragraphs.iter_mut() {
            paragraph.finalclass = paragraph.cfclass;
        }

        // Good headings
        let mut j = 0;
        let mut distance = 0;
        for i in 0..paragraphs.len() {
            if !paragraphs[i].heading || paragraphs[i].finalclass != "short" {
                continue;
            }
            j = i + 1;
            distance = 0;
            while j < paragraphs.len() && distance <= max_heading_distance {
                if paragraphs[j].finalclass == "good" {
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
            if paragraphs[i].finalclass != "short" {
                continue;
            }
            let prev_neighbour = get_prev_neighbour(i as i64, paragraphs, true);
            let next_neighbour = get_next_neighbour(i as i64, paragraphs, true);
            if prev_neighbour == "good" && next_neighbour == "good" {
                new_classes[i] = "good";
            } else if prev_neighbour == "bad" && next_neighbour == "bad" {
                new_classes[i] = "bad";
            } else if (prev_neighbour == "bad" && get_prev_neighbour(i as i64, paragraphs, false) == "neargood") || (next_neighbour == "bad" &&
                get_next_neighbour(i as i64, paragraphs, false) == "neargood") {
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
            if paragraphs[i].finalclass != "neargood" {
                continue;
            }
            let prev_neighbour = get_prev_neighbour(i as i64, paragraphs, true);
            let next_neighbour = get_next_neighbour(i as i64, paragraphs, true);
            if prev_neighbour == "bad" && next_neighbour == "bad" {
                paragraphs[i].finalclass = "bad";
            } else {
                paragraphs[i].finalclass = "good";
            }
        }

        // More good headings
        for i in 0..paragraphs.len() {
        // for (i, paragraph) in paragraphs.iter_mut().enumerate() {
            if !paragraphs[i].heading || paragraphs[i].finalclass != "bad" || paragraphs[i].cfclass == "bad" {
                continue;
            }
            j = i + 1;
            distance = 0;
            while j < paragraphs.len() && distance <= max_heading_distance {
                if paragraphs[j].finalclass == "good" {
                    paragraphs[i].finalclass = "good";
                    break;
                }
                distance += paragraphs[j].text.len() as i32;
                j += 1;
            }
        }
    }

    fn output_default(&self, paragraphs: &mut Vec<Paragraph>) -> String {
        let mut out = String::new();
        for mut paragraph in paragraphs {
            let &mut tag;
            if paragraph.finalclass == "good" {
                if paragraph.heading {
                    tag = "h";
                } else if paragraph.bullet {
                    tag = "l";
                } else {
                    tag = "p";
                }
                // if !full {
                //     out.push_str(&paragraph.text);
                //     out.push_str(" ");
                // }
            } else {
                // if no_boilerplate {
                //     continue;
                // } else {
                //     tag = "b";
                // }
            }
            // replace_a_to_b(&mut paragraph.text, "&nbsp;", " ");
            // replace_a_to_b(&mut paragraph.text, "&quot;", "\"");
            // replace_a_to_b(&mut paragraph.text, "&gt;", ">");
            // replace_a_to_b(&mut paragraph.text, "&lt;", "<");
            // if full {
            //     let tmp = wrap_text(&mut paragraph.text, 80);
            //     if paragraph.m_tag == "pre" {
            //         let re = regex::Regex::new(r"\n>?[ \t]?\n>?").unwrap();
            //         out.push_str(&re.replace_all(&tmp, format!("\n\n<{}>", tag).to_string()));
            //     } else {
            //         out.push_str(format!("<{}>", tag).as_str());
            //         out.push_str(&tmp);
            //         out.push_str("\n\n");
            //     }
            // }
        }
        out
    }
}

fn wrap_text(str: &mut String, len: usize) -> &str {
    let mut line_width = 0;
    let mut last_space = 0;
    let mut next_s = 0;
    let mut next_n = 0;
    for mut i in 0..str.len() {
        next_s = str[i..].find(' ').unwrap_or(str.len() - i);
        if str[next_s..next_s + 1] == *"\n" {
            line_width = 0;
            last_space = next_s;
            i += next_s + 1;
            continue;
        }

        line_width += next_s - last_space + 1;

        if line_width > len {
            str.insert_str(last_space + 1, "\n");
            line_width = next_s - last_space + 1;
        }

        i += next_s + 1;
        last_space = next_s;
    }
    str
}

fn _get_neighbour(i: i64, paragraphs: &mut Vec<Paragraph>, ignore_neargood: bool, inc: i64, boundary: i64) -> String {
    let mut i = i + inc;
    while i != boundary {
        let c = paragraphs[i as usize].finalclass;
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

fn get_prev_neighbour(i: i64, paragraphs: &mut Vec<Paragraph>, ignore_neargood: bool) -> String {
    // Return the class of the paragraph at the top end of the short/neargood
    // paragraphs block. If ignore_neargood is true, than only 'bad' or 'good'
    // can be returned, otherwise 'neargood' can be returned, too.
    _get_neighbour(i, paragraphs, ignore_neargood, -1, -1)
}

fn get_next_neighbour(i: i64, paragraphs: &mut Vec<Paragraph>, ignore_neargood: bool) -> String {
    //Return the class of the paragraph at the bottom end of the short/neargood
    //paragraphs block. If ignore_neargood is True, than only 'bad' or 'good'
    //can be returned, otherwise 'neargood' can be returned, too.
    _get_neighbour(i, paragraphs, ignore_neargood, 1, paragraphs.len() as i64)
}
