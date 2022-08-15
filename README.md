# Goal 

This repo is to provide a simple example web app(with actix) using mysql and redis as db and cache respectively. 
It is not a complete web app and I have not tested the binary in any way. 
It's just to to test how redis + sql + failsafe + actix can be implemented. 

This app will be a mock calorie calculation app since i'm trying to lose weight right now. 
Apis are written below and example dataset will be provided which can be injected through setupdb.sh script. 
For this app to run, you must have running mysql and redis servers somewhere. 

## Apis 

```
1. [GET] /apis/food
gets all available food list 

{
    [
        {
            <food object>
        }, 
        {
            <food object>
        }
    ]
}

2. [GET] /apis/ingredient
get all available ingredient list

{
    [
        {
            <ingredient object>
        }, 
        {
            <ingredient object>
        }
    ]
}

3. [GET] /apis/calorie/{food}
get calorie for {food} will fail if such food does not exist in the db 


```

## Some ideas

1. Sync 

It will be confusing to see that I've used both sync and async libraries when it could've been done only in async ways.
It's just because I've wanted to try out diesel not diesel_async. I'll review async diesel sometime in the future and may make minor changes to this repo. 

2. Circuit breaker 
"Circuit break can be implemented on both client and server side. 
When implemented in the client side, clients can keep track of postponed respones and save the requests call if corresponding server is deemed as "open". If implemented on the server side, it's usually wrapped around some target operation which might be faulty at some moment. For this example, since we do not have explicit client(we use Postman for testing), we instead used circuit breaker to wrap the db connection, but there's very low risk of the circuit breaker turing into open state since example dataset and queries cannot be too slow in our use case"

For this mock app, it's implemented around sql pool which is not necessary. 