#[macro_export]
macro_rules! required_env {
  ($name:literal) => {
    env::var($name).expect(&format!(
      "{} `{}`",
      "Missing environment variable".red(),
      $name.red()
    ))
  };
}
