use serde::Serialize;
use serde_json::Value as JsonValue;
use std::fmt::Display;

/// Formats a function name and argument to be evaluated as callback.
///
/// # Examples
/// ```
/// use tauri_api::rpc::format_callback;
/// // callback with a string argument
/// let cb = format_callback("callback-function-name", "the string response");
/// assert!(cb.contains(r#"window["callback-function-name"]("the string response")"#));
/// ```
///
/// ```
/// use tauri_api::rpc::format_callback;
/// use serde::Serialize;
/// // callback with JSON argument
/// #[derive(Serialize)]
/// struct MyResponse {
///   value: String
/// }
/// let cb = format_callback("callback-function-name", serde_json::to_value(&MyResponse {
///   value: "some value".to_string()
/// }).expect("failed to serialize"));
/// assert!(cb.contains(r#"window["callback-function-name"]({"value":"some value"})"#));
/// ```
pub fn format_callback<T: Into<JsonValue>, S: AsRef<str> + Display>(
  function_name: S,
  arg: T,
) -> String {
  format!(
    r#"
      if (window["{fn}"]) {{
        window["{fn}"]({arg})
      }} else {{
        console.warn("[TAURI] Couldn't find callback id {fn} in window. This happens when the app is reloaded while Rust is running an asynchronous operation.")
      }}
    "#,
    fn = function_name,
    arg = arg.into().to_string()
  )
}

/// Formats a Result type to its Promise response.
/// Useful for Promises handling.
/// If the Result `is_ok()`, the callback will be the `success_callback` function name and the argument will be the Ok value.
/// If the Result `is_err()`, the callback will be the `error_callback` function name and the argument will be the Err value.
///
/// * `result` the Result to check
/// * `success_callback` the function name of the Ok callback. Usually the `resolve` of the JS Promise.
/// * `error_callback` the function name of the Err callback. Usually the `reject` of the JS Promise.
///
/// Note that the callback strings are automatically generated by the `promisified` helper.
///
/// # Examples
/// ```
/// use tauri_api::rpc::format_callback_result;
/// let res: Result<u8, &str> = Ok(5);
/// let cb = format_callback_result(res, "success_cb".to_string(), "error_cb".to_string()).expect("failed to format");
/// assert!(cb.contains(r#"window["success_cb"](5)"#));
///
/// let res: Result<&str, &str> = Err("error message here");
/// let cb = format_callback_result(res, "success_cb".to_string(), "error_cb".to_string()).expect("failed to format");
/// assert!(cb.contains(r#"window["error_cb"]("error message here")"#));
/// ```
pub fn format_callback_result<T: Serialize, E: Serialize>(
  result: Result<T, E>,
  success_callback: String,
  error_callback: String,
) -> crate::Result<String> {
  let rpc = match result {
    Ok(res) => format_callback(success_callback, serde_json::to_value(res)?),
    Err(err) => format_callback(error_callback, serde_json::to_value(err)?),
  };
  Ok(rpc)
}

#[cfg(test)]
mod test {
  use crate::rpc::*;
  use quickcheck_macros::quickcheck;

  // check abritrary strings in the format callback function
  #[quickcheck]
  fn qc_formating(f: String, a: String) -> bool {
    // can not accept empty strings
    if f != "" && a != "" {
      // call format callback
      let fc = format_callback(f.clone(), a.clone());
      fc.contains(&format!(
        r#"window["{}"]({})"#,
        f,
        serde_json::Value::String(a),
      ))
    } else {
      true
    }
  }

  // check arbitrary strings in format_callback_result
  #[quickcheck]
  fn qc_format_res(result: Result<String, String>, c: String, ec: String) -> bool {
    let resp = format_callback_result(result.clone(), c.clone(), ec.clone())
      .expect("failed to format callback result");
    let (function, value) = match result {
      Ok(v) => (c, v),
      Err(e) => (ec, e),
    };

    resp.contains(&format!(
      r#"window["{}"]({})"#,
      function,
      serde_json::Value::String(value),
    ))
  }
}
