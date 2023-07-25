use std::collections::HashSet;
use std::iter::Iterator;
use lazy_static::lazy_static;
use libxml::tree::{Document, Node, NodeType};
use regex::Regex;

lazy_static! {
    pub static ref DONTCARE_TAGS: std::collections::HashSet<&'static str> = "head,script".split(",").collect();

    pub static ref PARAGRAPH_TAGS: std::collections::HashSet<&'static str> = "blockquote,caption,center,col,colgroup,dd,\
        div,dl,dt,fieldset,form,legend,optgroup,option,\
        p,pre,table,td,textarea,tfoot,th,thead,tr,\
        ul,li,h1,h2,h3,h4,h5,h6".split(",").collect();

    pub static ref PARSER_RE1: Regex = Regex::new(r"&nbsp;").unwrap();
    pub static ref PARSER_RE2: Regex = Regex::new(r"[ 	]+").unwrap();
    pub static ref PARSER_RE3: Regex = Regex::new(r"\s+").unwrap();
}

#[derive(Debug, Clone, Default)]
pub struct Paragraph {
    pub dom_path: String,
    pub text_nodes: Vec<String>,
    pub text: String,
    pub cfclass: &'static str,

    pub finalclass: &'static str,
    pub heading: bool,
    pub bullet: bool,
    pub word_count: i64,
    pub linked_char_count: i64,
    pub tag_count: i64,

    pub stopword_count: i64,
    pub stopword_density: f32,
    pub link_density: f32,

    pub m_tag: String,
    pub m_htmlSrc: String,
    pub m_originalTags: Vec<String>,
    pub m_htmlParents: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Parser {
    pub m_paragraphs: Vec<Paragraph>,
    pub m_currParagraph: Paragraph,
    pub m_link: bool,
    pub m_br: bool,
    // pub m_dontcare: i64,
    pub m_dom: Vec<String>,

    pub m_pTags: HashSet<&'static str>,

    pub m_dontcareTags: HashSet<&'static str>,

    pub m_url: String,
    pub m_convertedHtml: String,
    pub m_invalid_characters: bool,
    pub m_first_pass: bool,
    pub m_haveGood: bool,
    pub m_learning: bool,
    pub m_basicJustext: bool,
}

impl Parser {
    pub fn new() -> Self {
        let mut parser = Parser::default();
        parser.m_dontcareTags.extend(DONTCARE_TAGS.iter());
        parser.m_pTags.extend(PARAGRAPH_TAGS.iter());
        parser
    }

    pub fn walk_tree(&mut self, document: &Document) -> anyhow::Result<()> {
    	let root = document.get_root_element().ok_or(anyhow::anyhow!("get_root_element"))?;
    	self.walk_tree_helper(&root, 0);
        Ok(())
    }

    fn walk_tree_helper(&mut self, node: &Node, mut depth: usize) {
        for curr in node.get_child_nodes() {
            let curr_depth = depth + 1;
            if DONTCARE_TAGS.contains(curr.get_name().as_str()) {
                continue;
            }
            self.handle_node(&curr, curr_depth);
            self.walk_tree_helper(&curr, curr_depth);
        }

        let tag = node.get_name().to_lowercase();
        if PARAGRAPH_TAGS.contains(tag.as_str()) {
            self.start_new_paragraph()
        }
        if tag == "A" {
            self.m_link = false;
        }
    }

