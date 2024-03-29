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

pub fn into_censored_md(html: &str, user: &mut User) -> Result<String, Box<dyn Error>> {
    let mut document = kuchikiki::parse_html().one(html);

    // If there's no <p> tag, wrap the content in a <p> tag
    if !document.select_first("p").is_ok() {
        document = kuchikiki::parse_html().one(format!("<p>{}</p>", document.select_first("body").unwrap().as_node().to_string()));
    }

    let nodes_text: Vec<String> = document.descendants().text_nodes().map(|text| {<RefCell<String> as Clone>::clone(&text).into_inner()}).collect();
    let mut nodes_char: Vec<char>;

    match user.context.process(nodes_text.join("")) {
        Ok(text) => {
            nodes_char = text.chars().collect::<Vec<char>>();
        },
        Err(blockreason) => {
            return Err(blockreason);
        }
    }

    let mut index = 0;
    let mut new_text: Vec<String> = vec![];
    while index < nodes_text.len() {
        println!("{:?}", nodes_char);
        let replacement: String = nodes_char.chunks_exact(nodes_text[index].len()).next().unwrap().iter().collect();
        new_text.push(replacement);
        nodes_char.drain(0..nodes_text[index].len());
        index += 1;
    }

    for (index, text_node) in document.descendants().text_nodes().enumerate() {
        text_node.replace(new_text[index].clone());
    }
    if document.descendants().text_nodes().map(|text| {<RefCell<String> as Clone>::clone(&text).into_inner()}).collect::<Vec<String>>().join("").trim().is_empty() {
        Err()
    } else {
        Ok(document.select_first("p").unwrap().as_node().to_string())
    }
}