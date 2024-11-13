// examples/basic.rs
use cooklang_indexer::IngredientIndex;
use anyhow::Result;

fn main() -> Result<()> {
    let index = IngredientIndex::new("./recipes")?;
    
    // Print all ingredients
    for ingredient in index.ingredients() {
        println!("Ingredient: {}", ingredient);
        
        // Print recipes containing this ingredient
        if let Some(recipes) = index.get_recipes_for_ingredient(ingredient) {
            for recipe in recipes {
                println!("  Recipe: {:?}", recipe);
            }
        }
    }
    
    Ok(())
}
