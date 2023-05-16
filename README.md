# `errproto`---example showing how to handle errors in an Axum app

## Anti-Patterns in Axum API Error Handling (Not Prescriptive)

1. Creating your own API-exposed error type.
2. Creating your own internal error type (that's fine), AND using it
for the API-exposed type, *simultaneously*.
3. Creating your own structure or trait to wrap around a value that inherits
from one of the known error traits, such as `std::error::Error`
or even `std::fmt::Display`.
4. Creating your own structure or trait to wrap around `anyhow::Error`,
`eyre::Error`, or any custom type created using `thiserror::Error`.
5. A regular `String`.
6. Maybe more...

## Common Reasons Why Some or All of Them Are Anti-Patterns

### Not all I/O errors map to 404, and not all "X" errors go to "Y" status code

- In general, it's not wise to tie internal logic to HTTP semantics
one-to-one.
- It's because it often isn't one-to-one in reality.

### You may have to exhaustively enumerate all error conditions

- I refuse to do it.

### You may have to use downcasting to determine the true error message and other object-oriented patterns and it tends to get verbose in Rust

- I refuse to do this one, too.

### You lose full control over the HTTP `Response`, and you might need it

- Some HTTP error conditions require setting appropriate headers,
including cookies.
- A `Response` (part of Axum) contains the status code already.
- Sometimes, the error condition has to be hidden from the user.
- Yet, often, a custom error response has to be generated and shown
to the user.
- Can you change the status code while handling the error?

### Your code gets longer

- Do you like to do the Go way, which is like having a long chain of
interrupted `if err = asdf() { return error message }` statements?
- I prefer `let next_value = value.validate().map_err(stop(500))?`
kind of approach.
- The question-mark (`?`) operator is a Godsend.
- (The "`?`" (AFAIK) only works with one specific type, that is, `std::result::Result<_, _>`.)

### Some errors are not `Error`s, etc.

- Some common error types (those on the right side of a `Result`) are
  - Not `Sync`
  - Not `Send`
  - Not `std::error::Error`
  - Not `Display` or `Debug`
  - Not constructible (`Infallible`, for instance)
  - Zero-sized
- You still have to handle those.

### You may have to convert all the world's errors to your type first but the typing would get crazy

- And, there is no lowest denominator of all the Error's in the world.
- But you can pretty much assume that it is `Sized`. Zero-size types
(ZST) are also `Sized`.
- (You can't even `Display` them or call `Debug`'s implementation, either.)

## It's Better To Use `Response` For The Error Type

- Use `axum::response::Response` for the error type in your API endpoints.
- Have functions that can turn all kinds of error types to `Response`.
- My function `catch` handles two cases:
  - Your error converter was interested in and able to produce a custom
   HTTP response to displayed to the user. The response could have
   any status code, headers, response type, etc.
  - Or, your error converter didn't produce a response. In that case,
   the default handler kicks in.
- I assume that you may or may not be interested in creating your own
  default error handler function. This function takes in an HTTP
  status code and then produces a complete HTTP response.
- I assume that you may or may not want to hide or show the error
  and want a lot of control over it (while simplifying things down for the
  happy path).
- So along with `catch`, which takes in a suggested HTTP status code
  and an error and then creates a HTTP response (curried form, though),
  I created two helper functions:

1. A suggested default HTTP status to HTTP response converter.
I named it `default_response` and you don't have to use it.
2. A suggested error to HTTP response converter, if the error implements
`Display`.
I named it `transparent`, and, like #1, you don't have to use it.

- Incorporating `default_response` and `transparent`, I created two
  simplified functions, namely `stop` and `transparent_stop`.
- `stop` can take any error type (absolutely no restriction except
that it must be `Sized`) and then produce an HTTP response.
- `stop` works by consuming and ignoring the error value, and only
working with the status code. The status code is fed into
`default_response`.
- `transparent_stop` has the same API surface as `stop`, but it requires
the error type to implement `Display`.
- `transparent_stop` creates a plain-text error message using the
`Display` implementation; it then attaches the suggested status code
to finish the HTTP response.
- `stop` and `transparent_stop` are implemented trivially using
`catch`.

### An alternative to `stop` and `transparent_stop`

- You can bring your own equivalents to `default_response` and `transparent`, and implement your own versions of `stop` and `transparent_stop`.
- You can't use `stop` and `transparent_stop` directly since these function calls are hard-coded, but you can create your own versions of them.
- Utilize `catch` the same way the original implementations do and you're good.
