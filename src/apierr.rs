//! Api Error Handling
//!
//! You would be particularly interested in these functions
//! (and I recommend looking at them in this order):
//! - [`stop`]: Map error, hide it from the user, set status code.
//! - [`transparent_stop`]: Map error, show the user, set status code.
//! - [`catch`]: Map error, do custom handling, set status code.
//!
//! See also: [`Result::map_err`].

use std::fmt::{Debug, Display};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// API Result
pub type Result<R = Response, E = Response> = std::result::Result<R, E>;

/// Return a closure that takes what would be an error type
/// and then discards it, while giving a default error message to the user.
///
/// (A plain text message with the canonical reason, if exists.)
///
/// This default responder is used: [`default_response`].
pub fn stop<C, E, Z>(code: C) -> impl FnOnce(E) -> Response
where
    C: TryInto<StatusCode, Error = Z>,
    Z: Debug,
{
    catch(code, |_, _| None::<Response>, default_response)
}

/// A variety of [`stop`] that, rather than hiding the error from
/// the user, shows it using a call to [`transparent`].
pub fn transparent_stop<C, E, Z>(code: C) -> impl FnOnce(E) -> Response
where
    C: TryInto<StatusCode, Error = Z>,
    E: Display,
    Z: Debug,
{
    catch(code, transparent, default_response)
}

/// Return a closure that takes what would be an error type
/// and then consumes it, producing an optional custom response.
/// If the response is generated, it is served to the user.
/// If the response is not generated, a default error message is given to the user.
///
/// NOTE: when the custom response is generated, it's the responsibility of the
/// caller to set the status code. The `code` provided is only the default
/// status code to use when the custom response is NOT generated.
///
/// HINT: For the default response generator (`default`), you can use
/// [`default_response`] without any problem.
///
/// NOTE: There are no restrictions on the type of the error.
pub fn catch<C, D, E, F, R, Z>(code: C, handle: F, default: D) -> impl FnOnce(E) -> Response
where
    C: TryInto<StatusCode, Error = Z>,
    D: Fn(StatusCode) -> Response,
    F: FnOnce(StatusCode, E) -> Option<R>,
    R: IntoResponse,
    Z: Debug,
{
    move |e| {
        let code = cvcode(code);
        if let Some(r) = handle(code, e) {
            r.into_response()
        } else {
            default(code)
        }
    }
}

/// Create the default error response for a given status code.
///
/// HINT: Use as the third argument for the [`catch`] function.
pub fn default_response(code: StatusCode) -> Response
where
{
    let code = cvcode(code);
    let body = format!("{}", code); // It gives the numeric code + reason.
    (code, body).into_response()
}

/// Create a response by calling Display implementation on an
/// error type and returning a plain text message.
///
/// HINT: Use as the second argument for the [`catch`] function.
pub fn transparent<D>(code: StatusCode, error: D) -> Option<Response>
where
    D: Display,
{
    let body = format!("{}", error);
    Some((code, body).into_response())
}

/// Convert what could be a status code into a [`StatusCode`], or
/// panic if the conversion fails.
fn cvcode<C, Z>(code: C) -> StatusCode
where
    C: TryInto<StatusCode, Error = Z>,
    Z: Debug,
{
    code.try_into().expect("invalid status code")
}
