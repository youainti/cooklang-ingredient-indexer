use cooklang_indexer::IngredientIndex;
use std::path::Path;
 
fn main() -> anyhow::Result<()> { 
    let recipes_dir = std::env::args()
        .nth(1)
        .context("Please provide the recipe directory path")?;

    let index = IngredientIndex::new(recipes_dir)?;
    // Get all ingredients
    for ingredient in index.ingredients() {
    println!("Found ingredient: {}", ingredient);
}
