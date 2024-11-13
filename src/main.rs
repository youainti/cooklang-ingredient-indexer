use cooklang_indexer::IngredientIndex;
use anyhow::{Context}; //required to compile even though it throws a warning
use std::fs;
 
fn main() -> anyhow::Result<()> { 
    let recipes_dir = std::env::args()
        .nth(1)
        .context("Please provide the recipe directory path")?;

    let base_url = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "http://localhost:8080/r".to_string());

    let index = IngredientIndex::new(recipes_dir)?;

    // Get all ingredients
    for ingredient in index.ingredients() {
        println!("Found ingredient: {}", ingredient);
    }

    //create an html version and write it out
    let html = index.generate_html(&base_url)?;
    fs::write("ingredient-index.html", html)?;
    println!("Index generated at: ingredient-index.html");

    Ok(())
}
