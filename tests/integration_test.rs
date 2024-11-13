// tests/integration_test.rs
use cooklang_indexer::IngredientIndex;
use std::path::Path;

#[test]
fn test_index_creation() {
    let index = IngredientIndex::new("./test_recipes").unwrap();
    assert!(!index.ingredients().is_empty());
}