    fn handle_node(&mut self, it: &Node, depth: usize) {
        let name = it.get_name().to_lowercase();
        if name.is_empty() || it.get_type().unwrap() == NodeType::CommentNode {
            return;
        }
        while self.m_dom.len() >= depth {
            self.m_dom.pop();
        }
        if it.is_element_node() {
            if DONTCARE_TAGS.contains(&name.as_str()) {
                // it.skip_children();
                return;
            }

            self.m_dom.push(name.clone());

            if PARAGRAPH_TAGS.contains(&name.as_str()) || (name == "br" && self.m_br) {
                if name == "br" {
                    self.m_currParagraph.tag_count -= 1;
                    self.start_new_paragraph();
                } else {
                    if name == "br" {
                        self.m_br = true;
                    } else {
                        self.m_br = false;
                    }
                    if name == "a" {
                        self.m_link = true;
                    }
                    self.m_currParagraph.tag_count += 1;
                }

                if self.m_currParagraph.m_tag.is_empty() {
                    self.m_currParagraph.m_tag = name;
                }

                if self.m_currParagraph.m_originalTags.is_empty() {
                    self.m_currParagraph.m_originalTags.push(it.get_content());
                }
            }

            match it.get_name().as_str() {
                "img" if let Some(src) = it.get_attribute("src") => {
                    self.m_currParagraph.text_nodes.push(format!("<IMAGE>{}</IMAGE>", src).to_string());
                },
                "video" if let Some(src) = it.get_attribute("src") => {
                    for source in it.get_child_nodes() {
                        match it.get_name().as_str() {
                            "source" if let Some(src) = source.get_attribute("src") => {
                                self.m_currParagraph.text_nodes.push(format!("<VIDEO>{}</VIDEO>", src).to_string());
                            }
                            _ => {}
                        }
                    }
                },
                "audio" if let Some(src) = it.get_attribute("src") => {
                    for source in it.get_child_nodes() {
                        match it.get_name().as_str() {
                            "source" if let Some(src) = source.get_attribute("src") => {
                                self.m_currParagraph.text_nodes.push(format!("<AUDIO>{}</AUDIO>", src).to_string());
                            }
                            _ => {}
                        }
                    }
                },
                _ => {}
            }
        }

        if it.is_text_node() {
            // text data
            let mut content = it.get_content();
            PARSER_RE1.replace_all(&mut content, " ");
            let pre = self.m_currParagraph.m_tag == "pre";
            if pre {
                PARSER_RE2.replace_all(&mut content, " ");
            } else {
                PARSER_RE3.replace_all(&mut content, " ");
            }

            if content.trim().is_empty() {
                if self.m_br {
                    if self.m_dom.len() > 2 && self.m_dom[self.m_dom.len() - 2] == "b" && self.m_dom[self.m_dom.len() - 1] == "br" && self.m_currParagraph
                        .text_nodes
                        .len() == 1 {
                        self.m_currParagraph.heading = true;
                        self.start_new_paragraph();
                    } else if !self.m_currParagraph.text_nodes.is_empty() {
                        self.m_currParagraph.text_nodes.push("\n".to_string());
                    }
                } else if pre {
                    self.m_currParagraph.text_nodes.push(" ".to_string());
                }
                return;
            }

            if self.m_br {
                content = "\n".to_owned() + content.as_str();
                // curr_paragraph.m_htmlPosition2 = it.offset() + it.length();
            }

            self.m_currParagraph.text_nodes.push(content.clone());
            // curr_paragraph.m_htmlPosition2 = it.offset() + it.length();

            if self.m_link {
                self.m_currParagraph.linked_char_count += content.len() as i64;
            }

            self.m_br = false;
        }
    }

    fn start_new_paragraph(&mut self) {
        self.m_currParagraph.dom_path = self.m_dom.join(".");

        if !self.m_currParagraph.text_nodes.is_empty() {
            self.m_currParagraph.text = self.m_currParagraph.text_nodes.join("");

            // If the text was split into multiple chunks, count each chunk as a word
            let words: Vec<String> = self.m_currParagraph.text.split("").map(|s| s.to_string()).collect();
            self.m_currParagraph.word_count = words.len() as i64;

            self.m_paragraphs.push(self.m_currParagraph.clone());
        }

        self.m_currParagraph.text_nodes.clear();
        self.m_currParagraph.text.clear();
        self.m_currParagraph.linked_char_count = 0;
        self.m_currParagraph.word_count = 0;
        self.m_currParagraph.tag_count = 0;
        self.m_currParagraph.stopword_count = 0;
        self.m_currParagraph.stopword_density = 0.0;

        self.m_currParagraph.m_tag.clear();
        self.m_currParagraph.m_htmlSrc.clear();
        self.m_currParagraph.m_originalTags.clear();
        self.m_currParagraph.m_htmlParents.clear();
    }
}