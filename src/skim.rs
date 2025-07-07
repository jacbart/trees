extern crate skim;
use anyhow::{Error, Result};
use skim::prelude::*;
use std::io::Cursor;

#[allow(dead_code)]
pub fn select(input: String) -> Result<String, Error> {
    let mut selected = "".to_owned();

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));

    let options = SkimOptionsBuilder::default()
        .no_height(true)
        .no_multi(true)
        .exit_0(true)
        .build()?;

    let selected_items = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_else(|| Vec::new());

    for item in selected_items.iter() {
        selected.push_str(&item.output().into_owned());
    }

    Ok(String::from(selected))
}
