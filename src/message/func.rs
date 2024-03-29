use kuchikiki::traits::*;
use std::cell::RefCell;
use rustrict::BlockReason;
use std::error::Error;

use crate::user::model::*;

pub trait VecWithHardLimit<T: Clone> {
    fn push_with_hard_limit(&mut self, element: &T);
}

impl<T: Clone> VecWithHardLimit<T> for Vec<T> {
    fn push_with_hard_limit(&mut self, element: &T) {
        if self.len() == self.capacity() {
            self.remove(0); // Remove the oldest element
        }
        self.push(element.clone());
    }
}

pub fn into_censored_md(html: &str, user: &mut User) -> Result<String, BlockReason> {
    let mut document = kuchikiki::parse_html().one(html);

    // If there's no <p> tag, wrap the content in a <p> tag
    if !document.select_first("p").is_ok() {
        document = kuchikiki::parse_html().one(format!("<p>{}</p>", document.select_first("body").unwrap().as_node().to_string()));
    }

    let mut nodes_text: Vec<String> = document.descendants().text_nodes().map(|text| {<RefCell<String> as Clone>::clone(&text).into_inner()}).collect();
    nodes_text.pop();
    let mut nodes_char: Vec<char> = user.context.process(nodes_text.join("").trim().to_string())?.chars().collect();

    let mut new_text: Vec<String> = vec![];
    let mut consumed_chars = 0;

    for text in nodes_text.iter() {
        let len = text.len();
        let replacement: String = nodes_char[consumed_chars..consumed_chars + len].iter().collect();
        new_text.push(replacement);
        consumed_chars += len;
    }

    for (index, text_node) in document.descendants().text_nodes().enumerate() {
        text_node.replace(new_text[index].clone());
    }
    if document.descendants().text_nodes().map(|text| {<RefCell<String> as Clone>::clone(&text).into_inner()}).collect::<Vec<String>>().join("").trim().is_empty() {
        Err(BlockReason::Empty)
    } else {
        Ok(document.select_first("p").unwrap().as_node().to_string())
    }
}