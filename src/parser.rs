use std::collections::HashSet;
use std::iter::Iterator;
use libxml::tree::{Document, Node};

const DONTCARE_TAGS: std::collections::HashSet<&str> = "head,script".split(",").collect();

const PARAGRAPH_TAGS: std::collections::HashSet<&str> = "blockquote,caption,center,col,colgroup,dd,\
	div,dl,dt,fieldset,form,legend,optgroup,option,\
	p,pre,table,td,textarea,tfoot,th,thead,tr,\
	ul,li,h1,h2,h3,h4,h5,h6".split(",").collect();

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

    pub m_htmlPosition1: i64,
    pub m_htmlPosition2: i64,
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

    pub m_pTags: HashSet<String>,

    pub m_dontcareTags: HashSet<String>,

    pub m_url: String,
    pub m_convertedHtml: String,
    pub m_invalid_characters: bool,
    pub m_first_pass: bool,
    pub m_haveGood: bool,
    pub m_learning: bool,
    pub m_basicJustext: bool,
}

impl Parser {
    pub fn new(document: &Document) -> Self {
        let mut parser = Parser::default();
        parser.m_dontcareTags.extend(DONTCARE_TAGS);
        parser.m_pTags.extend(PARAGRAPH_TAGS);

        let root = document.get_root_element().ok_or(anyhow::anyhow!("get_root_element"))?;
        parser.walk_tree(&root, 0);

        parser
    }

    fn walk_tree(&mut self, node: &Node, mut depth: usize) {
        for curr in node.get_child_nodes() {
            let curr_depth = depth + 1;
            if DONTCARE_TAGS.contains(curr.get_name().as_str()) {
                continue;
            }
            handle_node(&curr, curr_depth);
            walk_tree(&curr, curr_depth);
        }

        let tag = node.get_name().to_lowercase();
        if PARAGRAPH_TAGS.contains(&tag) {
            start_new_paragraph()
        }
        if tag == "A" {
            self.m_link = false;
        }
    }

    fn handle_node(&mut self, it: &Node, depth: usize, dom: &mut Vec<String>, curr_paragraph: &mut Paragraph) {
        let name = it.name().to_lowercase();
        if name.is_empty() || it.is_comment() {
            return;
        }
        while dom.len() >= depth {
            dom.pop();
        }
        if it.is_tag() {
            if DONTCARE_TAGS.contains(&name) {
                it.skip_children();
                return;
            }

            if dom.len() < depth {
                dom.push(name);
            } else {
                dom.push(name);
            }

            if TAGS.contains(&name) || (name == "br" && BR) {
                if name == "br" {
                    curr_paragraph.tag_count -= 1;
                    start_new_paragraph();
                } else {
                    if name == "br" {
                        BR = true;
                    } else {
                        BR = false;
                    }
                    if name == "a" {
                        self.m_link = true;
                    }
                    curr_paragraph.tag_count += 1;
                }

                if curr_paragraph.tag.is_empty() {
                    curr_paragraph.tag = name;
                }

                if curr_paragraph.original_tags.is_empty() {
                    curr_paragraph.original_tags.push(it.text());
                }
            }

            if curr_paragraph.html_pos1 == 0 {
                curr_paragraph.html_pos1 = it.offset();
                curr_paragraph.html_pos2 = it.offset() + it.length();
            } else if BR {
                curr_paragraph.html_pos2 = it.offset() + it.length();
            }
        }

        if !it.is_comment() && !it.is_tag() {
            // text data
            let mut content = it.text();
            RE.replace_all(&mut content, r"&nbsp;", " ");
            let pre = curr_paragraph.tag == "pre";
            if pre {
                RE.replace_all(&mut content, r#"\s+"#, " ");
            } else {
                RE.replace_all(&mut content, r"\s+", " ");
            }

            if content.trim().is_empty() {
                if BR {
                    if dom.len() > 2 && dom[dom.len() - 2] == "b" && dom[dom.len() - 1] == "br" && curr_paragraph.text_nodes.len() == 1 {
                        curr_paragraph.heading = true;
                        start_new_paragraph();
                    } else if !curr_paragraph.text_nodes.is_empty() {
                        curr_paragraph.text_nodes.push("\n".to_string());
                    }
                } else if pre {
                    curr_paragraph.text_nodes.push(" ".to_string());
                }
                return;
            }

            if BR {
                content = "\n".to_owned() + content;
                curr_paragraph.html_pos2 = it.offset() + it.length();
            }

            curr_paragraph.text_nodes.push(content);
            curr_paragraph.html_pos2 = it.offset() + it.length();

            if self.m_link {
                curr_paragraph.linked_char_count += content.len() as i64;
            }

            BR = false;

            if LEARNING {
                // get parents for learning
                let mut parent_it = it;
                while curr_paragraph.html_parents.len() <= 8 {
                    parent_it = parent_it.parent();
                    if parent_it.is_none() {
                        break;
                    }
                    if parent_it.unwrap().is_tag() {
                        curr_paragraph.html_parents.push(parent_it.unwrap().name());
                    }
                }
            }
        }
    }

    fn start_new_paragraph(&mut self) {
        self.m_currParagraph.dom_path = self.m_dom.join(".");

        if !self.m_currParagraph.text_nodes.is_empty() {
            self.m_currParagraph.text = m_currParagraph.text_nodes.join("");

            // If the text was split into multiple chunks, count each chunk as a word
            let words: Vec<String> = self.m_currParagraph.text.split("").map(|s| s.to_string()).collect();
            self.m_currParagraph.word_count = words.len() as i64;

            self.m_paragraphs.push(m_currParagraph);
        }

        self.m_currParagraph.text_nodes.clear();
        self.m_currParagraph.text.clear();
        self.m_currParagraph.linked_char_count = 0;
        self.m_currParagraph.word_count = 0;
        self.m_currParagraph.tag_count = 0;
        self.m_currParagraph.stopword_count = 0;
        self.m_currParagraph.stopword_density = 0.0;

        self.m_currParagraph.m_htmlPosition1 = 0;
        self.m_currParagraph.m_htmlPosition2 = 0;
        self.m_currParagraph.m_tag.clear();
        self.m_currParagraph.m_htmlSrc.clear();
        self.m_currParagraph.m_originalTags.clear();
        self.m_currParagraph.m_htmlParents.clear();
    }
}