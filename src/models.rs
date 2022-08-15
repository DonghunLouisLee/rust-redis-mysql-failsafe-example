use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Food {
    pub id: i32,
    pub name: String,
}

impl Food {
    pub(crate) fn from_u8(bytes: Vec<u8>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(&bytes)?)
    }
}

// ingredients: Vec<(Ingredient, i64)>, //pair <ingredient, gram>
#[allow(dead_code)]
pub(crate) struct Relationship {
    pub food_id: i32,       //foreign key
    pub ingredient_id: i32, //foreign key
    pub grams: i32,         //amount of ingredient that is used for food
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Ingredient {
    pub id: i32,
    pub name: String,
    pub calorie_per_gram: i32,
}
