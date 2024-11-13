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
    
    // Generate and save HTML
    let html = generate_html_index(&ingredient_index)?;
    fs::write("ingredient-index.html", html)?;
    println!("Index generated at: ingredient-index.html");
    
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

fn generate_html_index(index: &HashMap<String, Vec<PathBuf>>) -> Result<String> {
    let mut ingredients: Vec<_> = index.keys().collect();
    ingredients.sort();
    
    let mut html = String::from(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Recipe Ingredient Index</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            line-height: 1.6;
        }
        h1 {
            color: #2c3e50;
            border-bottom: 2px solid #eee;
            padding-bottom: 10px;
        }
        .ingredient {
            margin: 20px 0;
        }
        .ingredient-name {
            font-weight: bold;
            color: #34495e;
            margin-bottom: 5px;
        }
        .recipe-list {
            margin-left: 20px;
            list-style-type: none;
        }
        .recipe-list li {
            margin: 5px 0;
        }
        a {
            color: #3498db;
            text-decoration: none;
        }
        a:hover {
            text-decoration: underline;
        }
    </style>
</head>
<body>
    <h1>Recipe Ingredient Index</h1>
"#);

    for ingredient in ingredients {
        html.push_str("<div class=\"ingredient\">\n");
        html.push_str(&format!("    <div class=\"ingredient-name\">{}</div>\n", ingredient));
        html.push_str("    <ul class=\"recipe-list\">\n");
        
        if let Some(recipes) = index.get(ingredient) {
            for recipe_path in recipes {
                let recipe_name = recipe_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown Recipe")
                    .replace("-", " ")
                    .replace("_", " ");
                
                // Create a relative path for the link
                let path_str = recipe_path.to_string_lossy();
                
                html.push_str(&format!(
                    "        <li><a href=\"{}\">{}</a></li>\n",
                    path_str,
                    recipe_name
                ));
            }
        }
        
        html.push_str("    </ul>\n");
        html.push_str("</div>\n");
    }

    html.push_str("</body>\n</html>");
    
    Ok(html)
}
