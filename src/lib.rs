// File: src/lib.rs

//! A library for indexing and generating HTML indexes of cooklang recipe ingredients
//! 
//! This library provides functionality to:
//! - Parse cooklang files for ingredients
//! - Create an searchable ingredient index
//! - Generate HTML documentation with links to recipes
//! 
//! # Example
//! ```no_run
//! use cooklang_indexer::IngredientIndex;
//! use std::path::Path;
//! 
//! # fn main() -> anyhow::Result<()> {
//! let index = IngredientIndex::new("path/to/recipes")?;
//! 
//! // Get all ingredients
//! for ingredient in index.ingredients() {
//!     println!("Found ingredient: {}", ingredient);
//! }
//! 
//! // Generate HTML index
//! let html = index.generate_html("http://example.com/recipes")?;
//! std::fs::write("index.html", html)?;
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use regex::Regex;
use anyhow::{Result, Context};

/// Represents a single recipe file and its ingredients
#[derive(Debug)]
pub struct Recipe {
    /// Path to the recipe file
    pub path: PathBuf,
    /// List of ingredients found in the recipe
    pub ingredients: Vec<String>,
}

/// Main struct for managing ingredient indexing and HTML generation
#[derive(Debug)]
pub struct IngredientIndex {
    index: HashMap<String, Vec<PathBuf>>,
    base_dir: PathBuf,
}

impl IngredientIndex {
    /// Creates a new IngredientIndex by scanning the given directory for cooklang files
    ///
    /// # Arguments
    /// * `recipes_dir` - Path to the directory containing cooklang recipe files
    ///
    /// # Returns
    /// * `Result<IngredientIndex>` - The index if successful, or an error if the directory
    ///   cannot be read or if there are issues parsing the files
    ///
    /// # Example
    /// ```no_run
    /// use cooklang_indexer::IngredientIndex;
    /// 
    /// let index = IngredientIndex::new("./recipes").unwrap();
    /// ```
    pub fn new(recipes_dir: impl AsRef<Path>) -> Result<Self> {
        let recipes = index_recipes(recipes_dir.as_ref())?;
        Ok(Self {
            index: create_ingredient_index(&recipes),
            base_dir: recipes_dir.as_ref().to_path_buf(),
        })
    }

    /// Generates an HTML index of all ingredients and their recipes
    ///
    /// # Arguments
    /// * `base_url` - Base URL where recipes will be hosted (e.g., "http://example.com/recipes")
    ///
    /// # Returns
    /// * `Result<String>` - HTML content as a string if successful
    ///
    /// # Example
    /// ```no_run
    /// # use cooklang_indexer::IngredientIndex;
    /// # let index = IngredientIndex::new("./recipes").unwrap();
    /// let html = index.generate_html("http://example.com/recipes").unwrap();
    /// std::fs::write("index.html", html).unwrap();
    /// ```
    pub fn generate_html(&self, base_url: &str) -> Result<String> {
        generate_html_index(&self.index, base_url, &self.base_dir)
    }

    /// Gets all recipes that contain a specific ingredient
    ///
    /// # Arguments
    /// * `ingredient` - Name of the ingredient to search for
    ///
    /// # Returns
    /// * `Option<&Vec<PathBuf>>` - Vector of paths to recipes containing the ingredient,
    ///   or None if the ingredient isn't found
    ///
    /// # Example
    /// ```no_run
    /// # use cooklang_indexer::IngredientIndex;
    /// # let index = IngredientIndex::new("./recipes").unwrap();
    /// if let Some(recipes) = index.get_recipes_for_ingredient("chicken") {
    ///     for recipe in recipes {
    ///         println!("Found chicken recipe: {:?}", recipe);
    ///     }
    /// }
    /// ```
    pub fn get_recipes_for_ingredient(&self, ingredient: &str) -> Option<&Vec<PathBuf>> {
        self.index.get(ingredient)
    }

    /// Gets a sorted list of all ingredients in the index
    ///
    /// # Returns
    /// * `Vec<&String>` - Sorted vector of ingredient names
    ///
    /// # Example
    /// ```no_run
    /// # use cooklang_indexer::IngredientIndex;
    /// # let index = IngredientIndex::new("./recipes").unwrap();
    /// for ingredient in index.ingredients() {
    ///     println!("Found ingredient: {}", ingredient);
    /// }
    /// ```
    pub fn ingredients(&self) -> Vec<&String> {
        let mut ingredients: Vec<_> = self.index.keys().collect();
        ingredients.sort();
        ingredients
    }
}

/// Converts a file path to a URL using the provided base URL
///
/// # Arguments
/// * `path` - Path to the recipe file
/// * `base_url` - Base URL where recipes are hosted
/// * `base_dir` - Base directory of recipes (used to create relative paths)
///
/// # Returns
/// * `String` - Full URL to the recipe
///
/// # Example
/// ```no_run
/// use cooklang_indexer::path_to_url;
/// use std::path::Path;
/// 
/// let url = path_to_url(
///     Path::new("recipes/chicken_pasta.cook"),
///     "http://example.com/recipes",
///     Path::new("recipes")
/// );
/// assert_eq!(url, "http://example.com/recipes/chicken_pasta");
/// ```
pub fn path_to_url(path: &Path, base_url: &str, base_dir: &Path) -> String {
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

/// Creates the Ingredient-Recipe index
///
/// Walks the provided directory, extracting cooklang ingredients
fn index_recipes(dir: &Path) -> Result<Vec<Recipe>> {
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

/// Build an ingredient index out of the list of recipes and the ingredients they contain
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

/// builds basic html with the list of ingredients and which recipes they 
/// are included in.
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
