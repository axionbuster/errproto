mod apierr;

use axum::{
    extract::Path, http::StatusCode, response::IntoResponse, routing::get, Json, Router, Server,
};
use serde::Serialize;

use crate::apierr::*;

fn bad() -> Result<&'static str, &'static str> {
    Err("bad")
}

fn good() -> Result<&'static str, &'static str> {
    Ok("good")
}

async fn always_500() -> Result<&'static str> {
    // The user sees: "500 Internal Server Error"
    // The user does NOT see: "bad"
    // So, by default, we hide the error from the user.
    bad().map_err(stop(500))
}

async fn always_200() -> Result<&'static str> {
    // The user sees "good" and the status code is 200.
    good().map_err(stop(500))
}

async fn error_with_custom_feedback(number: Option<Path<String>>) -> Result<impl IntoResponse> {
    // I produce a bunch of impl IntoResponse's.
    let validate_length = |number: String| match number.len() {
        0 => Err("You must provide a number."),
        n if n > 5 => Err("The number is too long. Try again with fewer digits."),
        _ => Ok(number),
    };
    let validate_range = |number: i32| match number {
        ..=68 => Err(format!("The number {number} is too low. Try higher :)")),
        69 => Ok(format!(
            "Nice! You guessed the right number, which is {number}!!!"
        )),
        70.. => Err(format!("The number {number} is too high. Try lower :)")),
    };
    let not_a_number = |c: StatusCode, err| {
        // Let's try JSON and some header manipulation just for fun.
        #[derive(Serialize)]
        struct X {
            err: String,
        }
        let msg = Json(X {
            err: format!("{err}"),
        });
        Some((
            c,
            [("Set-Cookie", "foo=bar; Max-Age=10; SameSite=Lax")],
            msg,
        ))
    };

    let number = match number {
        Some(Path(number)) => number,
        None => "".to_string(),
    };

    // Error handling can be customized using the `catch` function.
    // `catch` is a generalization of `stop`.
    //
    // The second argument (handle) produces an optional response
    // by combining the status code and the error.
    //
    // The default `transparent` function produces a response
    // by calling Display on the error, and then wrapping it in `Some(_)`.
    //
    // The third argument (default) is a function that produces a response
    // if the second argument (handle) returns `None`.
    // Here, it's useless, but in other times, it can be useful.
    let number = validate_length(number).map_err(catch(400, transparent, default_response))?;

    // Now, convert this to an integer!
    //
    // Here I use something different than `transparent` to show the error.
    let number = number
        .parse::<i32>()
        .map_err(catch(400, not_a_number, default_response))?;

    // Ok, now, let's check the ranges.
    // (NOTE: transparent_stop is an alias for catch(_, transparent, default_response)).
    validate_range(number).map_err(transparent_stop(StatusCode::BAD_REQUEST))
}

fn router() -> Router {
    Router::new()
        .route("/", get(always_200))
        .route("/500", get(always_500))
        .route("/custom", get(error_with_custom_feedback))
        .route("/custom/:number", get(error_with_custom_feedback))
}

#[tokio::main]
async fn main() {
    let router = router();
    let app = Server::bind(&"0.0.0.0:3000".parse().unwrap()).serve(router.into_make_service());
    app.await.unwrap();
}
