#[macro_use]
extern crate diesel;
use std::ops::{DerefMut};
use std::time::Duration;

use actix_web::{get, middleware, web, App, Error, HttpResponse, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod models;
mod query;
mod schema;

use failsafe::backoff::EqualJittered;
use failsafe::failure_policy::{ConsecutiveFailures, OrElse, SuccessRateOverTimeWindow};
use failsafe::{CircuitBreaker, Config, StateMachine};
use r2d2_redis::redis::{Commands, RedisError};
use r2d2_redis::RedisConnectionManager;

use crate::models::Food;

pub type RedisPool = r2d2::Pool<RedisConnectionManager>;

const CACHE_POOL_MAX_OPEN: u32 = 16;
const CACHE_POOL_MIN_IDLE: u32 = 8;
const CACHE_POOL_EXPIRE_SECONDS: u64 = 60;

//we skip dto for now

type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;


const GET_ALL_FOOD_KEY: &str = "all";
type CircuitBreakerType = StateMachine<
    OrElse<SuccessRateOverTimeWindow<EqualJittered>, ConsecutiveFailures<EqualJittered>>,
    (),
>;

#[get("/apis/food")]
async fn get_all_food(
    circuit_breaker: web::Data<CircuitBreakerType>,
    redis_pool: web::Data<RedisPool>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    //first check if data is in redis

    match redis_pool.get() {
        Ok(mut redis_conn) => {
            //first check if circuit breaker is permitted
            if !circuit_breaker.is_call_permitted() {
                //circuit breaker is not permitted so return error straight away
                //error message should be included
                return Err(actix_web::error::ErrorInternalServerError(""));
            }
            //call redis connection
            //to be fair, redis should also be wrapped around separate circuit breaker but let's skip that for now
            let redis_conn = redis_conn.deref_mut();
            let a: Result<Vec<u8>, RedisError> = redis_conn.get(GET_ALL_FOOD_KEY);
            match a {
                Ok(value) => {
                    if value.is_empty() {
                        //if cache does not exist, query the db but cache the result afterwards
                        let foods = web::block(move || {
                            let conn = pool.get().unwrap();
                            //let's just treat error message as String for simplicity
                            let result: Result<Vec<Food>, String> =
                                match circuit_breaker.call(|| query::find_all_foods(&conn)) {
                                    Err(failsafe::Error::Inner(_)) => {
                                        //error should be treated
                                        todo!()
                                    }
                                    Err(failsafe::Error::Rejected) => {
                                        //rejected which means sql db is not responsive
                                        Err("sql is not working".to_string())
                                    }
                                    Ok(val) => {
                                        return Ok(val);
                                        //this means success
                                    }
                                };
                            return result;
                            // let conn = pool.get()?;
                            // query::find_all_foods(&conn)
                        })
                        .await?
                        .map_err(actix_web::error::ErrorInternalServerError)?;
                        //data was found but since connection is lost, don't bother caching the data
                        let value = bincode::serialize(&foods).unwrap();
                        let _a: bool = redis_conn.set(GET_ALL_FOOD_KEY, value).unwrap();
                        return Ok(HttpResponse::Ok().json(foods));
                    }
                    return Ok(HttpResponse::Ok().json(Food::from_u8(value).unwrap()));
                }

                Err(_) => {
                    //need to hanlde error correctly based on the error types
                    unimplemented!()
                }
            }
        }
        Err(_) => {
            let foods = web::block(move || {
                let conn = pool.get().unwrap();
                //let's just treat error message as String for simplicity
                let result: Result<Vec<Food>, String> =
                    match circuit_breaker.call(|| query::find_all_foods(&conn)) {
                        Err(failsafe::Error::Inner(_)) => {
                            //error should be treated
                            todo!()
                        }
                        Err(failsafe::Error::Rejected) => {
                            //rejected which means sql db is not responsive
                            Err("sql is not working".to_string())
                        }
                        Ok(val) => {
                            return Ok(val);
                            //this means success
                        }
                    };
                return result;
                // let conn = pool.get()?;
                // query::find_all_foods(&conn)
            })
            .await?
            .map_err(actix_web::error::ErrorInternalServerError)?;
            //data was found but since connection is lost, don't bother caching the data
            Ok(HttpResponse::Ok().json(foods))
        }
    }
}

#[get("/apis/ingredients")]
async fn get_all_ingredients(
    _redis_pool: web::Data<RedisPool>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let ingredients = web::block(move || {
        let conn = pool.get()?;
        query::find_all_ingredients(&conn)
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(ingredients))
}

//api should include valid food_id
#[get("/apis/calorie/{food_id}")]
async fn get_calorie(
    food_id: web::Path<i32>,
    _redis_pool: web::Data<RedisPool>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let calorie = web::block(move || {
        let conn = pool.get()?;
        query::find_calorie(food_id.into_inner(), &conn)
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(calorie))
}

//todo need post method for updating data

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // set up database connection pool
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<MysqlConnection>::new(conn_spec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let redis_con_string = "";
    let manager = RedisConnectionManager::new(redis_con_string).unwrap();
    let redis_pool = r2d2::Pool::builder()
        .max_size(CACHE_POOL_MAX_OPEN)
        .max_lifetime(Some(Duration::from_secs(CACHE_POOL_EXPIRE_SECONDS)))
        .min_idle(Some(CACHE_POOL_MIN_IDLE))
        .build(manager)
        .unwrap();

    let circuit_breaker = Config::new().build();

    log::info!("starting HTTP server at http://localhost:8080");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // set up DB pool to be used with web::Data<Pool> extractor
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(redis_pool.clone()))
            .app_data(web::Data::new(circuit_breaker.clone()))
            .wrap(middleware::Logger::default())
            .service(get_all_food)
            .service(get_all_ingredients)
            .service(get_all_ingredients)
            .service(get_calorie)
        // .service() # add other methods in here
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
