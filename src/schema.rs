table! {
    food (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    ingredient (id) {
        id -> Int4,
        name -> Varchar,
        calorie_per_gram -> Int4,
    }
}

table! {
    relationship (food_id, ingredient_id) {
        food_id -> Int4,
        ingredient_id -> Int4,
        grams -> Int4,
    }
}
