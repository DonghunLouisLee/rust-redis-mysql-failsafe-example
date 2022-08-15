use crate::models::{self, Food, Ingredient};
use diesel::prelude::*;
type DbError = Box<dyn std::error::Error + Send + Sync>;

pub(crate) fn find_all_foods(conn: &MysqlConnection) -> Result<Vec<models::Food>, DbError> {
    use crate::schema::food::dsl::*;

    let all_foods = food.load::<(i32, String)>(conn)?;
    Ok(all_foods
        .iter()
        .map(|(a, b)| Food {
            id: *a,
            name: b.to_string(),
        })
        .collect())
}

pub(crate) fn find_all_ingredients(
    conn: &MysqlConnection,
) -> Result<Vec<models::Ingredient>, DbError> {
    use crate::schema::ingredient::dsl::*;

    let all_ingredients = ingredient.load::<(i32, String, i32)>(conn)?;
    Ok(all_ingredients
        .iter()
        .map(|(a, b, c)| Ingredient {
            id: *a,
            name: b.to_string(),
            calorie_per_gram: *c,
        })
        .collect())
}

pub(crate) fn find_calorie(_food_id: i32, conn: &MysqlConnection) -> Result<i32, DbError> {
    use crate::schema::relationship::dsl::*;

    let ingredients = relationship
        .filter(food_id.eq(food_id))
        .load::<(i32, i32, i32)>(conn)?;

    let a: Vec<i32> = ingredients
        .iter()
        .map(|x| get_ingredient_calorie(x.1, conn).unwrap())
        .collect();
    Ok(a.iter().sum())
}

fn get_ingredient_calorie(ingredient_id: i32, conn: &MysqlConnection) -> Result<i32, DbError> {
    use crate::schema::ingredient::dsl::*;

    let ingredient_item: i32 = ingredient
        .filter(id.eq(ingredient_id))
        .select(calorie_per_gram)
        .first(conn)?; //there should be only one so take the first result
    Ok(ingredient_item)
}
