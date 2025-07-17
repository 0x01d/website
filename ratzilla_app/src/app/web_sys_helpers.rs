use web_sys::window;

fn get_browser_url() -> Option<String> {
    let window = window()?;
    let location = window.location();
    location.href().ok()
}
