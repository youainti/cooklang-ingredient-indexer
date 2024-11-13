# Cooklang-ingredient-indexer

This extracts the ingredients from the cooklang files within a direcotry,
and then produce an index matching ingredients to recipes.
It also contains tools to produce html with the ingredients.

Use:

```
git clone https://github.com/youainti/cooklang-ingredient-indexer.git
cd cooklang-ingredient-indexer
cargo install --path .
```

Now start a `chef serve` instance for a given collection.
```
cd /path/to/collection
chef serve & # to put the server in the background. Use `jobs` and `fg` to recover control.
cooklang-ingredient-indexer . https://localhost:8080/r/
```
now open the `ingredient-index.html` file available in your collection.
Clicking on a recipe will take you to the appropriate recipe.
