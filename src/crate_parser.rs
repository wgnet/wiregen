// Copyright (C) 2018 Wargaming.net Limited. All rights reserved.

// This is a custom Nom parser for crates, to comply with GitHub syn version

use syn::Item;
use syn::parse::{lit, ident, item};

use synom::IResult;
use synom::space::{block_comment, whitespace, skip_whitespace};

named!(krate -> Vec<Item>, do_parse!(
    option!(byte_order_mark) >> option!(shebang)
        >> many0!(inner_attr)
        >> items: many0!(item) >> (items)
));

named!(byte_order_mark -> &str, tag!("\u{feff}"));

named!(shebang -> (), do_parse!(
        tag!("#!") >>
        not!(tag!("[")) >>
        take_until!("\n") >> (())
));

named!(inner_attr -> (), alt!(
    do_parse!(
        punct!("#") >> punct!("!") >> punct!("[") >> meta_item >> punct!("]") >> (()))
    |
    do_parse!(
        punct!("//!") >> take_until!("\n") >> (()))
    |
    do_parse!(
        option!(whitespace) >> peek!(tag!("/*!")) >> block_comment >> (()))
));

named!(meta_item -> (), alt!(
    do_parse!(ident
            >> punct!("(")
            >> terminated_list!(punct!(","), nested_meta_item)
            >> punct!(")") >> (()))
    |
    do_parse!(
        ident >> punct!("=") >> lit >> (()))
    |
    do_parse!(
        ident >> (()))
));

named!(nested_meta_item -> (), alt!(
    do_parse!(meta_item >> (()))
    |
    do_parse!(lit >> (()))
));

const CONTEXT: usize = 10;

pub fn get_items(input: &str) -> Result<Vec<Item>, String> {
    match krate(input) {
        IResult::Done(leftover, items) => {
            let eof = skip_whitespace(leftover);

            if eof.is_empty() {
                Ok(items)
            } else {
                Err(eof)
            }
        },

        IResult::Error => Err(input)
    }
    .map_err(
        |l| l.lines().take(CONTEXT).collect::<Vec<_>>().join("\n")
    )
}
