//! A basic example of parsing cosense syntax using the `cosy`.
//!
//! This example shows how to use the default `parse` function to process
//! a string containing consense-style markup and links.

use cosy;

fn main() {
    let mut input =
        "[* ある日の暮方]の事である。一人の[下人]が、[羅生門 https://ja.wikipedia.org/wiki/%E7%BE%85%E7%94%9F%E9%96%80_(%E5%B0%8F%E8%AA%AC)]の下で雨やみを待っていた。";

    let result = cosy::parse(&mut input, &());

    match result {
        Ok(nodes) => println!("{:#?}", nodes),
        Err(e) => println!("Error: {}", e),
    }
}
