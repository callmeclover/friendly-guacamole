use rustrict::{Type, Censor};
use kuchikiki::traits::*;
use std::{error::Error, cell::RefCell};
use crate::user::model::User;

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
        document = kuchikiki::parse_html().one(format!("<p>{}</p>", document.select_first("body").unwrap().as_node().to_string())).select_first("p").unwrap().as_node().clone();
    } else {
        document = document.select_first("p").unwrap().as_node().clone();
    }

    let nodes_text: Vec<String> = document.descendants().text_nodes().map(|text| {<RefCell<String> as Clone>::clone(&text).into_inner()}).collect();
    let mut nodes_char: Vec<char>;
    match user.glass.process(nodes_text.join("")) {
        Ok(val) => { nodes_char = val.chars().collect() },
        Err(err) => { return Err(err); }
    }
        
    let mut index = 0;
    let mut new_text: Vec<String> = vec![];
    while index < nodes_text.len() {
        let replacement: String = nodes_char.chunks_exact(nodes_text[index].len()).next().unwrap().iter().collect();
        new_text.push(replacement);
        nodes_char.drain(0..nodes_text[index].len());
        index += 1;
    }

    for (index, text_node) in document.descendants().text_nodes().enumerate() {
        text_node.replace(new_text[index].clone());
    }
    if document.descendants().text_nodes().map(|text| {<RefCell<String> as Clone>::clone(&text).into_inner()}).collect::<Vec<String>>().join("").trim().is_empty() {
        Err("Message is empty".into())
    } else {
        Ok(document.select_first("p").unwrap().as_node().to_string())
    }
}
