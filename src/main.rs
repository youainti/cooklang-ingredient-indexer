use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use regex::Regex;
use anyhow::{Result, Context};

#[derive(Debug)]
struct Recipe {
    path: PathBuf,
    ingredients: Vec<String>,
}

fn main() -> Result<()> {
    let recipes_dir = std::env::args()
        .nth(1)
        .context("Please provide the recipe directory path")?;
    
    let recipes = index_recipes(&recipes_dir)?;
    let ingredient_index = create_ingredient_index(&recipes);
    
    // Print the index
    for (ingredient, paths) in ingredient_index.iter() {
        println!("{}:", ingredient);
        for path in paths {
            println!("  - {}", path.display());
        }
    }
    
    Ok(())
}

fn index_recipes(dir: &str) -> Result<Vec<Recipe>> {
    let mut recipes = Vec::new();
    let ingredient_regex = Regex::new(r"@([^{@\n]+)(?:\{[^}]*\})?").unwrap();
    
    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("cook") {
                let content = fs::read_to_string(path)?;
                let ingredients: Vec<String> = ingredient_regex
                    .captures_iter(&content)
                    .map(|cap| cap[1].trim().to_lowercase())
                    .collect();
                
                if !ingredients.is_empty() {
                    recipes.push(Recipe {
                        path: path.to_owned(),
                        ingredients,
                    });
                }
            }
    }
    
    Ok(recipes)
}

fn create_ingredient_index(recipes: &[Recipe]) -> HashMap<String, Vec<PathBuf>> {
    let mut index: HashMap<String, Vec<PathBuf>> = HashMap::new();
    
    for recipe in recipes {
        for ingredient in &recipe.ingredients {
            index
                .entry(ingredient.clone())
                .or_default()
                .push(recipe.path.clone());
        }
    }
    
    // Sort the paths for each ingredient for consistent output
    for paths in index.values_mut() {
        paths.sort();
    }
    
    index
}
