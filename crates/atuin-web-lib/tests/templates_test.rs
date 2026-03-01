use atuin_web_lib::templates;

#[test]
fn test_create_environment() {
    let env = templates::create_environment();
    // In test mode (debug), templates load from disk
    // Just verify the environment can be created without panicking
    assert!(env.get_template("login.html").is_ok());
}

#[test]
fn test_render_login() {
    let env = templates::create_environment();
    let result = templates::render(
        &env,
        "login.html",
        minijinja::context! {
            error => false,
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("Sign in"));
    assert!(html.contains("Username"));
    assert!(html.contains("Password"));
}

#[test]
fn test_render_login_with_error() {
    let env = templates::create_environment();
    let result = templates::render(
        &env,
        "login.html",
        minijinja::context! {
            error => "Bad credentials",
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("Bad credentials"));
}
