use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::path::Path;
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
    
    let base_url = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "http://localhost:8080/r".to_string());

    let html = run_index_operation(&recipes_dir, &base_url)?;
    fs::write("ingredient-index.html", html)?;
    println!("Index generated at: ingredient-index.html");
    
    Ok(())
}
fn run_index_operation(
    recipes_dir: &str, 
    base_url: &str, 
) -> Result<String> {
    
    let base_dir = Path::new(&recipes_dir);
    let recipes = index_recipes(&recipes_dir)?;
    let ingredient_index = create_ingredient_index(&recipes);
    
    // Generate and save HTML
    let html = generate_html_index(&ingredient_index,&base_url, base_dir)?;
    Ok(html)
}


fn path_to_url(path: &Path, base_url: &str, base_dir: &Path) -> String {
    // Strip the base directory from the path to get the relative path
    let relative_path = path.strip_prefix(base_dir)
        .unwrap_or(path);  // Fallback to full path if strip fails
    
    // Get the stem (filename without extension)
    let file_stem = relative_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
    // Get the parent directory path if any, excluding the base directory
    let parent_path = relative_path
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("");
    
    // Combine parent path and file stem, ensuring proper formatting
    let final_path = if parent_path.is_empty() {
        file_stem.to_string()
    } else {
        format!("{}/{}", parent_path, file_stem)
    };
    
    // Construct the final URL, ensuring no double slashes
    let base = base_url.trim_end_matches('/');
    format!("{}/{}", base, urlencoding::encode(&final_path))
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

fn generate_html_index(
    index: &HashMap<String, Vec<PathBuf>>, 
    base_url: &str,
    base_dir: &Path
) -> Result<String> {
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
                
                let url = path_to_url(recipe_path, base_url, base_dir);
                
                html.push_str(&format!(
                    "        <li><a href=\"{}\">{}</a></li>\n",
                    url,
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
